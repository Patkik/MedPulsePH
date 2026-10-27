# 21. Cryptography, database isolation, and key operations

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I04

---

Specify a versioned encryption envelope: algorithm, key reference/version, nonce, ciphertext, authentication tag, and authenticated context. Bind ciphertext to its patient/resource identity, field purpose, and schema/version as appropriate through authenticated associated data. Reject authentication failures without exposing plaintext or verbose cryptographic internals.

For AES-256-GCM, use a reviewed 96-bit nonce strategy with uniqueness under each key across instances, retries, restored backups, and migrations; define invocation limits and rotation. Re-encryption uses a fresh nonce. Do not make a timestamp, patient identifier, or restart-reset counter the unreviewed nonce generator. Keep encryption and key-wrapping keys separate. Reference: [OWASP cryptographic storage guidance](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html).

Maintain a key inventory and procedures for generation, access, rotation, compromise response, retirement, backup decryption, and eventual destruction. Prove that restored data remains decryptable while obsolete keys cannot remain usable indefinitely without policy. Tag-key changes may require physical access; define replacement/recall when remote rotation cannot repair compromised bands. HMAC identifier rotation needs a stable mapping/migration plan rather than creating a second clinical identity.

Separate runtime, migration, backup, audit-writing, and operational database roles. Restrict network access and encrypted database connections. Runtime credentials do not grant schema changes or audit alteration. If PostgreSQL row-level security is used as defense in depth, test owner/bypass roles and connection-pool tenant-context cleanup; never let a previous request's session setting grant the next request access.

Document which fields are encrypted versus indexed plaintext and the resulting metadata exposure. Search and reporting must not quietly produce a new unencrypted clinical database. Keep internal patient identifiers, names, and raw tag UIDs out of unnecessary cache keys, traces, and operational metrics. Control plaintext exposure through memory diagnostics, core dumps, support bundles, and temporary files.
