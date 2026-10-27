---
name: split-vault-typestate
description: Enforce compile-time Rust typestate security, ATM PIN/OTP dual-custody handshakes, and PostgreSQL Vault A (Identity PII) vs. Vault B (Clinical EHR) database isolation. Use whenever writing database migrations, SQLx queries, authentication middleware, or clinical record endpoints.
allowed-tools: Read Edit Grep Glob Bash(cargo check*) Bash(cargo clippy*)
---

# Split-Vault & Typestate Security Skill

## 1. Strict Database Isolation (Vault A vs. Vault B)
- **Vault A (`IdentityDbPool`):** Stores `patient_pii`, `nfc_tags`, `band_status`, and `argon2_pin_hash`.
- **Vault B (`ClinicalDbPool`):** Stores `encounters`, `allergies`, `prescriptions`, `lab_results`, and `immutable_audit_logs` indexed ONLY by `blind_patient_id` (an HMAC-SHA256 derived UUID).
- **Hard Rule:** NEVER write a SQL `JOIN` across Vault A and Vault B. NEVER store patient names, addresses, or raw NFC UIDs inside Vault B.

## 2. Compile-Time Typestate Pattern
- Every function that queries or mutates Vault B (full EHR) MUST require a reference to `&VerifiedClinicSession` in its function signature.
- `VerifiedClinicSession` fields must remain private to the `auth::handshake` module so no route handler can instantiate the struct without passing:
  1. Valid NFC Tap Counter check
  2. Valid Doctor/Clinic JWT (`Role::AccreditedPhysician`)
  3. Valid Patient 6-digit PIN (`argon2::Argon2::default()`) or TOTP verification

## 3. Field-Level Encryption & Audit Logging
- Encrypt clinical notes and diagnoses in memory using `aes_gcm::Aes256Gcm` with a unique 96-bit nonce per record before inserting into Vault B.
- Every Tier-2 read or write MUST append an entry to `immutable_audit_logs` containing `blind_patient_id`, `doctor_id`, `clinic_id`, `action_type`, `ip_address`, and `timestamp`.

## Local Docker development

The local-only Compose stack uses separate PostgreSQL containers, networks, credentials, and volumes for identity and clinical data. Runtime accounts are distinct from bootstrap owners. `IdentityPool` and `ClinicalPool` are separate Rust types. The early API only connects the pools and exposes process liveness; it has no clinical queries or authenticated routes. Do not treat this setup, its plaintext local transport, or its local environment credentials as production security. Never load real patient data into this development stack.
