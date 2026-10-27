# 24. Audit behavior, operational observability, and incident response

**Status:** In progress; `backend/src/encounter.rs` enforces fail-closed clinical reads and writes when durable audit storage is unavailable (`AuditStorageHealth::Outage`) and provides minimized privacy auditor projections (`MinimizedAuditEventProjection`) omitting clinical payloads. Hospital incident ownership and breach-reporting sign-off remain open.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I07

---

Distinguish clinical provenance, disclosure audit, authentication/security events, administrative changes, and technical diagnostics. Each has its own readers and retention. Audit projections must omit clinical payloads and must not create a second broadly accessible identity-to-clinical map. Minimize exact IP/location data and justify retention.

Define the audit-outage policy before launch. Clinical writes must not commit without their required audit event, and protected reads must not release data without their required durable disclosure record. Never silently replace durable recording with best-effort logging. Define Tier 1 recording separately, including what happens if public-access event delivery fails, while retaining proof/status checks and the approved emergency scope. A public emergency page cannot silently become a full-record fallback during an outage.

Use bounded audit queues and storage-capacity alerts. If cryptographic chaining/checkpoints are used, state what tampering they detect and protect checkpoints independently; a chain and its keys under one compromised administrator do not establish immutability. Test missing events, duplicate events, rollback to an older archive, storage exhaustion, and verification-key compromise. Define how privileged investigations and audit exports are themselves audited.

Monitor latency percentiles, authorization failures, replay rejections, PIN attack indicators, key/decrypt errors, queue age, audit failures, certificate/key expiry, database saturation, and restore health without PHI in labels. Alert on actionable deviations; a high rejection rate alone is not proof of an attack. Assign an on-call owner, severity policy, response expectations, and runbooks for each critical alert.

Prepare incident procedures for account takeover, staff misuse, wrong-patient disclosure, lost/compromised tag keys, dependency compromise, ransomware, notification misdelivery, and cloud-account compromise. Include containment, session/key revocation, evidence preservation, hospital continuity, communication authority, recovery verification, and lessons learned. Use synthetic tabletop exercises before a live pilot.

The privacy lead must document whether a security incident triggers mandatory breach notification and the applicable clock, recipients, content, and escalation. NPC guidance describes a 72-hour notification window where notification is required; do not assume every event has identical legal treatment or postpone investigation until certainty. Confirm the current rules and hospital responsibilities. References: [NPC breach reporting](https://privacy.gov.ph/pips-and-pics/breach-reporting/) and [NPC reporting procedure guidance](https://privacy.gov.ph/exercising-breach-reporting-procedures/).
