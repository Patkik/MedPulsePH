# I01 pilot baseline

**Status:** Proposed for a synthetic-data pilot. Hospital, clinical, privacy, and operations owners are not yet assigned. This document does not authorize real-patient use or represent clinical approval.

## Intended pilot

The initial workflow is a doctor-led encounter using an NTAG 424 DNA band, verified tap, accredited doctor identity, patient six-digit PIN or enrolled OTP, a 15-minute clinic session, a narrow emergency summary, and durable audit evidence. It is intended for Android and iPhone browsers and a PC PWA, with phone-to-PC pairing where needed. Hardware and backend behavior must be tested with synthetic records before any real-patient pilot.

Public emergency content is limited to blood type, critical allergies, and ICE contact. The public and offline disclosure decisions are separate from clinical authorization. The offline record may remain readable and stale after a remote freeze.

## Disabled until separately designed and authorized

- Independent nurse, laboratory, pharmacy, and other non-physician clinical access.
- Full-record emergency override or automatic access without patient authorization.
- Self-service full-history patient access, partner data exchange, file upload, export, and printing.
- Offline clinical editing, background synchronization of clinical changes, and clinical decision support.
- Production enrollment or processing of real patient data.

Patient access and correction requests still need an independently authorized, tracked hospital process before real-patient use. Disabling a self-service screen does not remove that obligation.

## Decision and responsibility ledger

| Decision | Proposed boundary | Decision owner | Status |
| --- | --- | --- | --- |
| Hospital sharing | No cross-hospital clinical access until source and sharing scope are defined. | Hospital clinical and privacy leads | Open; owner not assigned |
| Identity and representation | Demographic matching alone cannot claim a record; staff do not set or view patient PINs. | Hospital identity and privacy leads | Open; owner not assigned |
| Emergency disclosure | Only approved Tier 1 fields; separate physical/offline disclosure. | Clinical and privacy leads | Open; owner not assigned |
| Full-record emergency access | No bypass in the initial scope. | Clinical governance and privacy leads | Open; owner not assigned |
| Care-team privileges | Non-physician clinical access disabled pending a named-actor authorization design. | Clinical governance lead | Open; owner not assigned |
| Patient rights and complaints | Establish verified request, correction, release, and escalation procedures. | Hospital privacy lead | Open; owner not assigned |
| Clinical safety | Confirm identity, allergy provenance, medication, amendment, and downtime controls. | Hospital clinical safety owner | Open; owner not assigned |
| Operating model | Agree hosting, recovery objectives, incident responsibility, staffing, and supplier contracts. | Hospital and service operations leads | Open; owner not assigned |

The full risk register and evidence requirements are in [16-risk-and-decision-register.md](16-risk-and-decision-register.md). Named people, review dates, and evidence must be recorded by the responsible organization; they must not be invented by the development team.

## Trust boundaries and assumptions

The band is a cryptographic pointer, not proof that its wearer is the registered patient. A counter rejects consumed or superseded proofs, but does not prove that an unused proof was generated immediately before submission. The browser and reader are untrusted clients. Doctor identity, hospital membership, clinical capability, patient authorization, expiry, and revocation require server-side validation.

Vault A and Vault B use separate PostgreSQL connection pools, credentials, and data boundaries. Encryption and separation reduce specific risks; a compromised process with both authorized accesses remains a shared boundary. The service must durably record required clinical audit events and fail safely if it cannot do so.

See [17-threat-model.md](17-threat-model.md) and [18-patient-sharing-and-clinical-safety.md](18-patient-sharing-and-clinical-safety.md) for the threat scenarios and clinical hazards. A formal threat-model review and clinical safety approval remain open.
