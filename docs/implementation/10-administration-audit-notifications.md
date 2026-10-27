# 10. Administration, audit, and notifications

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I07

---

Hospital administration includes staff invitations, membership lifecycle, department and scoped-role assignment, credential status, session revocation, band operations, operational reports, integration status, and administrative history. High-impact actions show the affected person, hospital, consequences, and confirmation. Suspension revokes relevant sessions and invitations promptly; one hospital cannot suspend another hospital's membership.

For clinical writes, commit the record and its audit event together. For reads, durably record the authorized disclosure before releasing the protected response. Event wording must distinguish an authorized response from proof that a person read it. Record the clinical patient identifier, actor, hospital, action, resource, time, outcome, and necessary correlation metadata. Exclude PINs, tokens, cryptographic keys, and unnecessary medical content.

Use restricted append-only permissions, retention controls, and independently protected archival evidence. Call the system tamper-resistant or tamper-evident only according to implemented controls. Deliver notifications through a durable outbox with retries and minimal lock-screen content. Label anonymous emergency access as anonymous.

The privacy workspace supports scoped access-event search, investigations, findings, and controlled exports using a minimized event projection. Audit permissions do not grant chart browsing. Clinical content required for an investigation needs a separate authorization pathway. Define narrow internal service capabilities for audit projection and workers rather than manufacturing doctor sessions or a general system bypass.
