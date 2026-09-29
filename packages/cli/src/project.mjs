import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

export const digest = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
export const json = value => JSON.stringify(value, null, 2) + '\n';
export const read = file => JSON.parse(fs.readFileSync(file, 'utf8'));
export function inside(root, relative) {
  if (typeof relative !== 'string' || !relative || relative.includes('\\') || path.isAbsolute(relative) || relative.split('/').some(p => !p || p === '.' || p === '..')) throw Error(`Expected a project-relative path: ${relative}`);
  const file = path.join(root, relative);
  let part = root;
  for (const name of relative.split('/')) {
    part = path.join(part, name);
    if (fs.existsSync(part) || (() => { try { fs.lstatSync(part); return true; } catch { return false; } })()) {
      if (fs.lstatSync(part).isSymbolicLink()) throw Error(`Symlinks are not allowed in managed paths: ${relative}`);
    }
  }
  return file;
}

// Validate every change before writing anything. Roll back a partial filesystem
// failure; never silently replace a file edited outside this tool.
export function writeBatch(root, files, expected = {}) {
  const plan = Object.entries(files).map(([relative, content]) => {
    const file = inside(root, relative), bytes = Buffer.from(content);
    const previous = fs.existsSync(file) ? fs.readFileSync(file) : null;
    if (previous && !previous.equals(bytes) && digest(previous) !== expected[relative]) throw Error(`Preserving modified file: ${relative}`);
    return { relative, file, bytes, previous };
  });
  const written = [];
  try {
    for (const item of plan) {
      if (item.previous?.equals(item.bytes)) continue;
      fs.mkdirSync(path.dirname(item.file), { recursive: true });
      // Recheck in case another process changed a file during preparation.
      const now = fs.existsSync(item.file) ? fs.readFileSync(item.file) : null;
      if (now === null ? item.previous !== null : !item.previous?.equals(now)) throw Error(`File changed during operation: ${item.relative}`);
      if (now === null) fs.writeFileSync(item.file, item.bytes, { flag: 'wx' });
      else fs.writeFileSync(item.file, item.bytes);
      written.push(item);
    }
  } catch (error) {
    for (const item of written.reverse()) if (item.previous) fs.writeFileSync(item.file, item.previous); else fs.unlinkSync(item.file);
    throw error;
  }
  return written.map(item => item.relative);
}

export function inspectProject(root) {
  const file = path.join(root, 'package.json');
  if (!fs.existsSync(file)) throw Error('Select an app folder containing package.json with --cwd <app>');
  const pkg = read(file), deps = { ...pkg.dependencies, ...pkg.devDependencies };
  if (pkg.workspaces) throw Error('This is a workspace root. Select an app with --cwd <app-folder>');
  const framework = deps.next ? 'next' : deps.react ? 'react' : deps.vue ? 'vue' : deps.svelte ? 'svelte' : 'vanilla';
  const manager = ['pnpm-lock.yaml', 'yarn.lock', 'bun.lock', 'package-lock.json'].find(name => fs.existsSync(path.join(root, name)));
  return { framework, packageManager: pkg.packageManager?.split('@')[0] || ({ 'pnpm-lock.yaml': 'pnpm', 'yarn.lock': 'yarn', 'bun.lock': 'bun' }[manager] || 'npm'), sourceDirectory: fs.existsSync(path.join(root, 'src')) ? 'src' : '.' };
}

export function readConfig(root) {
  const value = read(inside(root, 'lottie-icons.json'));
  if (value.schemaVersion !== 1) throw Error('Unsupported project configuration');
  for (const key of ['assets', 'descriptors']) inside(root, value[key]);
  if (!/^\/[a-zA-Z0-9_/-]+$/.test(value.publicPath) || value.publicPath.includes('..')) throw Error('publicPath must be a local URL path');
  const defaults = value.defaults;
  if (!defaults || !/^#[0-9a-f]{6}$/i.test(defaults.color) || !['outline', 'solid', 'duotone'].includes(defaults.style) || ![1.5, 2, 2.5].includes(defaults.weight) || !['light', 'strong'].includes(defaults.motion)) throw Error('Invalid project defaults');
  return value;
}
