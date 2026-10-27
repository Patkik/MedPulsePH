# 18. Patient identity, ownership, sharing, and clinical safety

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I01

---

### Longitudinal records and hospital boundaries

**Decision required:** Define whether MedPulsePH operates a shared longitudinal record, separate hospital-owned records linked to one patient, or a constrained combination. A stable clinical identifier must not become authority to read every hospital's contribution.

Recommended foundation: preserve source hospital, author, record version, and any applicable sensitivity label on every resource. Apply source-organization and patient-sharing policy alongside the clinical session. A doctor at Hospital A does not automatically gain Hospital B records because the patient wore a band. Explicitly define consent scope, restricted categories, referral access, and which source can correct its contributions.

Specify local hospital identifiers versus platform identifiers, normalization, duplicate-review evidence, disputed identity handling, and recovery after an incorrect link. Merges must preserve old references and audit history and have a reviewed correction procedure. Do not derive identity authority from phone-number possession or a demographic match. Do not globally reassign old band UIDs or external identifiers to new people.

### Patient access and representatives

Keep direct patient clinical-history access disabled until a separate capability exists, but provide an authenticated, tracked request-and-release workflow for access and correction from the initial live service. Deferring the self-service screen cannot mean ignoring patient requests. Define identity checks, authorization to release, permitted redactions, delivery, review responsibilities, and legally applicable response periods with the privacy lead.

Document guardian/delegate identity, relationship evidence, permitted actions, effective dates, expiry, disputes, withdrawal, and the transition when a minor becomes independently authorized. Do not share a patient's credentials with a representative. Define what happens for incapacity, absent guardians, deceased patients, and loss of all recovery factors. A receptionist or administrator cannot unilaterally establish clinical consent by selecting a relationship label.

### Clinical data correctness

Maintain a clinical hazard log: wrong patient, stale emergency summary, incorrect blood type, ambiguous allergies, lost documentation, wrong medication, unacknowledged critical result, and unreviewed imported history. The hospital clinical safety owner determines the verification and workflow controls for each hazard.

- Record who reported and who verified blood type and allergies, the source, the observation time, and review status. The wristband is a summary source; the hospital's independent clinical verification procedures still govern treatment and transfusion decisions.
- Do not collapse unknown allergy status into no known allergies. Support reaction, severity, verification, resolution, and disputed information where clinically appropriate.
- Store observations with units and appropriate coded meaning; validate realistic format/range without silently changing the entered value. Present measurement time separately from entry time.
- Medication requests need drug identity, dose/unit, route, frequency, duration where applicable, prescriber, status, and cancellation/amendment rules. Do not claim interaction checking, prescribing compliance, or clinical decision support unless implemented and clinically validated.
- If critical results or referrals are supported, specify recipient, acknowledgement, escalation, and handover. A delivered notification is not clinical acknowledgement. Keep these features disabled until a responsible service owns the closed-loop workflow.
- Preserve author, source organization, event time, entry time, resource version, amendments, and entered-in-error status. Correct clinical content through traceable amendments rather than deleting history to hide a mistake.

### Public and offline disclosure

Separate account terms, public emergency disclosure, offline physical disclosure, clinical authorization, and optional notifications. Explain what any nearby reader can see, the limitations of freeze, and the exposure of an ICE contact's phone number. Capture the appropriate basis and contact-holder handling with the privacy lead. Avoid publishing additional identity fields simply to make the emergency page more convenient.

Define withdrawal and update behavior: stop further online disclosure when revoked, schedule verified physical rewriting where required, and keep a visible distinction between the latest online projection and the version on the band. Do not present an offline record as current or cryptographically authenticated solely because the URL record was verified. Confirm exactly which bytes the chosen NDEF/SDM configuration authenticates. Never distribute tag master secrets to generic responder devices to make offline verification possible.
