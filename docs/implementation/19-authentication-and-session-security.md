# 19. Identity, authentication, and authorization protocol details

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I02

---

### Identity provider and credentials

Specify the chosen identity provider, supported hospital federation, account-linking proof, verified issuer, and immutable subject identifier. Do not merge accounts merely because two providers return the same email address. Bind hospital membership and clinical eligibility in the application's trusted directory rather than accepting an arbitrary role claim from a browser or unapproved identity provider.

Use an authorization-code flow with PKCE, transaction-bound state/nonce as appropriate, exact registered redirects, and issuer validation. Disallow arbitrary redirects. Establish required MFA for all staff and separate assurance for privileged actions. Prefer phishing-resistant authenticators where feasible; document supported fallback and recovery. Doctor login factors and patient PIN/OTP are separate controls for separate people, not interchangeable MFA factors for one account. Reference: [OAuth security BCP, RFC 9700](https://www.rfc-editor.org/info/rfc9700/).

For JWTs, specify accepted algorithms, issuer, audience, token purpose, subject, expiry, not-before behavior, and bounded clock skew. Never accept unsigned tokens or use an untrusted token-supplied URL to retrieve arbitrary signing keys. Distinguish ID tokens, doctor API tokens, invitation proofs, pairing contexts, and clinical capabilities so one cannot substitute for another. Plan signing-key rotation, bounded key-cache behavior, and emergency revocation. Reference: [JWT BCP, RFC 8725](https://www.rfc-editor.org/info/rfc8725/).

Use independent secrets for PIN hashing peppers where selected, token signing, tag verification, vault encryption, and clinical identifier derivation. Keep patient PINs out of login and analytics flows. Benchmark Argon2id on the deployment target and set concurrency limits; do not use an unreviewed library default as the complete operational policy.

### Timers and replay boundaries

Define separate policies for staff login absolute/idle lifetime, patient login, privileged reauthentication freshness, invitation expiry, pairing expiry, tap-context expiry, PIN/OTP attempt window, and the fixed 15-minute clinical session. Changing a login or renewing an OAuth token does not extend the clinical session.

All expiry is enforced server-side. Synchronize service clocks, monitor drift, and define failure behavior. TOTP windows must be bounded; consume accepted codes to prevent reuse under the defined policy. An OTP must be bound to its intended patient and request where the selected scheme permits it. Record the limitations of time-only OTPs rather than claiming full transaction binding.

Clinical contexts include patient, clinician, hospital, action/resource scope, authorization event, issue/expiry times, and an invalidation version or equivalent. Use high-entropy server-issued references, rotate sessions after authentication/privilege changes, and keep raw secrets out of stored audit correlation fields. An explicit chart context identifies the tab/encounter but is not trusted without checking the authenticated session.

### Revocation and concurrent operations

**Decision required:** Define the transaction boundary for permission checks and the maximum permitted revocation propagation time. Specify which in-flight operations can complete and how that behavior is reported.

Use a consistent revocation/version check at the relevant database transaction boundary before committing a clinical mutation. Define serialization with a simultaneous freeze, consent withdrawal, staff suspension, or credential expiry. Prevent an action authorized in one tab from being finalized under another patient's context. A previously rendered or downloaded response cannot be recalled; the service can stop further disclosures and the UI can clear its view.

Test changing privileges while reads, saves, exports, streams, and background jobs are active. Do not rely on a self-contained 15-minute JWT alone if immediate revocation is part of the product promise.
