# 14. Skill alignment and change governance

**Status:** The instruction rule and mirrored-file check are in place. Semantic alignment remains a review responsibility; the remaining skill revisions below are not complete.

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

Known alignment work before dependent implementation:

| Reference | Required change |
| --- | --- |
| `AGENTS.md` and `CLAUDE.md` | PWA-first direction and skill-alignment rule recorded; detailed role boundaries remain in the implementation specifications. |
| Frontend skill | Hospital context and preview limitations recorded; update with approved workflows as they are implemented. |
| Split-vault skill | Define RBAC conditions, capability expiry/revocation, and narrowly scoped internal processing. |
| FHIR skill | Remove its extra public chronic-alert field and align resource permissions/profiles with the project invariant. |
| NFC skill | Add durable replay recovery, ingestion/retry behavior, PWA limitations, and complete NDEF byte budgeting. |
| Deployment skill | Resolve Firestore versus required PostgreSQL. Replace guaranteed latency claims with measured targets and load tests. |
| Logo skill | Add the selected identity and verified usage references after visual review. |
