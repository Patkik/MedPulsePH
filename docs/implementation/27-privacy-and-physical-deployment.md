# 27. Privacy lifecycle, contracts, and physical deployment

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I01

---

Complete a data inventory and privacy impact assessment covering public and offline disclosure, patient linking, hospital sharing, minors/representatives, suppliers, and misuse by legitimate users. Identify controller/processor responsibilities for the platform and each hospital, contractual security duties, authorized subprocessors, permitted locations/transfers, data return, and service termination. These classifications and obligations require the responsible privacy/legal stakeholders; the architecture cannot declare compliance by itself.

Define retention by record class: clinical record, enrollment evidence, band/provisioning logs, authorization evidence, audit, operational diagnostics, exports, notifications, and backups. Address legal holds, corrections, account closure, hospital offboarding, patient death, and expired representative authority. Freezing a band or deleting an account does not delete clinical records. Define what can be deleted, retained, anonymized, or restricted and how backup retention/recovery avoids reintroducing deleted data. Explain the distinction to users.

Provide a tracked patient-rights and complaint process with accessible identity verification, status, responsible reviewer, approved delivery, and escalation. Offer privacy-preserving contact preferences; a shared phone or household email may be unsafe for sensitive notifications. Never use an ICE contact as a general recovery or disclosure delegate without separate authority.

Assess whether the product's actual claims and intended use require medical-device or other sector-specific review. Do not advertise diagnostic, treatment, prescribing, or certification capabilities beyond the validated scope. Obtain the hospital's clinical governance and legal determination before making those claims.

For wristbands, document supplier provenance, provisioning chain of custody, stock loss, counterfeit handling, serialization, tamper/replacement checks, cleaning/disinfection compatibility, skin/fit suitability, durability, water exposure, printing/engraving, and end-of-life disposal with hospital infection-control and procurement staff. A correct cryptographic implementation does not establish physical suitability. Treat required supplier certifications as procurement evidence to verify, not presumed properties.

Validate hospital Wi-Fi restrictions, captive portals, browser policies, NFC-disabled phones, private-browsing/storage restrictions, kiosk modes, screen visibility, borrowed devices, and assisted accessibility. Publish a tested support matrix with the selected device/browser versions at release and a route for unsupported devices. Provide training and short job aids for identity checks, private PIN entry, consent decline, band replacement, downtime, and incident reporting.
