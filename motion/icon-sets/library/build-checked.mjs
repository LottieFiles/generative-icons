// New generation entry: screen trusted local source before the existing builder.
// Keep versioned builders untouched so prior native provenance remains meaningful.
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { checkSet } from './preflight.mjs';

export async function buildChecked(root, launch = (file) => spawnSync(process.execPath, [file], { stdio: 'inherit' })) {
  const report = await checkSet(root);
  if (report.status !== 'PASS') return { ...report, built: false };
  const result = launch(path.join(root, 'build.mjs'));
  return { ...report, status: result.status === 0 ? 'PASS' : 'FAIL', built: result.status === 0, buildExitCode: result.status, error: result.error?.message };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    if (!process.argv[2] || process.argv.length > 3) throw Error('Usage: node build-checked.mjs <source-set-directory>');
    const report = await buildChecked(path.resolve(process.argv[2]));
    console.log(JSON.stringify(report, null, 2));
    if (report.status !== 'PASS') process.exitCode = 1;
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
