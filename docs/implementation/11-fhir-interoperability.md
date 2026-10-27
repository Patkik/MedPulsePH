# 11. FHIR and partner interoperability

**Status:** Planning requirements; implementation not started.

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
