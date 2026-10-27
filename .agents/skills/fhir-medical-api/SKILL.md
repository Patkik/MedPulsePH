---
name: fhir-medical-api
description: Build and validate standardized HL7 FHIR R4-inspired REST API endpoints in Rust Axum for clinic-to-clinic interoperability. Use when creating or modifying JSON payloads, OpenAPI schemas, or clinic checkup endpoints for Patient, AllergyIntolerance, Encounter, or MedicationRequest resources.
allowed-tools: Read Edit Grep Glob Bash(cargo test*)
---

# FHIR Interoperable Medical API Skill

## Two-Tier Endpoint Routing
1. **Tier-1 Public Triage (`GET /api/v1/emergency/{tap_payload}`):**
   - Requires ONLY a valid NFC tap signature and `ACTIVE` band status.
   - Returns strictly limited emergency fields: `blood_type`, `critical_allergies`, `chronic_alerts`, and `ice_contact`.
   - Must NEVER expose past clinic visits, doctor names, or full medical history.
2. **Tier-2 Clinic Checkup (`GET/POST /api/v1/fhir/...`):**
   - Requires `VerifiedClinicSession` (Bearer Encounter Token issued after the Patient PIN/OTP handshake).
   - Formats responses using standardized FHIR R4 JSON resource structures (`resourceType`, `id`, `meta`, `status`, `code`, `subject`, `period`).
