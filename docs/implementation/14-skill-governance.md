# 14. Skill alignment and change governance

**Status:** Baseline semantic alignment across `AGENTS.md`, `CLAUDE.md`, and all six mirrored workspace skills is in place. Ongoing semantic review remains required whenever features, security boundaries, or visual identity artifacts change.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I04

---

**Workspace rule:**

1. Before changing a feature, identify and read the workspace skills governing its UI, permissions, data access, security, interoperability, hardware, and deployment.
2. Treat approved product decisions and project security invariants as constraints. A skill cannot silently override them.
3. When a change alters behavior, permissions, data boundaries, supported platforms, or acceptance criteria, update the affected skill sections and referenced specifications in the same change. Keep corresponding `.agents/skills` and `.claude/skills` copies synchronized while both are maintained.
4. Identify the product decision, user journey, affected skills, permission or data-boundary changes, and verification evidence.
5. Resolve contradictions before implementing dependent behavior. Do not weaken an invariant merely by rewriting a skill.
6. Make narrow revisions. Unrelated wording or styling changes do not require rewriting unrelated skills.
7. Before acceptance, verify synchronization, relevant denial cases, and agreement with the documented workflow. Automated checks can detect structural drift; reviewers must assess semantic consistency.

`AGENTS.md` and `CLAUDE.md` contain the rule. Run `node scripts/check-skill-sync.mjs` to detect mismatched skill copies and project instruction files. This check cannot determine whether a skill semantically matches a feature or whether a required skill was updated; reviewers must verify those points.

Baseline alignment status and ongoing maintenance:

| Reference | Baseline status and ongoing requirement |
| --- | --- |
| `AGENTS.md` and `CLAUDE.md` | Aligned: PWA-first direction, PostgreSQL split-vault invariant, and skill-alignment rule recorded; detailed role boundaries remain in the implementation specifications. |
| Frontend skill (`frontend-design`) | Aligned: hospital context and synthetic preview limitations recorded; update as authenticated and clinical workflows are implemented. |
| Split-vault skill (`split-vault-typestate`) | Aligned: RBAC conditions, 15-minute `VerifiedClinicSession` expiry and transaction-boundary revocation, versioned AES-256-GCM envelope with AAD, and narrowly scoped outbox/worker boundaries recorded. |
| FHIR skill (`fhir-medical-api`) | Aligned: extra public `chronic_alerts` field removed; Tier 1 restricted to blood type, critical allergies, and ICE contact with provenance; Tier 2 boundary assembly and default-deny route posture recorded. |
| NFC skill (`nfc-sun-crypto`) | Aligned: Vault A durable replay acceptance authority, Redis coordination role, single-consumption bound tap context, complete 256-byte NDEF budget (no silent allergy truncation), and PWA Web NFC limitations recorded. |
| Deployment skill (`firebase-deploy-scale`) | Aligned: Firestore/RTDB replaced with required PostgreSQL split-vault and Redis coordination boundaries; unverified latency guarantees replaced with engineering targets requiring load and recovery tests. |
| Logo skill (`anti-slop-logo`) | Aligned: MedPulsePH Clasp direction (`13-logo-and-identity.md`) and provisional status of current preview marks recorded; update after the full 5-panel visual and physical review is completed. |
