# 16. Gap assessment and decision priorities

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I01

---

The original plan describes the intended happy paths and several failure states. It does not yet define a complete operating contract for security-critical boundaries. This review adds the requirements below without changing the existing project invariants or enabling new roles.

Priority meanings:

- **P0:** Resolve and verify before any pilot using real patient data, or before enabling the affected capability if explicitly deferred.
- **P1:** Required before broader hospital production use; may be excluded from a narrowly documented pilot only if the affected feature is disabled and hospital operations have an accepted alternative.
- **P2:** Later improvement with a recorded boundary; cannot be silently advertised as supported.

| ID | Gap | Priority and gate | Accountable responsibility | Required evidence |
| --- | --- | --- | --- | --- |
| G01 | Cross-hospital ownership, sharing, and sensitive-record scope | P0 before longitudinal history | Hospital clinical lead and privacy lead | Approved data-sharing and segmentation policy plus denial tests. |
| G02 | Patient self-access, representatives, incapacity, and recovery authority | P0 before patient onboarding | Hospital privacy and identity leads | Identity/representation matrix and supported patient-rights workflow. |
| G03 | Public emergency disclosure and third-party ICE information | P0 before band issuance | Privacy and clinical leads | Separate disclosure choices, provenance rules, and physical-band update process. |
| G04 | End-to-end trust boundaries and compromised-operator model | P0 before backend security implementation | Security and architecture leads | Data-flow threat model, trust assumptions, and residual-risk record. |
| G05 | Consistent durable replay acceptance and disaster recovery | P0 before live NFC verification | Backend and operations leads | Concurrency, crash, failover, and restored-backup replay tests. |
| G06 | Cryptographic key, nonce, and identifier lifecycle | P0 before encrypting real data | Security and operations leads | Encryption format, key inventory, rotation and restore drill. |
| G07 | Exact session, token-validation, and in-flight revocation semantics | P0 before clinical authorization | Identity and backend leads | Protocol specification and race-condition tests. |
| G08 | Browser, PWA, log, and telemetry leakage | P0 before publishing patient routes | Frontend and security leads | Browser lifecycle and sensitive-data capture tests. |
| G09 | Wrong-patient, stale-data, medication, and amendment hazards | P0 before encounter pilot | Clinical safety owner | Hazard log and clinician acceptance scenarios. |
| G10 | Worker capabilities and audit durability under failure | P0 before asynchronous clinical processing | Backend and security leads | Narrow service permissions and fault-injection results. |
| G11 | Hospital downtime and coordinated restoration | P0 before real-patient pilot | Hospital operations and service operator | Downtime workflow, approved recovery objectives, and restore rehearsal. |
| G12 | API, search, and dependency resource limits | P0 for exposed endpoints | Backend and operations leads | Abuse tests and measured capacity/cost limits. |
| G13 | File uploads, exports, printing, and imported content | P0 before enabling each feature | Security, privacy, and integration leads | Feature-specific threat tests and disclosure controls. |
| G14 | Build, deployment, migration, and dependency integrity | P0 before live deployment | Engineering and operations leads | Reproducible release record, access controls, and rollback rehearsal. |
| G15 | Incident handling, retention, deletion, and supplier responsibilities | P0 before real-patient pilot | Privacy and service owners | Responsibility matrix, retention schedule, and incident exercise. |
| G16 | Exact nurse/lab/pharmacy scope and clinical handover | P1; disabled until approved | Clinical governance lead | Revised capability model, skills, and profession-specific tests. |
| G17 | FHIR search, terminology, external references, and partner reconciliation | P0 before partner access | Integration and clinical leads | Contract tests and explicit supported operations. |
| G18 | Complete screen/action/state specification | P0 before affected screen implementation | Product and design leads | Screen catalog linked to permissions and acceptance tests. |
| G19 | Device usability, physical band suitability, and support accessibility | P0 for selected pilot devices/bands | Hospital operations and design leads | Real-device/physical validation and supported-device list. |
| G20 | Capacity, budget, ownership, and support coverage | P0 before promising service levels | Product, hospital, and service owners | Sizing assumptions, cost scenarios, escalation rota, and accepted targets. |

These are responsibilities to assign, not a claim that people or teams are already appointed. Record a named owner, decision date, evidence link, and status for each item before its gate. An automated check or a written policy alone does not close a clinical or operational risk.
