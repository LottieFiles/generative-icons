// A real temporary native export, not just an executable/version check.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { exporterConfig } from './exporter-config.mjs';

const NO_EXPORTER = `Dependencies OK. Native export is not configured (CREATOR_MCP_BIN is not set).

The Engine exporter turns icon source into native Lottie assets. It is not
publicly distributed, so most contributors will not have it.

You can still:
  - preview and use every icon in this repository
  - author icon source and run preflight: npm run check:source

To ship a new or changed icon, open a pull request with your source changes.
A maintainer runs the native export, reviews the rendered output and merges.

Maintainers with the exporter: set CREATOR_MCP_BIN to its absolute path,
then run npm run doctor again. See docs/development.md.`;

let scratch;
try {
  if (process.argv.length !== 2) throw Error('Usage: npm run doctor');
  if (Number(process.versions.node.split('.')[0]) < 22) throw Error('Node 22 or newer is required.');
  for (const command of ['python3', 'unzip']) {
    const r = spawnSync(command, [command === 'python3' ? '--version' : '-v'], { encoding: 'utf8', timeout: 10000 });
    if (r.error || r.status !== 0) throw Error(`${command} is required and must be available on PATH.`);
  }
  try { await import('svgpath'); }
  catch { throw Error('Authoring dependencies are missing. Run npm run setup.'); }
  if (!process.env.CREATOR_MCP_BIN && !process.env.ENGINE_MCP_ROOT) {
    console.log(NO_EXPORTER);
    process.exitCode = 1;
    process.exit();
  }
  exporterConfig();
  scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'icon-doctor-'));
  const here = path.dirname(fileURLToPath(import.meta.url));
  const scene = { name: 'Exporter compatibility probe', width: 32, height: 32,
    fps: 60, duration: 1, nodes: [{ name: 'probe', commands: [0, 1, 1, 3],
      points: [[0, [8, 24, 16, 8, 24, 24]], [1, [8, 24, 16, 8, 24, 24]]],
      type: 'fill', color: '#008c76', opacity: [[0, 1], [1, 1]], width: [[0, 2], [1, 2]] }] };
  const script = path.join(scratch, 'probe.engine.js'), input = path.join(scratch, 'probe.scene.json'), output = path.join(scratch, 'probe.json');
  fs.writeFileSync(script, fs.readFileSync(path.join(here, 'engine-scene.js'), 'utf8').replace('/* SCENE_DATA */', 'const scene = params.scene;'));
  fs.writeFileSync(input, JSON.stringify(scene));
  console.log('Checking dependencies and exporting a temporary native scene…');
  const result = spawnSync(process.execPath, [path.join(here, 'export.mjs'), script, output, input],
    { encoding: 'utf8', timeout: 120000, maxBuffer: 1024 * 1024 });
  if (result.error || result.status !== 0) throw Error(`Native export probe failed: ${result.error?.message || result.stderr || result.stdout}`);
  const native = JSON.parse(fs.readFileSync(output, 'utf8'));
  if (native.w !== 32 || native.h !== 32 || !native.layers?.length) throw Error('Probe returned an unexpected native composition.');
  console.log('PASS: dependencies and native export. Browser rendering and icon visual quality still require review.');
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
} finally {
  if (scratch) fs.rmSync(scratch, { recursive: true, force: true });
}
