# 17. Threat model and data-flow inventory

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I01

---

Map every flow involving the band, browser, identity provider, ingress proxy, Rust services, PostgreSQL vaults, Redis, key facility, object storage, audit archive, notification supplier, provisioning station, reader companion, partner hospital, CI/CD, and support tooling. Include metadata, backups, temporary files, dead-letter queues, exports, and diagnostic dumps.

For each flow record the data classification, purpose, actor, authentication, authorization, encryption, retention, logging/redaction, failure behavior, and responsible operator. Identify the components that can decrypt data and the administrators able to change those components.

Exercise these adversaries rather than assuming all enrolled users are trustworthy:

- A bystander photographing a tap URL, a lost-band finder, or a person harvesting repeated authentic reads.
- An outsider guessing accounts, records, OTPs, invitation links, pairing requests, or public emergency identifiers.
- A legitimate staff member accessing unrelated patients, a malicious approver, or a terminated user with an existing session.
- A compromised browser, extension, shared workstation, provisioning station, reader, or hospital identity provider.
- A compromised application dependency, build account, service account, cloud administrator, or notification supplier.
- An attacker exhausting verification, key-service, database, audit, storage, or notification capacity.

Describe residual limits explicitly: a verified band is not proof of the wearer's identity; SDM is not distance bounding; relaying or delayed first-use of a genuine proof is not eliminated by a counter. An authorized recipient can photograph, print, or retain displayed information. Field encryption does not protect records from a compromised service that is entitled to decrypt them. Split vaults reduce particular risks but do not make the combined runtime anonymous or breach-proof.

Disable unsupported capabilities rather than giving them placeholder permissions. Do not introduce production ML/AI summarization, external transcription, session replay analytics, or third-party clinical-data processing without a separately reviewed data flow and supplier agreement.
