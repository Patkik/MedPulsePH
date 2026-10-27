//! Cryptographic primitives for Vault B field-level encryption and blind patient identifiers.
//!
//! Enforces:
//! - Versioned AES-256-GCM envelopes with Authenticated Associated Data (AAD) binding
//!   ciphertext to `(BlindPatientId, ResourcePurpose, schema_version)`.
//! - Reviewed 96-bit nonce generation combining a random 32-bit instance salt with a
//!   strictly monotonic 64-bit counter and an explicit invocation limit before rotation.
//! - Key-ring rotation supporting decryption of historical key versions and fresh-nonce
//!   re-encryption under the active key.
//! - Keyed HMAC-SHA256 derivation of `BlindPatientId` with constant-time equality.

use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, Payload},
};
use hmac::{Hmac, Mac};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{
    collections::BTreeMap,
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};
use subtle::ConstantTimeEq;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Current binary/JSON envelope format version.
pub const ENVELOPE_FORMAT_VERSION: u16 = 1;

/// Maximum AES-256-GCM encryptions permitted for a single key instance before rotation is required.
pub const DEFAULT_MAX_NONCE_INVOCATIONS: u64 = 1 << 32;

/// Typed cryptographic errors that never leak plaintext or raw key material.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CryptoError {
    #[error("patient identifier input must not be empty")]
    EmptyPatientIdentifier,
    #[error("key identifier must not be empty")]
    EmptyKeyId,
    #[error("key version must be greater than zero")]
    InvalidKeyVersion,
    #[error("schema version must be greater than zero")]
    InvalidSchemaVersion,
    #[error("encryption key is not registered in the cipher ring")]
    UnknownKeyId,
    #[error("cannot remove or disable the currently active encryption key")]
    CannotDisableActiveKey,
    #[error("nonce invocation budget exhausted; rotate the active encryption key")]
    NonceBudgetExhausted,
    #[error("unsupported encryption envelope format version")]
    UnsupportedEnvelopeVersion,
    #[error("authenticated decryption failed or associated data did not match")]
    AuthenticationFailed,
    #[error("invalid blind patient identifier encoding")]
    InvalidBlindIdEncoding,
}

/// Logical clinical resource or field purpose bound into AES-256-GCM associated data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourcePurpose {
    EncounterNote,
    DiagnosisCondition,
    AllergyRecord,
    MedicationOrder,
    ObservationValue,
    AuditContext,
}

impl ResourcePurpose {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EncounterNote => "encounter_note",
            Self::DiagnosisCondition => "diagnosis_condition",
            Self::AllergyRecord => "allergy_record",
            Self::MedicationOrder => "medication_order",
            Self::ObservationValue => "observation_value",
            Self::AuditContext => "audit_context",
        }
    }
}

/// Pseudonymous identifier used as the sole patient index inside Vault B (Clinical EHR).
#[derive(Clone, Serialize, Deserialize)]
pub struct BlindPatientId {
    key_version: u16,
    digest: [u8; 32],
}

impl BlindPatientId {
    pub fn key_version(&self) -> u16 {
        self.key_version
    }

    /// Canonical string representation (`bp_v{version}_{64_hex_chars}`) safe for SQL parameters.
    pub fn to_canonical_string(&self) -> String {
        format!("bp_v{}_{}", self.key_version, hex::encode(self.digest))
    }

    pub fn from_canonical_str(value: &str) -> Result<Self, CryptoError> {
        let rest = value
            .strip_prefix("bp_v")
            .ok_or(CryptoError::InvalidBlindIdEncoding)?;
        let (version_part, hex_part) = rest
            .split_once('_')
            .ok_or(CryptoError::InvalidBlindIdEncoding)?;
        let key_version: u16 = version_part
            .parse()
            .map_err(|_| CryptoError::InvalidBlindIdEncoding)?;
        if key_version == 0 {
            return Err(CryptoError::InvalidKeyVersion);
        }
        let decoded = hex::decode(hex_part).map_err(|_| CryptoError::InvalidBlindIdEncoding)?;
        let digest: [u8; 32] = decoded
            .try_into()
            .map_err(|_| CryptoError::InvalidBlindIdEncoding)?;
        Ok(Self {
            key_version,
            digest,
        })
    }
}

impl PartialEq for BlindPatientId {
    fn eq(&self, other: &Self) -> bool {
        let version_eq = self
            .key_version
            .to_be_bytes()
            .ct_eq(&other.key_version.to_be_bytes());
        let digest_eq = self.digest.ct_eq(&other.digest);
        (version_eq & digest_eq).into()
    }
}

impl Eq for BlindPatientId {}

impl fmt::Debug for BlindPatientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlindPatientId")
            .field("key_version", &self.key_version)
            .field("prefix", &&hex::encode(&self.digest[..4]))
            .finish()
    }
}

/// Derives `BlindPatientId` values from Vault A internal patient IDs using keyed HMAC-SHA256.
pub struct BlindIdDeriver {
    key_version: u16,
    secret_key: [u8; 32],
}

impl BlindIdDeriver {
    pub fn new(key_version: u16, secret_key: [u8; 32]) -> Result<Self, CryptoError> {
        if key_version == 0 {
            return Err(CryptoError::InvalidKeyVersion);
        }
        Ok(Self {
            key_version,
            secret_key,
        })
    }

    pub fn derive(&self, internal_patient_id: &str) -> Result<BlindPatientId, CryptoError> {
        let normalized = internal_patient_id.trim();
        if normalized.is_empty() {
            return Err(CryptoError::EmptyPatientIdentifier);
        }
        let mut mac = <HmacSha256 as Mac>::new_from_slice(&self.secret_key)
            .map_err(|_| CryptoError::AuthenticationFailed)?;
        mac.update(b"medpulse:blind-patient-id:v1:");
        mac.update(&self.key_version.to_be_bytes());
        mac.update(b":");
        mac.update(normalized.as_bytes());
        let bytes = mac.finalize().into_bytes();
        let mut digest = [0u8; 32];
        digest.copy_from_slice(&bytes);
        Ok(BlindPatientId {
            key_version: self.key_version,
            digest,
        })
    }
}

/// Authenticated context bound to every encrypted clinical record in Vault B.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeAad {
    blind_patient_id: BlindPatientId,
    purpose: ResourcePurpose,
    schema_version: u16,
}

impl EnvelopeAad {
    pub fn new(
        blind_patient_id: BlindPatientId,
        purpose: ResourcePurpose,
        schema_version: u16,
    ) -> Result<Self, CryptoError> {
        if schema_version == 0 {
            return Err(CryptoError::InvalidSchemaVersion);
        }
        Ok(Self {
            blind_patient_id,
            purpose,
            schema_version,
        })
    }

    pub fn to_aad_bytes(&self) -> Vec<u8> {
        format!(
            "medpulse:aad:v1|{}|{}|{}",
            self.blind_patient_id.to_canonical_string(),
            self.purpose.as_str(),
            self.schema_version
        )
        .into_bytes()
    }
}

/// Versioned AES-256-GCM ciphertext envelope stored in Vault B.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    pub envelope_version: u16,
    pub key_id: String,
    pub nonce: [u8; 12],
    pub ciphertext_with_tag: Vec<u8>,
}

/// 96-bit nonce generator combining a 32-bit random instance prefix with a 64-bit atomic counter.
pub struct NonceGenerator {
    instance_prefix: [u8; 4],
    counter: AtomicU64,
    max_invocations: u64,
}

impl NonceGenerator {
    pub fn new_random(max_invocations: u64) -> Self {
        let mut instance_prefix = [0u8; 4];
        OsRng.fill_bytes(&mut instance_prefix);
        Self {
            instance_prefix,
            counter: AtomicU64::new(0),
            max_invocations,
        }
    }

    pub fn next_nonce(&self) -> Result<[u8; 12], CryptoError> {
        let seq = self.counter.fetch_add(1, Ordering::Relaxed);
        if seq >= self.max_invocations {
            return Err(CryptoError::NonceBudgetExhausted);
        }
        let mut nonce = [0u8; 12];
        nonce[..4].copy_from_slice(&self.instance_prefix);
        nonce[4..].copy_from_slice(&seq.to_be_bytes());
        Ok(nonce)
    }

    pub fn remaining_invocations(&self) -> u64 {
        let used = self.counter.load(Ordering::Relaxed);
        self.max_invocations.saturating_sub(used)
    }
}

/// Key material for initializing or rotating `VaultCipherRing`.
pub struct VaultEncryptionKey {
    key_id: String,
    key_bytes: [u8; 32],
}

impl VaultEncryptionKey {
    pub fn new(key_id: impl Into<String>, key_bytes: [u8; 32]) -> Result<Self, CryptoError> {
        let key_id = key_id.into();
        if key_id.trim().is_empty() {
            return Err(CryptoError::EmptyKeyId);
        }
        Ok(Self { key_id, key_bytes })
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }
}

/// Manages active and historical AES-256-GCM keys for Vault B encryption and key rotation.
pub struct VaultCipherRing {
    active_key_id: String,
    ciphers: BTreeMap<String, Aes256Gcm>,
    nonce_generator: NonceGenerator,
    max_invocations_per_key: u64,
}

impl VaultCipherRing {
    pub fn new(active_key: VaultEncryptionKey) -> Self {
        Self::with_invocation_limit(active_key, DEFAULT_MAX_NONCE_INVOCATIONS)
    }

    pub fn with_invocation_limit(
        active_key: VaultEncryptionKey,
        max_invocations_per_key: u64,
    ) -> Self {
        let mut ciphers = BTreeMap::new();
        let cipher = Aes256Gcm::new((&active_key.key_bytes).into());
        ciphers.insert(active_key.key_id.clone(), cipher);
        Self {
            active_key_id: active_key.key_id,
            ciphers,
            nonce_generator: NonceGenerator::new_random(max_invocations_per_key),
            max_invocations_per_key,
        }
    }

    pub fn active_key_id(&self) -> &str {
        &self.active_key_id
    }

    /// Registers a historical decryption-only key so restored backups remain decryptable.
    pub fn register_historical_key(&mut self, key: VaultEncryptionKey) {
        let cipher = Aes256Gcm::new((&key.key_bytes).into());
        self.ciphers.insert(key.key_id, cipher);
    }

    /// Rotates the active key used for all subsequent encryptions and resets the nonce generator.
    pub fn rotate_active_key(&mut self, new_key: VaultEncryptionKey) {
        let cipher = Aes256Gcm::new((&new_key.key_bytes).into());
        self.ciphers.insert(new_key.key_id.clone(), cipher);
        self.active_key_id = new_key.key_id;
        self.nonce_generator = NonceGenerator::new_random(self.max_invocations_per_key);
    }

    /// Destroys a retired key so obsolete keys cannot remain usable indefinitely.
    pub fn destroy_retired_key(&mut self, key_id: &str) -> Result<(), CryptoError> {
        if key_id == self.active_key_id {
            return Err(CryptoError::CannotDisableActiveKey);
        }
        self.ciphers
            .remove(key_id)
            .map(|_| ())
            .ok_or(CryptoError::UnknownKeyId)
    }

    pub fn encrypt(
        &self,
        plaintext: &[u8],
        aad: &EnvelopeAad,
    ) -> Result<EncryptedEnvelope, CryptoError> {
        let cipher = self
            .ciphers
            .get(&self.active_key_id)
            .ok_or(CryptoError::UnknownKeyId)?;
        let nonce_bytes = self.nonce_generator.next_nonce()?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let aad_bytes = aad.to_aad_bytes();
        let ciphertext_with_tag = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: plaintext,
                    aad: &aad_bytes,
                },
            )
            .map_err(|_| CryptoError::AuthenticationFailed)?;
        Ok(EncryptedEnvelope {
            envelope_version: ENVELOPE_FORMAT_VERSION,
            key_id: self.active_key_id.clone(),
            nonce: nonce_bytes,
            ciphertext_with_tag,
        })
    }

    pub fn decrypt(
        &self,
        envelope: &EncryptedEnvelope,
        expected_aad: &EnvelopeAad,
    ) -> Result<Vec<u8>, CryptoError> {
        if envelope.envelope_version != ENVELOPE_FORMAT_VERSION {
            return Err(CryptoError::UnsupportedEnvelopeVersion);
        }
        let cipher = self
            .ciphers
            .get(&envelope.key_id)
            .ok_or(CryptoError::UnknownKeyId)?;
        let nonce = Nonce::from_slice(&envelope.nonce);
        let aad_bytes = expected_aad.to_aad_bytes();
        cipher
            .decrypt(
                nonce,
                Payload {
                    msg: &envelope.ciphertext_with_tag,
                    aad: &aad_bytes,
                },
            )
            .map_err(|_| CryptoError::AuthenticationFailed)
    }

    /// Decrypts an envelope under its recorded key and re-encrypts under the active key with a fresh nonce.
    pub fn reencrypt(
        &self,
        envelope: &EncryptedEnvelope,
        aad: &EnvelopeAad,
    ) -> Result<EncryptedEnvelope, CryptoError> {
        let plaintext = self.decrypt(envelope, aad)?;
        self.encrypt(&plaintext, aad)
    }
}
