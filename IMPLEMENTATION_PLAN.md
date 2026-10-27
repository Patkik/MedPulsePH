# MedPulsePH implementation plan

**Status:** The pilot-boundary draft and an early synthetic-data interface preview are in progress. Authentication, clinical/NFC functionality, and production security controls have not been implemented. The skill-alignment instruction and mirror check are installed; semantic alignment still requires review.

**Product direction:** An installable PWA for Android, iPhone, and PC using React Native for Web and a Rust backend. The interface should feel calm, clear, and reassuring. Native apps and a reader companion remain later capabilities where browsers are insufficient.

The implementation plan is divided into 29 focused specifications below. Their original section numbers are preserved for traceability. Each requirement has one maintained specification; update it there rather than copying requirements between iterations.

Start with the [12 iteration packages](docs/implementation/ITERATIONS.md). They define deliverables, dependencies, and completion checks. Consult the [risk and decision register](docs/implementation/16-risk-and-decision-register.md) before choosing work, and the [readiness requirements](docs/implementation/29-readiness-and-open-decisions.md) before enabling a feature or a real-patient pilot.

## Specification map

| Specification | Primary iteration |
| --- | --- |
| [1. Product outcomes and constraints](docs/implementation/01-product-scope.md) | I01 |
| [2. Target system architecture](docs/implementation/02-system-architecture.md) | I04 |
| [3. Users, workspaces, and information architecture](docs/implementation/03-users-and-workspaces.md) | I03 |
| [4. Authorization and role boundaries](docs/implementation/04-roles-and-permissions.md) | I02 |
| [5. Registration, login, and recovery flows](docs/implementation/05-registration-login-recovery.md) | I02 |
| [6. Dashboards and primary actions](docs/implementation/06-dashboards.md) | I03 |
| [7. NFC, band issuance, and emergency access](docs/implementation/07-nfc-bands-and-emergency.md) | I05 |
| [8. Replay protection and clinical authorization](docs/implementation/08-replay-and-clinical-authorization.md) | I05 |
| [9. Clinical workspace and data boundaries](docs/implementation/09-clinical-workspace-and-vaults.md) | I08 |
| [10. Administration, audit, and notifications](docs/implementation/10-administration-audit-notifications.md) | I07 |
| [11. FHIR and partner interoperability](docs/implementation/11-fhir-interoperability.md) | I10 |
| [12. UI design and resilience](docs/implementation/12-ui-design-and-resilience.md) | I03 |
| [13. Logo and identity brief](docs/implementation/13-logo-and-identity.md) | I03 |
| [14. Skill alignment and change governance](docs/implementation/14-skill-governance.md) | I04 |
| [15. Testing, rollout, and acceptance](docs/implementation/15-testing-and-rollout.md) | I12 |
| [16. Gap assessment and decision priorities](docs/implementation/16-risk-and-decision-register.md) | I01 |
| [17. Threat model and data-flow inventory](docs/implementation/17-threat-model.md) | I01 |
| [18. Patient identity, ownership, sharing, and clinical safety](docs/implementation/18-patient-sharing-and-clinical-safety.md) | I01 |
| [19. Identity, authentication, and authorization protocol details](docs/implementation/19-authentication-and-session-security.md) | I02 |
| [20. Durable state machines and cross-vault consistency](docs/implementation/20-state-machines-and-consistency.md) | I04 |
| [21. Cryptography, database isolation, and key operations](docs/implementation/21-cryptography-and-key-operations.md) | I04 |
| [22. Browser, PWA, API, and abuse hardening](docs/implementation/22-pwa-api-and-abuse-security.md) | I09 |
| [23. Attachments, exports, integrations, and notification safety](docs/implementation/23-files-exports-and-integrations.md) | I10 |
| [24. Audit behavior, operational observability, and incident response](docs/implementation/24-observability-audit-and-incidents.md) | I07 |
| [25. Availability, disaster recovery, and hospital downtime](docs/implementation/25-availability-and-disaster-recovery.md) | I11 |
| [26. Deployment, supply chain, migrations, and support operations](docs/implementation/26-deployment-and-supply-chain.md) | I11 |
| [27. Privacy lifecycle, contracts, and physical deployment](docs/implementation/27-privacy-and-physical-deployment.md) | I01 |
| [28. Implementation specification packet and skill updates](docs/implementation/28-screen-and-api-specifications.md) | I03 |
| [29. Readiness checklist and unresolved decisions](docs/implementation/29-readiness-and-open-decisions.md) | I12 |

## How to iterate

1. Select one package and read its linked specifications and applicable workspace skills.
2. Resolve the decisions needed for that package; assign an owner and record the evidence in the decision register.
3. Break its deliverables into one reviewable change at a time. Preserve the patient, hospital, consent, and session boundaries in every change.
4. Run the package-specific checks and update affected specifications and skills together when implementation changes their rules.
5. Record results and remaining dependencies in the iteration ledger. Move to dependent work only when its prerequisite is met.

Security, accessibility, privacy, auditing, and recovery are continuous requirements. The package covering their final verification does not postpone their implementation. A deferred capability stays disabled until its own release gate is satisfied.
