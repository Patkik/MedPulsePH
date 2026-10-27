---
name: firebase-deploy-scale
description: Architect, configure, and deploy the MedBand-Rust NFC platform on Firebase and Google Cloud Run for high availability and horizontal scalability. Use whenever configuring firebase.json, Firestore named databases (Vault A & Vault B), Firestore/RTDB security rules, Cloud Run containerization for Rust Axum, or Next.js Firebase App Hosting.
allowed-tools: Read Edit Grep Glob Bash(firebase *) Bash(gcloud *) Bash(docker *) Bash(cargo build*)
---

# Firebase Deployment & High-Scale Architecture Skill

## 1. Firebase Split-Vault Isolation (Vault A vs. Vault B)
- **Named Firestore Databases:** Use Firestore multi-database isolation within the project:
  - **Vault A (`vault-a-identity`):** Stores `patient_pii`, `nfc_tags`, `band_status`, and `argon2_pin_hash`.
  - **Vault B (`vault-b-clinical`):** Stores `encounters`, `allergies`, `prescriptions`, `lab_results`, and `immutable_audit_logs` keyed ONLY by `blind_patient_id` (HMAC-SHA256 UUID).
- **Zero Direct Client Access (`firestore.rules`):**
  - Lock down both vaults to public/client SDKs (`allow read, write: if false;`).
  - All reads and writes MUST flow through the Rust `axum` backend using least-privilege Google Cloud IAM service accounts (`IdentityVaultServiceAccount` vs `ClinicalVaultServiceAccount`).
- **Immutable Audit Logs:** Enforce append-only semantics on `immutable_audit_logs` (deny `update` and `delete` operations at the rule and IAM level).

## 2. Scalable Anti-Replay NFC Counter (Sub-10ms Atomic Transactions)
- Store NFC tap counters in **Firebase Realtime Database (RTDB)** at `/nfc_counters/{tag_uid_hex}` or a dedicated Firestore collection using atomic transactions.
- **Strict Monotonicity:** Inside a transaction, read `stored_counter`, verify `incoming_counter > stored_counter`, and write `incoming_counter`. Abort immediately if `incoming_counter <= stored_counter` to block cloned URL replay attacks across distributed instances.

## 3. Cloud Run + Firebase Hosting Rewrite Architecture
- **Stateless Rust Container (`Dockerfile`):**
  - Use a multi-stage Rust build (`cargo build --release`) targeting a minimal `gcr.io/distroless/cc-debian12` runtime image.
  - Keep the Rust `axum` service completely stateless so any Cloud Run instance can verify an NFC tap or clinic session.
- **Zero Cold-Start for Emergency Triage:**
  - Configure Cloud Run with `min-instances = 1` (minimum) and `--cpu-boost` so Tier-1 emergency taps (`GET /v1/tap` and `/api/v1/emergency/*`) respond in <100ms 24/7.
  - Set Cloud Run concurrency (`--concurrency 250`) to leverage `tokio` async I/O on each instance before horizontally scaling out.
- **Unified Routing (`firebase.json`):**
  - Route `/v1/tap/**` and `/api/**` to the `medband-rust-api` Cloud Run service.
  - Deploy the React / Next.js Doctor & Patient portals via **Firebase App Hosting** or **Firebase Hosting** global CDN with strict `Strict-Transport-Security` and `Content-Security-Policy` headers.

## 4. Deployment & Verification Checklist
1. Validate `firebase.json`, `firestore.rules`, and `database.rules.json` before deploying.
2. Verify no secrets (AES master keys, HMAC peppers, Service Account JSONs) are baked into the Docker image or frontend bundle—inject all cryptographic keys via **Google Cloud Secret Manager** mounted into Cloud Run.
3. Run `cargo clippy --workspace -- -D warnings -D clippy::unwrap_used` and `cargo test --workspace` prior to building the release container.
