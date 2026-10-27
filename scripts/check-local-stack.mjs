import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';

function checkContainer(label, service, command, expected, input = undefined) {
  const result = spawnSync('docker', ['compose', 'exec', '-T', service, 'sh', '-c', command], {
    encoding: 'utf8',
    input,
    stdio: [input !== undefined ? 'pipe' : 'ignore', 'pipe', 'pipe'],
  });
  if (result.status !== 0 || (expected !== null && result.stdout.trim() !== expected)) {
    throw new Error(`${label} failed. Inspect the local container without exposing credentials.`);
  }
  console.log(`${label}: passed`);
}

const userQuery = 'SELECT current_user, rolsuper, rolcreatedb FROM pg_roles WHERE rolname = current_user';
for (const [service, user, database] of [
  ['identity-db', 'identity_app', 'identity_vault'],
  ['clinical-db', 'clinical_app', 'clinical_vault'],
]) {
  checkContainer(
    `${service} runtime account`, service,
    `PGPASSWORD="$APP_DB_PASSWORD" psql -w -h 127.0.0.1 -U ${user} -d ${database} -Atc '${userQuery}'`,
    `${user}|f|f`,
  );
}

checkContainer(
  'Cross-vault account denial', 'clinical-db',
  'if PGPASSWORD="$APP_DB_PASSWORD" psql -w -h 127.0.0.1 -U identity_app -d clinical_vault -c "SELECT 1" >/dev/null 2>&1; then exit 1; else echo denied; fi',
  'denied',
);
checkContainer(
  'Direct database network isolation', 'identity-db',
  'getent hosts identity-db >/dev/null 2>&1 || exit 1; if getent hosts clinical-db >/dev/null 2>&1; then exit 1; else echo isolated; fi',
  'isolated',
);
checkContainer(
  'Reverse database network isolation', 'clinical-db',
  'getent hosts clinical-db >/dev/null 2>&1 || exit 1; if getent hosts identity-db >/dev/null 2>&1; then exit 1; else echo isolated; fi',
  'isolated',
);

const vaultASql = readFileSync('backend/sql/vault_a_identity.sql', 'utf8');
checkContainer(
  'Vault A schema and least-privilege grants', 'identity-db',
  'PGPASSWORD="$POSTGRES_PASSWORD" psql -w -v ON_ERROR_STOP=1 -h 127.0.0.1 -U identity_owner -d identity_vault >/dev/null',
  null,
  vaultASql,
);

const vaultBSql = readFileSync('backend/sql/vault_b_clinical.sql', 'utf8');
checkContainer(
  'Vault B schema and append-only audit trigger', 'clinical-db',
  'PGPASSWORD="$POSTGRES_PASSWORD" psql -w -v ON_ERROR_STOP=1 -h 127.0.0.1 -U clinical_owner -d clinical_vault >/dev/null',
  null,
  vaultBSql,
);

checkContainer(
  'Vault A runtime DDL denial', 'identity-db',
  'if PGPASSWORD="$APP_DB_PASSWORD" psql -w -v ON_ERROR_STOP=1 -h 127.0.0.1 -U identity_app -d identity_vault -c "DROP TABLE nfc_bands" >/dev/null 2>&1; then exit 1; else echo denied; fi',
  'denied',
);

checkContainer(
  'Vault B non-blind patient ID rejection', 'clinical-db',
  `if PGPASSWORD="$APP_DB_PASSWORD" psql -w -v ON_ERROR_STOP=1 -h 127.0.0.1 -U clinical_app -d clinical_vault -c "INSERT INTO immutable_audit_logs (blind_patient_id, doctor_actor_id, hospital_id, session_reference, action_type, recorded_at_unix) VALUES ('raw-patient-name', 'doc-1', 'hosp-1', 'sess-1', 'read_chart', 1700000000)" >/dev/null 2>&1; then exit 1; else echo rejected; fi`,
  'rejected',
);

checkContainer(
  'Vault B append-only audit log enforcement', 'clinical-db',
  `PGPASSWORD="$APP_DB_PASSWORD" psql -w -v ON_ERROR_STOP=1 -h 127.0.0.1 -U clinical_app -d clinical_vault -c "INSERT INTO immutable_audit_logs (blind_patient_id, doctor_actor_id, hospital_id, session_reference, action_type, recorded_at_unix) VALUES ('bp_v1_0000000000000000000000000000000000000000000000000000000000000001', 'doc-check', 'hosp-check', 'sess-check', 'synthetic_verify', 1700000000)" >/dev/null && if PGPASSWORD="$APP_DB_PASSWORD" psql -w -v ON_ERROR_STOP=1 -h 127.0.0.1 -U clinical_app -d clinical_vault -c "DELETE FROM immutable_audit_logs WHERE session_reference = 'sess-check'" >/dev/null 2>&1; then exit 1; fi && if PGPASSWORD="$POSTGRES_PASSWORD" psql -w -v ON_ERROR_STOP=1 -h 127.0.0.1 -U clinical_owner -d clinical_vault -c "UPDATE immutable_audit_logs SET action_type = 'tampered' WHERE session_reference = 'sess-check'" >/dev/null 2>&1; then exit 1; else echo append_only; fi`,
  'append_only',
);

const live = await fetch('http://127.0.0.1:8080/health/live', { signal: AbortSignal.timeout(5000) });
if (live.status !== 204) throw new Error('API liveness check failed.');
const missing = await fetch('http://127.0.0.1:8080/api/v1/fhir/Patient', { signal: AbortSignal.timeout(5000) });
if (missing.status !== 404) throw new Error('The unfinished clinical route was unexpectedly exposed.');
console.log('API liveness and disabled clinical route: passed');
