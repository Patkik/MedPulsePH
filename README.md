# MedPulsePH

MedPulsePH is an early-stage hospital and patient platform centered on an NFC wristband and patient-authorized clinical access.

**Current status:** Development prototype only. Do not use real patient data. Login, RBAC enforcement, NFC verification, clinical records, and production security are not implemented.

- [Implementation plan](IMPLEMENTATION_PLAN.md)
- [Iteration ledger](docs/implementation/ITERATIONS.md)
- [Local Docker development](docs/implementation/I04-DOCKER-DEVELOPMENT.md)
- [React Native web preview](app/README.md)

The local Docker stack provides separate development PostgreSQL containers for the identity and clinical vaults, Redis, and a health-only Rust API. Generate local credentials with `node scripts/setup-local-env.mjs`, then run `docker compose up --build -d`. Consult the Docker development document for verification, limitations, and safe shutdown.
