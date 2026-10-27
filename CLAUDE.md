# MedBand-Rust: Zero-Trust NFC Medical Architecture

## Project Overview
A high-security healthcare wristband platform built in Rust. Physical 13.56 MHz NFC wristbands (NTAG 424 DNA / NTAG215) act as cryptographic pointers to a centralized, FHIR-compatible REST API with an ATM-style PIN/OTP handshake.

## Tech Stack
- **Backend:** Rust (Edition 2024), `axum`, `tokio`, `sqlx` (PostgreSQL), `redis`
- **Cryptography:** `aes-gcm` (AES-256-GCM vault cipher), `aes` + `cmac` (NTAG 424 DNA SUN verification), `argon2` (PIN hashing), `subtle` (constant-time comparisons)
- **Frontend Portals:** React / Next.js, TypeScript, Tailwind CSS
- **Hardware Target:** 13.56 MHz ISO/IEC 14443-A NFC (NXP NTAG 424 DNA & NTAG215)

## Non-Negotiable Security Invariants
1. **No Raw Medical Data on Chip:** Only store the dynamic NDEF URL (Record 1) and minimal offline triage text (Record 2) on the wristband.
2. **Two-Tier Access Control:**
   - **Tier 1 (Emergency Tap):** Public read-only access to Blood Type, Critical Allergies, and ICE Contact.
   - **Tier 2 (Clinic Checkup):** Requires BOTH Doctor JWT + Patient 6-digit PIN/OTP to mint a 15-minute `VerifiedClinicSession`.
3. **Split-Vault Isolation:** Vault A (Identity PII) and Vault B (Clinical EHR) use separate PostgreSQL connection pools (`IdentityPool` vs `ClinicalPool`).
4. **No `unwrap()` in Production Paths:** All handlers and crypto routines must return typed errors via `thiserror` and map cleanly to Axum `IntoResponse`.

## Commands
- Typecheck & SQLx verify: `cargo check --workspace`
- Lint (strict): `cargo clippy --workspace -- -D warnings -D clippy::unwrap_used`
- Run crypto & handshake tests: `cargo test --workspace`
