# MedPulsePH web application

The public home page describes the planned service. Sign-in, emergency access, and patient and staff workspaces are currently unavailable. The website does not authenticate users or connect to patient records. Do not use it with real patient data.

Install dependencies with `npm ci`, run locally with `npm run web`, and verify with `npm run typecheck`, `npm run lint`, and `npm run build:web`.

The web manifest supports a standalone presentation where the browser supports installation. There is no service worker or offline clinical functionality. Branding is provisional and still needs the physical and visual validation in the logo specification.

The dependency audit previously reported a high-severity advisory ([GHSA-mrhx-c7xc-m3w5](https://github.com/advisories/GHSA-mrhx-c7xc-m3w5)) in the Expo CLI's transitive `zod` dependency. This application is not ready for production or real patient data.
