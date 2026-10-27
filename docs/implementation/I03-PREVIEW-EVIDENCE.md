# I03 preview evidence

**Scope:** The first Expo/React Native web design preview only. It uses synthetic interface copy and does not implement accounts, authorization, NFC, patient records, or clinical workflows. I03 remains in progress.

## Checks completed

- `npm run typecheck` passed.
- `npm run lint` passed.
- `npm run build:web` produced a static Expo web export containing the preview and web manifest.
- The built site was served locally and inspected in a browser. Selecting the Doctor role changed the heading and planned workspace actions. This is client-side preview state, not an access-control check.
- `node scripts/check-skill-sync.mjs` passed for the six mirrored skill instructions and the project instruction files. Semantic review remains necessary.

## Open items

- The current workstation has no host Rust toolchain. The separate I04 Docker setup now provides a health-only Rust API and containerized checks; it does not provide clinical or authentication functionality.
- The npm audit reports [GHSA-mrhx-c7xc-m3w5](https://github.com/advisories/GHSA-mrhx-c7xc-m3w5) through Expo CLI's transitive `zod` dependency. The suggested automatic remediation changes Expo's major version; a compatible, verified fix remains open.
- Browser inspection covered one desktop view and the Doctor selection. Mobile viewport, screen readers, keyboard interaction, and other target devices have not yet been verified.
- The SVG icon (`app/public/icon.svg`), standalone mark (`app/public/mark.svg`), horizontal lockup (`app/public/logo-lockup.svg`), and 5-panel visual stress-test sheet (`app/public/logo-stress-test.html`) implement the `64×64` integer-grid Clasp mark with an `8`-unit central channel; hospital stakeholder sign-off and physical `8mm` wristband engraving inspection remain open.
- The screen and state permission catalog (`S01`–`S10`) is documented in `docs/implementation/28-screen-and-api-specifications.md`; hospital user wireframe reviews and live authenticated screen routes remain open.
- The web manifest is present, but installability has not been verified across browsers. No service worker or offline clinical support exists.
- I01 hospital and privacy decisions have not been approved, and the I02, I04, and I05 security prerequisites remain incomplete. Do not use real patient data.
