# 25. Availability, disaster recovery, and hospital downtime

**Status:** In progress; `backend/src/recovery.rs` implements coordinated backup/restore reconciliation (`reconcile_band_restore`) ensuring restored Vault A snapshots never roll back NFC `high_water_counter` below protected recovery evidence, resurrect `Frozen`/`Retired` bands, or keep pre-restore clinical sessions active (`post_restore_revocation_version` bump), and `scripts/check-local-stack.mjs` verifies monotonic counter reconciliation in `identity_vault`. Hospital RTO/RPO sign-off and live downtime drills remain open.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I11

---

Approve measurable objectives for emergency access, login, clinical operations, audit ingestion, and notifications separately. Record expected hospitals, concurrent staff, tap rate, peak burst, clinical writes, attachment sizes, retention growth, and dependency quotas. Measure p95/p99 latency and failure rates under normal load, overload, dependency loss, and cold starts. Define an availability calculation and maintenance policy. No universal sub-100ms or zero-downtime claim is justified by selecting a hosting option.

**Decisions required:** Acceptable recovery point (data loss), recovery time, regional/data-location constraints, support hours, on-call coverage, and monthly cost limits. Include key-service, identity-provider, DNS, certificates, PostgreSQL, Redis, audit storage, and notification suppliers in failure tests. Define fail-safe behavior if each is unavailable. Do not build multi-region writes without a proven monotonic-counter and consistency design.

Back up Vault A, Vault B, key versions, audit evidence, and recovery metadata in separately protected storage. Test restoration in isolation. A coordinated restore must reconcile patient mappings, clinical identifiers, counter high-water marks, consent/revocation state, and delivered events. Restoring an old database must not resurrect old sessions, reactivate retired bands, or allow captured proofs. Invalidate prior online sessions during recovery and reconcile counter/band state from protected newer evidence; if safety cannot be established, keep affected bands unavailable pending verified recovery or replacement.

Separate ransomware-resistant copies and restore credentials from routine runtime access. Validate actual decryption, audit integrity, correct patient linkage, and accepted recovery objectives, not merely backup-file existence. Exercise restoration before the first live pilot and on an agreed recurring schedule.

Hospital downtime needs an owned manual procedure for patient identification, consultation documentation, contact escalation, and later reconciliation. Do not promise offline clinical chart availability in the PWA. If the 15-minute authorization limit or the lack of offline access creates an unacceptable ward workflow, resolve the product policy with clinical stakeholders rather than silently extending sessions or caching charts. A technical failure must not be mistaken for a medical instruction to delay necessary care.
