# 6. Dashboards and primary actions

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I03

---

| Workspace | Main information | Primary action |
| --- | --- | --- |
| Patient | Band status, authorization requests, emergency-profile status, access activity | Review authorization or manage band. |
| Doctor | Assigned visits, authorization status, unfinished documentation | Begin or resume an authorized encounter. |
| Reception | Arrivals, identity-verification tasks, enrollment progress | Prepare the correct patient visit. |
| Band officer | Issuance and replacement tasks, verification failures | Issue or replace a band. |
| Hospital administrator | Staff lifecycle, pending approvals, operational issues | Resolve an access or enrollment task. |
| Credential reviewer | Submissions, expiring credentials, review history | Review eligibility. |
| Privacy auditor | Assigned investigations and flagged access events | Review evidence. |
| Platform operator | Service status, incidents, integration failures | Investigate an operational issue. |
| Nurse, future clinical milestone | Assigned care tasks and handovers | Complete a specifically authorized task. |

Doctor queues show only minimum identity and visit metadata needed for assignment; do not preload clinical histories before authorization. Operational dashboards show service health and failures rather than patient notes. Empty states explain a role-appropriate next action. Avoid decorative dashboard metrics.
