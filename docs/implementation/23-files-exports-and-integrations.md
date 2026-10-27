# 23. Attachments, exports, integrations, and notification safety

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I10

---

Disable attachments until their full lifecycle is ready. If enabled, allowlist formats, inspect file signatures/content rather than trusting extensions, constrain compressed and decompressed sizes, generate storage names, quarantine before release, scan in an isolated worker, and store outside public web roots in private encrypted storage. A scan error is not a clean result. Strip unnecessary metadata where appropriate and serve active formats as safe downloads or isolated previews. Reference: [OWASP file-upload guidance](https://cheatsheetseries.owasp.org/cheatsheets/File_Upload_Cheat_Sheet.html).

Downloads require object-level authorization. Prefer authorization at retrieval; if signed links are needed, document their short lifetime, bearer nature, and revocation limitations. Exports have an explicit purpose, minimum field set, recipient/destination verification, audit event, expiry, and cleanup. Guard spreadsheet exports against formula injection. Redact filenames, document properties, and print headers. Printing creates a physical disclosure and must not silently inherit a general chart-read permission.

For FHIR partners, define supported version/profile packages, terminology versions and licensing where relevant, validation failures, identity matching, stable references, provenance, conflict resolution, duplicate imports, and reconciliation. Authorize `_include`, `_revinclude`, history, batch/transaction, search counts, attachments, and any bulk operations separately; an ordinary read permission must not enable unrestricted discovery. Reject unsupported operations. Validate all references against patient and organization scope. Reference: [HL7 FHIR R4 security](https://hl7.org/fhir/R4/security.html).

Use scoped machine identities for integrations; do not share human doctor credentials. A machine client cannot bypass the patient-authorization invariant. Define an approved capability for each automated workflow before enabling it, including any work performed after a clinical session expires. Authenticate and replay-protect webhooks, bound retries, and verify event origins. Notifications contain minimal information, verify destinations, and handle changed numbers, shared devices, opt-out, duplicates, and delivery failure. SMS/push success is not proof of recipient identity or acknowledgement.
