//! Split-vault isolation, transaction-boundary capability enforcement, idempotency,
//! optimistic resource versioning, and cross-vault outbox contracts (`20-state-machines-and-consistency.md`).

use crate::{
    auth::{AuthError, TransactionBoundaryState, VerifiedClinicSession},
    crypto::BlindPatientId,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, postgres::PgPoolOptions};
use subtle::ConstantTimeEq;
use thiserror::Error;

/// Typed errors for vault capability, idempotency, and optimistic concurrency enforcement.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum VaultError {
    #[error("clinical capability check failed: {0}")]
    Authorization(#[from] AuthError),
    #[error("idempotency key must not be empty")]
    EmptyIdempotencyKey,
    #[error("idempotency key was reused with a different request payload or scope")]
    IdempotencyPayloadMismatch,
    #[error("stale resource version: expected {expected}, actual {actual}")]
    StaleResourceVersion { expected: u64, actual: u64 },
    #[error("audit event action type must not be empty")]
    EmptyAuditAction,
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

    pub async fn close(&self) {
        self.0.close().await;
    }
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
