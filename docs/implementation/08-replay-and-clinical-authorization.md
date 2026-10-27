# 8. Replay protection and clinical authorization

**Status:** In progress (I05 foundation). Durable high-water counter acceptance (`DurableBandReplayState`), Redis eviction recovery (preventing cache loss from authorizing old counters), single-consumption `AcceptedTapContext` issuance, and the Argon2id PIN handshake minting a 15-minute `VerifiedClinicSession` are implemented in `backend/src/nfc.rs` and `backend/src/auth.rs` with unit tests. PostgreSQL/Redis SQL persistence and HTTP endpoints remain unimplemented.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I05

---

The backend validates the NXP-defined payload and CMAC, checks band status, atomically enforces a strictly increasing counter, and issues a short-lived tap context. Maintain durable high-water state in Vault A; Redis can support a fast atomic path but must not be the sole surviving authority. Define restart, failover, and reconstruction behavior. An empty cache cannot authorize an old proof. Skipped counters are acceptable.

Handle concurrent taps, retries, refreshes, link previews, interrupted requests, and counter exhaustion. Choose a deliberate ingestion policy so a preview does not unnecessarily consume the user's workflow. Resume only an established bound context. Do not accept replayed proofs broadly for convenience. A captured unused proof can be submitted later; the counter is not a timestamp.

Clinical flow:

1. The doctor signs in; the backend verifies identity, clinic membership, and current eligibility.
2. The band is read and its proof consumed once.
3. The backend creates a short-lived tap context, separate from login and the clinical session.
4. The patient sees clinician, hospital, purpose, scope, and the 15-minute duration.
5. The patient approves using a private PIN or enrolled OTP, or declines.
6. The server creates a 15-minute session bound to the patient, clinician, hospital, permissions, and revocation state.
7. Every protected operation verifies those conditions.

Do not recheck the same raw counter on PIN submission. Prefer patient-facing entry or a paired patient phone; do not ask the patient to disclose a permanent PIN aloud. Use Argon2id plus server-side throttling, delays, and secure recovery. TOTP and SMS codes have different enrollment and recovery risks; specify them separately. PIN-first is a reasonable initial scope.

States: awaiting tap, verified tap, awaiting authorization, approved, declined, cancelled, expired, invalid proof, frozen band, revoked clinician, attempt limit, and connection failure. Decline is a valid patient choice, not a technical error. Full clinical break-glass access remains outside the current invariant and requires a separately approved policy.
