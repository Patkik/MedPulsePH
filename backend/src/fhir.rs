//! Scoped HL7 FHIR R4 interoperability contracts, boundary `Patient` assembly, reference
//! validation, and narrative sanitization (`I10`, `11-fhir-interoperability.md`,
//! `23-files-exports-and-integrations.md`, `.agents/skills/fhir-medical-api/SKILL.md`).
//!
//! Enforces:
//! - Strict split-vault isolation (`AGENTS.md` Invariant #3): Vault B stores only
//!   `BlindPatientId` and encrypted clinical payloads. Authorized FHIR `Patient` responses
//!   are assembled at the application boundary using a live-revalidated `&VerifiedClinicSession`
//!   and a narrow Vault A identity slice (`VaultAPatientIdentitySlice`).
//! - `#[serde(deny_unknown_fields)]` on incoming FHIR payloads to reject undeclared fields
//!   and prevent mass-assignment or unvalidated extension smuggling (`22-pwa-api-and-abuse-security.md`).
//! - Reference validation (`Patient/{blind_patient_id}`) so a clinical resource can never
//!   reference a different patient or arbitrary external URL.
//! - Narrative/text sanitization rejecting active script or frame markup (`<script`,
//!   `javascript:`, `<iframe`, `onerror=`, `onload=`).
//! - Explicit `FhirCapabilityStatement` publishing only the supported R4 resources while
//!   declaring bulk export (`$export`), unconstrained `_include`, and file uploads disabled.

use crate::{
    auth::{AuthError, TransactionBoundaryState, VerifiedClinicSession},
    crypto::BlindPatientId,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Supported FHIR R4 release version string.
pub const FHIR_R4_VERSION: &str = "4.0.1";

/// Typed errors for FHIR R4 validation, reference scoping, and boundary assembly.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum FhirError {
    #[error("clinical session authorization failed: {0}")]
    Auth(#[from] AuthError),
    #[error("unexpected FHIR resourceType: expected {expected}, got {actual}")]
    UnexpectedResourceType { expected: String, actual: String },
    #[error("FHIR subject reference '{reference}' does not match authorized patient scope")]
    CrossPatientReferenceForbidden { reference: String },
    #[error("FHIR code or status field must not be empty")]
    EmptyRequiredField,
    #[error("FHIR text or narrative contains forbidden executable markup")]
    UnsafeNarrativeContent,
    #[error("bulk FHIR export and unconstrained graph searches are disabled in the pilot scope")]
    DeferredOperationDisabled,
}

/// Narrow demographic slice read from Vault A (`IdentityPool`) and passed to the API
/// boundary assembler after `VerifiedClinicSession` revalidation (`11-fhir-interoperability.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultAPatientIdentitySlice {
    pub blind_patient_id: BlindPatientId,
    pub family_name: String,
    pub given_name: String,
    pub birth_date_iso: String,
    pub administrative_gender: String,
}

/// FHIR R4 `Meta` element carrying resource version and source hospital provenance (`11-fhir-interoperability.md`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FhirMeta {
    #[serde(rename = "versionId")]
    pub version_id: String,
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
    pub source: String,
}

/// FHIR R4 `HumanName` element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FhirHumanName {
    pub family: String,
    pub given: Vec<String>,
}

/// FHIR R4 `Coding` element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FhirCoding {
    pub system: String,
    pub code: String,
    pub display: String,
}

/// FHIR R4 `CodeableConcept` element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FhirCodeableConcept {
    pub coding: Vec<FhirCoding>,
    pub text: String,
}

/// FHIR R4 `Reference` element restricted to local patient/encounter scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FhirReference {
    pub reference: String,
}

/// FHIR R4 `Period` element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FhirPeriod {
    pub start: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

/// Scoped FHIR R4 `Patient` resource assembled at the authorized application boundary (`11-fhir-interoperability.md`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FhirPatientResource {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    pub meta: FhirMeta,
    pub active: bool,
    pub name: Vec<FhirHumanName>,
    pub gender: String,
    #[serde(rename = "birthDate")]
    pub birth_date: String,
}

/// Scoped FHIR R4 `AllergyIntolerance` resource (`11-fhir-interoperability.md`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FhirAllergyIntoleranceResource {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    pub meta: FhirMeta,
    #[serde(rename = "clinicalStatus")]
    pub clinical_status: FhirCodeableConcept,
    #[serde(rename = "verificationStatus")]
    pub verification_status: FhirCodeableConcept,
    pub criticality: String,
    pub code: FhirCodeableConcept,
    pub patient: FhirReference,
}

/// Scoped FHIR R4 `Encounter` resource (`11-fhir-interoperability.md`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FhirEncounterResource {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub id: String,
    pub meta: FhirMeta,
    pub status: String,
    pub code: FhirCodeableConcept,
    pub subject: FhirReference,
    pub period: FhirPeriod,
}

/// Scoped FHIR R4 `CapabilityStatement` metadata (`11-fhir-interoperability.md`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FhirCapabilityStatement {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub status: String,
    #[serde(rename = "fhirVersion")]
    pub fhir_version: String,
    pub format: Vec<String>,
    #[serde(rename = "supportedResources")]
    pub supported_resources: Vec<String>,
    #[serde(rename = "disabledOperations")]
    pub disabled_operations: Vec<String>,
}

impl FhirCapabilityStatement {
    /// Returns the truthful pilot FHIR R4 capability statement (`11-fhir-interoperability.md`).
    pub fn pilot_baseline() -> Self {
        Self {
            resource_type: "CapabilityStatement".to_string(),
            status: "active".to_string(),
            fhir_version: FHIR_R4_VERSION.to_string(),
            format: vec!["application/fhir+json".to_string()],
            supported_resources: vec![
                "Patient".to_string(),
                "AllergyIntolerance".to_string(),
                "Encounter".to_string(),
                "Condition".to_string(),
                "Observation".to_string(),
                "MedicationRequest".to_string(),
            ],
            disabled_operations: vec![
                "$export".to_string(),
                "$everything".to_string(),
                "_include=*".to_string(),
                "_revinclude=*".to_string(),
                "DocumentReference.upload".to_string(),
            ],
        }
    }
}

/// Assembles a FHIR R4 `Patient` resource at the API boundary from a Vault A identity slice
/// only after revalidating `&VerifiedClinicSession` at the transaction boundary.
/// Vault B never receives or stores the demographic fields (`AGENTS.md` Invariant #3).
pub fn assemble_fhir_patient_at_boundary(
    session: &VerifiedClinicSession,
    live_state: &TransactionBoundaryState,
    hospital_id: &str,
    vault_a_slice: &VaultAPatientIdentitySlice,
    version_id: u64,
    last_updated_iso: &str,
) -> Result<FhirPatientResource, FhirError> {
    session.revalidate_at_transaction_boundary(
        &vault_a_slice.blind_patient_id,
        hospital_id,
        live_state,
    )?;

    sanitize_fhir_text(&vault_a_slice.family_name)?;
    sanitize_fhir_text(&vault_a_slice.given_name)?;
    sanitize_fhir_text(&vault_a_slice.birth_date_iso)?;
    sanitize_fhir_text(&vault_a_slice.administrative_gender)?;

    Ok(FhirPatientResource {
        resource_type: "Patient".to_string(),
        id: vault_a_slice.blind_patient_id.to_canonical_string(),
        meta: FhirMeta {
            version_id: version_id.to_string(),
            last_updated: last_updated_iso.to_string(),
            source: format!("urn:medpulse:hospital:{hospital_id}"),
        },
        active: true,
        name: vec![FhirHumanName {
            family: vault_a_slice.family_name.clone(),
            given: vec![vault_a_slice.given_name.clone()],
        }],
        gender: vault_a_slice.administrative_gender.clone(),
        birth_date: vault_a_slice.birth_date_iso.clone(),
    })
}

/// Validates an incoming or outgoing `FhirEncounterResource` against the active
/// `VerifiedClinicSession`'s patient and hospital scope (`11-fhir-interoperability.md`).
pub fn validate_scoped_fhir_encounter(
    session: &VerifiedClinicSession,
    live_state: &TransactionBoundaryState,
    hospital_id: &str,
    authorized_patient: &BlindPatientId,
    encounter: &FhirEncounterResource,
) -> Result<(), FhirError> {
    session.revalidate_at_transaction_boundary(authorized_patient, hospital_id, live_state)?;

    if encounter.resource_type != "Encounter" {
        return Err(FhirError::UnexpectedResourceType {
            expected: "Encounter".to_string(),
            actual: encounter.resource_type.clone(),
        });
    }
    if encounter.status.trim().is_empty() || encounter.code.text.trim().is_empty() {
        return Err(FhirError::EmptyRequiredField);
    }

    let expected_ref = format!("Patient/{}", authorized_patient.to_canonical_string());
    if encounter.subject.reference != expected_ref {
        return Err(FhirError::CrossPatientReferenceForbidden {
            reference: encounter.subject.reference.clone(),
        });
    }

    sanitize_fhir_text(&encounter.code.text)?;
    for coding in &encounter.code.coding {
        sanitize_fhir_text(&coding.system)?;
        sanitize_fhir_text(&coding.code)?;
        sanitize_fhir_text(&coding.display)?;
    }

    Ok(())
}

/// Rejects active script, event handler, or frame markup inside FHIR text fields (`11-fhir-interoperability.md`, `22-pwa-api-and-abuse-security.md`).
pub fn sanitize_fhir_text(input: &str) -> Result<(), FhirError> {
    let lower = input.to_ascii_lowercase();
    for forbidden in [
        "<script",
        "</script",
        "javascript:",
        "vbscript:",
        "data:text/html",
        "<iframe",
        "<object",
        "<embed",
        "onerror=",
        "onload=",
    ] {
        if lower.contains(forbidden) {
            return Err(FhirError::UnsafeNarrativeContent);
        }
    }
    Ok(())
}
