# I04 local Docker development

This stack is for development and synthetic data only. It is not a deployment architecture for real patient information. Authentication, patient consent, NFC verification, clinical queries, encryption, auditing, Redis replay protection, and migrations are not implemented.

## Local services

- `identity-db` and `clinical-db` are separate PostgreSQL 17 containers with separate volumes, internal networks, owner credentials, and non-superuser runtime accounts.
- `redis` is on its own internal network. It is not yet used by the API and is not a durable replay authority.
- `api` connects the two distinct Rust pool types and exposes only `GET /health/live` on `127.0.0.1:8080`. The endpoint confirms process liveness, not database readiness.
- Database and Redis ports are not published to the host. The API joins all three internal networks and a separate edge network for its loopback-published port. The edge network also permits outbound access in this local setup. The API process remains a shared trust boundary.

## Start

1. Install and start Docker Desktop and ensure Node.js is available.
2. Run `node scripts/setup-local-env.mjs` from the repository root. The script creates random local credentials in the ignored `.env` file without overwriting an existing file or printing secrets.
3. Run `docker compose up --build -d`.
4. Run `docker compose ps` and request `http://127.0.0.1:8080/health/live`. A `204` response means the API process is alive; it is not a security or clinical readiness certification.
5. Run `node scripts/check-local-stack.mjs` to verify the local runtime roles, a cross-vault denial, network separation, and the health-only API. The Docker daemon must be accessible to this process.
6. Run `docker compose down` when finished. Named database volumes remain for the next run. Removing volumes deletes their data and must be a deliberate separate action.

The generated passwords are hexadecimal so they can be safely embedded in local connection URLs. Compose environment variables and container administration expose credentials to local Docker administrators. Internal database transport is plaintext. File permissions on `.env` depend on the host OS. None of these choices is acceptable as a production secrets or transport design.

PostgreSQL's initialization script runs only when its volume is first initialized. Editing `.env` later does not rotate passwords in an existing database. Do not delete or reset volumes as a troubleshooting shortcut; follow an explicit development reset procedure after checking for data to preserve.

## Verification and remaining work

Run Rust checks inside the optional tools container when Rust is not installed on the host:

```text
docker compose --profile tools build tools
docker compose --profile tools run --rm tools check --workspace --locked --offline
docker compose --profile tools run --rm tools clippy --workspace --locked --offline -- -D warnings -D clippy::unwrap_used
docker compose --profile tools run --rm tools test --workspace --locked --offline
```

The tool container uses a named build-cache volume and no network when running checks. Rebuild the tools image after source or dependency changes. Record results before marking an iteration complete.

On 2026-10-28, the locked offline workspace check, strict Clippy check, and workspace tests passed in the tools container. The test suite verifies:
- Only `GET /health/live` is exposed (`204`) and unfinished clinical, login, emergency, and tap routes return `404`.
- Default-deny RBAC (`backend/src/auth.rs`) blocks cross-hospital access, self-approved credentials, non-clinical roles (`HospitalAdmin`, `PrivacyAuditor`, `Reception`, `BandOfficer`, `Patient`, `PlatformOperator`), disabled care-team clinical access (`Nurse`, `LaboratoryStaff`, `PharmacyStaff`), and unimplemented exports.
- The dual-custody handshake (`auth::handshake::mint_verified_clinic_session`) consumes an `AcceptedTapContext` once, verifies a 6-digit PIN via Argon2id with attempt lockout, mints a 15-minute `VerifiedClinicSession`, and enforces transaction-boundary revalidation for expiry, revocation, band freeze, and scope mismatches.
- Cryptographic primitives (`backend/src/crypto.rs`) derive constant-time `BlindPatientId` identifiers via keyed HMAC-SHA256, bind AES-256-GCM ciphertexts to `(BlindPatientId, ResourcePurpose, schema_version)` via AAD, enforce per-key nonce invocation budgets, and support key rotation and fresh-nonce re-encryption.
- NFC SUN verification, NDEF budgeting, and durable replay state (`backend/src/nfc.rs`) verify NTAG 424 DNA AES-128 PICCData and 8-byte truncated CMAC in constant time, enforce the 256-byte dual-record NDEF budget without truncating allergies, reject replays even after simulated Redis eviction, enforce `Active`/`Frozen`/`Retired` band transitions, and restrict Tier-1 emergency triage serialization to blood type, critical allergies, and ICE contact with provenance.
- Vault capability and consistency contracts (`backend/src/vault.rs`) gate `ClinicalPool` behind `authorize_clinical_access`, verify scoped idempotency payloads, and reject stale resource versions.
The local stack check passed for both non-superuser runtime accounts, cross-vault account denial, bidirectional database-network separation, API liveness, and the disabled clinical route. These checks do not prove production security or implement live HTTP clinical routes.

This is a small I04 foundation. Database roles, schema migrations, identity and clinical authorization, durable audit, key operations, replay acceptance, backup/restore, and production deployment are still required. Do not interpret the separate pool types or healthy containers as complete vault isolation.
