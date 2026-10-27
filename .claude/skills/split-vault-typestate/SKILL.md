---
name: split-vault-typestate
description: Enforce compile-time Rust typestate security, ATM PIN/OTP dual-custody handshakes, RBAC and capability revocation, and PostgreSQL Vault A (Identity PII) vs. Vault B (Clinical EHR) database isolation. Use whenever writing database migrations, SQLx queries, authentication middleware, or clinical record endpoints.
allowed-tools: Read Edit Grep Glob Bash(cargo check*) Bash(cargo clippy*)
---

# Split-Vault & Typestate Security Skill

## 1. Strict Database Isolation (Vault A vs. Vault B)
- **Vault A (`IdentityPool`):** Stores identity records, hospital memberships, credential status, `nfc_tags`, durable tap counter high-water state, `band_status`, and `argon2_pin_hash`.
- **Vault B (`ClinicalPool`):** Stores encrypted `encounters`, `allergies`, `conditions`, `observations`, `prescriptions`, and `immutable_audit_logs` indexed ONLY by `blind_patient_id` (derived via keyed HMAC-SHA256 with a stable mapping/version contract).
- **Hard Rule:** NEVER write a SQL `JOIN` or distributed transaction across Vault A and Vault B. NEVER store patient names, addresses, phone numbers, or raw NFC UIDs inside Vault B.
- **Cross-Vault Coordination & Internal Workers:** Use durable outbox/inbox events with idempotency keys, version tracking, and narrowly scoped worker capabilities (`docs/implementation/20-state-machines-and-consistency.md`) for approved cross-vault projections (such as emergency summary updates or notifications). Internal workers must never hold blanket chart-browsing privileges.

## 2. RBAC, Compile-Time Typestate & Session Revocation
- **Server-Side Default-Deny RBAC (`docs/implementation/04-roles-and-permissions.md`):**
  - Evaluate active hospital membership, role, independent credential approval, patient relationship, encounter scope, and resource state on every request.
  - Hospital administrators, reception, band officers, platform operators, and privacy auditors MUST NOT be granted clinical chart access.
  - Non-physician care-team (nurse, laboratory, pharmacy) clinical access remains disabled until separately specified and approved.
- **Compile-Time Typestate (`&VerifiedClinicSession`):**
  - Every function that queries or mutates Vault B clinical records MUST require `&VerifiedClinicSession` (or a narrowly scoped internal worker capability for non-chart projections) in its signature.
  - `VerifiedClinicSession` fields remain private to the authorization/handshake module so no route handler can construct it without:
    1. A valid, unconsumed accepted-proof tap context from Vault A (`band_status == 'ACTIVE'`).
    2. An authenticated, actively accredited physician in the target hospital (`Role::Doctor` with verified credential eligibility).
    3. Verified Patient 6-digit PIN (`argon2` Argon2id with tuned parameters and server-side throttling) or enrolled OTP approval for the displayed clinician, hospital, and scope.
- **Fixed 15-Minute Expiry & Transaction-Boundary Revocation (`docs/implementation/19-authentication-and-session-security.md`):**
  - `VerifiedClinicSession` enforces a fixed 15-minute server-side lifetime that cannot be extended by renewing a login token.
  - Before committing any clinical read or write, revalidate session expiry, session revocation version, band freeze, consent withdrawal, and clinician suspension at the database transaction boundary.

## 3. Versioned Encryption Envelope, Key Separation & Durable Audit
- **Versioned AES-256-GCM Envelope (`docs/implementation/21-cryptography-and-key-operations.md`):**
  - Encrypt clinical notes, diagnoses, and sensitive payloads using `aes_gcm::Aes256Gcm` inside a versioned envelope recording `envelope_version`, `key_id`, a unique 96-bit nonce, and `ciphertext_with_tag`.
  - Bind the ciphertext via Authenticated Associated Data (AAD) to `blind_patient_id`, resource/field purpose, and schema version so ciphertexts cannot be transplanted across patients or fields.
  - Maintain independent secrets for PIN peppers, token signing, NFC tag verification, Vault B encryption, and HMAC blind-ID derivation.
- **Mandatory Durable Audit:**
  - Every Tier-2 clinical read or write MUST durably append an entry to `immutable_audit_logs` containing `blind_patient_id`, `actor_id`, `hospital_id`, `session_reference` (never raw secrets), `action_type`, and `timestamp`, and fail safely if the audit record cannot be persisted.

## 4. Local Docker Development
The local-only Compose stack uses separate PostgreSQL containers, networks, credentials, and volumes for identity and clinical data. Runtime accounts are distinct from bootstrap owners. `IdentityPool` and `ClinicalPool` are separate Rust types. Do not treat this local setup, its plaintext local transport, or its local environment credentials as production security. Never load real patient data into this development stack.
