//! Split-vault isolation, transaction-boundary capability enforcement, idempotency,
//! optimistic resource versioning, and PostgreSQL repository transactions (`20-state-machines-and-consistency.md`).

use crate::{
    auth::{AcceptedTapContext, AuthError, BandStatus, TransactionBoundaryState, VerifiedClinicSession},
    crypto::{BlindPatientId, CryptoError, EncryptedEnvelope, ResourcePurpose},
    nfc::NfcError,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use subtle::ConstantTimeEq;
use thiserror::Error;

/// Typed errors for vault capability, idempotency, optimistic concurrency, and SQL persistence.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum VaultError {
    #[error("clinical capability check failed: {0}")]
    Authorization(#[from] AuthError),
    #[error("cryptographic identifier or envelope error: {0}")]
    Crypto(#[from] CryptoError),
    #[error("NFC replay or band status error: {0}")]
    Nfc(#[from] NfcError),
    #[error("idempotency key must not be empty")]
    EmptyIdempotencyKey,
    #[error("idempotency key was reused with a different request payload or scope")]
    IdempotencyPayloadMismatch,
    #[error("stale resource version: expected {expected}, actual {actual}")]
    StaleResourceVersion { expected: u64, actual: u64 },
    #[error("audit event action type must not be empty")]
    EmptyAuditAction,
    #[error("requested record was not found in the vault")]
    RecordNotFound,
    #[error("database transaction or query failed")]
    DatabaseOperationFailed,
}

/// A connection pool exclusively for Vault A (Identity PII, hospital memberships, and durable NFC counter state).
///
/// Clinical repositories must never accept this type.
#[derive(Clone)]
pub struct IdentityPool(PgPool);

/// A connection pool exclusively for Vault B (Encrypted clinical EHR and immutable audit logs).
///
/// The inner `PgPool` is private and can only be borrowed through `authorize_clinical_access`
/// with a live-revalidated `&VerifiedClinicSession`.
#[derive(Clone)]
pub struct ClinicalPool(PgPool);

impl IdentityPool {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map(Self)
    }

    pub fn pool(&self) -> &PgPool {
        &self.0
    }

    /// Atomically verifies `band_status == 'active'` and `incoming_counter > high_water_counter`
    /// inside a Vault A PostgreSQL transaction (`SELECT ... FOR UPDATE`), advances the durable
    /// high-water counter, and inserts an `accepted_tap_contexts` row (`08-replay-and-clinical-authorization.md`).
    #[allow(clippy::too_many_arguments)]
    pub async fn accept_tap_proof_durable(
        &self,
        tag_uid: [u8; 7],
        incoming_counter: u32,
        context_id: &str,
        hospital_id: &str,
        doctor_actor_id: &str,
        issued_at_unix: u64,
        ttl_secs: u64,
    ) -> Result<AcceptedTapContext, VaultError> {
        let tag_uid_hex = hex::encode(tag_uid);
        let mut tx = self
            .0
            .begin()
            .await
            .map_err(|_| VaultError::DatabaseOperationFailed)?;

        let row = sqlx::query(
            "SELECT blind_patient_id, band_status, high_water_counter \
             FROM nfc_bands WHERE tag_uid_hex = $1 FOR UPDATE",
        )
        .bind(&tag_uid_hex)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| VaultError::DatabaseOperationFailed)?
        .ok_or(VaultError::RecordNotFound)?;

        let blind_id_str: String = row
            .try_get("blind_patient_id")
            .map_err(|_| VaultError::DatabaseOperationFailed)?;
        let status_str: String = row
            .try_get("band_status")
            .map_err(|_| VaultError::DatabaseOperationFailed)?;
        let high_water_i64: i64 = row
            .try_get("high_water_counter")
            .map_err(|_| VaultError::DatabaseOperationFailed)?;

        let blind_patient_id = BlindPatientId::from_canonical_str(&blind_id_str)?;
        let band_status = parse_band_status(&status_str)?;
        if band_status != BandStatus::Active {
            return Err(VaultError::Nfc(NfcError::BandNotActive(band_status)));
        }

        let high_water = u32::try_from(high_water_i64).unwrap_or(u32::MAX);
        if incoming_counter <= high_water {
            return Err(VaultError::Nfc(NfcError::ReplayRejected {
                incoming: incoming_counter,
                high_water,
            }));
        }

        let incoming_i64 = i64::from(incoming_counter);
        let issued_i64 = i64::try_from(issued_at_unix).map_err(|_| VaultError::DatabaseOperationFailed)?;
        let expires_at_unix = issued_at_unix.saturating_add(ttl_secs);
        let expires_i64 =
            i64::try_from(expires_at_unix).map_err(|_| VaultError::DatabaseOperationFailed)?;

        sqlx::query(
            "UPDATE nfc_bands \
             SET high_water_counter = $1, resource_version = resource_version + 1, updated_at_unix = $2 \
             WHERE tag_uid_hex = $3",
        )
        .bind(incoming_i64)
        .bind(issued_i64)
        .bind(&tag_uid_hex)
        .execute(&mut *tx)
        .await
        .map_err(|_| VaultError::DatabaseOperationFailed)?;

        sqlx::query(
            "INSERT INTO accepted_tap_contexts \
             (context_id, blind_patient_id, tag_uid_hex, accepted_counter, hospital_id, doctor_actor_id, state, failed_pin_attempts, issued_at_unix, expires_at_unix) \
             VALUES ($1, $2, $3, $4, $5, $6, 'proof_accepted', 0, $7, $8)",
        )
        .bind(context_id)
        .bind(&blind_id_str)
        .bind(&tag_uid_hex)
        .bind(incoming_i64)
        .bind(hospital_id)
        .bind(doctor_actor_id)
        .bind(issued_i64)
        .bind(expires_i64)
        .execute(&mut *tx)
        .await
        .map_err(|_| VaultError::DatabaseOperationFailed)?;

        tx.commit()
            .await
            .map_err(|_| VaultError::DatabaseOperationFailed)?;

        Ok(AcceptedTapContext::new(
            context_id.to_string(),
            blind_patient_id,
            tag_uid,
            incoming_counter,
            band_status,
            hospital_id.to_string(),
            doctor_actor_id.to_string(),
            issued_at_unix,
            ttl_secs,
        )?)
    }

    pub async fn close(&self) {
        self.0.close().await;
    }
}

fn parse_band_status(status: &str) -> Result<BandStatus, VaultError> {
    match status {
        "inventory" => Ok(BandStatus::Inventory),
        "provisioning" => Ok(BandStatus::Provisioning),
        "verification_pending" => Ok(BandStatus::VerificationPending),
        "active" => Ok(BandStatus::Active),
        "frozen" => Ok(BandStatus::Frozen),
        "replacement_pending" => Ok(BandStatus::ReplacementPending),
        "retired" => Ok(BandStatus::Retired),
        "failed" => Ok(BandStatus::Failed),
        _ => Err(VaultError::DatabaseOperationFailed),
    }
}

/// Stored encrypted clinical record metadata and envelope in Vault B.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredClinicalRecord {
    pub record_id: String,
    pub blind_patient_id: BlindPatientId,
    pub hospital_id: String,
    pub purpose: ResourcePurpose,
    pub schema_version: u16,
    pub envelope: EncryptedEnvelope,
    pub resource_version: ResourceVersion,
    pub updated_at_unix: u64,
}

/// Guard proving that a `VerifiedClinicSession` passed transaction-boundary revalidation
/// and that an immutable audit record has been bound to the operation.
pub struct AuthorizedClinicalAccess<'a> {
    pool: &'a PgPool,
    audit_record: ClinicalAuditRecord,
}

impl<'a> AuthorizedClinicalAccess<'a> {
    pub fn pool(&self) -> &'a PgPool {
        self.pool
    }

    pub fn audit_record(&self) -> &ClinicalAuditRecord {
        &self.audit_record
    }

    /// Inserts the mandatory `immutable_audit_logs` entry and a new `clinical_encrypted_records`
    /// row inside a single Vault B transaction. Fails closed if the audit write fails.
    pub async fn insert_encrypted_record_with_audit(
        &self,
        record_id: &str,
        purpose: ResourcePurpose,
        schema_version: u16,
        envelope: &EncryptedEnvelope,
    ) -> Result<ResourceVersion, VaultError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| VaultError::DatabaseOperationFailed)?;

        let recorded_at_i64 = i64::try_from(self.audit_record.recorded_at_unix)
            .map_err(|_| VaultError::DatabaseOperationFailed)?;

        sqlx::query(
            "INSERT INTO immutable_audit_logs \
             (blind_patient_id, doctor_actor_id, hospital_id, session_reference, action_type, recorded_at_unix) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&self.audit_record.blind_patient_id)
        .bind(&self.audit_record.doctor_actor_id)
        .bind(&self.audit_record.hospital_id)
        .bind(&self.audit_record.session_reference)
        .bind(&self.audit_record.action_type)
        .bind(recorded_at_i64)
        .execute(&mut *tx)
        .await
        .map_err(|_| VaultError::DatabaseOperationFailed)?;

        sqlx::query(
            "INSERT INTO clinical_encrypted_records \
             (record_id, blind_patient_id, hospital_id, resource_purpose, schema_version, envelope_version, key_id, nonce, ciphertext_with_tag, resource_version, updated_at_unix) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 1, $10)",
        )
        .bind(record_id)
        .bind(&self.audit_record.blind_patient_id)
        .bind(&self.audit_record.hospital_id)
        .bind(purpose.as_str())
        .bind(i32::from(schema_version))
        .bind(i32::from(envelope.envelope_version))
        .bind(&envelope.key_id)
        .bind(&envelope.nonce[..])
        .bind(&envelope.ciphertext_with_tag)
        .bind(recorded_at_i64)
        .execute(&mut *tx)
        .await
        .map_err(|_| VaultError::DatabaseOperationFailed)?;

        tx.commit()
            .await
            .map_err(|_| VaultError::DatabaseOperationFailed)?;

        Ok(ResourceVersion::INITIAL)
    }

    /// Inserts the mandatory `immutable_audit_logs` entry and reads an encrypted clinical record
    /// scoped to the session's `blind_patient_id` inside a single Vault B transaction.
    pub async fn fetch_encrypted_record_with_audit(
        &self,
        record_id: &str,
    ) -> Result<(EncryptedEnvelope, ResourceVersion), VaultError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| VaultError::DatabaseOperationFailed)?;

        let recorded_at_i64 = i64::try_from(self.audit_record.recorded_at_unix)
            .map_err(|_| VaultError::DatabaseOperationFailed)?;

        sqlx::query(
            "INSERT INTO immutable_audit_logs \
             (blind_patient_id, doctor_actor_id, hospital_id, session_reference, action_type, recorded_at_unix) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&self.audit_record.blind_patient_id)
        .bind(&self.audit_record.doctor_actor_id)
        .bind(&self.audit_record.hospital_id)
        .bind(&self.audit_record.session_reference)
        .bind(&self.audit_record.action_type)
        .bind(recorded_at_i64)
        .execute(&mut *tx)
        .await
        .map_err(|_| VaultError::DatabaseOperationFailed)?;

        let row = sqlx::query(
            "SELECT envelope_version, key_id, nonce, ciphertext_with_tag, resource_version \
             FROM clinical_encrypted_records \
             WHERE record_id = $1 AND blind_patient_id = $2",
        )
        .bind(record_id)
        .bind(&self.audit_record.blind_patient_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| VaultError::DatabaseOperationFailed)?
        .ok_or(VaultError::RecordNotFound)?;

        let envelope_version_i32: i32 = row
            .try_get("envelope_version")
            .map_err(|_| VaultError::DatabaseOperationFailed)?;
        let key_id: String = row
            .try_get("key_id")
            .map_err(|_| VaultError::DatabaseOperationFailed)?;
        let nonce_vec: Vec<u8> = row
            .try_get("nonce")
            .map_err(|_| VaultError::DatabaseOperationFailed)?;
        let ciphertext_with_tag: Vec<u8> = row
            .try_get("ciphertext_with_tag")
            .map_err(|_| VaultError::DatabaseOperationFailed)?;
        let resource_version_i64: i64 = row
            .try_get("resource_version")
            .map_err(|_| VaultError::DatabaseOperationFailed)?;

        let nonce: [u8; 12] = nonce_vec
            .try_into()
            .map_err(|_| VaultError::DatabaseOperationFailed)?;
        let envelope_version = u16::try_from(envelope_version_i32)
            .map_err(|_| VaultError::DatabaseOperationFailed)?;
        let resource_version = u64::try_from(resource_version_i64)
            .map_err(|_| VaultError::DatabaseOperationFailed)?;

        tx.commit()
            .await
            .map_err(|_| VaultError::DatabaseOperationFailed)?;

        Ok((
            EncryptedEnvelope {
                envelope_version,
                key_id,
                nonce,
                ciphertext_with_tag,
            },
            ResourceVersion::new(resource_version),
        ))
    }
}

/// Immutable audit record required for every Tier-2 clinical read or write (`AGENTS.md`, `24-observability-audit-and-incidents.md`).
///
/// Stores `session_reference` rather than raw tokens or PIN secrets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClinicalAuditRecord {
    pub blind_patient_id: String,
    pub doctor_actor_id: String,
    pub hospital_id: String,
    pub session_reference: String,
    pub action_type: String,
    pub recorded_at_unix: u64,
}

impl ClinicalPool {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map(Self)
    }

    /// Enforces compile-time and runtime transaction-boundary revalidation before exposing
    /// `AuthorizedClinicalAccess` for Vault B operations.
    pub fn authorize_clinical_access<'a>(
        &'a self,
        session: &VerifiedClinicSession,
        target_patient: &BlindPatientId,
        target_hospital_id: &str,
        live_state: &TransactionBoundaryState,
        action_type: &str,
    ) -> Result<AuthorizedClinicalAccess<'a>, VaultError> {
        let normalized_action = action_type.trim();
        if normalized_action.is_empty() {
            return Err(VaultError::EmptyAuditAction);
        }

        session.revalidate_at_transaction_boundary(
            target_patient,
            target_hospital_id,
            live_state,
        )?;

        let audit_record = ClinicalAuditRecord {
            blind_patient_id: target_patient.to_canonical_string(),
            doctor_actor_id: session.doctor_actor_id().to_string(),
            hospital_id: session.hospital_id().to_string(),
            session_reference: session.session_reference().to_string(),
            action_type: normalized_action.to_string(),
            recorded_at_unix: live_state.current_unix_secs,
        };

        Ok(AuthorizedClinicalAccess {
            pool: &self.0,
            audit_record,
        })
    }

    pub async fn close(&self) {
        self.0.close().await;
    }
}

/// Scoped idempotency fingerprint binding `(actor_id, hospital_id, operation, idempotency_key)` to a SHA-256 payload digest (`20-state-machines-and-consistency.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedIdempotencyRecord {
    pub actor_id: String,
    pub hospital_id: String,
    pub operation: String,
    pub idempotency_key: String,
    pub payload_sha256: [u8; 32],
}

impl ScopedIdempotencyRecord {
    pub fn new(
        actor_id: impl Into<String>,
        hospital_id: impl Into<String>,
        operation: impl Into<String>,
        idempotency_key: impl Into<String>,
        canonical_payload: &[u8],
    ) -> Result<Self, VaultError> {
        let key = idempotency_key.into();
        if key.trim().is_empty() {
            return Err(VaultError::EmptyIdempotencyKey);
        }
        let digest = Sha256::digest(canonical_payload);
        let mut payload_sha256 = [0u8; 32];
        payload_sha256.copy_from_slice(&digest);
        Ok(Self {
            actor_id: actor_id.into(),
            hospital_id: hospital_id.into(),
            operation: operation.into(),
            idempotency_key: key,
            payload_sha256,
        })
    }

    /// Verifies that a retried request with the same idempotency key has the exact same scope and payload.
    pub fn verify_retry(&self, candidate: &Self) -> Result<(), VaultError> {
        let same_scope = self.actor_id == candidate.actor_id
            && self.hospital_id == candidate.hospital_id
            && self.operation == candidate.operation
            && self.idempotency_key == candidate.idempotency_key;
        let same_payload: bool = self.payload_sha256.ct_eq(&candidate.payload_sha256).into();
        if same_scope && same_payload {
            Ok(())
        } else {
            Err(VaultError::IdempotencyPayloadMismatch)
        }
    }
}

/// Optimistic concurrency version (`ETag` / resource version) required for mutating clinical or band state (`20-state-machines-and-consistency.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceVersion(u64);

impl ResourceVersion {
    pub const INITIAL: Self = Self(1);

    pub const fn new(version: u64) -> Self {
        Self(version)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub fn verify_and_advance(self, expected_by_caller: Self) -> Result<Self, VaultError> {
        if self.0 != expected_by_caller.0 {
            return Err(VaultError::StaleResourceVersion {
                expected: expected_by_caller.0,
                actual: self.0,
            });
        }
        Ok(Self(self.0.saturating_add(1)))
    }
}

/// Delivery lifecycle state for cross-vault outbox/inbox coordination (`20-state-machines-and-consistency.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxDeliveryState {
    Pending,
    Leased,
    Delivered,
    RetryableFailure,
    DeadLettered,
}
