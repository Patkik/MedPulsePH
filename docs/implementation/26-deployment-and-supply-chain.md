# 26. Deployment, supply chain, migrations, and support operations

**Status:** In progress; local Docker isolation (`compose.yaml`), separate database owner vs. runtime credentials (`identity_owner`/`identity_app`, `clinical_owner`/`clinical_app`), non-root API container execution (`backend/Dockerfile`), and locked dependency manifests (`Cargo.lock`, `app/package-lock.json`) are enforced. Open supply-chain item: Expo CLI transitive `zod` advisory (`GHSA-mrhx-c7xc-m3w5`) tracked in `docs/implementation/I03-PREVIEW-EVIDENCE.md`.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I11

---

Separate development, test, staging, and production identities, databases, keys, logs, storage, and domains. Use synthetic or appropriately governed de-identified fixtures outside production. Do not copy live databases into a developer environment or preview deployment for convenience. Keep production secrets and clinical datasets out of source control, frontend bundles, container layers, build logs, and SQLx metadata.

Use reviewed dependency versions and lockfiles, vulnerability and secret scanning, a software bill of materials, provenance for releases, and a defined patch process. Review native/FFI components and cryptographic crates carefully; Rust does not remove unsafe-code or dependency risk. Limit CI tokens and cloud roles, protect release branches, and require independent review for privileged infrastructure and authorization changes. Authenticate deployment artifacts and maintain the ability to identify exactly what is running.

Give application runtime, migrations, backup, and deployment separate credentials. Rehearse expand/migrate/contract database changes, key-version migrations, and data backfills. Mixed old/new PWA clients must remain compatible during rollout or receive a safe upgrade requirement. Feature flags are server-controlled and not a substitute for authorization. Rollback must account for schema/data changes and service-worker caches; not every migration can be reversed by redeploying an old binary.

Readiness, liveness, and diagnostics endpoints expose no PHI or secrets. Configure least-privilege network ingress/egress, non-root/minimally privileged containers where supported, encrypted transport, certificate renewal, trusted-proxy header handling, and production-safe error output. Test the chosen hosting platform rather than copying generic deployment settings blindly.

Support uses an audited, time-bounded process with minimum metadata. No routine production impersonation, shared administrator password, or downloadable patient-filled debug bundle. If supervised access is ever needed, specify patient/hospital authority, scope, expiry, recording, and independent review. Restrict infrastructure emergency access separately and review its use afterward.
