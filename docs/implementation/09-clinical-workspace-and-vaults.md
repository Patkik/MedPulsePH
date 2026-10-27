# 9. Clinical workspace and data boundaries

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I08

---

On PC, show a persistent hospital/workspace header, patient identity strip, session time, clinical navigation, main content, and clear save/finalize controls. On phones, stack the same information in a focused layout while retaining patient and session context.

Clinical sections: summary, allergies, encounter history, current encounter, observations/vitals, conditions, medication requests, and supported results/documents. A summary permission must not imply fetching every resource.

Documentation lifecycle: **draft → reviewed → finalized → amended when necessary**. Show unsaved, durably saved, and finalized states. Save only under valid authorization; do not cache clinical drafts in browser storage initially. Finalize only after explicit confirmation and server acknowledgement. Amend finalized records with provenance rather than silently overwriting them. Prescription actions remain distinct from saving notes. Present concurrent update conflicts for review.

Warn before session expiry, but do not extend it without the required patient authorization. On expiry, stop protected reads and writes. Saved drafts remain sealed until appropriate authorization returns; unsent text is not guaranteed durable recovery. Prevent stale tabs, patient switching, and hospital switching from applying an action to the wrong context.

| Boundary | Contents |
| --- | --- |
| Vault A | Identity, account linkage, band registry and status, PIN hash, identity-to-clinical mapping, authorized emergency projection, durable counter state. |
| Vault B | Stable pseudonymous clinical identifier, encounters, allergies, observations, medication requests, and clinical audit. |
| Redis | Expiring tap contexts, session coordination, and rate limits. |
| Protected key facility | Encryption and diversification secrets. |
| Audit archive | Independently protected audit copies or checkpoints. |

Never join the vaults. Use a stable HMAC-derived clinical identifier with documented domain separation, encoding, key versioning, and migration; do not use the random identifiers in the draft example. Pseudonymous records remain sensitive. Control identifying data in free text, filenames, attachments, and FHIR narrative. Publish emergency projections through a defined, provenance-preserving workflow.

Keep `VerifiedClinicSession` construction private and require it for clinical repository operations. Runtime checks must still enforce expiry and revocation. Test that ordinary modules cannot construct the capability or access the clinical repository without it. Patient access to complete history needs its own authorization design; do not add unrestricted Vault B queries to the patient dashboard.
