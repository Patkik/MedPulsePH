---
name: nfc-sun-crypto
description: Implement, audit, or debug 13.56 MHz NFC wristband payloads, NXP NTAG 424 DNA SUN (Secure Unique NFC) AES-128 decryption, CMAC verification, NDEF dual-record payloads, and Redis tap-counter anti-replay logic. Use whenever touching NFC tap routes, cryptographic tag verification, or USB PC/SC reader modules.
allowed-tools: Read Edit Grep Glob Bash(cargo test*) Bash(cargo check*)
---

# NFC Wristband & SUN Cryptographic Skill

## Core Hardware & Protocol Rules
1. **Frequency & Standard:** Always target 13.56 MHz HF NFC (ISO/IEC 14443-A / NFC Forum Type 2 or Type 4). Never generate code for 125 kHz LF RFID.
2. **Dual NDEF Structure:**
   - **Record 1 (Online URI):** `https://{domain}/v1/tap?d={picc_data_hex}&c={cmac_hex}`
   - **Record 2 (Offline Fallback):** Compact MIME/Text triage string (`BloodType|Allergies|ICE_Phone`) under 120 bytes.

## Cryptographic Verification Checklist (Rust)
- **AES-128 Decryption:** Decrypt `picc_data` using the tag's derived AES key to extract the 7-byte `uid` and 24-bit little-endian `read_counter`.
- **Constant-Time CMAC:** Verify the 8-byte truncated AES-CMAC (`c`) using `subtle::ConstantTimeEq` (`ct_eq`). Never use standard `==` on cryptographic signatures or MACs.
- **Redis Anti-Replay Check:**
  - Fetch `nfc:counter:{tag_uid_hex}` from Redis.
  - Require `incoming_counter` to be strictly greater than `stored_counter`.
  - Atomically update the counter in Redis using a Lua script or `SET` with condition before returning Tier-1 data.
- **Band Freeze Check:** Immediately reject taps where `band_status != 'ACTIVE'` in Vault A, returning a safe 403 locked response.
