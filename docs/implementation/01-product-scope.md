# 1. Product outcomes and constraints

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I01

---

MedPulsePH connects a patient's NFC wristband to a centralized service. It supports a narrow emergency summary and a separately authorized clinical encounter. The hospital is the organizational stakeholder; patients, clinicians, reception staff, band officers, administrators, auditors, and operators have different responsibilities.

The existing project instructions establish these non-negotiable constraints:

- The band stores a dynamic URL and only minimal offline triage text. The offline text is an explicit exception to the general prohibition on storing medical information on the chip.
- Tier 1 provides public read-only access to blood type, critical allergies, and the ICE contact after a valid tap and active-band check.
- Tier 2 requires an accredited doctor's authentication and the patient's six-digit PIN or OTP to create a 15-minute `VerifiedClinicSession`.
- Vault A holds identity information; Vault B holds the clinical record. They use separate PostgreSQL pools and must not be joined.
- Rust production handlers and cryptographic paths return typed errors rather than using `unwrap()`.

The system is an encounter and emergency-access platform. Hospital admission, billing, insurance, laboratory operations, pharmacy dispensing, and bed management require separate specifications or integrations.

### Corrections to the source blueprint

| Source assumption | Planning decision |
| --- | --- |
| NTAG215 provides the same rolling cryptographic proof as NTAG 424 DNA | Use NTAG 424 DNA for authenticated production taps. NTAG215 may support clearly labeled static-link demonstrations only. |
| Any phone automatically displays the second NDEF record offline | Offline reading depends on the device, browser, and reader software. Do not promise it on every phone. |
| A higher counter proves a tap happened just now | An unseen valid proof may have been captured earlier. Counters reject consumed or superseded proofs, not every delayed first use. |
| FHIR-shaped JSON guarantees universal interoperability | Define supported profiles, terminology, operations, validation, and partner tests. SMART authorization is a separate layer. |
| Rust makes unauthorized access impossible | Use type boundaries as defense in depth alongside runtime authorization, isolation, and testing. |
| An asynchronous middleware task creates an immutable audit trail | Require durable audit recording and separately protected evidence. |

References: [NXP NTAG 424 application note](https://www.nxp.com/docs/en/application-note/AN12196.pdf), [NXP NTAG215 product information](https://www.nxp.com/products/NTAG213_215_216?tab=Documentation_Tab), [Apple background NFC reading](https://developer.apple.com/documentation/CoreNFC/adding-support-for-background-tag-reading), [FHIR R4 REST specification](https://hl7.org/fhir/R4/http.html), and [SMART App Launch](https://hl7.org/fhir/smart-app-launch/app-launch.html).
