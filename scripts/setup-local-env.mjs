import { randomBytes } from 'node:crypto';
import { writeFile } from 'node:fs/promises';

const names = ['IDENTITY_OWNER_PASSWORD', 'IDENTITY_APP_PASSWORD', 'CLINICAL_OWNER_PASSWORD', 'CLINICAL_APP_PASSWORD'];
const contents = `${names.map((name) => `${name}=${randomBytes(24).toString('hex')}`).join('\n')}\n`;
try {
  await writeFile('.env', contents, { encoding: 'utf8', flag: 'wx', mode: 0o600 });
  console.log('Created local development credentials in .env. Do not use real patient data.');
} catch (error) {
  if (error.code === 'EEXIST') {
    console.log('.env already exists; it was not modified.');
  } else {
    throw error;
  }
}
