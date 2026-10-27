# 4. Authorization and role boundaries

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I02

---

Use RBAC plus resource-specific conditions: active hospital membership, role, department, professional eligibility, patient relationship, patient authorization, encounter scope, session status, and resource state. Deny by default. Validate permissions server-side on every request, including exports, printing, attachments, background jobs, bulk actions, and integration endpoints. Hiding UI controls alone is insufficient. See [OWASP authorization guidance](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html).

| Role | Baseline responsibility | Boundary |
| --- | --- | --- |
| Platform operator | Approved hospitals and technical services | No routine chart access. Infrastructure access needs separate controls. |
| Hospital administrator | Hospital users and operational settings | Administration does not grant clinical access. |
| Credential reviewer | Review professional eligibility | Cannot approve own credentials. |
| Doctor | Authorized encounter access and documentation | Requires valid clinician status, verified tap, and patient-authorized session. |
| Reception | Minimum identity registration and visit preparation | Cannot browse clinical history. |
| Band officer | Band binding, verification, replacement, and retirement | Cannot read full clinical records or retrieve PINs. |
| Privacy auditor | Scoped audit-event review | Cannot browse clinical notes through audit permissions. |
| Patient | Own account, band, emergency disclosure, and access activity | Cannot alter clinician-authored history. |
| Nurse | Assigned operational tasks initially | Clinical actions remain disabled pending an approved care-team capability model. |
| Laboratory and pharmacy staff | Future scoped workflows | No general chart access. |

There is no everyday role that combines unrestricted administration, clinical care, privacy investigation, and technical operations. Start with reviewed role templates and scoped assignments, not an unrestricted permission editor. No self-approval of privileged role changes or credentials; privileged grants require an independent approver. Routine suspension can take effect promptly. Removing the last responsible hospital administrator requires an ownership-transfer process.

Representative permission decisions:

| Action | Eligible actor | Additional conditions |
| --- | --- | --- |
| Invite staff | Hospital administrator | Same hospital and permitted target role. |
| Approve clinical eligibility | Credential reviewer | Authorized hospital; not own submission. |
| Register patient identity | Reception | Minimum identity fields and approved verification workflow. |
| Bind a band | Band officer | Verified patient assignment and tag. |
| Freeze a band | Patient or explicitly authorized band operator | Verified ownership or scoped authority; recent authentication. |
| Open clinical history | Doctor | Current credential and active patient-authorized session for this patient and hospital. |
| Finalize encounter | Doctor | Correct patient and encounter, required fields, active session. |
| Review audit events | Privacy auditor | Authorized organizational and investigation scope. |
| View technical health | Platform operator | Operational data only. |
| Export clinical records | Separately authorized actor | Defined purpose and scope, reauthentication, audit, and secure delivery. |

### Care-team extension

The current invariant does not define independent nurse, laboratory, or pharmacy access. A doctor-focused pilot can use the baseline. A broader hospital release must first specify named individual authentication, patient consent scope, professional actions, assignments, supervision, handover, expiry, revocation, and audit attribution. Never share a doctor's token. Clinical data cannot be moved into an operational store to bypass clinical authorization. Update the relevant project instructions and skills before enabling the extension.
