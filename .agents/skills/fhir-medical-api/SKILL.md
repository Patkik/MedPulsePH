---
name: fhir-medical-api
description: Build and validate standardized HL7 FHIR R4-inspired REST API endpoints in Rust Axum for clinic-to-clinic interoperability. Use when creating or modifying JSON payloads, OpenAPI schemas, or clinic checkup endpoints for Patient, AllergyIntolerance, Encounter, or MedicationRequest resources.
allowed-tools: Read Edit Grep Glob Bash(cargo test*)
---

# FHIR Interoperable Medical API Skill

## 1. Two-Tier Endpoint Routing & Default-Deny Posture
1. **Tier-1 Public Triage (`GET /api/v1/emergency/{tap_payload}`):**
   - Requires a valid NFC tap proof (or bound accepted-proof tap context) and `ACTIVE` band status in Vault A.
   - Returns **strictly** the approved Tier-1 emergency fields: `blood_type`, `critical_allergies`, and `ice_contact` (plus provenance/update metadata distinguishing `unknown`, `not_recorded`, `none_reported`, and `clinician_verified`).
   - Do **not** include unapproved extra fields (such as `chronic_alerts`), past clinic visits, doctor names, medication orders, or full medical history.
2. **Tier-2 Clinic Checkup (`/api/v1/fhir/...`):**
   - Requires an active, non-expired, unrevoked 15-minute `VerifiedClinicSession` bound to the specific patient (`blind_patient_id`), accredited physician, hospital, and permitted resource scope.
   - Keep unfinished FHIR and clinical endpoints disabled (`404 Not Found`) until authentication, patient PIN/OTP authorization, schema validation, and durable audit logging are implemented and verified.

## 2. Resource Profiles & Split-Vault Assembly
- **Supported Pilot Resources:** `Patient` (authorized demographics), `AllergyIntolerance`, `Encounter`, `Condition`, `Observation`, and `MedicationRequest` (`DiagnosticReport` is deferred to a later slice).
- **Boundary Assembly (No PII in Vault B):**
  - Vault B stores encrypted clinical resources keyed only by `blind_patient_id` (derived via keyed HMAC-SHA256).
  - Construct authorized `Patient` demographic fields at the API boundary from Vault A only when permitted by the active session. Never copy patient names, addresses, phone numbers, or raw NFC tag UIDs into Vault B resource snapshots, narratives, or audit payloads.
- **Provenance & Concurrency:**
  - Preserve source hospital/organization, authoring clinician, and verification status on clinical resources.
  - Require versioned updates (`ETag` / `If-Match` or resource version checks) and scoped idempotency keys on mutating requests (`POST`/`PUT`/`PATCH`).
  - See `docs/implementation/11-fhir-interoperability.md` and `docs/implementation/09-clinical-workspace-and-vaults.md` for full resource and encounter rules.
