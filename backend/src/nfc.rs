//! NTAG 424 DNA SUN (Secure Unique NFC) cryptographic verification, 256-byte dual-record
//! NDEF budget validation, band lifecycle transitions, and Vault A durable replay acceptance
//! (`07-nfc-bands-and-emergency.md`, `08-replay-and-clinical-authorization.md`).

use crate::{
    auth::{AcceptedTapContext, BandStatus},
    crypto::BlindPatientId,
};
use aes::{
    Aes128,
    cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray},
};
use cmac::{Cmac, Mac};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use thiserror::Error;

type Aes128Cmac = Cmac<Aes128>;

/// NXP AN12196 PICCData tag byte indicating 7-byte UID + 24-bit SDMReadCtr mirrored in encrypted PICCData.
pub const NTAG424_PICC_DATA_TAG_UID_AND_CTR: u8 = 0xC7;

/// Maximum NTAG 424 DNA standard NDEF file size in bytes (`07-nfc-bands-and-emergency.md`).
pub const NTAG424_NDEF_FILE_CAPACITY_BYTES: usize = 256;

/// Maximum permitted byte length for Record 2 offline fallback text (`nfc-sun-crypto`).
pub const MAX_OFFLINE_RECORD2_TEXT_BYTES: usize = 120;

/// Maximum 24-bit NFC read counter value (`0x00FF_FFFF`) before counter exhaustion requires replacement.
pub const MAX_NTAG424_READ_COUNTER: u32 = 0x00FF_FFFF;

/// Typed errors for NFC SUN cryptography, NDEF byte budgeting, band lifecycle, and replay enforcement.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum NfcError {
    #[error("invalid hex encoding in NFC tap parameters")]
    InvalidHexPayload,
    #[error("encrypted PICCData must be 16 bytes (32 hex chars) and CMAC must be 8 bytes (16 hex chars)")]
    InvalidPayloadLength,
    #[error("NTAG 424 DNA SUN CMAC authentication failed")]
    InvalidSunCmac,
    #[error("decrypted PICCData header tag is invalid")]
    InvalidPiccDataHeader,
    #[error("decrypted tag UID does not match the registered wristband")]
    UidMismatch,
    #[error("NFC read counter has reached the 24-bit hardware limit")]
    CounterExhausted,
    #[error("wristband is not active (current status: {0:?})")]
    BandNotActive(BandStatus),
    #[error("replayed or stale NFC tap counter rejected (incoming {incoming} <= durable high-water {high_water})")]
    ReplayRejected { incoming: u32, high_water: u32 },
    #[error("invalid wristband lifecycle state transition from {from:?} to {to:?}")]
    InvalidBandTransition { from: BandStatus, to: BandStatus },
    #[error("NDEF payload fields must not be empty or contain the '|' delimiter")]
    InvalidOfflineFieldFormat,
    #[error("critical NDEF content requires {required_bytes} bytes, exceeding the {max_bytes}-byte limit; refusing to truncate allergies")]
    NdefBudgetExceeded {
        required_bytes: usize,
        max_bytes: usize,
    },
}

/// Decrypted and CMAC-authenticated NTAG 424 DNA SUN tap proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedSunTap {
    pub uid: [u8; 7],
    pub read_counter: u32,
}

/// Per-band NTAG 424 DNA AES-128 keys (diversified or per-tag provisioned).
#[derive(Clone)]
pub struct Ntag424TagKeys {
    pub expected_uid: [u8; 7],
    pub sdm_meta_read_key: [u8; 16],
    pub sdm_file_read_key: [u8; 16],
}

impl Ntag424TagKeys {
    /// Computes the NXP AN12196 SV2 session MAC key `K_SesSDMFileReadMAC` for `(uid, read_counter)`.
    fn derive_session_mac_key(&self, uid: &[u8; 7], read_counter: u32) -> Result<[u8; 16], NfcError> {
        let ctr_le = read_counter.to_le_bytes();
        let mut sv2 = [0u8; 16];
        sv2[0] = 0x3C;
        sv2[1] = 0xC3;
        sv2[2] = 0x00;
        sv2[3] = 0x01;
        sv2[4] = 0x00;
        sv2[5] = 0x80;
        sv2[6..13].copy_from_slice(uid);
        sv2[13..16].copy_from_slice(&ctr_le[..3]);

        let mut mac = <Aes128Cmac as Mac>::new_from_slice(&self.sdm_file_read_key)
            .map_err(|_| NfcError::InvalidSunCmac)?;
        mac.update(&sv2);
        let bytes = mac.finalize().into_bytes();
        let mut session_key = [0u8; 16];
        session_key.copy_from_slice(&bytes);
        Ok(session_key)
    }

    /// Computes the 8-byte truncated SUN CMAC (odd-indexed bytes `1, 3, 5, ..., 15` per NXP AN12196).
    fn compute_truncated_cmac(
        &self,
        uid: &[u8; 7],
        read_counter: u32,
        authenticated_ascii_prefix: &[u8],
    ) -> Result<[u8; 8], NfcError> {
        let session_key = self.derive_session_mac_key(uid, read_counter)?;
        let mut mac = <Aes128Cmac as Mac>::new_from_slice(&session_key)
            .map_err(|_| NfcError::InvalidSunCmac)?;
        mac.update(authenticated_ascii_prefix);
        let full = mac.finalize().into_bytes();
        Ok([
            full[1], full[3], full[5], full[7], full[9], full[11], full[13], full[15],
        ])
    }

    /// Decrypts `picc_data_hex` (32 hex chars) and verifies `cmac_hex` (16 hex chars) in constant time.
    pub fn verify_sun_tap(
        &self,
        picc_data_hex: &str,
        cmac_hex: &str,
        authenticated_ascii_prefix: &[u8],
    ) -> Result<VerifiedSunTap, NfcError> {
        let picc_bytes = hex::decode(picc_data_hex).map_err(|_| NfcError::InvalidHexPayload)?;
        let cmac_bytes = hex::decode(cmac_hex).map_err(|_| NfcError::InvalidHexPayload)?;
        if picc_bytes.len() != 16 || cmac_bytes.len() != 8 {
            return Err(NfcError::InvalidPayloadLength);
        }

        let cipher = Aes128::new(GenericArray::from_slice(&self.sdm_meta_read_key));
        let mut block = GenericArray::clone_from_slice(&picc_bytes);
        cipher.decrypt_block(&mut block);

        let tag_header = block[0];
        let mut uid = [0u8; 7];
        uid.copy_from_slice(&block[1..8]);
        let read_counter = u32::from_le_bytes([block[8], block[9], block[10], 0]);

        let expected_cmac =
            self.compute_truncated_cmac(&uid, read_counter, authenticated_ascii_prefix)?;

        // Verify CMAC, header tag, and UID in constant time before branching on counter value.
        let cmac_ok = expected_cmac.ct_eq(&cmac_bytes);
        let header_ok = tag_header.ct_eq(&NTAG424_PICC_DATA_TAG_UID_AND_CTR);
        let uid_ok = uid.ct_eq(&self.expected_uid);

        if cmac_ok.unwrap_u8() == 0 {
            return Err(NfcError::InvalidSunCmac);
        }
        if header_ok.unwrap_u8() == 0 {
            return Err(NfcError::InvalidPiccDataHeader);
        }
        if uid_ok.unwrap_u8() == 0 {
            return Err(NfcError::UidMismatch);
        }
        if read_counter == 0 || read_counter >= MAX_NTAG424_READ_COUNTER {
            return Err(NfcError::CounterExhausted);
        }

        Ok(VerifiedSunTap { uid, read_counter })
    }

    /// Generates an encrypted `(picc_data_hex, cmac_hex)` pair for provisioning verification and test vectors.
    pub fn seal_sun_tap_for_vector(
        &self,
        read_counter: u32,
        authenticated_ascii_prefix: &[u8],
    ) -> Result<(String, String), NfcError> {
        if read_counter == 0 || read_counter >= MAX_NTAG424_READ_COUNTER {
            return Err(NfcError::CounterExhausted);
        }
        let mut plain = [0u8; 16];
        plain[0] = NTAG424_PICC_DATA_TAG_UID_AND_CTR;
        plain[1..8].copy_from_slice(&self.expected_uid);
        let ctr_le = read_counter.to_le_bytes();
        plain[8..11].copy_from_slice(&ctr_le[..3]);

        let cipher = Aes128::new(GenericArray::from_slice(&self.sdm_meta_read_key));
        let mut block = GenericArray::clone_from_slice(&plain);
        cipher.encrypt_block(&mut block);

        let cmac = self.compute_truncated_cmac(
            &self.expected_uid,
            read_counter,
            authenticated_ascii_prefix,
        )?;

        Ok((hex::encode(block), hex::encode(cmac)))
    }
}

/// Validates and executes explicit wristband lifecycle state transitions (`07-nfc-bands-and-emergency.md`, `20-state-machines-and-consistency.md`).
///
/// Terminal states (`Retired`, `Failed`) can never transition back to `Active`.
pub fn validate_band_status_transition(from: BandStatus, to: BandStatus) -> Result<BandStatus, NfcError> {
    let allowed = matches!(
        (from, to),
        (BandStatus::Inventory, BandStatus::Provisioning)
            | (BandStatus::Provisioning, BandStatus::VerificationPending)
            | (BandStatus::Provisioning, BandStatus::Failed)
            | (BandStatus::VerificationPending, BandStatus::Active)
            | (BandStatus::VerificationPending, BandStatus::Failed)
            | (BandStatus::Active, BandStatus::Frozen)
            | (BandStatus::Active, BandStatus::ReplacementPending)
            | (BandStatus::Active, BandStatus::Retired)
            | (BandStatus::Frozen, BandStatus::Active)
            | (BandStatus::Frozen, BandStatus::ReplacementPending)
            | (BandStatus::Frozen, BandStatus::Retired)
            | (BandStatus::ReplacementPending, BandStatus::Retired)
    );
    if allowed {
        Ok(to)
    } else {
        Err(NfcError::InvalidBandTransition { from, to })
    }
}

/// Provenance classification required on Tier-1 emergency triage fields (`07-nfc-bands-and-emergency.md`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmergencyFieldProvenance {
    Unknown,
    NotRecorded,
    NoneReported,
    ClinicianVerified {
        hospital_id: String,
        verified_at_unix: u64,
        value: String,
    },
}

/// Strictly bounded Tier-1 public emergency triage response (`AGENTS.md` Invariant #2).
///
/// Contains ONLY `blood_type`, `critical_allergies`, and `ice_contact` with provenance.
/// Never includes chronic alerts, past encounters, diagnoses, or medications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmergencyTriageSummary {
    pub blood_type: EmergencyFieldProvenance,
    pub critical_allergies: EmergencyFieldProvenance,
    pub ice_contact: EmergencyFieldProvenance,
    pub offline_summary_updated_at_unix: u64,
}

/// Verified dual-record NDEF payload budgeted against the 256-byte NTAG 424 DNA file limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NdefDualRecordBudget {
    pub record1_online_uri: String,
    pub record2_offline_text: String,
    pub total_ndef_file_bytes: usize,
}

impl NdefDualRecordBudget {
    /// Budgets Record 1 (HTTPS tap URL with 32-char `d` and 16-char `c` placeholders) and
    /// Record 2 (`BloodType|Allergies|ICE_Phone|UpdatedIndicator`) within the 256-byte NTAG 424 DNA NDEF file.
    ///
    /// Returns `Err(NfcError::NdefBudgetExceeded)` rather than silently truncating critical allergies.
    pub fn build_and_verify(
        domain: &str,
        blood_type: &str,
        critical_allergies: &str,
        ice_phone: &str,
        updated_indicator: &str,
    ) -> Result<Self, NfcError> {
        for field in [domain, blood_type, critical_allergies, ice_phone, updated_indicator] {
            let trimmed = field.trim();
            if trimmed.is_empty() || trimmed.contains('|') {
                return Err(NfcError::InvalidOfflineFieldFormat);
            }
        }

        // Record 1 uses fixed-length hex placeholders (32 hex chars for PICCData, 16 hex chars for CMAC).
        let record1_online_uri = format!(
            "https://{}/v1/tap?d={}&c={}",
            domain.trim(),
            "0".repeat(32),
            "0".repeat(16)
        );
        let record2_offline_text = format!(
            "{}|{}|{}|{}",
            blood_type.trim(),
            critical_allergies.trim(),
            ice_phone.trim(),
            updated_indicator.trim()
        );

        let record2_text_bytes = record2_offline_text.len();
        if record2_text_bytes > MAX_OFFLINE_RECORD2_TEXT_BYTES {
            return Err(NfcError::NdefBudgetExceeded {
                required_bytes: record2_text_bytes,
                max_bytes: MAX_OFFLINE_RECORD2_TEXT_BYTES,
            });
        }

        // NTAG 424 DNA NDEF file overhead calculation:
        // - 2 bytes NLEN file header
        // - Record 1 (Short Record Well-Known URI 'U'): 4-byte header + 1-byte URI prefix code + URI body (minus "https://")
        // - Record 2 (Short Record Well-Known Text 'T'): 4-byte header + 1-byte status + 2-byte 'en' lang + UTF-8 text bytes
        let uri_without_scheme_len = record1_online_uri
            .strip_prefix("https://")
            .map_or(record1_online_uri.len(), str::len);
        let record1_bytes = 4 + 1 + uri_without_scheme_len;
        let record2_bytes = 4 + 1 + 2 + record2_text_bytes;
        let total_ndef_file_bytes = 2 + record1_bytes + record2_bytes;

        if total_ndef_file_bytes > NTAG424_NDEF_FILE_CAPACITY_BYTES {
            return Err(NfcError::NdefBudgetExceeded {
                required_bytes: total_ndef_file_bytes,
                max_bytes: NTAG424_NDEF_FILE_CAPACITY_BYTES,
            });
        }

        Ok(Self {
            record1_online_uri,
            record2_offline_text,
            total_ndef_file_bytes,
        })
    }
}

/// Durable Vault A band record acting as the primary replay acceptance authority (`08-replay-and-clinical-authorization.md`, `20-state-machines-and-consistency.md`).
///
/// Redis (`cached_redis_counter`) is an optional fast-path cache that is reconstructed from
/// `durable_high_water_counter` whenever evicted or restarted, so cache loss never authorizes a replay.
pub struct DurableBandReplayState {
    blind_patient_id: BlindPatientId,
    tag_keys: Ntag424TagKeys,
    band_status: BandStatus,
    durable_high_water_counter: u32,
    cached_redis_counter: Option<u32>,
}

impl DurableBandReplayState {
    pub fn new(
        blind_patient_id: BlindPatientId,
        tag_keys: Ntag424TagKeys,
        band_status: BandStatus,
        durable_high_water_counter: u32,
    ) -> Self {
        Self {
            blind_patient_id,
            tag_keys,
            band_status,
            durable_high_water_counter,
            cached_redis_counter: Some(durable_high_water_counter),
        }
    }

    pub fn band_status(&self) -> BandStatus {
        self.band_status
    }

    pub fn durable_high_water_counter(&self) -> u32 {
        self.durable_high_water_counter
    }

    pub fn cached_redis_counter(&self) -> Option<u32> {
        self.cached_redis_counter
    }

    /// Simulates a Redis cache eviction, restart, or failover (`08-replay-and-clinical-authorization.md`).
    pub fn simulate_redis_eviction(&mut self) {
        self.cached_redis_counter = None;
    }

    /// Transitions band status using the verified state machine (`07-nfc-bands-and-emergency.md`).
    pub fn transition_status(&mut self, next_status: BandStatus) -> Result<BandStatus, NfcError> {
        let updated = validate_band_status_transition(self.band_status, next_status)?;
        self.band_status = updated;
        Ok(updated)
    }

    /// Verifies the NTAG 424 DNA SUN tap proof, enforces `BandStatus::Active`, atomically advances
    /// the durable Vault A high-water counter, resynchronizes the Redis cache counter, and issues
    /// a short-lived `AcceptedTapContext` for the subsequent PIN handshake.
    #[allow(clippy::too_many_arguments)]
    pub fn verify_and_accept_tap(
        &mut self,
        picc_data_hex: &str,
        cmac_hex: &str,
        authenticated_ascii_prefix: &[u8],
        context_id: String,
        hospital_id: String,
        doctor_actor_id: String,
        issued_at_unix: u64,
        ttl_secs: u64,
    ) -> Result<AcceptedTapContext, NfcError> {
        if self.band_status != BandStatus::Active {
            return Err(NfcError::BandNotActive(self.band_status));
        }

        let verified = self.tag_keys.verify_sun_tap(
            picc_data_hex,
            cmac_hex,
            authenticated_ascii_prefix,
        )?;

        // Vault A durable state is the authoritative high-water mark, even if Redis was evicted.
        let effective_high_water = self
            .cached_redis_counter
            .map_or(self.durable_high_water_counter, |cached| {
                cached.max(self.durable_high_water_counter)
            });

        if verified.read_counter <= effective_high_water {
            return Err(NfcError::ReplayRejected {
                incoming: verified.read_counter,
                high_water: effective_high_water,
            });
        }

        // Commit durable high-water counter in Vault A first, then update Redis cache.
        self.durable_high_water_counter = verified.read_counter;
        self.cached_redis_counter = Some(verified.read_counter);

        AcceptedTapContext::new(
            context_id,
            self.blind_patient_id.clone(),
            verified.uid,
            verified.read_counter,
            self.band_status,
            hospital_id,
            doctor_actor_id,
            issued_at_unix,
            ttl_secs,
        )
        .map_err(|_| NfcError::BandNotActive(self.band_status))
    }
}
