//! MedPulsePH backend library.
//!
//! Exposes only `GET /health/live` on the HTTP router while providing the tested
//! compile-time capability boundaries, RBAC rules, Argon2id PIN handshake,
//! HMAC-SHA256 blind identifier derivation, and versioned AES-256-GCM vault envelopes
//! required before any authentication, NFC, or clinical routes are enabled.

pub mod auth;
pub mod crypto;
pub mod vault;

pub use vault::{ClinicalPool, IdentityPool};

use axum::{Router, http::StatusCode, routing::get};

/// Only process liveness is exposed. It does not assert database readiness,
/// and all unfinished authentication, NFC tap, emergency, and FHIR clinical routes remain disabled (404).
pub fn router() -> Router {
    Router::new().route("/health/live", get(|| async { StatusCode::NO_CONTENT }))
}

#[cfg(test)]
mod tests {
    use super::router;
    use crate::{
        auth::{
            AcceptedTapContext, AuthError, AuthenticatedActor, AuthorizationRequestState,
            BandStatus, CLINICAL_SESSION_DURATION_SECS, CredentialStatus, MAX_PIN_ATTEMPTS,
            MembershipStatus, PatientPinVerifier, ProtectedAction, Role, TransactionBoundaryState,
            authorize_actor_action, handshake::mint_verified_clinic_session,
        },
        crypto::{
            BlindIdDeriver, BlindPatientId, CryptoError, EnvelopeAad, ResourcePurpose,
            VaultCipherRing, VaultEncryptionKey,
        },
        vault::{ResourceVersion, ScopedIdempotencyRecord, VaultError},
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn sample_doctor(hospital_id: &str) -> AuthenticatedActor {
        AuthenticatedActor {
            actor_id: "doc-101".to_string(),
            hospital_id: Some(hospital_id.to_string()),
            role: Role::Doctor,
            membership_status: MembershipStatus::Active,
            credential_status: CredentialStatus::Approved {
                reviewer_actor_id: "cred-reviewer-1".to_string(),
            },
            mfa_verified: true,
        }
    }

    #[tokio::test]
    async fn only_liveness_is_exposed() -> Result<(), Box<dyn std::error::Error>> {
        let live = Request::builder().uri("/health/live").body(Body::empty())?;
        assert_eq!(router().oneshot(live).await?.status(), StatusCode::NO_CONTENT);

        for path in [
            "/",
            "/api/v1/fhir/Patient",
            "/api/v1/auth/login",
            "/v1/tap",
            "/api/v1/emergency/sample",
        ] {
            let request = Request::builder().uri(path).body(Body::empty())?;
            assert_eq!(router().oneshot(request).await?.status(), StatusCode::NOT_FOUND);
        }
        Ok(())
    }

    #[test]
    fn rbac_enforces_default_deny_and_blocks_self_escalation() -> Result<(), Box<dyn std::error::Error>> {
        let doctor = sample_doctor("hosp-manila");
        authorize_actor_action(
            &doctor,
            Some("hosp-manila"),
            &ProtectedAction::OpenClinicalHistory,
        )?;

        // Cross-hospital access is denied.
        assert_eq!(
            authorize_actor_action(
                &doctor,
                Some("hosp-cebu"),
                &ProtectedAction::OpenClinicalHistory
            ),
            Err(AuthError::CrossHospitalAccessDenied)
        );

        // Self-approved doctor credentials are rejected.
        let mut self_approved_doctor = doctor.clone();
        self_approved_doctor.credential_status = CredentialStatus::Approved {
            reviewer_actor_id: "doc-101".to_string(),
        };
        assert_eq!(
            authorize_actor_action(
                &self_approved_doctor,
                Some("hosp-manila"),
                &ProtectedAction::OpenClinicalHistory
            ),
            Err(AuthError::SelfApprovalForbidden)
        );

        // Credential reviewer cannot approve their own eligibility.
        let reviewer = AuthenticatedActor {
            actor_id: "rev-1".to_string(),
            hospital_id: Some("hosp-manila".to_string()),
            role: Role::CredentialReviewer,
            membership_status: MembershipStatus::Active,
            credential_status: CredentialStatus::NotApplicable,
            mfa_verified: true,
        };
        assert_eq!(
            authorize_actor_action(
                &reviewer,
                Some("hosp-manila"),
                &ProtectedAction::ApproveClinicalEligibility {
                    subject_actor_id: "rev-1".to_string()
                }
            ),
            Err(AuthError::SelfApprovalForbidden)
        );
        authorize_actor_action(
            &reviewer,
            Some("hosp-manila"),
            &ProtectedAction::ApproveClinicalEligibility {
                subject_actor_id: "doc-101".to_string(),
            },
        )?;

        // Hospital administrators, privacy auditors, reception, band officers, and patients cannot read clinical charts.
        for non_clinical_role in [
            Role::HospitalAdmin,
            Role::PrivacyAuditor,
            Role::Reception,
            Role::BandOfficer,
            Role::Patient,
            Role::PlatformOperator,
        ] {
            let actor = AuthenticatedActor {
                actor_id: "actor-x".to_string(),
                hospital_id: Some("hosp-manila".to_string()),
                role: non_clinical_role,
                membership_status: MembershipStatus::Active,
                credential_status: CredentialStatus::NotApplicable,
                mfa_verified: true,
            };
            assert_eq!(
                authorize_actor_action(
                    &actor,
                    Some("hosp-manila"),
                    &ProtectedAction::OpenClinicalHistory
                ),
                Err(AuthError::RoleDenied)
            );
        }

        // Care-team roles (Nurse, LaboratoryStaff, PharmacyStaff) remain explicitly disabled in the pilot baseline.
        for care_team_role in [Role::Nurse, Role::LaboratoryStaff, Role::PharmacyStaff] {
            let actor = AuthenticatedActor {
                actor_id: "care-1".to_string(),
                hospital_id: Some("hosp-manila".to_string()),
                role: care_team_role,
                membership_status: MembershipStatus::Active,
                credential_status: CredentialStatus::Approved {
                    reviewer_actor_id: "rev-1".to_string(),
                },
                mfa_verified: true,
            };
            assert_eq!(
                authorize_actor_action(
                    &actor,
                    Some("hosp-manila"),
                    &ProtectedAction::OpenClinicalHistory
                ),
                Err(AuthError::CareTeamClinicalAccessDisabled)
            );
        }

        // Exports remain disabled by default.
        assert_eq!(
            authorize_actor_action(
                &doctor,
                Some("hosp-manila"),
                &ProtectedAction::ExportClinicalRecords
            ),
            Err(AuthError::RoleDenied)
        );

        Ok(())
    }

    #[test]
    fn handshake_mints_15_minute_session_and_enforces_revocation_and_pin_limits()
    -> Result<(), Box<dyn std::error::Error>> {
        let deriver = BlindIdDeriver::new(1, [0x42; 32])?;
        let blind_id = deriver.derive("patient-synthetic-001")?;
        let other_blind_id = deriver.derive("patient-synthetic-002")?;

        let pin_verifier = PatientPinVerifier::new([0x99; 32]);
        let pin_hash = pin_verifier.hash_pin("482910")?;

        let doctor = sample_doctor("hosp-manila");
        let mut tap_ctx = AcceptedTapContext::new(
            "tap-ctx-001".to_string(),
            blind_id.clone(),
            [0x04, 0xA1, 0xB2, 0xC3, 0xD4, 0xE5, 0xF6],
            12,
            BandStatus::Active,
            "hosp-manila".to_string(),
            doctor.actor_id.clone(),
            1_700_000_000,
            120,
        )?;

        // Wrong PIN fails and increments attempt counter without consuming the tap proof.
        assert_eq!(
            mint_verified_clinic_session(
                &doctor,
                &mut tap_ctx,
                "000000",
                &pin_hash,
                &pin_verifier,
                "sess-invalid".to_string(),
                1_700_000_010,
                1,
            ),
            Err(AuthError::PinVerificationFailed)
        );
        assert_eq!(tap_ctx.failed_pin_attempts(), 1);
        assert_eq!(tap_ctx.state(), AuthorizationRequestState::PatientPending);

        // Correct PIN mints a 15-minute VerifiedClinicSession and consumes the tap context.
        let session = mint_verified_clinic_session(
            &doctor,
            &mut tap_ctx,
            "482910",
            &pin_hash,
            &pin_verifier,
            "sess-001".to_string(),
            1_700_000_020,
            1,
        )?;
        assert_eq!(tap_ctx.state(), AuthorizationRequestState::Consumed);
        assert_eq!(
            session.expires_at_unix() - session.issued_at_unix(),
            CLINICAL_SESSION_DURATION_SECS
        );

        // Reusing the consumed tap context is rejected.
        assert_eq!(
            mint_verified_clinic_session(
                &doctor,
                &mut tap_ctx,
                "482910",
                &pin_hash,
                &pin_verifier,
                "sess-002".to_string(),
                1_700_000_025,
                1,
            ),
            Err(AuthError::TapContextNotConsumable)
        );

        // Transaction-boundary revalidation passes while active and within 15 minutes.
        let mut live_state = TransactionBoundaryState {
            current_unix_secs: 1_700_000_500,
            band_status: BandStatus::Active,
            doctor_membership: MembershipStatus::Active,
            doctor_credential: CredentialStatus::Approved {
                reviewer_actor_id: "cred-reviewer-1".to_string(),
            },
            session_revoked: false,
            patient_consent_withdrawn: false,
            revocation_version: 1,
        };
        session.revalidate_at_transaction_boundary(&blind_id, "hosp-manila", &live_state)?;

        // Wrong patient context is rejected at the transaction boundary.
        assert_eq!(
            session.revalidate_at_transaction_boundary(&other_blind_id, "hosp-manila", &live_state),
            Err(AuthError::SessionScopeMismatch)
        );

        // Freezing the band immediately invalidates in-flight clinical session operations.
        live_state.band_status = BandStatus::Frozen;
        assert_eq!(
            session.revalidate_at_transaction_boundary(&blind_id, "hosp-manila", &live_state),
            Err(AuthError::BandNotActive)
        );
        live_state.band_status = BandStatus::Active;

        // Advancing revocation version or withdrawing consent invalidates the session.
        live_state.revocation_version = 2;
        assert_eq!(
            session.revalidate_at_transaction_boundary(&blind_id, "hosp-manila", &live_state),
            Err(AuthError::ClinicalSessionRevoked)
        );
        live_state.revocation_version = 1;

        // Expiry at 15 minutes is strictly enforced.
        live_state.current_unix_secs = session.expires_at_unix();
        assert_eq!(
            session.revalidate_at_transaction_boundary(&blind_id, "hosp-manila", &live_state),
            Err(AuthError::ClinicalSessionExpired)
        );

        // Verify PIN attempt lockout after MAX_PIN_ATTEMPTS failed attempts.
        let mut lockout_ctx = AcceptedTapContext::new(
            "tap-ctx-lockout".to_string(),
            blind_id,
            [0x04, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66],
            13,
            BandStatus::Active,
            "hosp-manila".to_string(),
            doctor.actor_id.clone(),
            1_700_000_000,
            120,
        )?;
        for attempt in 1..=MAX_PIN_ATTEMPTS {
            let result = mint_verified_clinic_session(
                &doctor,
                &mut lockout_ctx,
                "111111",
                &pin_hash,
                &pin_verifier,
                "sess-lock".to_string(),
                1_700_000_010,
                1,
            );
            if attempt < MAX_PIN_ATTEMPTS {
                assert_eq!(result, Err(AuthError::PinVerificationFailed));
            } else {
                assert_eq!(result, Err(AuthError::PinAttemptLimitExceeded));
            }
        }
        assert_eq!(lockout_ctx.state(), AuthorizationRequestState::Revoked);

        Ok(())
    }

    #[test]
    fn aes_gcm_envelope_binds_aad_rotates_keys_and_enforces_nonce_budget()
    -> Result<(), Box<dyn std::error::Error>> {
        let deriver = BlindIdDeriver::new(1, [0x11; 32])?;
        let patient_a = deriver.derive("synthetic-patient-a")?;
        let patient_b = deriver.derive("synthetic-patient-b")?;

        let parsed = BlindPatientId::from_canonical_str(&patient_a.to_canonical_string())?;
        assert_eq!(patient_a, parsed);

        let aad_a_note = EnvelopeAad::new(patient_a.clone(), ResourcePurpose::EncounterNote, 1)?;
        let aad_b_note = EnvelopeAad::new(patient_b, ResourcePurpose::EncounterNote, 1)?;
        let aad_a_allergy = EnvelopeAad::new(patient_a, ResourcePurpose::AllergyRecord, 1)?;

        let key_v1 = VaultEncryptionKey::new("vault-b-key-v1", [0x22; 32])?;
        let mut ring = VaultCipherRing::with_invocation_limit(key_v1, 2);

        let plaintext = b"Synthetic clinical encounter note: patient stable.";
        let env1 = ring.encrypt(plaintext, &aad_a_note)?;
        let env2 = ring.encrypt(plaintext, &aad_a_note)?;
        assert_ne!(env1.nonce, env2.nonce);

        // Exhausting the per-key invocation budget requires key rotation.
        assert_eq!(
            ring.encrypt(plaintext, &aad_a_note),
            Err(CryptoError::NonceBudgetExhausted)
        );

        // Wrong patient or wrong field purpose in AAD fails authentication.
        assert_eq!(
            ring.decrypt(&env1, &aad_b_note),
            Err(CryptoError::AuthenticationFailed)
        );
        assert_eq!(
            ring.decrypt(&env1, &aad_a_allergy),
            Err(CryptoError::AuthenticationFailed)
        );

        // Rotate to v2 key: historical v1 envelopes remain decryptable and can be re-encrypted under v2.
        let key_v2 = VaultEncryptionKey::new("vault-b-key-v2", [0x33; 32])?;
        ring.rotate_active_key(key_v2);
        assert_eq!(ring.active_key_id(), "vault-b-key-v2");
        assert_eq!(ring.decrypt(&env1, &aad_a_note)?, plaintext);

        let reencrypted = ring.reencrypt(&env1, &aad_a_note)?;
        assert_eq!(reencrypted.key_id, "vault-b-key-v2");
        assert_ne!(reencrypted.nonce, env1.nonce);
        assert_eq!(ring.decrypt(&reencrypted, &aad_a_note)?, plaintext);

        // Retiring v1 makes v1 ciphertexts undecryptable while v2 continues to work.
        assert_eq!(
            ring.destroy_retired_key("vault-b-key-v2"),
            Err(CryptoError::CannotDisableActiveKey)
        );
        ring.destroy_retired_key("vault-b-key-v1")?;
        assert_eq!(
            ring.decrypt(&env1, &aad_a_note),
            Err(CryptoError::UnknownKeyId)
        );
        assert_eq!(ring.decrypt(&reencrypted, &aad_a_note)?, plaintext);

        Ok(())
    }

    #[test]
    fn idempotency_and_optimistic_resource_versioning_reject_conflicts()
    -> Result<(), Box<dyn std::error::Error>> {
        let original = ScopedIdempotencyRecord::new(
            "doc-101",
            "hosp-manila",
            "encounter.finalize",
            "idem-key-001",
            br#"{"encounter_id":"enc-1","status":"finalized"}"#,
        )?;
        let retry_same = ScopedIdempotencyRecord::new(
            "doc-101",
            "hosp-manila",
            "encounter.finalize",
            "idem-key-001",
            br#"{"encounter_id":"enc-1","status":"finalized"}"#,
        )?;
        let retry_different_payload = ScopedIdempotencyRecord::new(
            "doc-101",
            "hosp-manila",
            "encounter.finalize",
            "idem-key-001",
            br#"{"encounter_id":"enc-2","status":"finalized"}"#,
        )?;

        original.verify_retry(&retry_same)?;
        assert_eq!(
            original.verify_retry(&retry_different_payload),
            Err(VaultError::IdempotencyPayloadMismatch)
        );

        let v1 = ResourceVersion::INITIAL;
        let v2 = v1.verify_and_advance(ResourceVersion::new(1))?;
        assert_eq!(v2.get(), 2);
        assert_eq!(
            v2.verify_and_advance(ResourceVersion::new(1)),
            Err(VaultError::StaleResourceVersion {
                expected: 1,
                actual: 2
            })
        );

        Ok(())
    }
}
