//! Role-based access control, independent credential review, Argon2id PIN verification,
//! and the compile-time `VerifiedClinicSession` typestate boundary.
//!
//! Enforces:
//! - Default-deny authorization across hospital roles (`04-roles-and-permissions.md`).
//! - Strict separation between staff membership and clinical credential approval, with
//!   explicit rejection of self-approval.
//! - Disabled non-physician care-team (`Nurse`, `LaboratoryStaff`, `PharmacyStaff`) clinical
//!   access in the initial doctor-led pilot scope (`I01-PILOT-BASELINE.md`).
//! - `VerifiedClinicSession` with private fields, mintable only inside `auth::handshake`
//!   after consuming a valid `AcceptedTapContext`, verifying an accredited `Doctor` in the
//!   target hospital, and verifying the patient's 6-digit PIN via Argon2id.
//! - Fixed 15-minute (`900`-second) session duration and transaction-boundary revalidation.

use crate::crypto::BlindPatientId;
use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::SaltString,
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use thiserror::Error;

/// Fixed 15-minute clinical encounter session lifetime in seconds (`AGENTS.md` Invariant #2).
pub const CLINICAL_SESSION_DURATION_SECS: u64 = 15 * 60;

/// Maximum consecutive failed PIN attempts before the tap context is locked out.
pub const MAX_PIN_ATTEMPTS: u8 = 5;

/// Roles defined in `docs/implementation/04-roles-and-permissions.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    PlatformOperator,
    HospitalAdmin,
    CredentialReviewer,
    Doctor,
    Reception,
    BandOfficer,
    PrivacyAuditor,
    Patient,
    Nurse,
    LaboratoryStaff,
    PharmacyStaff,
}

/// Hospital membership lifecycle state (`docs/implementation/20-state-machines-and-consistency.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipStatus {
    Invited,
    AcceptedPendingChecks,
    Active,
    Suspended,
    Expired,
    Terminated,
}

/// Professional credential verification state, kept distinct from hospital membership.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialStatus {
    NotApplicable,
    Pending,
    Approved { reviewer_actor_id: String },
    Suspended,
    Revoked,
}

/// Physical NFC wristband lifecycle state (`07-nfc-bands-and-emergency.md`, `20-state-machines-and-consistency.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BandStatus {
    Inventory,
    Provisioning,
    VerificationPending,
    Active,
    Frozen,
    ReplacementPending,
    Retired,
    Failed,
}

/// Actions evaluated by the server-side default-deny policy engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtectedAction {
    InviteStaff { target_role: Role },
    ApproveClinicalEligibility { subject_actor_id: String },
    RegisterPatientIdentity,
    BindBand,
    FreezeBand,
    OpenClinicalHistory,
    FinalizeEncounter,
    ReviewAuditEvents,
    ViewTechnicalHealth,
    ExportClinicalRecords,
}

/// Authenticated actor context evaluated against hospital and resource boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedActor {
    pub actor_id: String,
    pub hospital_id: Option<String>,
    pub role: Role,
    pub membership_status: MembershipStatus,
    pub credential_status: CredentialStatus,
    pub mfa_verified: bool,
}

/// Typed authorization and handshake errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthError {
    #[error("actor authentication and MFA verification are required")]
    UnauthenticatedOrMissingMfa,
    #[error("staff membership is not active")]
    InactiveMembership,
    #[error("actor does not belong to the requested hospital scope")]
    CrossHospitalAccessDenied,
    #[error("self-approval of clinical credentials or privileges is forbidden")]
    SelfApprovalForbidden,
    #[error("actor lacks an independently approved clinical credential")]
    UnapprovedClinicalCredential,
    #[error("non-physician care-team clinical access is disabled in the pilot scope")]
    CareTeamClinicalAccessDisabled,
    #[error("role is denied access to the requested action by default-deny policy")]
    RoleDenied,
    #[error("clinical chart access requires a patient-authorized VerifiedClinicSession")]
    MissingVerifiedClinicSession,
    #[error("patient PIN must be exactly 6 ASCII digits")]
    InvalidPinFormat,
    #[error("PIN hashing or verification failed")]
    PinVerificationFailed,
    #[error("PIN attempt limit exceeded for this authorization request")]
    PinAttemptLimitExceeded,
    #[error("patient declined the clinical authorization request")]
    PatientDeclined,
    #[error("tap context has already been consumed or is not in proof-accepted state")]
    TapContextNotConsumable,
    #[error("tap context has expired")]
    TapContextExpired,
    #[error("wristband is not active")]
    BandNotActive,
    #[error("clinical session has expired")]
    ClinicalSessionExpired,
    #[error("clinical session has been revoked or superseded")]
    ClinicalSessionRevoked,
    #[error("patient or hospital context does not match the authorized clinical session")]
    SessionScopeMismatch,
}

/// Evaluates RBAC and resource conditions (`04-roles-and-permissions.md`).
///
/// Note that `OpenClinicalHistory` and `FinalizeEncounter` require BOTH passing this
/// actor-level check AND presenting a valid `&VerifiedClinicSession` at the repository boundary.
pub fn authorize_actor_action(
    actor: &AuthenticatedActor,
    target_hospital_id: Option<&str>,
    action: &ProtectedAction,
) -> Result<(), AuthError> {
    if !actor.mfa_verified {
        return Err(AuthError::UnauthenticatedOrMissingMfa);
    }
    if actor.membership_status != MembershipStatus::Active {
        return Err(AuthError::InactiveMembership);
    }

    if let Some(required_hospital) = target_hospital_id {
        let actor_hospital = actor
            .hospital_id
            .as_deref()
            .ok_or(AuthError::CrossHospitalAccessDenied)?;
        if actor_hospital != required_hospital {
            return Err(AuthError::CrossHospitalAccessDenied);
        }
    }

    match action {
        ProtectedAction::InviteStaff { target_role } => {
            if actor.role != Role::HospitalAdmin || target_hospital_id.is_none() {
                return Err(AuthError::RoleDenied);
            }
            if matches!(target_role, Role::PlatformOperator) {
                return Err(AuthError::RoleDenied);
            }
            Ok(())
        }
        ProtectedAction::ApproveClinicalEligibility { subject_actor_id } => {
            if actor.role != Role::CredentialReviewer || target_hospital_id.is_none() {
                return Err(AuthError::RoleDenied);
            }
            if actor.actor_id == *subject_actor_id {
                return Err(AuthError::SelfApprovalForbidden);
            }
            Ok(())
        }
        ProtectedAction::RegisterPatientIdentity => {
            if actor.role == Role::Reception && target_hospital_id.is_some() {
                Ok(())
            } else {
                Err(AuthError::RoleDenied)
            }
        }
        ProtectedAction::BindBand => {
            if actor.role == Role::BandOfficer && target_hospital_id.is_some() {
                Ok(())
            } else {
                Err(AuthError::RoleDenied)
            }
        }
        ProtectedAction::FreezeBand => {
            if matches!(actor.role, Role::Patient | Role::BandOfficer) {
                Ok(())
            } else {
                Err(AuthError::RoleDenied)
            }
        }
        ProtectedAction::OpenClinicalHistory | ProtectedAction::FinalizeEncounter => {
            if matches!(
                actor.role,
                Role::Nurse | Role::LaboratoryStaff | Role::PharmacyStaff
            ) {
                return Err(AuthError::CareTeamClinicalAccessDisabled);
            }
            if actor.role != Role::Doctor || target_hospital_id.is_none() {
                return Err(AuthError::RoleDenied);
            }
            match &actor.credential_status {
                CredentialStatus::Approved { reviewer_actor_id }
                    if reviewer_actor_id != &actor.actor_id
                        && !reviewer_actor_id.trim().is_empty() =>
                {
                    Ok(())
                }
                CredentialStatus::Approved { .. } => Err(AuthError::SelfApprovalForbidden),
                _ => Err(AuthError::UnapprovedClinicalCredential),
            }
        }
        ProtectedAction::ReviewAuditEvents => {
            if actor.role == Role::PrivacyAuditor && target_hospital_id.is_some() {
                Ok(())
            } else {
                Err(AuthError::RoleDenied)
            }
        }
        ProtectedAction::ViewTechnicalHealth => {
            if actor.role == Role::PlatformOperator {
                Ok(())
            } else {
                Err(AuthError::RoleDenied)
            }
        }
        // Exports are disabled in the initial pilot baseline (`I01-PILOT-BASELINE.md`).
        ProtectedAction::ExportClinicalRecords => Err(AuthError::RoleDenied),
    }
}

/// Validates that a PIN is strictly 6 ASCII digits (`000000`–`999999`).
pub fn validate_six_digit_pin(pin: &str) -> Result<(), AuthError> {
    if pin.len() == 6 && pin.bytes().all(|b| b.is_ascii_digit()) {
        Ok(())
    } else {
        Err(AuthError::InvalidPinFormat)
    }
}

/// Tuned Argon2id PIN hasher and verifier with an independent secret pepper (`19-authentication-and-session-security.md`).
pub struct PatientPinVerifier {
    pepper: [u8; 32],
}

impl PatientPinVerifier {
    pub fn new(pepper: [u8; 32]) -> Self {
        Self { pepper }
    }

    fn argon2_instance(&self) -> Result<Argon2<'_>, AuthError> {
        // Explicit Argon2id parameters: 19 MiB memory, 2 iterations, 1 degree of parallelism (OWASP baseline).
        let params =
            Params::new(19 * 1024, 2, 1, Some(32)).map_err(|_| AuthError::PinVerificationFailed)?;
        Argon2::new_with_secret(
            &self.pepper,
            Algorithm::Argon2id,
            Version::V0x13,
            params,
        )
        .map_err(|_| AuthError::PinVerificationFailed)
    }

    pub fn hash_pin(&self, pin: &str) -> Result<String, AuthError> {
        validate_six_digit_pin(pin)?;
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = self.argon2_instance()?;
        argon2
            .hash_password(pin.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| AuthError::PinVerificationFailed)
    }

    pub fn verify_pin(&self, pin: &str, stored_phc_hash: &str) -> Result<(), AuthError> {
        validate_six_digit_pin(pin)?;
        let parsed =
            PasswordHash::new(stored_phc_hash).map_err(|_| AuthError::PinVerificationFailed)?;
        let argon2 = self.argon2_instance()?;
        argon2
            .verify_password(pin.as_bytes(), &parsed)
            .map_err(|_| AuthError::PinVerificationFailed)
    }
}

/// Lifecycle state of a tap-backed clinical authorization request (`20-state-machines-and-consistency.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationRequestState {
    ProofAccepted,
    PatientPending,
    Approved,
    Declined,
    Expired,
    Consumed,
    Revoked,
}

/// Short-lived context created after Vault A accepts a strictly increasing NFC tap counter.
///
/// The PIN handshake consumes this context rather than re-submitting the raw NFC counter.
#[derive(Debug, Clone)]
pub struct AcceptedTapContext {
    context_id: String,
    blind_patient_id: BlindPatientId,
    tag_uid: [u8; 7],
    accepted_counter: u32,
    band_status: BandStatus,
    hospital_id: String,
    doctor_actor_id: String,
    issued_at_unix: u64,
    expires_at_unix: u64,
    failed_pin_attempts: u8,
    state: AuthorizationRequestState,
}

impl AcceptedTapContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        context_id: String,
        blind_patient_id: BlindPatientId,
        tag_uid: [u8; 7],
        accepted_counter: u32,
        band_status: BandStatus,
        hospital_id: String,
        doctor_actor_id: String,
        issued_at_unix: u64,
        ttl_secs: u64,
    ) -> Result<Self, AuthError> {
        if band_status != BandStatus::Active {
            return Err(AuthError::BandNotActive);
        }
        Ok(Self {
            context_id,
            blind_patient_id,
            tag_uid,
            accepted_counter,
            band_status,
            hospital_id,
            doctor_actor_id,
            issued_at_unix,
            expires_at_unix: issued_at_unix.saturating_add(ttl_secs),
            failed_pin_attempts: 0,
            state: AuthorizationRequestState::ProofAccepted,
        })
    }

    pub fn context_id(&self) -> &str {
        &self.context_id
    }

    pub fn hospital_id(&self) -> &str {
        &self.hospital_id
    }

    pub fn doctor_actor_id(&self) -> &str {
        &self.doctor_actor_id
    }

    pub fn accepted_counter(&self) -> u32 {
        self.accepted_counter
    }

    pub fn tag_uid(&self) -> [u8; 7] {
        self.tag_uid
    }

    pub fn issued_at_unix(&self) -> u64 {
        self.issued_at_unix
    }

    pub fn expires_at_unix(&self) -> u64 {
        self.expires_at_unix
    }

    pub fn state(&self) -> AuthorizationRequestState {
        self.state
    }

    pub fn failed_pin_attempts(&self) -> u8 {
        self.failed_pin_attempts
    }

    pub fn record_patient_decline(&mut self) {
        self.state = AuthorizationRequestState::Declined;
    }
}

/// Live revocation and status snapshot checked at the database transaction boundary (`19-authentication-and-session-security.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionBoundaryState {
    pub current_unix_secs: u64,
    pub band_status: BandStatus,
    pub doctor_membership: MembershipStatus,
    pub doctor_credential: CredentialStatus,
    pub session_revoked: bool,
    pub patient_consent_withdrawn: bool,
    pub revocation_version: u64,
}

/// Compile-time capability token required by every Vault B clinical read or write (`AGENTS.md` Invariant #2).
///
/// All fields are private so route handlers cannot construct this struct outside `auth::handshake`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedClinicSession {
    session_reference: String,
    blind_patient_id: BlindPatientId,
    doctor_actor_id: String,
    hospital_id: String,
    source_tap_context_id: String,
    issued_at_unix: u64,
    expires_at_unix: u64,
    revocation_version_at_issue: u64,
}

impl VerifiedClinicSession {
    pub fn session_reference(&self) -> &str {
        &self.session_reference
    }

    pub fn blind_patient_id(&self) -> &BlindPatientId {
        &self.blind_patient_id
    }

    pub fn doctor_actor_id(&self) -> &str {
        &self.doctor_actor_id
    }

    pub fn hospital_id(&self) -> &str {
        &self.hospital_id
    }

    pub fn source_tap_context_id(&self) -> &str {
        &self.source_tap_context_id
    }

    pub fn issued_at_unix(&self) -> u64 {
        self.issued_at_unix
    }

    pub fn expires_at_unix(&self) -> u64 {
        self.expires_at_unix
    }

    /// Revalidates expiry, revocation, band freeze, consent withdrawal, and doctor status
    /// at the database transaction boundary before committing any Vault B operation.
    pub fn revalidate_at_transaction_boundary(
        &self,
        expected_patient: &BlindPatientId,
        expected_hospital_id: &str,
        live_state: &TransactionBoundaryState,
    ) -> Result<(), AuthError> {
        if self.blind_patient_id != *expected_patient
            || self.hospital_id.as_bytes().ct_eq(expected_hospital_id.as_bytes()).unwrap_u8() == 0
        {
            return Err(AuthError::SessionScopeMismatch);
        }
        if live_state.current_unix_secs >= self.expires_at_unix {
            return Err(AuthError::ClinicalSessionExpired);
        }
        if live_state.session_revoked
            || live_state.patient_consent_withdrawn
            || live_state.revocation_version > self.revocation_version_at_issue
        {
            return Err(AuthError::ClinicalSessionRevoked);
        }
        if live_state.band_status != BandStatus::Active {
            return Err(AuthError::BandNotActive);
        }
        if live_state.doctor_membership != MembershipStatus::Active {
            return Err(AuthError::InactiveMembership);
        }
        match &live_state.doctor_credential {
            CredentialStatus::Approved { reviewer_actor_id }
                if reviewer_actor_id != &self.doctor_actor_id
                    && !reviewer_actor_id.trim().is_empty() =>
            {
                Ok(())
            }
            _ => Err(AuthError::UnapprovedClinicalCredential),
        }
    }
}

/// Exclusive module capable of minting a `VerifiedClinicSession`.
pub mod handshake {
    use super::{
        AcceptedTapContext, AuthError, AuthenticatedActor, AuthorizationRequestState, BandStatus,
        CLINICAL_SESSION_DURATION_SECS, MAX_PIN_ATTEMPTS, PatientPinVerifier, ProtectedAction,
        VerifiedClinicSession, authorize_actor_action,
    };

    /// Mints a fixed 15-minute `VerifiedClinicSession` only when:
    /// 1. `doctor` passes server-side RBAC and independent credential verification for `tap_context.hospital_id`.
    /// 2. `tap_context` is in `ProofAccepted` or `PatientPending` state, non-expired, and bound to `doctor` and an `ACTIVE` band.
    /// 3. `patient_pin` verifies against `stored_argon2_pin_hash` within the attempt limit.
    ///
    /// Consumes `tap_context` on success so the same tap proof cannot mint a second session.
    #[allow(clippy::too_many_arguments)]
    pub fn mint_verified_clinic_session(
        doctor: &AuthenticatedActor,
        tap_context: &mut AcceptedTapContext,
        patient_pin: &str,
        stored_argon2_pin_hash: &str,
        pin_verifier: &PatientPinVerifier,
        session_reference: String,
        current_unix_secs: u64,
        current_revocation_version: u64,
    ) -> Result<VerifiedClinicSession, AuthError> {
        authorize_actor_action(
            doctor,
            Some(&tap_context.hospital_id),
            &ProtectedAction::OpenClinicalHistory,
        )?;

        if doctor.actor_id != tap_context.doctor_actor_id {
            return Err(AuthError::SessionScopeMismatch);
        }
        if tap_context.band_status != BandStatus::Active {
            return Err(AuthError::BandNotActive);
        }
        if current_unix_secs >= tap_context.expires_at_unix {
            tap_context.state = AuthorizationRequestState::Expired;
            return Err(AuthError::TapContextExpired);
        }
        match tap_context.state {
            AuthorizationRequestState::ProofAccepted
            | AuthorizationRequestState::PatientPending => {}
            AuthorizationRequestState::Declined => return Err(AuthError::PatientDeclined),
            AuthorizationRequestState::Expired => return Err(AuthError::TapContextExpired),
            AuthorizationRequestState::Consumed
            | AuthorizationRequestState::Approved
            | AuthorizationRequestState::Revoked => {
                return Err(AuthError::TapContextNotConsumable);
            }
        }
        if tap_context.failed_pin_attempts >= MAX_PIN_ATTEMPTS {
            tap_context.state = AuthorizationRequestState::Revoked;
            return Err(AuthError::PinAttemptLimitExceeded);
        }

        if let Err(err) = pin_verifier.verify_pin(patient_pin, stored_argon2_pin_hash) {
            tap_context.failed_pin_attempts = tap_context.failed_pin_attempts.saturating_add(1);
            if tap_context.failed_pin_attempts >= MAX_PIN_ATTEMPTS {
                tap_context.state = AuthorizationRequestState::Revoked;
                return Err(AuthError::PinAttemptLimitExceeded);
            }
            tap_context.state = AuthorizationRequestState::PatientPending;
            return Err(err);
        }

        tap_context.state = AuthorizationRequestState::Consumed;
        Ok(VerifiedClinicSession {
            session_reference,
            blind_patient_id: tap_context.blind_patient_id.clone(),
            doctor_actor_id: doctor.actor_id.clone(),
            hospital_id: tap_context.hospital_id.clone(),
            source_tap_context_id: tap_context.context_id.clone(),
            issued_at_unix: current_unix_secs,
            expires_at_unix: current_unix_secs.saturating_add(CLINICAL_SESSION_DURATION_SECS),
            revocation_version_at_issue: current_revocation_version,
        })
    }
}
