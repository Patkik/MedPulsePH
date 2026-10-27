//! MedPulsePH backend library.
//!
//! Exposes only `GET /health/live` on the HTTP router while providing the tested
//! compile-time capability boundaries, RBAC rules, Argon2id PIN handshake,
//! HMAC-SHA256 blind identifier derivation, and versioned AES-256-GCM vault envelopes
//! required before any authentication, NFC, or clinical routes are enabled.

pub mod auth;
pub mod crypto;
pub mod encounter;
pub mod enrollment;
pub mod fhir;
pub mod hardening;
pub mod nfc;
pub mod vault;

pub use vault::{ClinicalPool, IdentityPool};

use axum::{Router, http::StatusCode, middleware, routing::get};

/// Only process liveness is exposed. It does not assert database readiness,
/// and all unfinished authentication, NFC tap, emergency, and FHIR clinical routes remain disabled (404).
/// Strict browser/API security headers (`Cache-Control: no-store`, `Content-Security-Policy`,
/// `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`)
/// are applied to every response (`22-pwa-api-and-abuse-security.md`).
pub fn router() -> Router {
    Router::new()
        .route("/health/live", get(|| async { StatusCode::NO_CONTENT }))
        .layer(middleware::from_fn(hardening::security_headers_middleware))
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
        encounter::{
            AuditStorageHealth, ClinicalEncounterRecord, ClinicalEncounterState, EncounterError,
            HospitalSharingConsent, OutboxEventRecord,
        },
        enrollment::{
            EnrollmentChannel, EnrollmentError, PatientEnrollmentRecord, PatientIdentityLinkState,
            WorkstationPairingToken,
        },
        fhir::{
            FhirCapabilityStatement, FhirCodeableConcept, FhirCoding, FhirEncounterResource,
            FhirError, FhirMeta, FhirPeriod, FhirReference, VaultAPatientIdentitySlice,
            assemble_fhir_patient_at_boundary, validate_scoped_fhir_encounter,
        },
        hardening::{
            CompositeRateLimitKey, HardeningError, redact_sensitive_url_params,
            validate_outbound_https_destination,
        },
        nfc::{
            DurableBandReplayState, EmergencyFieldProvenance, EmergencyTriageSummary,
            MAX_OFFLINE_RECORD2_TEXT_BYTES, NTAG424_NDEF_FILE_CAPACITY_BYTES, NdefDualRecordBudget,
            NfcError, Ntag424TagKeys, validate_band_status_transition,
        },
        vault::{OutboxDeliveryState, ResourceVersion, ScopedIdempotencyRecord, VaultError},
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
        let response = router().oneshot(live).await?;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            response
                .headers()
                .get("cache-control")
                .and_then(|v| v.to_str().ok()),
            Some("no-store, no-cache, must-revalidate")
        );
        assert_eq!(
            response
                .headers()
                .get("x-frame-options")
                .and_then(|v| v.to_str().ok()),
            Some("DENY")
        );
        assert_eq!(
            response
                .headers()
                .get("x-content-type-options")
                .and_then(|v| v.to_str().ok()),
            Some("nosniff")
        );
        assert_eq!(
            response
                .headers()
                .get("referrer-policy")
                .and_then(|v| v.to_str().ok()),
            Some("no-referrer")
        );

        for path in [
            "/",
            "/api/v1/fhir/Patient",
            "/api/v1/auth/login",
            "/v1/tap",
            "/api/v1/emergency/sample",
        ] {
            let request = Request::builder().uri(path).body(Body::empty())?;
            let denied_resp = router().oneshot(request).await?;
            assert_eq!(denied_resp.status(), StatusCode::NOT_FOUND);
            assert_eq!(
                denied_resp
                    .headers()
                    .get("cache-control")
                    .and_then(|v| v.to_str().ok()),
                Some("no-store, no-cache, must-revalidate")
            );
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

    #[test]
    fn ntag424_sun_verification_durable_replay_and_redis_eviction()
    -> Result<(), Box<dyn std::error::Error>> {
        let deriver = BlindIdDeriver::new(1, [0x55; 32])?;
        let blind_id = deriver.derive("synthetic-patient-nfc-001")?;
        let uid = [0x04, 0x91, 0x82, 0x73, 0x64, 0x55, 0x46];
        let keys = Ntag424TagKeys {
            expected_uid: uid,
            sdm_meta_read_key: [0xA5; 16],
            sdm_file_read_key: [0x5A; 16],
        };
        let prefix = b"https://medpulse.example/v1/tap?d=";

        let mut band_state =
            DurableBandReplayState::new(blind_id, keys.clone(), BandStatus::Active, 0);

        // Counter 1 succeeds and advances Vault A durable high-water state.
        let (picc_1, cmac_1) = keys.seal_sun_tap_for_vector(1, prefix)?;
        let ctx_1 = band_state.verify_and_accept_tap(
            &picc_1,
            &cmac_1,
            prefix,
            "tap-ctx-1".to_string(),
            "hosp-manila".to_string(),
            "doc-101".to_string(),
            1_700_000_000,
            120,
        )?;
        assert_eq!(ctx_1.accepted_counter(), 1);
        assert_eq!(band_state.durable_high_water_counter(), 1);

        // Tampered CMAC is rejected in constant time.
        let mut bad_cmac = cmac_1.clone();
        bad_cmac.replace_range(0..2, if &cmac_1[0..2] == "00" { "ff" } else { "00" });
        assert!(matches!(
            band_state.verify_and_accept_tap(
                &picc_1,
                &bad_cmac,
                prefix,
                "tap-ctx-bad".to_string(),
                "hosp-manila".to_string(),
                "doc-101".to_string(),
                1_700_000_001,
                120,
            ),
            Err(NfcError::InvalidSunCmac)
        ));

        // Skipped counter (counter = 5) is valid and advances high-water mark to 5.
        let (picc_5, cmac_5) = keys.seal_sun_tap_for_vector(5, prefix)?;
        let ctx_5 = band_state.verify_and_accept_tap(
            &picc_5,
            &cmac_5,
            prefix,
            "tap-ctx-5".to_string(),
            "hosp-manila".to_string(),
            "doc-101".to_string(),
            1_700_000_005,
            120,
        )?;
        assert_eq!(ctx_5.accepted_counter(), 5);
        assert_eq!(band_state.durable_high_water_counter(), 5);

        // Simulate Redis eviction/restart: Vault A durable high-water state still rejects counter <= 5.
        band_state.simulate_redis_eviction();
        assert_eq!(band_state.cached_redis_counter(), None);
        assert!(matches!(
            band_state.verify_and_accept_tap(
                &picc_5,
                &cmac_5,
                prefix,
                "tap-ctx-replay".to_string(),
                "hosp-manila".to_string(),
                "doc-101".to_string(),
                1_700_000_006,
                120,
            ),
            Err(NfcError::ReplayRejected {
                incoming: 5,
                high_water: 5
            })
        ));

        // Counter 6 succeeds after Redis eviction and heals the cached Redis counter.
        let (picc_6, cmac_6) = keys.seal_sun_tap_for_vector(6, prefix)?;
        let ctx_6 = band_state.verify_and_accept_tap(
            &picc_6,
            &cmac_6,
            prefix,
            "tap-ctx-6".to_string(),
            "hosp-manila".to_string(),
            "doc-101".to_string(),
            1_700_000_007,
            120,
        )?;
        assert_eq!(ctx_6.accepted_counter(), 6);
        assert_eq!(band_state.cached_redis_counter(), Some(6));

        // Freezing the band immediately blocks new taps even with a fresh counter (counter = 7).
        band_state.transition_status(BandStatus::Frozen)?;
        let (picc_7, cmac_7) = keys.seal_sun_tap_for_vector(7, prefix)?;
        assert!(matches!(
            band_state.verify_and_accept_tap(
                &picc_7,
                &cmac_7,
                prefix,
                "tap-ctx-7".to_string(),
                "hosp-manila".to_string(),
                "doc-101".to_string(),
                1_700_000_008,
                120,
            ),
            Err(NfcError::BandNotActive(BandStatus::Frozen))
        ));

        // Terminal Retired state cannot transition back to Active.
        band_state.transition_status(BandStatus::Retired)?;
        assert_eq!(
            validate_band_status_transition(BandStatus::Retired, BandStatus::Active),
            Err(NfcError::InvalidBandTransition {
                from: BandStatus::Retired,
                to: BandStatus::Active
            })
        );

        Ok(())
    }

    #[test]
    fn ndef_dual_record_256_byte_budget_refuses_allergy_truncation()
    -> Result<(), Box<dyn std::error::Error>> {
        let valid_budget = NdefDualRecordBudget::build_and_verify(
            "medpulse.example",
            "O+",
            "Penicillin,Cefazolin",
            "+639170000123",
            "2026-10-28",
        )?;
        assert!(valid_budget.total_ndef_file_bytes <= NTAG424_NDEF_FILE_CAPACITY_BYTES);
        assert!(valid_budget.record2_offline_text.len() <= MAX_OFFLINE_RECORD2_TEXT_BYTES);

        // Oversized allergy string must fail provisioning rather than silently truncating allergies.
        let long_allergies =
            "Penicillin,Cefazolin,Sulfonamides,Carbamazepine,Phenytoin,Allopurinol,Lamotrigine,Abacavir,Nevirapine,Dapsone,Vancomycin";
        assert!(matches!(
            NdefDualRecordBudget::build_and_verify(
                "medpulse.example",
                "O+",
                long_allergies,
                "+639170000123",
                "2026-10-28",
            ),
            Err(NfcError::NdefBudgetExceeded { .. })
        ));

        // Verify Tier-1 emergency triage summary serializes only the approved fields and provenance.
        let triage = EmergencyTriageSummary {
            blood_type: EmergencyFieldProvenance::ClinicianVerified {
                hospital_id: "hosp-manila".to_string(),
                verified_at_unix: 1_700_000_000,
                value: "O+".to_string(),
            },
            critical_allergies: EmergencyFieldProvenance::ClinicianVerified {
                hospital_id: "hosp-manila".to_string(),
                verified_at_unix: 1_700_000_000,
                value: "Penicillin".to_string(),
            },
            ice_contact: EmergencyFieldProvenance::NotRecorded,
            offline_summary_updated_at_unix: 1_700_000_000,
        };
        let serialized = serde_json::to_string(&triage)?;
        assert!(serialized.contains("blood_type"));
        assert!(serialized.contains("critical_allergies"));
        assert!(serialized.contains("ice_contact"));
        assert!(!serialized.contains("chronic_alerts"));

        Ok(())
    }

    #[test]
    fn enrollment_requires_verified_identity_private_pin_and_retires_replaced_band()
    -> Result<(), Box<dyn std::error::Error>> {
        let deriver = BlindIdDeriver::new(1, [0x77; 32])?;
        let blind_id = deriver.derive("synthetic-patient-enroll-001")?;
        let pin_verifier = PatientPinVerifier::new([0x88; 32]);

        let reception = AuthenticatedActor {
            actor_id: "rec-201".to_string(),
            hospital_id: Some("hosp-manila".to_string()),
            role: Role::Reception,
            membership_status: MembershipStatus::Active,
            credential_status: CredentialStatus::NotApplicable,
            mfa_verified: true,
        };
        let band_officer = AuthenticatedActor {
            actor_id: "band-301".to_string(),
            hospital_id: Some("hosp-manila".to_string()),
            role: Role::BandOfficer,
            membership_status: MembershipStatus::Active,
            credential_status: CredentialStatus::NotApplicable,
            mfa_verified: true,
        };

        let mut enrollment = PatientEnrollmentRecord::register_assisted(
            &reception,
            "hosp-manila",
            blind_id,
            EnrollmentChannel::AssistedNoPhoneAtHospital,
        )?;
        assert_eq!(
            enrollment.identity_link_state(),
            &PatientIdentityLinkState::Proposed
        );

        let initial_keys = Ntag424TagKeys {
            expected_uid: [0x04, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
            sdm_meta_read_key: [0x11; 16],
            sdm_file_read_key: [0x22; 16],
        };

        // Proposed / EvidencePending links cannot configure a PIN or bind a wristband.
        enrollment.mark_evidence_pending(&reception)?;
        assert_eq!(
            enrollment.configure_patient_pin_privately(Role::Patient, "591827", &pin_verifier),
            Err(EnrollmentError::IdentityLinkNotVerified)
        );
        assert!(matches!(
            enrollment.issue_initial_band(&band_officer, "hosp-manila", initial_keys.clone()),
            Err(EnrollmentError::IdentityLinkNotVerified)
        ));

        // Verify the identity link with explicit evidence reference.
        enrollment.verify_identity_link(
            &reception,
            "hospital-id-check-ref-2026-001",
            1_700_000_000,
        )?;

        // Reception or BandOfficer cannot set a patient's PIN on their behalf.
        for staff_role in [Role::Reception, Role::BandOfficer, Role::Doctor, Role::HospitalAdmin] {
            assert_eq!(
                enrollment.configure_patient_pin_privately(staff_role, "591827", &pin_verifier),
                Err(EnrollmentError::StaffCannotSetPatientPin)
            );
        }

        // Band cannot be issued before the patient sets their private PIN.
        assert!(matches!(
            enrollment.issue_initial_band(&band_officer, "hosp-manila", initial_keys.clone()),
            Err(EnrollmentError::PatientPinNotConfigured)
        ));

        // Patient privately configures their 6-digit PIN.
        enrollment.configure_patient_pin_privately(Role::Patient, "591827", &pin_verifier)?;
        assert!(enrollment.pin_hash().is_some());
        let rev_before_replace = enrollment.revocation_version();

        // Issue initial active band and accept tap counter 1.
        let mut band_v1 =
            enrollment.issue_initial_band(&band_officer, "hosp-manila", initial_keys.clone())?;
        let prefix = b"https://medpulse.example/v1/tap?d=";
        let (picc_1, cmac_1) = initial_keys.seal_sun_tap_for_vector(1, prefix)?;
        band_v1.verify_and_accept_tap(
            &picc_1,
            &cmac_1,
            prefix,
            "tap-v1-1".to_string(),
            "hosp-manila".to_string(),
            "doc-101".to_string(),
            1_700_000_010,
            120,
        )?;

        // Replace band: old band is atomically retired and revocation_version advances.
        let replacement_keys = Ntag424TagKeys {
            expected_uid: [0x04, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
            sdm_meta_read_key: [0x33; 16],
            sdm_file_read_key: [0x44; 16],
        };
        let mut band_v2 = enrollment.replace_band(
            &band_officer,
            "hosp-manila",
            &mut band_v1,
            replacement_keys.clone(),
        )?;
        assert_eq!(band_v1.band_status(), BandStatus::Retired);
        assert_eq!(band_v2.band_status(), BandStatus::Active);
        assert!(enrollment.revocation_version() > rev_before_replace);

        // Old retired band rejects any subsequent tap, even with a fresh counter.
        let (picc_old_2, cmac_old_2) = initial_keys.seal_sun_tap_for_vector(2, prefix)?;
        assert!(matches!(
            band_v1.verify_and_accept_tap(
                &picc_old_2,
                &cmac_old_2,
                prefix,
                "tap-v1-2".to_string(),
                "hosp-manila".to_string(),
                "doc-101".to_string(),
                1_700_000_020,
                120,
            ),
            Err(NfcError::BandNotActive(BandStatus::Retired))
        ));

        // Replacement band accepts its own fresh counter.
        let (picc_new_1, cmac_new_1) = replacement_keys.seal_sun_tap_for_vector(1, prefix)?;
        let ctx_new = band_v2.verify_and_accept_tap(
            &picc_new_1,
            &cmac_new_1,
            prefix,
            "tap-v2-1".to_string(),
            "hosp-manila".to_string(),
            "doc-101".to_string(),
            1_700_000_025,
            120,
        )?;
        assert_eq!(ctx_new.accepted_counter(), 1);

        // Disputed identity link immediately blocks further band replacement.
        enrollment.mark_disputed("potential duplicate demographic match under review")?;
        assert!(matches!(
            enrollment.replace_band(
                &band_officer,
                "hosp-manila",
                &mut band_v2,
                initial_keys
            ),
            Err(EnrollmentError::IdentityLinkNotVerified)
        ));

        Ok(())
    }

    #[test]
    fn workstation_pairing_bridges_tap_without_bypassing_or_redirecting_authority()
    -> Result<(), Box<dyn std::error::Error>> {
        let deriver = BlindIdDeriver::new(1, [0x66; 32])?;
        let blind_id = deriver.derive("synthetic-patient-pair-001")?;
        let doctor_a = sample_doctor("hosp-manila");
        let mut doctor_b = sample_doctor("hosp-manila");
        doctor_b.actor_id = "doc-202".to_string();

        let nonce = [0x5C; 16];
        let mut pairing = WorkstationPairingToken::issue(
            &doctor_a,
            "hosp-manila",
            "pair-001".to_string(),
            "ws-opd-04".to_string(),
            nonce,
            1_700_000_000,
            60,
        )?;

        let tap_for_doc_a = AcceptedTapContext::new(
            "tap-ctx-doc-a".to_string(),
            blind_id.clone(),
            [0x04, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
            9,
            BandStatus::Active,
            "hosp-manila".to_string(),
            doctor_a.actor_id.clone(),
            1_700_000_010,
            120,
        )?;
        let tap_for_doc_b = AcceptedTapContext::new(
            "tap-ctx-doc-b".to_string(),
            blind_id,
            [0x04, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
            10,
            BandStatus::Active,
            "hosp-manila".to_string(),
            doctor_b.actor_id,
            1_700_000_010,
            120,
        )?;

        // Wrong challenge nonce is rejected.
        assert_eq!(
            pairing.bind_verified_tap(&tap_for_doc_a, &[0x00; 16], 1_700_000_015),
            Err(EnrollmentError::PairingNonceMismatch)
        );

        // Redirecting another doctor's tap to this workstation pairing is rejected.
        assert_eq!(
            pairing.bind_verified_tap(&tap_for_doc_b, &nonce, 1_700_000_015),
            Err(EnrollmentError::PairingScopeMismatch)
        );

        // Valid doctor + hospital + nonce binds the verified tap to the workstation once.
        let paired = pairing.bind_verified_tap(&tap_for_doc_a, &nonce, 1_700_000_015)?;
        assert_eq!(paired.workstation_id, "ws-opd-04");
        assert_eq!(paired.tap_context_id, "tap-ctx-doc-a");
        assert!(pairing.is_consumed());

        // Reusing a consumed pairing token is rejected.
        assert_eq!(
            pairing.bind_verified_tap(&tap_for_doc_a, &nonce, 1_700_000_020),
            Err(EnrollmentError::PairingTokenExpiredOrConsumed)
        );

        Ok(())
    }

    #[test]
    fn clinical_encounter_lifecycle_enforces_audit_outage_sharing_and_amendments()
    -> Result<(), Box<dyn std::error::Error>> {
        let deriver = BlindIdDeriver::new(1, [0xA1; 32])?;
        let blind_id = deriver.derive("synthetic-patient-enc-001")?;
        let pin_verifier = PatientPinVerifier::new([0xB2; 32]);
        let pin_hash = pin_verifier.hash_pin("384756")?;

        let doctor_manila = sample_doctor("hosp-manila");
        let doctor_cebu = sample_doctor("hosp-cebu");

        let mut tap_manila = AcceptedTapContext::new(
            "tap-enc-1".to_string(),
            blind_id.clone(),
            [0x04, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67],
            1,
            BandStatus::Active,
            "hosp-manila".to_string(),
            doctor_manila.actor_id.clone(),
            1_700_000_000,
            120,
        )?;
        let session_manila = mint_verified_clinic_session(
            &doctor_manila,
            &mut tap_manila,
            "384756",
            &pin_hash,
            &pin_verifier,
            "sess-manila-1".to_string(),
            1_700_000_010,
            1,
        )?;

        let live_state = TransactionBoundaryState {
            current_unix_secs: 1_700_000_100,
            band_status: BandStatus::Active,
            doctor_membership: MembershipStatus::Active,
            doctor_credential: CredentialStatus::Approved {
                reviewer_actor_id: "cred-reviewer-1".to_string(),
            },
            session_revoked: false,
            patient_consent_withdrawn: false,
            revocation_version: 1,
        };

        let mut ring = VaultCipherRing::new(VaultEncryptionKey::new("vault-b-enc-k1", [0x44; 32])?);

        // Audit storage outage must fail closed before creating any clinical record.
        assert!(matches!(
            ClinicalEncounterRecord::create_draft(
                &session_manila,
                &live_state,
                AuditStorageHealth::Outage,
                &mut ring,
                "enc-1001".to_string(),
                blind_id.clone(),
                "hosp-manila",
                b"Initial synthetic draft note",
            ),
            Err(EncounterError::AuditStorageUnavailable)
        ));

        // Create Draft -> Review -> Finalize -> Amend with optimistic concurrency.
        let mut encounter = ClinicalEncounterRecord::create_draft(
            &session_manila,
            &live_state,
            AuditStorageHealth::Healthy,
            &mut ring,
            "enc-1001".to_string(),
            blind_id.clone(),
            "hosp-manila",
            b"Initial synthetic draft note",
        )?;
        assert_eq!(encounter.current_state(), ClinicalEncounterState::Draft);
        assert_eq!(encounter.current_version(), ResourceVersion::new(1));

        let v2 = encounter.update_draft_or_review(
            &session_manila,
            &live_state,
            AuditStorageHealth::Healthy,
            &mut ring,
            ResourceVersion::new(1),
            true,
            b"Reviewed synthetic clinical assessment",
        )?;
        assert_eq!(v2, ResourceVersion::new(2));
        assert_eq!(encounter.current_state(), ClinicalEncounterState::Reviewed);

        // Stale tab version (v1 instead of v2) is rejected.
        assert_eq!(
            encounter.finalize(
                &session_manila,
                &live_state,
                AuditStorageHealth::Healthy,
                &mut ring,
                ResourceVersion::new(1),
                b"Stale finalize attempt",
            ),
            Err(EncounterError::Vault(VaultError::StaleResourceVersion {
                expected: 1,
                actual: 2
            }))
        );

        let v3 = encounter.finalize(
            &session_manila,
            &live_state,
            AuditStorageHealth::Healthy,
            &mut ring,
            ResourceVersion::new(2),
            b"Finalized synthetic encounter summary",
        )?;
        assert_eq!(v3, ResourceVersion::new(3));
        assert_eq!(encounter.current_state(), ClinicalEncounterState::Finalized);

        // Finalized encounter cannot be overwritten via draft update; requires traceable amendment.
        assert!(matches!(
            encounter.update_draft_or_review(
                &session_manila,
                &live_state,
                AuditStorageHealth::Healthy,
                &mut ring,
                ResourceVersion::new(3),
                false,
                b"Illegal draft overwrite",
            ),
            Err(EncounterError::InvalidEncounterStateTransition { .. })
        ));

        let v4 = encounter.amend_finalized(
            &session_manila,
            &live_state,
            AuditStorageHealth::Healthy,
            &mut ring,
            ResourceVersion::new(3),
            "Addendum: synthetic potassium recheck noted",
            b"Finalized synthetic encounter summary + addendum",
        )?;
        assert_eq!(v4, ResourceVersion::new(4));
        assert_eq!(encounter.current_state(), ClinicalEncounterState::Amended);
        assert_eq!(encounter.revisions().len(), 4);

        // Cross-hospital session at hosp-cebu is blocked when sharing policy is SourceHospitalOnly.
        let mut tap_cebu = AcceptedTapContext::new(
            "tap-enc-cebu".to_string(),
            blind_id,
            [0x04, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67],
            2,
            BandStatus::Active,
            "hosp-cebu".to_string(),
            doctor_cebu.actor_id.clone(),
            1_700_000_050,
            120,
        )?;
        let session_cebu = mint_verified_clinic_session(
            &doctor_cebu,
            &mut tap_cebu,
            "384756",
            &pin_hash,
            &pin_verifier,
            "sess-cebu-1".to_string(),
            1_700_000_060,
            1,
        )?;
        assert_eq!(
            encounter.read_latest_decrypted(
                &session_cebu,
                "hosp-cebu",
                &HospitalSharingConsent::SourceHospitalOnly,
                &live_state,
                AuditStorageHealth::Healthy,
                &ring,
            ),
            Err(EncounterError::CrossHospitalRecordNotShared)
        );

        // Explicit longitudinal sharing consent allows hosp-cebu to read and records audit provenance.
        let decrypted = encounter.read_latest_decrypted(
            &session_cebu,
            "hosp-cebu",
            &HospitalSharingConsent::ExplicitHospitalList(vec!["hosp-cebu".to_string()]),
            &live_state,
            AuditStorageHealth::Healthy,
            &ring,
        )?;
        assert_eq!(
            decrypted,
            b"Finalized synthetic encounter summary + addendum"
        );

        // PrivacyAuditor receives minimized audit metadata without clinical note plaintext/ciphertext.
        let auditor = AuthenticatedActor {
            actor_id: "aud-1".to_string(),
            hospital_id: Some("hosp-manila".to_string()),
            role: Role::PrivacyAuditor,
            membership_status: MembershipStatus::Active,
            credential_status: CredentialStatus::NotApplicable,
            mfa_verified: true,
        };
        let projected =
            encounter.project_minimized_audit_for_privacy_auditor(&auditor, "hosp-manila")?;
        assert_eq!(projected.len(), 4);
        let serialized_audit = serde_json::to_string(&projected)?;
        assert!(!serialized_audit.contains("addendum"));

        Ok(())
    }

    #[test]
    fn outbox_event_lifecycle_bounds_retries_and_dead_letters()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut event = OutboxEventRecord::new(
            "evt-001".to_string(),
            "patient.access_notification".to_string(),
            2,
        );
        assert_eq!(event.state(), OutboxDeliveryState::Pending);

        assert_eq!(event.lease()?, OutboxDeliveryState::Leased);
        assert_eq!(
            event.record_failure("gateway_timeout")?,
            OutboxDeliveryState::RetryableFailure
        );
        assert_eq!(event.attempt_count(), 1);

        // Second failed attempt exhausts max_attempts (2) and transitions to DeadLettered.
        assert_eq!(event.lease()?, OutboxDeliveryState::Leased);
        assert_eq!(
            event.record_failure("gateway_timeout")?,
            OutboxDeliveryState::DeadLettered
        );
        assert_eq!(
            event.lease(),
            Err(EncounterError::InvalidOutboxTransition(
                OutboxDeliveryState::DeadLettered
            ))
        );

        Ok(())
    }

    #[test]
    fn security_headers_url_redaction_ssrf_and_composite_rate_limits()
    -> Result<(), Box<dyn std::error::Error>> {
        // Sensitive NFC tap parameters (d, c, pin, token) are redacted before logging.
        let raw_url =
            "https://medpulse.example/v1/tap?d=0123456789abcdef0123456789abcdef&c=fedcba9876543210&hospital=hosp-manila#frag-secret";
        let redacted = redact_sensitive_url_params(raw_url);
        assert_eq!(
            redacted,
            "https://medpulse.example/v1/tap?d=[REDACTED]&c=[REDACTED]&hospital=hosp-manila#[REDACTED]"
        );

        // Outbound SSRF validator blocks HTTP, loopback, RFC1918, cloud metadata IP, and unapproved hosts.
        let allowlist = ["fhir.partner-hospital.ph"];
        assert_eq!(
            validate_outbound_https_destination("http://fhir.partner-hospital.ph/r4", &allowlist),
            Err(HardeningError::InsecureScheme)
        );
        for forbidden_url in [
            "https://127.0.0.1/admin",
            "https://localhost:8443/hook",
            "https://169.254.169.254/latest/meta-data/",
            "https://10.0.0.15/internal",
            "https://192.168.1.10/api",
            "https://[::1]/loopback",
            "https://vault.internal/keys",
        ] {
            assert_eq!(
                validate_outbound_https_destination(forbidden_url, &allowlist),
                Err(HardeningError::ForbiddenAddressRange)
            );
        }
        assert_eq!(
            validate_outbound_https_destination("https://evil.example.com/r4", &allowlist),
            Err(HardeningError::HostNotAllowlisted)
        );
        assert_eq!(
            validate_outbound_https_destination(
                "https://FHIR.Partner-Hospital.ph/r4/Patient",
                &allowlist
            )?,
            "fhir.partner-hospital.ph"
        );

        // Hospital-aware composite rate limit keys isolate clinicians sharing a hospital NAT IP.
        let key_doc_1 = CompositeRateLimitKey::new(
            "pin_handshake",
            "doc-101",
            Some("hosp-manila"),
            "203.0.113.10",
        )?;
        let key_doc_2 = CompositeRateLimitKey::new(
            "pin_handshake",
            "doc-202",
            Some("hosp-manila"),
            "203.0.113.10",
        )?;
        assert_ne!(key_doc_1, key_doc_2);

        Ok(())
    }

    #[test]
    fn fhir_r4_boundary_assembly_reference_scope_and_mass_assignment_guards()
    -> Result<(), Box<dyn std::error::Error>> {
        let deriver = BlindIdDeriver::new(1, [0xC3; 32])?;
        let blind_id_a = deriver.derive("synthetic-patient-fhir-a")?;
        let blind_id_b = deriver.derive("synthetic-patient-fhir-b")?;

        let pin_verifier = PatientPinVerifier::new([0xD4; 32]);
        let pin_hash = pin_verifier.hash_pin("615243")?;
        let doctor = sample_doctor("hosp-manila");

        let mut tap_ctx = AcceptedTapContext::new(
            "tap-fhir-1".to_string(),
            blind_id_a.clone(),
            [0x04, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36],
            1,
            BandStatus::Active,
            "hosp-manila".to_string(),
            doctor.actor_id.clone(),
            1_700_000_000,
            120,
        )?;
        let session = mint_verified_clinic_session(
            &doctor,
            &mut tap_ctx,
            "615243",
            &pin_hash,
            &pin_verifier,
            "sess-fhir-1".to_string(),
            1_700_000_005,
            1,
        )?;
        let live_state = TransactionBoundaryState {
            current_unix_secs: 1_700_000_100,
            band_status: BandStatus::Active,
            doctor_membership: MembershipStatus::Active,
            doctor_credential: CredentialStatus::Approved {
                reviewer_actor_id: "cred-reviewer-1".to_string(),
            },
            session_revoked: false,
            patient_consent_withdrawn: false,
            revocation_version: 1,
        };

        // Boundary assembly produces a FHIR R4 Patient resource without storing PII in Vault B.
        let vault_a_slice = VaultAPatientIdentitySlice {
            blind_patient_id: blind_id_a.clone(),
            family_name: "Dela Cruz".to_string(),
            given_name: "Maria Synthetic".to_string(),
            birth_date_iso: "1988-04-12".to_string(),
            administrative_gender: "female".to_string(),
        };
        let fhir_patient = assemble_fhir_patient_at_boundary(
            &session,
            &live_state,
            "hosp-manila",
            &vault_a_slice,
            1,
            "2026-10-28T03:30:00Z",
        )?;
        assert_eq!(fhir_patient.resource_type, "Patient");
        assert_eq!(fhir_patient.id, blind_id_a.to_canonical_string());
        assert_eq!(fhir_patient.meta.source, "urn:medpulse:hospital:hosp-manila");

        // Undeclared fields (mass-assignment attempt) are rejected by serde deny_unknown_fields.
        let mass_assignment_json = r#"{
            "resourceType": "Patient",
            "id": "bp_v1_test",
            "meta": {"versionId": "1", "lastUpdated": "2026-10-28T00:00:00Z", "source": "urn:test"},
            "active": true,
            "name": [{"family": "Cruz", "given": ["Maria"]}],
            "gender": "female",
            "birthDate": "1988-04-12",
            "isAdminOverride": true
        }"#;
        assert!(serde_json::from_str::<crate::fhir::FhirPatientResource>(mass_assignment_json).is_err());

        // Scoped FHIR Encounter validation accepts matching patient reference and rejects cross-patient reference.
        let mut fhir_enc = FhirEncounterResource {
            resource_type: "Encounter".to_string(),
            id: "enc-fhir-1".to_string(),
            meta: FhirMeta {
                version_id: "1".to_string(),
                last_updated: "2026-10-28T03:30:00Z".to_string(),
                source: "urn:medpulse:hospital:hosp-manila".to_string(),
            },
            status: "finished".to_string(),
            code: FhirCodeableConcept {
                coding: vec![FhirCoding {
                    system: "http://terminology.hl7.org/CodeSystem/v3-ActCode".to_string(),
                    code: "AMB".to_string(),
                    display: "ambulatory".to_string(),
                }],
                text: "Outpatient synthetic follow-up".to_string(),
            },
            subject: FhirReference {
                reference: format!("Patient/{}", blind_id_a.to_canonical_string()),
            },
            period: FhirPeriod {
                start: "2026-10-28T03:15:00Z".to_string(),
                end: Some("2026-10-28T03:30:00Z".to_string()),
            },
        };
        validate_scoped_fhir_encounter(&session, &live_state, "hosp-manila", &blind_id_a, &fhir_enc)?;

        // Cross-patient subject reference is rejected.
        fhir_enc.subject.reference = format!("Patient/{}", blind_id_b.to_canonical_string());
        assert!(matches!(
            validate_scoped_fhir_encounter(
                &session,
                &live_state,
                "hosp-manila",
                &blind_id_a,
                &fhir_enc
            ),
            Err(FhirError::CrossPatientReferenceForbidden { .. })
        ));

        // Script injection in FHIR text is rejected.
        fhir_enc.subject.reference = format!("Patient/{}", blind_id_a.to_canonical_string());
        fhir_enc.code.text = "<script>alert(1)</script>".to_string();
        assert_eq!(
            validate_scoped_fhir_encounter(
                &session,
                &live_state,
                "hosp-manila",
                &blind_id_a,
                &fhir_enc
            ),
            Err(FhirError::UnsafeNarrativeContent)
        );

        // CapabilityStatement explicitly lists disabled bulk/unconstrained operations.
        let cap = FhirCapabilityStatement::pilot_baseline();
        assert!(cap.disabled_operations.contains(&"$export".to_string()));
        assert!(cap.disabled_operations.contains(&"_include=*".to_string()));

        Ok(())
    }
}
