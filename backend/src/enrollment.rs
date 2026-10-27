//! Patient enrollment, identity-link verification, private PIN setup, band replacement,
//! and PC workstation pairing state machines (`I06`).
//!
//! Enforces:
//! - Explicit `PatientIdentityLinkState` (`Proposed`, `EvidencePending`, `Verified`,
//!   `Disputed`, `Superseded`) so demographic similarity or unverified links never grant
//!   chart access or allow band binding (`05-registration-login-recovery.md`,
//!   `18-patient-sharing-and-clinical-safety.md`, `20-state-machines-and-consistency.md`).
//! - Assisted / no-phone patient enrollment where reception staff can create the enrollment
//!   envelope, while only the patient (or verified legal representative) can submit the
//!   6-digit PIN (`05-registration-login-recovery.md`).
//! - Atomic band replacement that retires the previous band's online authority and advances
//!   the patient's `revocation_version` before activating the replacement band (`07-nfc-bands-and-emergency.md`).
//! - Short-lived, single-use PC workstation pairing tokens bound to `(doctor_actor_id,
//!   hospital_id, workstation_id, challenge_nonce)` that bridge phone NFC reader taps to a
//!   paired workstation without bypassing the cryptographic NFC tap proof or redirecting
//!   authorization to another clinician (`08-handshake-and-sessions.md`, `I06`).

use crate::{
    auth::{
        AcceptedTapContext, AuthError, AuthenticatedActor, BandStatus, PatientPinVerifier,
        ProtectedAction, Role, authorize_actor_action,
    },
    crypto::BlindPatientId,
    nfc::{DurableBandReplayState, NfcError, Ntag424TagKeys},
};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use thiserror::Error;

/// Maximum workstation pairing challenge lifetime in seconds (2 minutes).
pub const MAX_WORKSTATION_PAIRING_TTL_SECS: u64 = 120;

/// Patient identity link lifecycle state (`20-state-machines-and-consistency.md`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatientIdentityLinkState {
    Proposed,
    EvidencePending,
    Verified {
        verifier_actor_id: String,
        evidence_ref: String,
        verified_at_unix: u64,
    },
    Disputed {
        reason: String,
    },
    Superseded {
        replacement_link_id: String,
    },
}

/// Enrollment channel used during patient onboarding (`05-registration-login-recovery.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentChannel {
    SelfServiceWithPhone,
    AssistedNoPhoneAtHospital,
    VerifiedRepresentativeAssisted,
}

/// Typed errors for enrollment, identity-link verification, band replacement, and workstation pairing.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EnrollmentError {
    #[error("authorization failed: {0}")]
    Auth(#[from] AuthError),
    #[error("NFC state transition failed: {0}")]
    Nfc(#[from] NfcError),
    #[error("identity link must be in Verified state before binding a band or opening clinical records")]
    IdentityLinkNotVerified,
    #[error("invalid identity link state transition")]
    InvalidIdentityLinkTransition,
    #[error("verifier actor cannot self-verify or supply empty evidence")]
    InvalidVerificationEvidence,
    #[error("staff members are forbidden from setting or reading a patient's 6-digit PIN")]
    StaffCannotSetPatientPin,
    #[error("patient 6-digit PIN must be configured before activating a wristband")]
    PatientPinNotConfigured,
    #[error("workstation pairing token TTL exceeds maximum allowed window")]
    PairingTtlTooLong,
    #[error("workstation pairing token has expired or already been consumed")]
    PairingTokenExpiredOrConsumed,
    #[error("workstation pairing token challenge nonce mismatch")]
    PairingNonceMismatch,
    #[error("workstation pairing token actor or hospital scope does not match the tap context")]
    PairingScopeMismatch,
}

/// Patient enrollment and identity-link record held in Vault A (`identity_vault`).
#[derive(Debug, Clone)]
pub struct PatientEnrollmentRecord {
    blind_patient_id: BlindPatientId,
    enrolling_hospital_id: String,
    channel: EnrollmentChannel,
    identity_link_state: PatientIdentityLinkState,
    pin_hash: Option<String>,
    revocation_version: u64,
}

impl PatientEnrollmentRecord {
    /// Registers a new patient identity envelope in `Proposed` state.
    /// Requires `Role::Reception` in the target hospital (`ProtectedAction::RegisterPatientIdentity`).
    pub fn register_assisted(
        staff_actor: &AuthenticatedActor,
        hospital_id: &str,
        blind_patient_id: BlindPatientId,
        channel: EnrollmentChannel,
    ) -> Result<Self, EnrollmentError> {
        authorize_actor_action(
            staff_actor,
            Some(hospital_id),
            &ProtectedAction::RegisterPatientIdentity,
        )?;
        Ok(Self {
            blind_patient_id,
            enrolling_hospital_id: hospital_id.to_string(),
            channel,
            identity_link_state: PatientIdentityLinkState::Proposed,
            pin_hash: None,
            revocation_version: 1,
        })
    }

    pub fn blind_patient_id(&self) -> &BlindPatientId {
        &self.blind_patient_id
    }

    pub fn enrolling_hospital_id(&self) -> &str {
        &self.enrolling_hospital_id
    }

    pub fn channel(&self) -> EnrollmentChannel {
        self.channel
    }

    pub fn identity_link_state(&self) -> &PatientIdentityLinkState {
        &self.identity_link_state
    }

    pub fn pin_hash(&self) -> Option<&str> {
        self.pin_hash.as_deref()
    }

    pub fn revocation_version(&self) -> u64 {
        self.revocation_version
    }

    /// Advances the identity link to `EvidencePending` while staff collect identity proofing artifacts.
    pub fn mark_evidence_pending(
        &mut self,
        staff_actor: &AuthenticatedActor,
    ) -> Result<(), EnrollmentError> {
        authorize_actor_action(
            staff_actor,
            Some(&self.enrolling_hospital_id),
            &ProtectedAction::RegisterPatientIdentity,
        )?;
        match self.identity_link_state {
            PatientIdentityLinkState::Proposed => {
                self.identity_link_state = PatientIdentityLinkState::EvidencePending;
                Ok(())
            }
            _ => Err(EnrollmentError::InvalidIdentityLinkTransition),
        }
    }

    /// Marks the identity link `Verified` after explicit evidence checks.
    /// Demographic similarity alone never satisfies this transition.
    pub fn verify_identity_link(
        &mut self,
        staff_actor: &AuthenticatedActor,
        evidence_ref: &str,
        verified_at_unix: u64,
    ) -> Result<(), EnrollmentError> {
        authorize_actor_action(
            staff_actor,
            Some(&self.enrolling_hospital_id),
            &ProtectedAction::RegisterPatientIdentity,
        )?;
        if evidence_ref.trim().is_empty() {
            return Err(EnrollmentError::InvalidVerificationEvidence);
        }
        match self.identity_link_state {
            PatientIdentityLinkState::Proposed | PatientIdentityLinkState::EvidencePending => {
                self.identity_link_state = PatientIdentityLinkState::Verified {
                    verifier_actor_id: staff_actor.actor_id.clone(),
                    evidence_ref: evidence_ref.trim().to_string(),
                    verified_at_unix,
                };
                Ok(())
            }
            _ => Err(EnrollmentError::InvalidIdentityLinkTransition),
        }
    }

    /// Flags a possible duplicate or contested identity link as `Disputed`, immediately
    /// blocking new band binding or chart linkage until resolved (`18-patient-sharing-and-clinical-safety.md`).
    pub fn mark_disputed(&mut self, reason: &str) -> Result<(), EnrollmentError> {
        if reason.trim().is_empty() {
            return Err(EnrollmentError::InvalidVerificationEvidence);
        }
        if matches!(
            self.identity_link_state,
            PatientIdentityLinkState::Superseded { .. }
        ) {
            return Err(EnrollmentError::InvalidIdentityLinkTransition);
        }
        self.identity_link_state = PatientIdentityLinkState::Disputed {
            reason: reason.trim().to_string(),
        };
        self.revocation_version = self.revocation_version.saturating_add(1);
        Ok(())
    }

    /// Configures the patient's 6-digit PIN via private patient-held entry.
    /// Rejects any attempt by Reception, BandOfficer, Doctor, or Admin roles to submit the PIN.
    pub fn configure_patient_pin_privately(
        &mut self,
        submitting_role: Role,
        raw_pin: &str,
        pin_verifier: &PatientPinVerifier,
    ) -> Result<(), EnrollmentError> {
        if submitting_role != Role::Patient {
            return Err(EnrollmentError::StaffCannotSetPatientPin);
        }
        if !matches!(
            self.identity_link_state,
            PatientIdentityLinkState::Verified { .. }
        ) {
            return Err(EnrollmentError::IdentityLinkNotVerified);
        }
        let hash = pin_verifier.hash_pin(raw_pin)?;
        self.pin_hash = Some(hash);
        self.revocation_version = self.revocation_version.saturating_add(1);
        Ok(())
    }

    /// Issues an initial active NFC wristband for a verified patient who has configured a private PIN.
    pub fn issue_initial_band(
        &self,
        staff_actor: &AuthenticatedActor,
        hospital_id: &str,
        tag_keys: Ntag424TagKeys,
    ) -> Result<DurableBandReplayState, EnrollmentError> {
        authorize_actor_action(staff_actor, Some(hospital_id), &ProtectedAction::BindBand)?;
        if !matches!(
            self.identity_link_state,
            PatientIdentityLinkState::Verified { .. }
        ) {
            return Err(EnrollmentError::IdentityLinkNotVerified);
        }
        if self.pin_hash.is_none() {
            return Err(EnrollmentError::PatientPinNotConfigured);
        }
        Ok(DurableBandReplayState::new(
            self.blind_patient_id.clone(),
            tag_keys,
            BandStatus::Active,
            0,
        ))
    }

    /// Atomically replaces an existing wristband (`Active`, `Frozen`, or `ReplacementPending`)
    /// by transitioning the old band to terminal `Retired`, advancing `revocation_version`
    /// so in-flight sessions are invalidated, and returning the newly activated replacement band.
    pub fn replace_band(
        &mut self,
        staff_actor: &AuthenticatedActor,
        hospital_id: &str,
        previous_band: &mut DurableBandReplayState,
        replacement_tag_keys: Ntag424TagKeys,
    ) -> Result<DurableBandReplayState, EnrollmentError> {
        authorize_actor_action(staff_actor, Some(hospital_id), &ProtectedAction::BindBand)?;
        if !matches!(
            self.identity_link_state,
            PatientIdentityLinkState::Verified { .. }
        ) {
            return Err(EnrollmentError::IdentityLinkNotVerified);
        }
        if self.pin_hash.is_none() {
            return Err(EnrollmentError::PatientPinNotConfigured);
        }
        if previous_band.blind_patient_id() != &self.blind_patient_id {
            return Err(EnrollmentError::Auth(AuthError::SessionScopeMismatch));
        }

        // Move active/frozen bands through ReplacementPending if needed, then retire permanently.
        match previous_band.band_status() {
            BandStatus::Active | BandStatus::Frozen => {
                previous_band.transition_status(BandStatus::ReplacementPending)?;
                previous_band.transition_status(BandStatus::Retired)?;
            }
            BandStatus::ReplacementPending | BandStatus::VerificationPending => {
                previous_band.transition_status(BandStatus::Retired)?;
            }
            other => {
                return Err(EnrollmentError::Nfc(NfcError::InvalidBandTransition {
                    from: other,
                    to: BandStatus::Retired,
                }));
            }
        }

        self.revocation_version = self.revocation_version.saturating_add(1);
        Ok(DurableBandReplayState::new(
            self.blind_patient_id.clone(),
            replacement_tag_keys,
            BandStatus::Active,
            0,
        ))
    }
}

/// Short-lived, single-use PC workstation pairing token (`08-handshake-and-sessions.md`, `I06`).
/// Bridges a phone NFC reader tap to a specific authenticated doctor's PC workstation
/// without replacing cryptographic NFC tap verification.
#[derive(Debug, Clone)]
pub struct WorkstationPairingToken {
    pairing_id: String,
    doctor_actor_id: String,
    hospital_id: String,
    workstation_id: String,
    challenge_nonce: [u8; 16],
    issued_at_unix: u64,
    expires_at_unix: u64,
    consumed: bool,
}

/// Verified binding between an `AcceptedTapContext` and a paired PC workstation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairedWorkstationTapContext {
    pub pairing_id: String,
    pub workstation_id: String,
    pub doctor_actor_id: String,
    pub hospital_id: String,
    pub tap_context_id: String,
    pub accepted_counter: u32,
}

impl WorkstationPairingToken {
    /// Issues a short-lived workstation pairing challenge for an authenticated `Doctor` in `hospital_id`.
    pub fn issue(
        doctor_actor: &AuthenticatedActor,
        hospital_id: &str,
        pairing_id: String,
        workstation_id: String,
        challenge_nonce: [u8; 16],
        issued_at_unix: u64,
        ttl_secs: u64,
    ) -> Result<Self, EnrollmentError> {
        authorize_actor_action(
            doctor_actor,
            Some(hospital_id),
            &ProtectedAction::OpenClinicalHistory,
        )?;
        if ttl_secs == 0 || ttl_secs > MAX_WORKSTATION_PAIRING_TTL_SECS {
            return Err(EnrollmentError::PairingTtlTooLong);
        }
        Ok(Self {
            pairing_id,
            doctor_actor_id: doctor_actor.actor_id.clone(),
            hospital_id: hospital_id.to_string(),
            workstation_id,
            challenge_nonce,
            issued_at_unix,
            expires_at_unix: issued_at_unix.saturating_add(ttl_secs),
            consumed: false,
        })
    }

    pub fn issued_at_unix(&self) -> u64 {
        self.issued_at_unix
    }

    pub fn expires_at_unix(&self) -> u64 {
        self.expires_at_unix
    }

    pub fn is_consumed(&self) -> bool {
        self.consumed
    }

    /// Binds a cryptographically verified `AcceptedTapContext` from the doctor's paired phone
    /// to the target PC workstation.
    ///
    /// Security properties enforced (`I06`):
    /// - Requires a real `AcceptedTapContext` (pairing QR alone can never open a session).
    /// - Constant-time verification of `presented_nonce`.
    /// - Rejects any mismatch between the pairing token's `(doctor_actor_id, hospital_id)` and
    ///   the tap context's bound `(expected_doctor_actor_id, expected_hospital_id)`.
    /// - Single-use and strictly bounded by `expires_at_unix`.
    pub fn bind_verified_tap(
        &mut self,
        tap_context: &AcceptedTapContext,
        presented_nonce: &[u8; 16],
        current_unix_secs: u64,
    ) -> Result<PairedWorkstationTapContext, EnrollmentError> {
        if self.consumed || current_unix_secs >= self.expires_at_unix {
            return Err(EnrollmentError::PairingTokenExpiredOrConsumed);
        }
        if !bool::from(self.challenge_nonce.ct_eq(presented_nonce)) {
            return Err(EnrollmentError::PairingNonceMismatch);
        }
        if tap_context.doctor_actor_id() != self.doctor_actor_id
            || tap_context.hospital_id() != self.hospital_id
        {
            return Err(EnrollmentError::PairingScopeMismatch);
        }

        self.consumed = true;
        Ok(PairedWorkstationTapContext {
            pairing_id: self.pairing_id.clone(),
            workstation_id: self.workstation_id.clone(),
            doctor_actor_id: self.doctor_actor_id.clone(),
            hospital_id: self.hospital_id.clone(),
            tap_context_id: tap_context.context_id().to_string(),
            accepted_counter: tap_context.accepted_counter(),
        })
    }
}
