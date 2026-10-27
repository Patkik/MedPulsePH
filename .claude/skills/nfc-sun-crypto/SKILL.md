---
name: nfc-sun-crypto
description: Implement, audit, or debug 13.56 MHz NFC wristband payloads, NXP NTAG 424 DNA SUN (Secure Unique NFC) AES-128 decryption, CMAC verification, NDEF dual-record payloads, and durable Vault A + Redis tap-counter anti-replay logic. Use whenever touching NFC tap routes, cryptographic tag verification, or USB PC/SC reader modules.
allowed-tools: Read Edit Grep Glob Bash(cargo test*) Bash(cargo check*)
---

# NFC Wristband & SUN Cryptographic Skill

## 1. Core Hardware, NDEF Budget & Platform Reality
1. **Frequency & Standard:** Always target 13.56 MHz HF NFC (ISO/IEC 14443-A / NFC Forum Type 4 NTAG 424 DNA for authenticated production taps; NTAG215 only where documented for non-SUN test fixtures). Never generate code for 125 kHz LF RFID.
2. **Dual NDEF Structure & Complete 256-Byte File Budget:**
   - **Record 1 (Online URI):** `https://{domain}/v1/tap?d={picc_data_hex}&c={cmac_hex}`
   - **Record 2 (Offline Fallback):** Minimal permitted offline triage text (`BloodType|CriticalAllergies|ICE_Contact|UpdatedIndicator`).
   - **Strict 256-Byte Budget:** NTAG 424 DNA provides a 256-byte NDEF file. Budget NDEF file headers, TLV wrappers, Record 1 URL + dynamic hex parameters, and Record 2 payload together (`docs/implementation/07-nfc-bands-and-emergency.md`). **Never silently truncate critical allergies**—reject provisioning if the required payload exceeds the verified byte budget.
   - **Offline Limitations:** Record 2 remains readable by compatible readers while offline, can become stale, and cannot be remotely wiped when a band is frozen online.
3. **Browser & PWA Capability Boundaries:**
   - Opening the HTTPS band URL works on compatible Android and iPhone NFC devices.
   - Direct in-browser Web NFC (`NDEFReader`) is limited (supported on Android Chromium, unavailable on iPhone/iOS and desktop browsers). Do not rely on Web NFC for iOS/PC workflows or for NTAG 424 DNA key provisioning, which requires a dedicated provisioning tool or PC/SC companion.

## 2. Cryptographic Verification & Durable Replay Authority (Rust)
- **Key Lookup & AES-128 Decryption:** Resolve the tag's key reference or diversification input without brute-forcing every patient's key. Decrypt `picc_data` to extract the 7-byte `uid` and 24-bit little-endian `read_counter`.
- **Constant-Time CMAC:** Verify the 8-byte truncated AES-CMAC (`c`) using `subtle::ConstantTimeEq` (`ct_eq`). Never use standard `==` on cryptographic signatures or MACs.
- **Durable Vault A Replay Acceptance (Primary Authority):**
  - **Vault A (`IdentityPool`) is the durable acceptance authority** (`docs/implementation/08-replay-and-clinical-authorization.md`, `20-state-machines-and-consistency.md`).
  - Inside a Vault A transaction, verify `band_status == 'ACTIVE'`, require `incoming_counter > stored_high_water_counter`, atomically advance the durable high-water counter, and persist a short-lived accepted-proof **tap context**.
  - **Redis Role:** Redis may coordinate fast-path checks and rate limits, but an empty, evicted, or restarted Redis cache must **never** authorize an old or replayed counter. Reconstruct or fall back to Vault A's durable high-water counter.
- **Single Consumption & Ingestion Resilience:**
  - Issue a short-lived bound tap context upon accepting a proof so refreshes, link previews, or the subsequent Patient PIN/OTP handshake reference the bound tap context rather than re-submitting and failing on the already-consumed raw NFC counter.
  - Remember that a valid counter proves a monotonic tag read after the previous accepted tap, not wall-clock recency (a captured unused proof could be delayed).
- **Band Status Enforcement:** Reject taps where the band is not `ACTIVE` (e.g., `FROZEN`, `REPLACED`, `RETIRED`, or `UNASSIGNED`) with an explicit, safe status response that discloses no clinical or identity data.
