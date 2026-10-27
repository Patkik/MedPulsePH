# 7. NFC, band issuance, and emergency access

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I05

---

Use NTAG 424 DNA for production authenticated taps. Provisioning must register the tag, replace factory keys, configure per-tag keys or an approved diversification scheme, configure the dynamic URL and authenticated byte ranges, write permitted offline text, verify the finished tag, and activate it only after success. Separate provisioning authority from routine reading. Protect master secrets in a managed key facility or equivalent; store references or wrapped material rather than plaintext master keys. Design key lookup so an incoming URL does not require trying every patient's key.

Band lifecycle: **unassigned → assigned pending verification → active → frozen → replaced or retired**. Replacement revokes the old band's online authority. A band is not reidentified as a different physical tag.

Online emergency journey: **tap → validate proof and active status → show blood type, critical allergies, and ICE contact**. Show provenance and update status. Distinguish unknown, not recorded, none reported, and clinician-verified information. Avoid mandatory installation, login, marketing, or dashboard navigation.

Offline text is readable by compatible readers, can become stale, and cannot be remotely erased or checked for current revocation. Display its update indicator when possible. A first-time visitor without connectivity cannot download the PWA. NTAG 424 DNA has a 256-byte NDEF file: budget URL, dynamic parameters, both records, and NDEF overhead together. Stop provisioning if critical content does not fit; do not silently truncate allergies.

| Capability | Android PWA | iPhone PWA | PC PWA |
| --- | --- | --- | --- |
| Open HTTPS band link | Supported on compatible NFC phones. | Supported on compatible iPhones subject to OS interaction. | Requires an external reading method. |
| Direct in-browser NDEF reading | Supported in some browsers, especially Chrome on Android. | Do not rely on Web NFC. | Do not rely on Web NFC. |
| Offline tag summary | Possible with supported Web NFC and an available app shell. | Suitable reader or later native app needed. | Suitable reader software needed. |
| Configure secure tag keys | Dedicated provisioning tool. | Dedicated provisioning tool. | Dedicated provisioning tool. |

Web NFC supports NDEF rather than the low-level operations needed for full secure-tag provisioning. Verify behavior on actual devices. See [Chrome Web NFC](https://developer.chrome.com/docs/capabilities/nfc) and [Apple background reading](https://developer.apple.com/documentation/CoreNFC/adding-support-for-background-tag-reading).

### PC pairing and readers

For the first PC workflow, the authenticated workstation creates a short-lived pairing request; a compatible phone joins, reads the band, and the backend verifies and binds the tap to the requesting clinician and workstation. The patient authorizes the resulting request and the workstation receives confirmation. The pairing QR is an invitation, not a substitute for cryptographic tap proof. A static band QR also cannot satisfy that proof requirement.

Investigate a narrowly scoped Rust PC/SC companion for clinics requiring USB readers. Prefer backend-mediated delivery; if a local connection is necessary, specify pairing, origin checks, and authentication. Verify reader, driver, tag, and OS compatibility with real hardware before procurement.
