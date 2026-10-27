# MedPulsePH iteration packages

[Plan index](../../IMPLEMENTATION_PLAN.md)

**Status:** Engineering contracts, split-vault PostgreSQL schemas, cryptographic envelopes, NTAG 424 DNA SUN verification, 256-byte NDEF budget checks, enrollment/pairing state machines, clinical encounter lifecycles, browser/API hardening, scoped FHIR R4 boundary assembly, coordinated restore reconciliation, and the end-to-end synthetic acceptance test matrix are implemented across `I01`–`I12`. All unfinished HTTP clinical/auth routes remain disabled (`404`), and packages requiring hospital clinical, privacy, procurement, or physical hardware sign-off remain `in progress (blocked on external hospital/hardware sign-off)` rather than inventing approvals.

The 29 specifications are the requirement sources. This file describes manageable work packages and their dependencies. Each package can take several small changes; it is not a promised calendar sprint. Package numbers are stable identifiers, not a requirement to finish every package strictly in numerical order.

## Iteration ledger

| Package | Focus | Prerequisites for implementation | Current state |
| --- | --- | --- | --- |
| I01 | Scope, risk, and hospital policy | None | In progress; pilot baseline drafted; hospital owners and approvals open |
| I02 | Roles, accounts, and authentication | Relevant I01 identity and authorization decisions | In progress; in-memory RBAC, Argon2id PIN verifier, and 15-minute `VerifiedClinicSession` typestate contracts implemented; HTTP auth/MFA routes pending |
| I03 | Screen flows, dashboard specifications, and brand | Relevant I01 user and scope decisions; use I02 permission contract | In progress; role preview, `S01`–`S10` screen-state catalog (`28-screen-and-api-specifications.md`), and `64×64` Clasp SVG assets + 5-panel stress-test sheet (`app/public/logo-stress-test.html`) implemented; hospital user review and physical band sample check pending |
| I04 | Vaults, keys, state machines, and skill alignment | Relevant I01 boundaries; I02 capability contract | In progress; skill alignment, Docker vaults, PostgreSQL schemas (`vault_a_identity.sql`, `vault_b_clinical.sql`), `BlindPatientId` HMAC, versioned AES-256-GCM envelopes, and transactional capability/audit/replay repositories implemented; outbox workers and backup restore drills pending |
| I05 | NFC proof and emergency access | I04 durable state and keys; I02 actor/context contracts for clinical handoff | In progress; NTAG 424 DNA SUN crypto, 256-byte NDEF budget validator, durable replay state with Redis eviction recovery, and Vault A SQL tap acceptance transaction implemented; HTTP routes and hardware tests pending |
| I06 | Patient enrollment, bands, and workstation pairing | I02 identity; I04 state transitions; I05 verified reading | In progress; `backend/src/enrollment.rs` implements `PatientIdentityLinkState` (`Proposed`, `EvidencePending`, `Verified`, `Disputed`, `Superseded`), assisted enrollment with private patient-only PIN setup, atomic band replacement retiring old band authority, and single-use `WorkstationPairingToken` tap binding; live HTTP endpoints and physical band tests pending |
| I07 | Administration, durable audit, and operational events | I02 permissions; I04 repositories and service capabilities | In progress; append-only `immutable_audit_logs` schema/trigger, transactional audit+record repository methods, `MinimizedAuditEventProjection` for `Role::PrivacyAuditor`, fail-closed `AuditStorageHealth::Outage` enforcement, and bounded `OutboxEventRecord` retry/dead-letter state machine implemented; live HTTP admin routes and hospital incident sign-off pending |
| I08 | Complete authorized encounter | I03 encounter specification; I05–I07 relevant workflows | In progress; `ClinicalEncounterRecord` (`Draft -> Reviewed -> Finalized -> Amended`), optimistic concurrency (`ResourceVersion`), traceable AES-256-GCM revisions, and cross-hospital `HospitalSharingConsent` enforcement implemented in `backend/src/encounter.rs`; live HTTP clinical routes and hospital clinical review pending |
| I09 | PWA integration and abuse/security verification | Relevant implemented slices from I02–I08 | In progress; HTTP security headers middleware, NFC tap URL parameter redaction, outbound SSRF destination validator, and hospital-aware composite rate-limit keys implemented in `backend/src/hardening.rs`; real-device browser/NFC audit and Expo CLI transitive `zod` advisory (`GHSA-mrhx-c7xc-m3w5`) remain open |
| I10 | FHIR partners, files, and exports | I08 authorized record operations; relevant I09 protections | In progress; scoped FHIR R4 models (`Patient`, `AllergyIntolerance`, `Encounter`, `CapabilityStatement`), Vault A boundary `Patient` assembly, `deny_unknown_fields` mass-assignment guards, cross-patient reference rejection, and narrative sanitization implemented in `backend/src/fhir.rs` while keeping uploads and bulk exports disabled |
| I11 | Deployment, recovery, and operational acceptance | Relevant policy decisions; implemented slices to exercise | In progress; coordinated Vault A backup/restore counter and status reconciliation (`backend/src/recovery.rs`) and live SQL monotonic counter restore checks (`scripts/check-local-stack.mjs`) implemented; hospital RTO/RPO and live downtime sign-off pending |
| I12 | End-to-end acceptance and controlled pilot | All applicable dependencies and P0 gates for the selected pilot | In progress; full positive/denied synthetic journey matrix (`end_to_end_synthetic_journey_matrix_and_coordinated_restore` in `backend/src/lib.rs`) and skill/spec synchronization checks pass; live-patient pilot remains blocked on hospital clinical, privacy, and physical hardware sign-off |

Design and policy work can overlap. Foundational browser security begins with I02, audit requirements apply to each affected write/read from its first implementation, and release integrity applies before the first deployed build. I09 and I11 consolidate verification rather than defer these controls. Tests are included with the feature changes they verify, not saved entirely for I12.

## I01 — Agree on scope and resolve blocking policy decisions

**Read:** [Product scope](01-product-scope.md), [risk and decision register](16-risk-and-decision-register.md), [threat model](17-threat-model.md), [patient sharing and clinical safety](18-patient-sharing-and-clinical-safety.md), and [privacy and physical deployment](27-privacy-and-physical-deployment.md).

**Current artifact:** [I01 pilot baseline and decision ledger](I01-PILOT-BASELINE.md). This is a proposal; no hospital owner or approval is recorded.

Small deliverables:

- Record the doctor-focused pilot scope and disabled capabilities, including care-team access and full-record emergency override.
- Assign owners and resolve the identity, patient representation, public/offline disclosure, and hospital-sharing decisions needed by the next slice.
- Draw the data flows and trust boundaries; record clinical hazards and residual limitations.
- Establish the patient-rights process and the hospital/privacy/operations responsibilities.

**Completion check:** The pilot has an explicit scope and named decision owners; no dependent feature proceeds on an unresolved permission, identity, or data-ownership assumption. Later decisions may remain open with a clear gate.

## I02 — Define and implement roles, accounts, and authentication

**Read:** [Roles and permissions](04-roles-and-permissions.md), [registration/login/recovery](05-registration-login-recovery.md), and [authentication/session security](19-authentication-and-session-security.md).

Small deliverables:

- Define the action/resource/hospital permission contract and denial cases.
- Implement hospital onboarding, staff invitations, membership, and independent credential review as separate lifecycle steps.
- Implement account authentication, required MFA, controlled recovery, and hospital/workspace selection.
- Define clinical capability, revocation, and expiry contracts before dependent handlers use them.

**Completion check:** No self-escalation, invitation tampering, cross-hospital access, or account-linking shortcut succeeds. Administrator access does not confer chart access; patient account login does not substitute for clinical consent.

## I03 — Specify screen flows, dashboards, and visual identity

**Read:** [Users/workspaces](03-users-and-workspaces.md), [dashboards](06-dashboards.md), [UI design/resilience](12-ui-design-and-resilience.md), [logo brief](13-logo-and-identity.md), and [screen/API specifications](28-screen-and-api-specifications.md).

**Current artifact:** An early Expo web design preview in `app/`. It uses no real patient data, does not authenticate users, and does not implement clinical or NFC access. See [preview evidence and open items](I03-PREVIEW-EVIDENCE.md). Formal dashboard flows, logo stress tests, stakeholder reviews, and access control remain outstanding.

Small deliverables:

- Build the screen catalog for entry, onboarding, account recovery, role dashboards, enrollment, authorization, and encounter work.
- Attach permissions, data fields, loading/empty/denial/error states, patient context, and interruption behavior to every screen.
- Review phone and PC wireframes with hospital users; define tested design tokens and accessibility behavior.
- Compare the three logo directions, choose and refine one, and complete the specified monochrome, small-size, and physical checks.

**Completion check:** Each primary action has a defined permission and recovery path; the design does not imply disabled functionality. Logo production follows the workspace logo skill. Any prototype at this stage uses synthetic data and does not imply backend enforcement.

## I04 — Establish vaults, cryptographic operations, and durable state

**Read:** [Architecture](02-system-architecture.md), [state machines/consistency](20-state-machines-and-consistency.md), [cryptography/key operations](21-cryptography-and-key-operations.md), and [skill governance](14-skill-governance.md).

**Current slice:** [Local Docker development](I04-DOCKER-DEVELOPMENT.md), separate connection-pool types, and a health-only API. This does not complete vault isolation, clinical authorization, or the iteration.

Small deliverables:

- Resolve the PostgreSQL-versus-Firestore skill conflict and align affected project instructions and mirrored skills before dependent implementation.
- Define separate pool types, database roles, encrypted record envelopes, identifier mapping, and key/nonce lifecycle.
- Establish transaction, idempotency, resource-version, outbox/inbox, and service-capability contracts.
- Prove isolation, key rotation/decryption, failure recovery, and replay-state durability with synthetic fixtures.

**Completion check:** Ordinary handlers cannot bypass clinical capability boundaries, vault credentials remain distinct, and crash/retry behavior is specified and tested. Updating a skill never silently weakens a project invariant.

## I05 — Prove NFC verification and the emergency path

**Read:** [NFC/bands/emergency](07-nfc-bands-and-emergency.md), [replay/clinical authorization](08-replay-and-clinical-authorization.md), and [threat model](17-threat-model.md).

Small deliverables:

- Verify real NTAG 424 provisioning, NXP vectors, dynamic URL construction, and the complete two-record byte budget.
- Implement durable proof acceptance and bounded tap contexts with replay, concurrency, and crash tests.
- Implement the narrow emergency response and explicit invalid/frozen/offline states.
- Demonstrate the actual phone/browser behavior and record delayed-proof and offline-revocation limitations.

**Completion check:** Consumed proofs cannot be reused after restart/failover; no counter is consumed twice during the later PIN handshake. Emergency disclosure remains limited to the approved fields. Unsupported devices are identified rather than given false capability promises.

## I06 — Complete patient enrollment, band operations, and PC pairing

**Read:** [Patient registration](05-registration-login-recovery.md), [band operations/pairing](07-nfc-bands-and-emergency.md), [patient identity/sharing](18-patient-sharing-and-clinical-safety.md), and [state machines](20-state-machines-and-consistency.md).

Small deliverables:

- Implement verified patient linking, assisted/no-phone enrollment, private PIN setup, and duplicate/disputed identity review.
- Implement provisioning verification, activation, freeze, replacement, retirement, and physical-summary update status.
- Bind short-lived PC/phone pairing to the correct clinician, workstation, hospital, and patient context.
- Verify recovery, cancellation, expired pairing, stolen invitations, and old-band behavior.

**Completion check:** Staff cannot learn a patient's PIN or claim a record using demographic matches alone. Replacement revokes the previous band. Pairing QR codes cannot replace verified NFC proof or redirect authorization to another workstation.

## I07 — Add administration, audit evidence, and event delivery

**Read:** [Administration/audit/notifications](10-administration-audit-notifications.md), [observability/incidents](24-observability-audit-and-incidents.md), and [state consistency](20-state-machines-and-consistency.md).

Small deliverables:

- Implement scoped staff administration, independent approvals, suspension, and session revocation.
- Implement durable clinical audit primitives before I08 depends on them.
- Add minimized privacy-event views and bounded notification delivery with retry/dead-letter handling.
- Add operational metrics, audit-outage behavior, and first incident runbooks.

**Completion check:** No protected disclosure or clinical commit bypasses its required durable audit. Privacy and technical dashboards cannot browse chart content. Suspension and audit/storage failure scenarios have verified behavior.

## I08 — Deliver one complete authorized encounter

**Read:** [Clinical workspace/vault boundaries](09-clinical-workspace-and-vaults.md), [authorization](08-replay-and-clinical-authorization.md), [clinical safety](18-patient-sharing-and-clinical-safety.md), and [state consistency](20-state-machines-and-consistency.md).

Small deliverables:

- Connect verified tap, displayed request scope, patient PIN authorization, and the 15-minute clinical session.
- Read only authorized patient/resource data; preserve source organization and provenance.
- Implement draft saving, explicit finalization, amendment, and conflict handling.
- Demonstrate durable audit and patient access activity across the full synthetic encounter.

**Completion check:** Wrong-patient, wrong-hospital, expired, revoked, stale-tab, and duplicate-write cases are denied or reconciled correctly. Network loss never produces a false saved state. No unapproved care-team access is enabled.

## I09 — Integrate the PWA and verify browser/API resilience

**Read:** [PWA/API/abuse security](22-pwa-api-and-abuse-security.md), [UI resilience](12-ui-design-and-resilience.md), and [session security](19-authentication-and-session-security.md).

Small deliverables:

- Complete installability, safe service-worker scope/versioning, and browser-resume revalidation.
- Verify sensitive-data exclusion from browser storage, pre-rendering, history, caches, logs, traces, and third-party capture.
- Test CSRF, framing, untrusted content, schema/mass-assignment controls, request limits, and dependency backpressure.
- Complete keyboard, screen-reader, text enlargement, high-contrast, localization, phone, and PC checks.

**Completion check:** Protected data does not persist through unintended browser paths; unsupported NFC behavior is explained accurately. API limits protect expensive verification without treating hospital-shared IPs as individual identities.

## I10 — Enable only specified interoperability and disclosure features

**Read:** [FHIR interoperability](11-fhir-interoperability.md), [files/exports/integrations](23-files-exports-and-integrations.md), and [clinical safety/sharing](18-patient-sharing-and-clinical-safety.md).

Small deliverables:

- Implement the selected FHIR profiles, capability statement, scoped search, validation, and partner identity/provenance rules.
- Add a partner integration only with approved machine capabilities and patient-authorization behavior.
- If uploads are in scope, implement quarantine, content validation, private storage, scanning, and authorized downloads.
- If exports/printing are in scope, implement independent disclosure permissions, destination checks, expiry, and audit.

**Completion check:** Partner requests, references, searches, files, and exports cannot bypass patient/hospital scope. A deferred subfeature stays disabled; this package can be narrowed to the pilot's FHIR requirements without implying file/export support.

## I11 — Demonstrate deployment, recovery, and hospital operations

**Read:** [Availability/disaster recovery](25-availability-and-disaster-recovery.md), [deployment/supply chain](26-deployment-and-supply-chain.md), and [privacy/physical deployment](27-privacy-and-physical-deployment.md).

Small deliverables:

- Maintain environment separation, least-privilege deployment/runtime/migration identities, dependency integrity, and secret protection from the first deployed build.
- Exercise safe schema changes, mixed-client compatibility, service-worker updates, and rollback.
- Rehearse coordinated vault/key/audit recovery without restoring old sessions, band authority, or replay acceptance.
- Measure capacity/cost and recovery objectives; finalize hospital downtime, support, supplier, and incident responsibilities.

**Completion check:** Restore and incident exercises meet accepted objectives and retain security boundaries. Required physical-band/device checks and operational ownership exist. No deployment setting is treated as proof of a guaranteed service level.

## I12 — Complete evidence review and the controlled pilot gate

**Read:** [Testing/rollout](15-testing-and-rollout.md), [risk register](16-risk-and-decision-register.md), and [readiness/open decisions](29-readiness-and-open-decisions.md).

Small deliverables:

- Run the full allowed/denied journey matrix for the selected pilot scope on real devices with synthetic records.
- Complete hospital usability review, applicable security review, clinical hazard review, privacy decisions, and training.
- Collect required implementation checks, restore/load evidence, release provenance, and synchronized skill/specification review.
- Record outstanding decisions, explicit deferred features, named owners, and accepted residual risks.

**Completion check:** Every applicable P0 gate has evidence. P1 deferrals have an accepted operational alternative and disabled affected functionality. Proceed to live-patient use only under the hospital's completed acceptance process.

## Iteration completion record

For each package, maintain its ledger state and a short record containing:

- Named owner and bounded scope for this iteration.
- Decisions resolved and the risk-register items affected.
- Links to changed specifications, affected skills, and implementation changes when they exist.
- Verification results and evidence, including denied/failure cases.
- Deferred capabilities, remaining dependencies, and next small action.

Use states such as **planned**, **in progress**, **ready for review**, **complete**, or **blocked by a named decision**. A package is complete only when its scoped deliverables and checks are satisfied; do not mark a whole package complete merely because one file or screen exists.

### Package completion records (`I01`–`I12`)

| Package | Ledger state | Implemented engineering artifacts & specs | Verified positive & denied cases | Remaining external hospital / hardware gates |
| --- | --- | --- | --- | --- |
| **I01** | In progress (blocked on hospital decision owners) | `docs/implementation/I01-PILOT-BASELINE.md`, `01-product-scope.md`, `16-risk-and-decision-register.md`, `27-privacy-and-physical-deployment.md` | Doctor-only Tier-2 baseline and disabled care-team/override rules codified in `backend/src/auth.rs` | Assign named hospital clinical, privacy/DPO, and operational decision owners (`D-01`–`D-08`) |
| **I02** | In progress (ready for HTTP auth wiring after IdP selection) | `backend/src/auth.rs`, `04-roles-and-permissions.md`, `19-authentication-and-session-security.md` | `rbac_enforces_default_deny_and_blocks_self_escalation`, `handshake_mints_15_minute_session_and_enforces_revocation_and_pin_limits` | Select hospital staff IdP/MFA mechanism (`D-01`) before exposing HTTP login routes |
| **I03** | In progress (ready for clinical & physical stencil review) | `app/src/app/index.tsx`, `app/public/icon.svg`, `app/public/mark.svg`, `app/public/logo-lockup.svg`, `app/public/logo-stress-test.html`, `28-screen-and-api-specifications.md` | `npm --prefix app run typecheck` and `lint`; `64×64` integer-grid Clasp (`x=28..36`) verified across 5-panel stress test | Hospital clinician/patient usability review and physical 12 mm wristband laser-stencil sample check |
| **I04** | In progress (core vault & crypto contracts verified) | `backend/src/crypto.rs`, `backend/src/vault.rs`, `backend/sql/vault_a_identity.sql`, `backend/sql/vault_b_clinical.sql`, `.agents/skills/*`, `.claude/skills/*` | `aes_gcm_envelope_binds_aad_rotates_keys_and_enforces_nonce_budget`, `idempotency_and_optimistic_resource_versioning_reject_conflicts`, `node scripts/check-local-stack.mjs`, `node scripts/check-skill-sync.mjs` | Cloud KMS/HSM production key ceremony and multi-host staging database provisioning |
| **I05** | In progress (ready for physical NTAG 424 DNA bench test) | `backend/src/nfc.rs`, `07-nfc-bands-and-emergency.md`, `08-replay-and-clinical-authorization.md` | `ntag424_sun_verification_durable_replay_and_redis_eviction`, `ndef_dual_record_256_byte_budget_refuses_allergy_truncation` | Physical NXP NTAG 424 DNA wristband encoding and real Android/iPhone tap latency matrix |
| **I06** | In progress (state machines & pairing verified) | `backend/src/enrollment.rs`, `05-registration-login-recovery.md` | `enrollment_requires_verified_identity_private_pin_and_retires_replaced_band`, `workstation_pairing_bridges_tap_without_bypassing_or_redirecting_authority` | Hospital reception identity-proofing SOP and guardian/representative policy (`D-03`) |
| **I07** | In progress (durable audit & outbox state verified) | `backend/src/encounter.rs`, `backend/sql/vault_b_clinical.sql`, `10-administration-audit-notifications.md`, `24-observability-audit-and-incidents.md` | `outbox_event_lifecycle_bounds_retries_and_dead_letters`, `Vault B append-only audit log enforcement` (`check-local-stack.mjs`) | Hospital privacy auditor workflow sign-off and 72-hour NPC breach-notification escalation owner |
| **I08** | In progress (encounter lifecycle & sharing verified) | `backend/src/encounter.rs`, `09-clinical-workspace-and-vaults.md`, `18-patient-sharing-and-clinical-safety.md` | `clinical_encounter_lifecycle_enforces_audit_outage_sharing_and_amendments` | Clinical governance sign-off on encounter templates and cross-hospital sharing policy (`D-05`) |
| **I09** | In progress (headers, SSRF, & rate-limit keys verified) | `backend/src/hardening.rs`, `backend/src/lib.rs`, `22-pwa-api-and-abuse-security.md` | `only_liveness_is_exposed`, `security_headers_url_redaction_ssrf_and_composite_rate_limits` | Upstream Expo CLI transitive `zod` advisory (`GHSA-mrhx-c7xc-m3w5`) and real-device browser cache/resume audit |
| **I10** | In progress (scoped FHIR R4 boundary assembly verified) | `backend/src/fhir.rs`, `11-fhir-interoperability.md`, `23-files-exports-and-integrations.md` | `fhir_r4_boundary_assembly_reference_scope_and_mass_assignment_guards` | Partner hospital FHIR endpoint selection and SMART App Launch scope agreement |
| **I11** | In progress (coordinated restore reconciliation verified) | `backend/src/recovery.rs`, `scripts/check-local-stack.mjs`, `25-availability-and-disaster-recovery.md`, `26-deployment-and-supply-chain.md` | `Vault A high-water counter monotonic restore reconciliation` (`check-local-stack.mjs`) and `reconcile_band_restore` unit/E2E tests | Hospital RTO/RPO acceptance, cloud staging load test, and paper downtime drill |
| **I12** | In progress (synthetic E2E matrix passing; pilot gated on hospital owners) | `backend/src/lib.rs`, `15-testing-and-rollout.md`, `29-readiness-and-open-decisions.md` | `end_to_end_synthetic_journey_matrix_and_coordinated_restore` (14/14 Rust tests + 10/10 live stack checks passing) | Formal hospital clinical safety, privacy/DPO, and physical device acceptance sign-off before any real-patient use |
