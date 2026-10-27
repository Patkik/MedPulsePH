# 3. Users, workspaces, and information architecture

**Status:** In progress; `app/src/app/` implements interactive synthetic workspace routes for all `S01`–`S10` screens (`/`, `/emergency`, `/login`, `/reception`, `/patient`, `/doctor`, `/admin`) sharing `app/src/components/AppShell.tsx`. Live HTTP clinical/auth routes remain disabled (`404`).

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I03

---

| User | Setting and primary need |
| --- | --- |
| Patient | Manage band, emergency disclosure, authorization requests, and account security, including without a personal smartphone. |
| Reception staff | Identify the correct person and prepare a visit at a busy desk. |
| Doctor | Open the correct authorized chart and document care through interruptions. |
| Nurse | Manage assigned operational tasks; future clinical work requires an approved care-team model. |
| Band officer | Issue, verify, replace, and retire bands. |
| Hospital administrator | Manage users, hospital membership, and operational settings. |
| Credential reviewer | Verify professional eligibility independently of registration. |
| Privacy auditor | Investigate access evidence within an assigned scope. |
| Platform operator | Monitor service health, integrations, and security. |
| Emergency responder or bystander | Find the permitted emergency information quickly. |

Provide public access, patient, care, hospital administration, privacy, and platform operations areas. Share a component library, not unrestricted data responses. An individual with multiple responsibilities explicitly switches role or hospital context; switching clears the prior patient context and re-evaluates permissions.

Keep patient identity, active hospital, clinician, and session state unmistakable. Design for shared workstations, slow connections, shift changes, frequent interruption, keyboard use, and long lists. Use plain action labels such as **Request patient authorization**, **Save draft**, and **Finalize encounter**.
