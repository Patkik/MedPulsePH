# 11. FHIR and partner interoperability

**Status:** In progress; `backend/src/fhir.rs` implements `FhirCapabilityStatement` (explicitly disabling `$export`, `$everything`, `_include=*`, and uploads), boundary assembly of `FhirPatientResource` from Vault A (`assemble_fhir_patient_at_boundary`) without storing PII in Vault B, `#[serde(deny_unknown_fields)]` mass-assignment guards, patient/hospital reference validation (`validate_scoped_fhir_encounter`), and narrative script sanitization (`sanitize_fhir_text`). Live HTTP FHIR routes and external partner SMART workflows remain disabled (`404`).

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I10

---

FHIR R4 structures and interactions are distinct from SMART App Launch authorization. Support the existing patient PIN rule within any partner authorization design.

| Information | FHIR resource |
| --- | --- |
| Authorized demographics | `Patient` |
| Allergies | `AllergyIntolerance` |
| Visit | `Encounter` |
| Diagnoses | `Condition` |
| Measurements | `Observation` |
| Medication orders | `MedicationRequest` |
| Results reports, later | `DiagnosticReport` |

Define profiles, terminology, required fields, search parameters, pagination, version handling, and error responses. Publish a `CapabilityStatement`, validate with an HL7 validator, and test actual partner workflows. Construct authorized demographics at the API boundary from Vault A; do not duplicate identifying information into Vault B snapshots or narrative. Full SMART integration follows the core workflow and requires registration, discovery, scopes, PKCE, and patient authorization enforcement.
