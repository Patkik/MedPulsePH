# 12. UI design and resilience

**Status:** In progress; the public home page is informational and sign-in, emergency, and protected workspace routes explicitly report that the service is unavailable. Synthetic interactive controls are no longer exposed. Authenticated workflows, accessibility review, and clinical interactions remain to be implemented and verified.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I03

---

The interface should feel calm, clear, and reassuring. Use legible patient context, clear task status, restrained alerts, sentence case, stable form labels, a consistent type scale, and readable line lengths. Use color with text and symbols, not alone.

The public pages follow the device color scheme by default and offer a light/dark control. An explicit choice is saved in the browser when storage is available; otherwise it lasts for the current session. This preference contains no patient data. Both themes still require full accessibility and device review.

Candidate palette, subject to actual contrast testing:

| Purpose | Color |
| --- | --- |
| Main surface | `#F5F8F7` |
| Main text | `#15362F` |
| Primary action | `#176856` |
| Boundary | `#C7D5D0` |
| Attention | `#8B5C00` |
| Critical state | `#B42332` |

Evaluate Atkinson Hyperlegible Next for clinical and interface reading; craft the logo wordmark separately. Use phone layouts with a focused single column; use desktop space for persistent navigation and chart context. Transform wide tables into labeled mobile rows or detail views without hiding essential content.

Aim for WCAG 2.2 AA. Verify keyboard operation, focus visibility, screen readers, text enlargement, reduced motion, high-contrast settings, accessible authentication, and touch targets generally at least 44 CSS pixels where feasible. Prepare localization for English, Filipino, and Cebuano; test long names, long translations, and special characters. See [WCAG 2.2](https://www.w3.org/TR/WCAG22/).

| State | Required behavior |
| --- | --- |
| Session expired or permission revoked | Stop protected activity and explain reauthorization or loss of access. |
| Connection lost | Show offline status; never claim an unconfirmed write succeeded. |
| Submission timed out | Reconcile operation status before retrying. |
| Double submission | Prevent duplicate encounters, records, or prescriptions. |
| Wrong hospital or stale tab | Block the action and require deliberate context switching. |
| Unauthorized record link | Show a safe unavailable state without revealing patient details. |
| Patient declines | End the request without blame or repeated prompts. |
| Patient lacks a phone | Provide a supported patient-facing authorization station. |
| PWA update available | Avoid interrupting an encounter; update at a safe boundary. |
| Shared workstation changes user | Clear previous patient content and revoke the prior session as appropriate. |
| Long allergy or medication text | Wrap and expand; do not silently truncate critical content. |

On resume from background, recheck authorization before displaying protected content. Block protected actions during disconnect; do not silently queue clinical writes under a potentially expired session.
