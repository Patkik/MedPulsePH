# 20. Durable state machines and cross-vault consistency

**Status:** In progress (I04 foundation). Compile-time `ClinicalPool` capability gating (`authorize_clinical_access`), transaction-boundary session revalidation, scoped idempotency verification (`ScopedIdempotencyRecord`), optimistic concurrency version checks (`ResourceVersion`), and PostgreSQL schemas/transactional repositories (`backend/sql/vault_a_identity.sql`, `backend/sql/vault_b_clinical.sql`, `IdentityPool::accept_tap_proof_durable`, and `AuthorizedClinicalAccess` audit+record transactions) are implemented. Outbox worker loops and HTTP endpoints remain in progress.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I04

---

Write state-transition specifications before their handlers. Include allowed actor, prerequisites, transaction boundary, idempotency, audit event, compensation, and recovery for each transition.

| Entity | Additional states/conditions to specify |
| --- | --- |
| Staff membership | Invited, accepted pending checks, active, suspended, expired, terminated; credential and membership state are distinct. |
| Patient identity link | Proposed, evidence pending, verified, disputed, superseded; no chart access from a proposal. |
| Band | Inventory, provisioning, verification pending, active, frozen, replacement pending, retired, failed; define who may unfreeze and which terminal states cannot return. |
| Authorization request | Issued, paired, proof accepted, patient pending, approved, declined, expired, consumed, revoked. |
| Encounter | Draft, reviewed, finalized, amended, entered-in-error; expiry locks access rather than erasing the record. |
| Export | Requested, authorized, preparing, ready, expired, cancelled, failed, retrieved; revalidate before release. |
| Background delivery | Pending, leased, delivered, retryable failure, dead-lettered; delivery is not patient receipt. |

Use idempotency keys scoped to actor, hospital, operation, and payload identity for high-impact writes. A repeated request returns the original result where authorized; the same key with a different payload is rejected. Set retention and retry semantics. Use resource versions/ETags or an equivalent optimistic-concurrency contract for updates. Do not rely on disabled buttons alone.

For replay acceptance, the durable Vault A transaction is the acceptance authority. It must atomically advance the counter and create the recoverable accepted-proof context. Redis is an optimization/coordination layer with defined invalidation, not a second independent acceptance decision. Preserve strict monotonicity and fail safely if consistent durable acceptance cannot be established. Document crashes before commit, after commit before response, and during cache update; resume only using the authorized bound context.

Do not attempt an ordinary SQL transaction across the two isolated vaults. Use approved state transitions and durable outbox/inbox events for emergency-projection updates, notifications, and other coordination. Record event identity, origin, minimum payload, delivery status, retry limit, and reconciliation. Consumers must be idempotent and handle duplicates and out-of-order messages. Define how identity corrections and clinical-record corrections propagate without briefly attaching information to the wrong patient.

Read replicas and caches must not supply stale authorization, revocation, band status, or replay state. Any cached emergency projection needs a version and invalidation contract. Test at-least-once event delivery and database/cache failure independently.
