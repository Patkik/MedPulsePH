# 29. Readiness checklist and unresolved decisions

**Status:** In progress; engineering contracts, split-vault schemas, cryptographic envelopes, NTAG 424 DNA SUN verification, enrollment/pairing state machines, clinical encounter lifecycles, browser/API hardening, scoped FHIR R4 boundary assembly, and coordinated restore reconciliation are implemented and verified on synthetic fixtures (`I01`–`I12`). Live-patient readiness remains gated on external hospital clinical, privacy, procurement, and physical hardware sign-off.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I12

---

The first real-patient pilot requires evidence for all applicable P0 items, an agreed scope, and named clinical, privacy, engineering, and operational owners. Any deferred feature stays disabled. Record residual risks and the owner's acceptance; do not turn a failed test into a weaker skill requirement.

Minimum evidence:

1. Approved clinical/identity workflows and a hospital-reviewed hazard log.
2. A data-flow threat model and tested role/action/resource matrix, including denied requests.
3. Exact authentication, tap ingestion, consent, session, revocation, and retry specifications.
4. Key-management, encrypted-storage, counter-recovery, and coordinated-restore proof.
5. Browser/PWA leakage, malicious input, attachment/export, and API abuse results for enabled features.
6. Real-device NFC and accessibility tests, clinically reviewed encounter and recovery journeys, and supported-device documentation.
7. Durable audit and outbox fault tests; independently reviewed security findings resolved or explicitly accepted at the proper gate.
8. Measured load/cost results, accepted service/recovery objectives, hospital downtime procedures, and incident/restore exercises.
9. Privacy responsibilities, disclosures, patient-rights handling, retention, supplier terms, and clinical/regulatory determinations as applicable.
10. Synchronized architecture/skills, reviewed release artifacts, safe migration/rollback, training, and support ownership.

Decisions still to record: identity provider and MFA options; patient authentication and no-phone recovery; hospital record-sharing model; sensitive-record segmentation; guardian authority; PIN versus enrolled OTP scope; clinical team access; exact tap/pairing/idle lifetimes; allowed revocation propagation; tag/key provisioning design; encryption key granularity; coordination and replay restore design; file/export enablement; initial FHIR partners; hosting/data location; capacity/cost/recovery targets; and pilot device/hospital selection. The only fixed clinical lifetime in the current invariant is 15 minutes; other numbers must be selected, documented, and tested rather than implied.

This is a design gap review, not a completed penetration test, privacy assessment, clinical safety certification, or proof of production readiness. Revisit the threat model and gates when the deployment topology, partner interfaces, intended clinical use, or authorization policy changes.
