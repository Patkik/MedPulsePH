# MedPulsePH design preview

This is an early Expo/React Native web preview. Role selection changes example content locally; it does not authenticate a user or grant permissions. No patient records, NFC verification, clinical workflows, or backend connections are implemented. Do not use this preview with real patient data.

Install dependencies with `npm ci`, start the web preview with `npm run web`, and check types with `npm run typecheck`. Build the static web output with `npm run build:web`.

The web manifest supports a standalone presentation where the browser supports installation. There is no service worker or offline clinical functionality. Icon and branding are provisional and still need the physical and visual validation in the logo specification.

The dependency audit currently reports a high-severity advisory ([GHSA-mrhx-c7xc-m3w5](https://github.com/advisories/GHSA-mrhx-c7xc-m3w5)) in the Expo CLI's transitive `zod` dependency. No compatible non-breaking fix has been established yet. This preview is not ready for production or real patient data.
