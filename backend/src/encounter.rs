//! Complete authorized clinical encounter lifecycle (`I08`), cross-hospital longitudinal
//! sharing boundaries (`18-patient-sharing-and-clinical-safety.md`), audit-outage fail-safe
//! and minimized privacy auditor views (`I07`, `10-administration-audit-notifications.md`,
//! `24-observability-audit-and-incidents.md`), and bounded outbox event delivery (`20-state-machines-and-consistency.md`).

use crate::{
    auth::{
        AuthError, AuthenticatedActor, ProtectedAction, TransactionBoundaryState,
        VerifiedClinicSession, authorize_actor_action,
    },
    crypto::{
        BlindPatientId, CryptoError, EncryptedEnvelope, EnvelopeAad, ResourcePurpose,
        VaultCipherRing,
    },
    vault::{ClinicalAuditRecord, OutboxDeliveryState, ResourceVersion, VaultError},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Clinical encounter lifecycle state (`09-clinical-workspace-and-vaults.md`, `20-state-machines-and-consistency.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClinicalEncounterState {
    Draft,
    Reviewed,
    Finalized,
    Amended,
    EnteredInError,
}

/// Runtime health of the durable Vault B audit store (`24-observability-audit-and-incidents.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditStorageHealth {
    Healthy,
    Outage,
}

/// Longitudinal cross-hospital sharing policy for a patient's clinical record (`18-patient-sharing-and-clinical-safety.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HospitalSharingConsent {
    /// Only the authoring source hospital may read or amend this record.
    SourceHospitalOnly,
    /// Patient has explicitly authorized specific partner hospitals to view longitudinal records.
    ExplicitHospitalList(Vec<String>),
}

impl HospitalSharingConsent {
    pub fn permits_hospital(&self, source_hospital_id: &str, requesting_hospital_id: &str) -> bool {
        if source_hospital_id == requesting_hospital_id {
            return true;
        }
        match self {
            Self::SourceHospitalOnly => false,
            Self::ExplicitHospitalList(allowed) => {
                allowed.iter().any(|h| h == requesting_hospital_id)
            }
        }
    }
}

/// Typed errors for clinical encounter operations, audit-outage fail-safe, and outbox delivery.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EncounterError {
    #[error("authorization failed: {0}")]
    Auth(#[from] AuthError),
    #[error("vault capability or concurrency check failed: {0}")]
    Vault(#[from] VaultError),
    #[error("cryptographic envelope operation failed: {0}")]
    Crypto(#[from] CryptoError),
    #[error("durable audit storage is unavailable; clinical reads and writes fail closed")]
    AuditStorageUnavailable,
    #[error("cross-hospital longitudinal record access is not authorized for this encounter")]
    CrossHospitalRecordNotShared,
    #[error("invalid clinical encounter state transition from {from:?} to {to:?}")]
    InvalidEncounterStateTransition {
        from: ClinicalEncounterState,
        to: ClinicalEncounterState,
    },
    #[error("amendment or error-entry reason must not be empty")]
    EmptyAmendmentReason,
    #[error("outbox event transition is invalid from current state {0:?}")]
    InvalidOutboxTransition(OutboxDeliveryState),
}

/// Single immutable encrypted revision in an encounter's clinical history (`09-clinical-workspace-and-vaults.md`).
#[derive(Debug, Clone)]
pub struct EncryptedEncounterRevision {
    pub version: ResourceVersion,
    pub state_at_revision: ClinicalEncounterState,
    pub author_doctor_id: String,
    pub source_hospital_id: String,
    pub recorded_at_unix: u64,
    pub amendment_reason: Option<String>,
    pub encrypted_note: EncryptedEnvelope,
}

/// Vault B clinical encounter aggregate enforcing compile-time `&VerifiedClinicSession`,
/// optimistic concurrency (`ResourceVersion`), audit-outage fail-safe, and traceable amendments.
#[derive(Debug, Clone)]
pub struct ClinicalEncounterRecord {
    encounter_id: String,
    blind_patient_id: BlindPatientId,
    source_hospital_id: String,
    current_state: ClinicalEncounterState,
    current_version: ResourceVersion,
    revisions: Vec<EncryptedEncounterRevision>,
    audit_trail: Vec<ClinicalAuditRecord>,
}

impl ClinicalEncounterRecord {
    /// Creates an initial `Draft` encounter inside an active `VerifiedClinicSession`.
    /// Fails closed if `audit_health == AuditStorageHealth::Outage`.
    #[allow(clippy::too_many_arguments)]
    pub fn create_draft(
        session: &VerifiedClinicSession,
        live_state: &TransactionBoundaryState,
        audit_health: AuditStorageHealth,
        cipher_ring: &mut VaultCipherRing,
        encounter_id: String,
        blind_patient_id: BlindPatientId,
        hospital_id: &str,
        note_plaintext: &[u8],
    ) -> Result<Self, EncounterError> {
        if audit_health == AuditStorageHealth::Outage {
            return Err(EncounterError::AuditStorageUnavailable);
        }
        session.revalidate_at_transaction_boundary(&blind_patient_id, hospital_id, live_state)?;

        let version = ResourceVersion::INITIAL;
        let aad = EnvelopeAad::new(
            blind_patient_id.clone(),
            ResourcePurpose::EncounterNote,
            u16::try_from(version.get()).unwrap_or(u16::MAX),
        )?;
        let encrypted_note = cipher_ring.encrypt(note_plaintext, &aad)?;

        let revision = EncryptedEncounterRevision {
            version,
            state_at_revision: ClinicalEncounterState::Draft,
            author_doctor_id: session.doctor_actor_id().to_string(),
            source_hospital_id: hospital_id.to_string(),
            recorded_at_unix: live_state.current_unix_secs,
            amendment_reason: None,
            encrypted_note,
        };

        let audit_entry = ClinicalAuditRecord {
            blind_patient_id: blind_patient_id.to_canonical_string(),
            doctor_actor_id: session.doctor_actor_id().to_string(),
            hospital_id: hospital_id.to_string(),
            session_reference: session.session_reference().to_string(),
            action_type: "encounter.create_draft".to_string(),
            recorded_at_unix: live_state.current_unix_secs,
        };

        Ok(Self {
            encounter_id,
            blind_patient_id,
            source_hospital_id: hospital_id.to_string(),
            current_state: ClinicalEncounterState::Draft,
            current_version: version,
            revisions: vec![revision],
            audit_trail: vec![audit_entry],
        })
    }

    pub fn encounter_id(&self) -> &str {
        &self.encounter_id
    }

    pub fn blind_patient_id(&self) -> &BlindPatientId {
        &self.blind_patient_id
    }

    pub fn source_hospital_id(&self) -> &str {
        &self.source_hospital_id
    }

    pub fn current_state(&self) -> ClinicalEncounterState {
        self.current_state
    }

    pub fn current_version(&self) -> ResourceVersion {
        self.current_version
    }

    pub fn revisions(&self) -> &[EncryptedEncounterRevision] {
        &self.revisions
    }

    pub fn audit_trail(&self) -> &[ClinicalAuditRecord] {
        &self.audit_trail
    }

    /// Saves an updated `Draft` or transitions `Draft -> Reviewed` using optimistic concurrency.
    #[allow(clippy::too_many_arguments)]
    pub fn update_draft_or_review(
        &mut self,
        session: &VerifiedClinicSession,
        live_state: &TransactionBoundaryState,
        audit_health: AuditStorageHealth,
        cipher_ring: &mut VaultCipherRing,
        expected_version: ResourceVersion,
        mark_reviewed: bool,
        note_plaintext: &[u8],
    ) -> Result<ResourceVersion, EncounterError> {
        if audit_health == AuditStorageHealth::Outage {
            return Err(EncounterError::AuditStorageUnavailable);
        }
        session.revalidate_at_transaction_boundary(
            &self.blind_patient_id,
            &self.source_hospital_id,
            live_state,
        )?;

        let next_state = if mark_reviewed {
            ClinicalEncounterState::Reviewed
        } else {
            ClinicalEncounterState::Draft
        };
        if !matches!(
            self.current_state,
            ClinicalEncounterState::Draft | ClinicalEncounterState::Reviewed
        ) {
            return Err(EncounterError::InvalidEncounterStateTransition {
                from: self.current_state,
                to: next_state,
            });
        }

        let next_version = self.current_version.verify_and_advance(expected_version)?;
        let aad = EnvelopeAad::new(
            self.blind_patient_id.clone(),
            ResourcePurpose::EncounterNote,
            u16::try_from(next_version.get()).unwrap_or(u16::MAX),
        )?;
        let encrypted_note = cipher_ring.encrypt(note_plaintext, &aad)?;

        self.current_state = next_state;
        self.current_version = next_version;
        self.revisions.push(EncryptedEncounterRevision {
            version: next_version,
            state_at_revision: next_state,
            author_doctor_id: session.doctor_actor_id().to_string(),
            source_hospital_id: self.source_hospital_id.clone(),
            recorded_at_unix: live_state.current_unix_secs,
            amendment_reason: None,
            encrypted_note,
        });
        self.audit_trail.push(ClinicalAuditRecord {
            blind_patient_id: self.blind_patient_id.to_canonical_string(),
            doctor_actor_id: session.doctor_actor_id().to_string(),
            hospital_id: session.hospital_id().to_string(),
            session_reference: session.session_reference().to_string(),
            action_type: if mark_reviewed {
                "encounter.review".to_string()
            } else {
                "encounter.save_draft".to_string()
            },
            recorded_at_unix: live_state.current_unix_secs,
        });

        Ok(next_version)
    }

    /// Explicitly finalizes the encounter (`Draft`/`Reviewed` -> `Finalized`).
    /// Once finalized, the encounter cannot be overwritten in-place; changes require `amend_finalized`.
    pub fn finalize(
        &mut self,
        session: &VerifiedClinicSession,
        live_state: &TransactionBoundaryState,
        audit_health: AuditStorageHealth,
        cipher_ring: &mut VaultCipherRing,
        expected_version: ResourceVersion,
        final_note_plaintext: &[u8],
    ) -> Result<ResourceVersion, EncounterError> {
        if audit_health == AuditStorageHealth::Outage {
            return Err(EncounterError::AuditStorageUnavailable);
        }
        session.revalidate_at_transaction_boundary(
            &self.blind_patient_id,
            &self.source_hospital_id,
            live_state,
        )?;

        if !matches!(
            self.current_state,
            ClinicalEncounterState::Draft | ClinicalEncounterState::Reviewed
        ) {
            return Err(EncounterError::InvalidEncounterStateTransition {
                from: self.current_state,
                to: ClinicalEncounterState::Finalized,
            });
        }

        let next_version = self.current_version.verify_and_advance(expected_version)?;
        let aad = EnvelopeAad::new(
            self.blind_patient_id.clone(),
            ResourcePurpose::EncounterNote,
            u16::try_from(next_version.get()).unwrap_or(u16::MAX),
        )?;
        let encrypted_note = cipher_ring.encrypt(final_note_plaintext, &aad)?;

        self.current_state = ClinicalEncounterState::Finalized;
        self.current_version = next_version;
        self.revisions.push(EncryptedEncounterRevision {
            version: next_version,
            state_at_revision: ClinicalEncounterState::Finalized,
            author_doctor_id: session.doctor_actor_id().to_string(),
            source_hospital_id: self.source_hospital_id.clone(),
            recorded_at_unix: live_state.current_unix_secs,
            amendment_reason: None,
            encrypted_note,
        });
        self.audit_trail.push(ClinicalAuditRecord {
            blind_patient_id: self.blind_patient_id.to_canonical_string(),
            doctor_actor_id: session.doctor_actor_id().to_string(),
            hospital_id: session.hospital_id().to_string(),
            session_reference: session.session_reference().to_string(),
            action_type: "encounter.finalize".to_string(),
            recorded_at_unix: live_state.current_unix_secs,
        });

        Ok(next_version)
    }

    /// Appends a traceable amendment to a `Finalized` or `Amended` encounter without deleting
    /// prior revisions (`09-clinical-workspace-and-vaults.md`, `18-patient-sharing-and-clinical-safety.md`).
    #[allow(clippy::too_many_arguments)]
    pub fn amend_finalized(
        &mut self,
        session: &VerifiedClinicSession,
        live_state: &TransactionBoundaryState,
        audit_health: AuditStorageHealth,
        cipher_ring: &mut VaultCipherRing,
        expected_version: ResourceVersion,
        amendment_reason: &str,
        amended_note_plaintext: &[u8],
    ) -> Result<ResourceVersion, EncounterError> {
        if audit_health == AuditStorageHealth::Outage {
            return Err(EncounterError::AuditStorageUnavailable);
        }
        session.revalidate_at_transaction_boundary(
            &self.blind_patient_id,
            &self.source_hospital_id,
            live_state,
        )?;
        if amendment_reason.trim().is_empty() {
            return Err(EncounterError::EmptyAmendmentReason);
        }
        if !matches!(
            self.current_state,
            ClinicalEncounterState::Finalized | ClinicalEncounterState::Amended
        ) {
            return Err(EncounterError::InvalidEncounterStateTransition {
                from: self.current_state,
                to: ClinicalEncounterState::Amended,
            });
        }

        let next_version = self.current_version.verify_and_advance(expected_version)?;
        let aad = EnvelopeAad::new(
            self.blind_patient_id.clone(),
            ResourcePurpose::EncounterNote,
            u16::try_from(next_version.get()).unwrap_or(u16::MAX),
        )?;
        let encrypted_note = cipher_ring.encrypt(amended_note_plaintext, &aad)?;

        self.current_state = ClinicalEncounterState::Amended;
        self.current_version = next_version;
        self.revisions.push(EncryptedEncounterRevision {
            version: next_version,
            state_at_revision: ClinicalEncounterState::Amended,
            author_doctor_id: session.doctor_actor_id().to_string(),
            source_hospital_id: self.source_hospital_id.clone(),
            recorded_at_unix: live_state.current_unix_secs,
            amendment_reason: Some(amendment_reason.trim().to_string()),
            encrypted_note,
        });
        self.audit_trail.push(ClinicalAuditRecord {
            blind_patient_id: self.blind_patient_id.to_canonical_string(),
            doctor_actor_id: session.doctor_actor_id().to_string(),
            hospital_id: session.hospital_id().to_string(),
            session_reference: session.session_reference().to_string(),
            action_type: "encounter.amend".to_string(),
            recorded_at_unix: live_state.current_unix_secs,
        });

        Ok(next_version)
    }

    /// Reads and decrypts the latest encounter note inside a valid `VerifiedClinicSession`,
    /// enforcing cross-hospital longitudinal sharing rules and appending an immutable audit entry.
    pub fn read_latest_decrypted(
        &mut self,
        session: &VerifiedClinicSession,
        requesting_hospital_id: &str,
        sharing_consent: &HospitalSharingConsent,
        live_state: &TransactionBoundaryState,
        audit_health: AuditStorageHealth,
        cipher_ring: &VaultCipherRing,
    ) -> Result<Vec<u8>, EncounterError> {
        if audit_health == AuditStorageHealth::Outage {
            return Err(EncounterError::AuditStorageUnavailable);
        }
        session.revalidate_at_transaction_boundary(
            &self.blind_patient_id,
            requesting_hospital_id,
            live_state,
        )?;
        if !sharing_consent.permits_hospital(&self.source_hospital_id, requesting_hospital_id) {
            return Err(EncounterError::CrossHospitalRecordNotShared);
        }

        let latest = self
            .revisions
            .last()
            .ok_or(EncounterError::Vault(VaultError::RecordNotFound))?;
        let aad = EnvelopeAad::new(
            self.blind_patient_id.clone(),
            ResourcePurpose::EncounterNote,
            u16::try_from(latest.version.get()).unwrap_or(u16::MAX),
        )?;
        let plaintext = cipher_ring.decrypt(&latest.encrypted_note, &aad)?;

        self.audit_trail.push(ClinicalAuditRecord {
            blind_patient_id: self.blind_patient_id.to_canonical_string(),
            doctor_actor_id: session.doctor_actor_id().to_string(),
            hospital_id: requesting_hospital_id.to_string(),
            session_reference: session.session_reference().to_string(),
            action_type: "encounter.read".to_string(),
            recorded_at_unix: live_state.current_unix_secs,
        });

        Ok(plaintext)
    }

    /// Projects minimized audit metadata for an authorized `Role::PrivacyAuditor` in the hospital
    /// without exposing any clinical ciphertexts, plaintexts, or patient PII (`10-administration-audit-notifications.md`).
    pub fn project_minimized_audit_for_privacy_auditor(
        &self,
        auditor: &AuthenticatedActor,
        hospital_id: &str,
    ) -> Result<Vec<MinimizedAuditEventProjection>, EncounterError> {
        authorize_actor_action(
            auditor,
            Some(hospital_id),
            &ProtectedAction::ReviewAuditEvents,
        )?;
        Ok(self
            .audit_trail
            .iter()
            .filter(|entry| entry.hospital_id == hospital_id)
            .map(|entry| MinimizedAuditEventProjection {
                encounter_id: self.encounter_id.clone(),
                blind_patient_id: entry.blind_patient_id.clone(),
                doctor_actor_id: entry.doctor_actor_id.clone(),
                hospital_id: entry.hospital_id.clone(),
                session_reference: entry.session_reference.clone(),
                action_type: entry.action_type.clone(),
                recorded_at_unix: entry.recorded_at_unix,
            })
            .collect())
    }
}

/// Minimized audit event view exposed to `Role::PrivacyAuditor` (`10-administration-audit-notifications.md`).
/// Never contains clinical note content or Vault A identity PII.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinimizedAuditEventProjection {
    pub encounter_id: String,
    pub blind_patient_id: String,
    pub doctor_actor_id: String,
    pub hospital_id: String,
    pub session_reference: String,
    pub action_type: String,
    pub recorded_at_unix: u64,
}

/// Bounded notification / outbox event state machine (`20-state-machines-and-consistency.md`).
#[derive(Debug, Clone)]
pub struct OutboxEventRecord {
    event_id: String,
    event_type: String,
    state: OutboxDeliveryState,
    attempt_count: u32,
    max_attempts: u32,
    last_error_code: Option<String>,
}

impl OutboxEventRecord {
    pub fn new(event_id: String, event_type: String, max_attempts: u32) -> Self {
        Self {
            event_id,
            event_type,
            state: OutboxDeliveryState::Pending,
            attempt_count: 0,
            max_attempts: max_attempts.max(1),
            last_error_code: None,
        }
    }

    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    pub fn state(&self) -> OutboxDeliveryState {
        self.state
    }

    pub fn attempt_count(&self) -> u32 {
        self.attempt_count
    }

    pub fn last_error_code(&self) -> Option<&str> {
        self.last_error_code.as_deref()
    }

    /// Leases a `Pending` or `RetryableFailure` event for delivery.
    pub fn lease(&mut self) -> Result<OutboxDeliveryState, EncounterError> {
        match self.state {
            OutboxDeliveryState::Pending | OutboxDeliveryState::RetryableFailure => {
                self.attempt_count = self.attempt_count.saturating_add(1);
                self.state = OutboxDeliveryState::Leased;
                Ok(self.state)
            }
            other => Err(EncounterError::InvalidOutboxTransition(other)),
        }
    }

    /// Marks a `Leased` event as `Delivered`.
    pub fn mark_delivered(&mut self) -> Result<OutboxDeliveryState, EncounterError> {
        if self.state != OutboxDeliveryState::Leased {
            return Err(EncounterError::InvalidOutboxTransition(self.state));
        }
        self.state = OutboxDeliveryState::Delivered;
        self.last_error_code = None;
        Ok(self.state)
    }

    /// Records a delivery failure on a `Leased` event, transitioning to `RetryableFailure`
    /// while under `max_attempts` or `DeadLettered` once the retry budget is exhausted.
    pub fn record_failure(&mut self, error_code: &str) -> Result<OutboxDeliveryState, EncounterError> {
        if self.state != OutboxDeliveryState::Leased {
            return Err(EncounterError::InvalidOutboxTransition(self.state));
        }
        self.last_error_code = Some(error_code.trim().to_string());
        if self.attempt_count >= self.max_attempts {
            self.state = OutboxDeliveryState::DeadLettered;
        } else {
            self.state = OutboxDeliveryState::RetryableFailure;
        }
        Ok(self.state)
    }
}
