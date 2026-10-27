# 14. Skill alignment and change governance

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I04

---

**Proposed rule; not yet installed or enforced:**

1. Before changing a feature, identify and read the workspace skills governing its UI, permissions, data access, security, interoperability, hardware, and deployment.
2. Treat approved product decisions and project security invariants as constraints. A skill cannot silently override them.
3. When a change alters behavior, permissions, data boundaries, supported platforms, or acceptance criteria, update the affected skill sections and referenced specifications in the same change. Keep corresponding `.agents/skills` and `.claude/skills` copies synchronized while both are maintained.
4. Identify the product decision, user journey, affected skills, permission or data-boundary changes, and verification evidence.
5. Resolve contradictions before implementing dependent behavior. Do not weaken an invariant merely by rewriting a skill.
6. Make narrow revisions. Unrelated wording or styling changes do not require rewriting unrelated skills.
7. Before acceptance, verify synchronization, relevant denial cases, and agreement with the documented workflow. Automated checks can detect structural drift; reviewers must assess semantic consistency.

Use `.agents/skills` as the proposed maintained workspace source, with `.claude/skills` synchronized copies. Update `AGENTS.md` and `CLAUDE.md` when implementing this rule; a statement in this plan does not itself enforce it.

Known alignment work before dependent implementation:

| Reference | Required change |
| --- | --- |
| `AGENTS.md` and `CLAUDE.md` | Record the PWA-first direction, role boundaries, and skill alignment rule. |
| Frontend skill | Add hospital context, approved workflows, accessibility, and clinical states. |
| Split-vault skill | Define RBAC conditions, capability expiry/revocation, and narrowly scoped internal processing. |
| FHIR skill | Remove its extra public chronic-alert field and align resource permissions/profiles with the project invariant. |
| NFC skill | Add durable replay recovery, ingestion/retry behavior, PWA limitations, and complete NDEF byte budgeting. |
| Deployment skill | Resolve Firestore versus required PostgreSQL. Replace guaranteed latency claims with measured targets and load tests. |
| Logo skill | Add the selected identity and verified usage references after visual review. |
