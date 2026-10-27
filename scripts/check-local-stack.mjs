import { spawnSync } from 'node:child_process';

function checkContainer(label, service, command, expected) {
  const result = spawnSync('docker', ['compose', 'exec', '-T', service, 'sh', '-c', command], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  if (result.status !== 0 || result.stdout.trim() !== expected) {
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

const live = await fetch('http://127.0.0.1:8080/health/live', { signal: AbortSignal.timeout(5000) });
if (live.status !== 204) throw new Error('API liveness check failed.');
const missing = await fetch('http://127.0.0.1:8080/api/v1/fhir/Patient', { signal: AbortSignal.timeout(5000) });
if (missing.status !== 404) throw new Error('The unfinished clinical route was unexpectedly exposed.');
console.log('API liveness and disabled clinical route: passed');
