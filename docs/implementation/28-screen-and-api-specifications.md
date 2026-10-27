# 28. Implementation specification packet and skill updates

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I03

---

Before implementing a screen, maintain a catalog entry with its role, permitted hospital/patient scope, entry and exit conditions, required fields, field-level disclosure, main action, validation, loading/empty/error/denial states, expiry/revocation behavior, audit event, accessibility behavior, and acceptance tests. Include safe behavior for direct URLs, refresh, browser back, new tabs, and account switching. Connect each entry to the permission catalog rather than scattering role checks through components.

Prepare versioned API/OpenAPI contracts and state-transition specifications with explicit error types, pagination, idempotency, optimistic concurrency, clock semantics, and resource limits. Identify the pilot's actual feature list and disabled routes; a mock screen or placeholder button must not imply production capability. Finish the care-team policy before implementing its protected data paths.

Extend the skill-alignment work in section 14 to cover this review: durable counter acceptance and restore, encryption envelopes/nonces, clinical safety and provenance, identity/consent scope, background-service capabilities, browser leakage, file/export security, operational recovery, and release gates. Keep detailed, changeable procedures in referenced documents rather than duplicating the entire plan inside every `SKILL.md`. Update affected mirrors and instructions in the same future implementation change. This review changes planning documentation only; skill enforcement remains uninstalled.
