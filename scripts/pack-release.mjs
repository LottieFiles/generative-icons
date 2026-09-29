import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

// Usage: npm run pack:release -- [output-dir] [--allow-dirty]
const root = fileURLToPath(new URL('../', import.meta.url));
const args = process.argv.slice(2);
const allowDirty = args.includes('--allow-dirty');
const positional = args.filter(arg => arg !== '--allow-dirty');
if (positional.length > 1 || positional.some(arg => arg.startsWith('--'))) throw Error('Usage: npm run pack:release -- [output-dir] [--allow-dirty]');
const output = path.resolve(positional[0] || path.join(root, 'dist/packages'));
const release = JSON.parse(fs.readFileSync(path.join(root, 'packages/icons/dist/release.json')));
const git = spawnSync('git', ['status', '--porcelain'], { cwd: root, encoding: 'utf8' });
if (git.status !== 0) throw Error('Could not read git status: ' + git.stderr);
const dirtyNow = Boolean(git.stdout.trim());
if ((release.sourceDirty || dirtyNow || release.status !== 'release') && !allowDirty) {
  console.error('Refusing to pack: the packages were built from, or the tree now has, uncommitted changes. Commit, run npm run build:packages, then pack again. Use --allow-dirty only for local testing; the result is status prerelease-dev and must not be published.');
  process.exit(1);
}
if (fs.existsSync(output)) throw Error('Release output exists. Use a fresh directory to preserve the previous pack.');
fs.mkdirSync(path.dirname(output), { recursive: true });
const stage = fs.mkdtempSync(path.join(path.dirname(output), '.pack-'));
const packages = [];
try {
  for (const name of ['icons', 'runtime', 'react', 'cli']) {
    const cwd = path.join(root, 'packages', name);
    const result = spawnSync('npm', ['pack', '--ignore-scripts', '--json', '--pack-destination', stage], { cwd, encoding: 'utf8' });
    if (result.status !== 0) throw Error(result.stderr);
    const metadata = JSON.parse(result.stdout)[0];
    for (const file of metadata.files) if (/(^|\/)(\.local|wip|unreleased|node_modules)(\/|$)/i.test(file.path)) throw Error('Private file entered a package: ' + file.path);
    const bytes = fs.readFileSync(path.join(stage, metadata.filename));
    packages.push({ name: metadata.name, version: metadata.version, file: metadata.filename, sha256: crypto.createHash('sha256').update(bytes).digest('hex'), bytes: bytes.length });
  }
  const status = release.status === 'release' && !dirtyNow ? 'release' : 'prerelease-dev';
  fs.writeFileSync(path.join(stage, 'release.json'), JSON.stringify({ ...release, status, sourceDirty: release.sourceDirty || dirtyNow, packages }, null, 2) + '\n');
  fs.renameSync(stage, output);
  console.log(`Packed ${packages.length} packages into ${output} (status ${status}). Nothing was published.`);
} finally { fs.rmSync(stage, { recursive: true, force: true }); }
