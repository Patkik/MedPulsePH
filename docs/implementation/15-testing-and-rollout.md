# 15. Testing, rollout, and acceptance

**Status:** In progress; automated synthetic verification (`cargo check --workspace`, strict `cargo clippy --workspace -- -D warnings -D clippy::unwrap_used`, 14 unit and end-to-end tests in `backend/src/lib.rs`, `node scripts/check-local-stack.mjs`, `node scripts/check-skill-sync.mjs`, and `npm --prefix app run typecheck` / `lint`) passes across `I01`–`I12`. Real-device NFC hardware usability and hospital clinical/privacy governance sign-off remain required before any live-patient pilot.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I12

---

Use synthetic patients until operational and security controls are ready for a controlled pilot. Run required Rust commands once implementation exists: `cargo check --workspace`, `cargo clippy --workspace -- -D warnings -D clippy::unwrap_used`, and `cargo test --workspace`.

Required security and authorization coverage:

- NXP vectors, malformed payloads, invalid MACs, old and concurrent counters, skipped counters, Redis recovery, and delayed first use of a captured proof.
- PIN guessing limits, OTP reuse, recovery, and wrong actor, clinic, patient, or session binding.
- Expiry, freeze, suspension, and revocation during an active operation.
- Reception, administrators, auditors, and unsupported roles denied clinical access; cross-hospital and cross-patient requests denied.
- URL manipulation, stale tabs, changed patient identifiers, bulk operations, exports, printing, attachments, and integration requests.
- Draft and finalized-record conflicts, duplicate submission, audit consistency, notifications, and outage recovery.
- Service workers excluding protected responses; logout, role switch, and patient switch clearing prior chart content.
- FHIR validation, terminology, pagination, version conflicts, and a real partner workflow.
- Backups, restoration, key availability, and independently protected audit evidence.

Run usability sessions with patients, doctors, reception staff, nurses, administrators, and auditors. Test on real Android phones, iPhones, and PC browsers; emulation cannot establish NFC compatibility. Measure task completion, wrong-patient corrections, authorization confusion, interruptions, and recovery. Set performance thresholds from pilot evidence.

| Stage | Deliverable | Exit criterion |
| --- | --- | --- |
| Architecture baseline | Role catalog, permission matrix, journeys, threat model, data boundaries, skill alignment plan. | No unresolved contradiction in the core access model. |
| Experience specification | Screen inventory, navigation, state diagrams, logo directions, patient and staff lifecycle. | Every primary action has a permission and failure path. |
| Hardware feasibility | Real NTAG configuration, test vectors, link behavior, offline reading, full NDEF budget. | Demonstrated on target devices. |
| Identity and administration | Hospital onboarding, invitations, credentials, MFA, suspension. | No self-escalation or cross-hospital access. |
| Patient and band enrollment | Identity verification, linking, provisioning, replacement, recovery. | Correct binding and replacement demonstrated. |
| Backend foundation | Split vaults, keys, tap verification, durable replay state, sessions, auditing. | Core failure and isolation tests pass. |
| Authorized encounter slice | Tap, consent, chart read, draft, finalization, audit, patient activity. | Complete synthetic encounter passes positive and denial tests. |
| Portable PWA | Responsive workspaces, installation, pairing, accessibility, caching and interruption controls. | Passes agreed phone and PC matrix. |
| Care-team and interoperability | Approved nursing and partner workflows, revised policies, validation. | Policies and relevant skills updated before enabling access. |
| Controlled pilot | Monitoring, training, operational support, recovery exercises. | Hospital acceptance criteria met. |

Initial scope includes the authenticated NTAG 424 workflow, narrow emergency summary, patient band controls, doctor eligibility, patient PIN authorization, 15-minute sessions, basic encounters, durable audit, phone-to-PC pairing, and the PWA. Native apps, broad hospital integrations, full SMART onboarding, offline clinical editing, and full-record emergency override require separately specified later phases.

Before real patient data, establish the hospital and operator responsibilities, lawful processing basis, emergency-disclosure policy, retention, recovery, incident response, and processor agreements. Review the [Philippine Data Privacy Act](https://privacy.gov.ph/data-privacy-act/) with the responsible privacy and legal stakeholders.
