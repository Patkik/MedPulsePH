---
name: firebase-deploy-scale
description: Architect, configure, and deploy the MedPulsePH PWA and Rust Axum backend with PostgreSQL split-vault isolation, Redis coordination, and cloud hosting. Use whenever configuring deployment files, containerization, hosting rewrites, or operational scaling and recovery checks.
allowed-tools: Read Edit Grep Glob Bash(firebase *) Bash(gcloud *) Bash(docker *) Bash(cargo build*)
---

# Deployment, Hosting & Operational Scaling Skill

## 1. PostgreSQL Split-Vault Isolation (Not Firestore)
- **Authoritative Data Stores:** In accordance with project security invariant #3 (`AGENTS.md`), Vault A (`IdentityPool`) and Vault B (`ClinicalPool`) MUST use separate **PostgreSQL** databases (`sqlx`), distinct credentials, isolated network paths, and independent backup access—never Cloud Firestore or Firebase Realtime Database for vault or replay authority.
- **Role Separation:** Separate runtime, migration, audit-writing, backup, and deployment database identities (`docs/implementation/21-cryptography-and-key-operations.md`, `26-deployment-and-supply-chain.md`). Runtime accounts (`identity_app`, `clinical_app`) must never hold superuser, schema-DDL, or audit-truncation privileges.
- **No Direct Client Database Access:** Browser/PWA clients never connect directly to PostgreSQL or Redis; all requests pass through the Rust `axum` API.

## 2. Durable Replay Authority vs. Redis Coordination
- **Vault A Durable Authority:** NFC tap counter advancement and accepted-proof tap contexts are committed durably in **Vault A PostgreSQL** (`docs/implementation/08-replay-and-clinical-authorization.md`, `20-state-machines-and-consistency.md`).
- **Redis Role:** Redis is used only for short-lived coordination, rate limiting, and fast-path caching with explicit invalidation. An evicted or restarted Redis instance must never permit a previously consumed NFC counter to be accepted again.

## 3. Container, PWA Hosting & Edge Routing
- **Stateless Rust Container (`backend/Dockerfile`):**
  - Build reproducibly with `--locked` dependencies and run as a non-root user (`medpulse`) with read-only root filesystem and dropped Linux capabilities.
  - Keep the Rust `axum` API stateless so horizontal instances do not rely on local in-memory session or replay state.
- **PWA & Static Asset Hosting:**
  - Serve the Expo / React Native for Web PWA over HTTPS with strict `Strict-Transport-Security`, `Content-Security-Policy`, framing protections, and `Cache-Control: no-store` on authentication, tap, and clinical responses (`docs/implementation/22-pwa-api-and-abuse-security.md`).
  - If Firebase Hosting or Cloud Run rewrites are used for the web frontend and API ingress, treat them strictly as static asset delivery and HTTPS reverse-proxy routing—not as a replacement for PostgreSQL Vault A/B.
  - Exclude clinical and authentication responses from service-worker caches.

## 4. Measured Targets, Recovery & Supply-Chain Verification
- **No Unverified Latency or SLA Claims:** Do not claim guaranteed `<100ms 24/7` response times or production readiness from configuration flags (such as `min-instances`) alone. Treat latency, concurrency, and cold-start figures as engineering targets that require load testing, abuse testing, and hospital acceptance evidence (`docs/implementation/25-availability-and-disaster-recovery.md`, `29-readiness-and-open-decisions.md`).
- **Coordinated Restore Safety:** Rehearse backup restoration across Vault A, Vault B, and key versions so restored backups never resurrect revoked sessions, unfrozen bands, or lower NFC tap counters.
- **Pre-Release Checks:**
  1. Ensure no secrets, master keys, HMAC peppers, or clinical datasets exist in source control, frontend bundles, or container layers.
  2. Run `cargo check --workspace --locked`, `cargo clippy --workspace --locked -- -D warnings -D clippy::unwrap_used`, and `cargo test --workspace --locked`.
