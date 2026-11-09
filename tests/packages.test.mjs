import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { connectIcon, validateIcon } from '../packages/runtime/dist/index.mjs';
import { rewriteStrokeWidth, readZip } from '../packages/runtime/dist/weight.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const catalog = JSON.parse(fs.readFileSync(new URL('../packages/icons/dist/catalog.json', import.meta.url)));
const stateful = catalog.icons.find(icon => icon.id === 'menu');
class Player {
  listeners = new Map(); currentFrame = 0; activeAnimationId = ''; destroyed = 0; slots = {};
  addEventListener(name, fn) { this.listeners.set(name, fn); }
  removeEventListener(name, fn) { if (this.listeners.get(name) === fn) this.listeners.delete(name); }
  pause() {} play() {} setLoop() {} setSegment() {} setSpeed() {}
  setFrame(frame) { this.currentFrame = frame; }
  loadAnimation(id) { this.activeAnimationId = id; }
  setTheme() {} setColorSlot(id, value) { this.slots[id] = value; }
  setScalarSlot(id, value) { this.slots[id] = value; }
  destroy() { this.destroyed++; }
}
test('every catalog icon satisfies the runtime contract; controller is canonical', () => {
  for (const icon of catalog.icons) validateIcon(icon);
  assert.deepEqual(fs.readFileSync(path.join(root, 'packages/runtime/dist/controller.mjs')), fs.readFileSync(path.join(root, 'prototype/icons/03/controller.mjs')));
  assert.equal(new Set(catalog.icons.map(i => i.id)).size, catalog.icons.length);
});
test('app state survives motion and reduced-motion changes without preview reset', async () => {
  const player = new Player(), media = new Player(); media.matches = false;
  const handle = connectIcon(stateful, player, { state: stateful.startState }, media);
  assert.equal(handle.state, stateful.startState);
  assert.equal(player.activeAnimationId, stateful.motions[0].reverse.name);
  const promise = handle.setState(stateful.endState);
  media.matches = true; media.listeners.get('change')();
  assert.equal(await promise, false); assert.equal(handle.state, stateful.endState);
  handle.update({ motion: 'strong', color: '#123456', style: 'duotone', weight: 2.5 });
  assert.equal(handle.state, stateful.endState); assert.equal(player.slots['style.tint'], 18);
  assert.equal(player.slots['style.outline.bold'], 100);
  await handle.setState(stateful.startState);
  assert.equal(handle.state, stateful.startState); assert.equal(player.currentFrame, stateful.motions[1].reverse.rest);
  handle.destroy(); handle.destroy(); assert.equal(player.destroyed, 1); assert.equal(media.listeners.size, 0);
});
test('runtime rejects invalid colors, state names and timing before use', () => {
  assert.throws(() => connectIcon(stateful, new Player(), { color: 'red' }), /hex/);
  assert.throws(() => connectIcon(stateful, new Player(), { state: 'invented' }), /state/);
  const invalid = structuredClone(stateful); invalid.motions[0].rest = Infinity;
  assert.throws(() => validateIcon(invalid), /timing/);
});
test('unsupported reverse leaves a gesture safe to change motion and play', async () => {
  const icon = catalog.icons.find(item => item.id === 'heart'), player = new Player();
  const handle = connectIcon(icon, player, { reducedMotion: true });
  assert.equal(await handle.play('reverse'), false);
  handle.update({ motion: 'strong' });
  assert.equal(await handle.play(), true);
  assert.equal(player.activeAnimationId, icon.motions[1].name);
  handle.destroy();
});
test('per-icon entry loads no catalog and npm contents exclude injected WIP', () => {
  const code = fs.readFileSync(path.join(root, 'packages/icons/dist/icons/heart.mjs'), 'utf8');
  assert.doesNotMatch(code, /import |catalog\.json/);
  const sentinels = ['.local/unreleased/sentinel.txt', 'unreleased/sentinel.txt', 'dist/unreleased/sentinel.txt', 'wip/sentinel.txt'];
  const created = [];
  try {
    for (const rel of sentinels) {
      const file = path.join(root, 'packages/runtime', rel), top = path.join(root, 'packages/runtime', rel.split('/').slice(0, rel.startsWith('dist/') ? 2 : 1).join('/'));
      if (!fs.existsSync(top)) created.push(top);
      fs.mkdirSync(path.dirname(file), { recursive: true }); fs.writeFileSync(file, 'UNRELEASED_SENTINEL');
    }
    const result = spawnSync('npm', ['pack', '--dry-run', '--ignore-scripts', '--json'], { cwd: path.join(root, 'packages/runtime'), encoding: 'utf8' });
    assert.equal(result.status, 0, result.stderr);
    const files = JSON.parse(result.stdout)[0].files;
    assert(files.some(file => file.path === 'dist/index.mjs'));
    assert(!files.some(file => /unreleased|wip\/|\.local|src\//.test(file.path)));
  } finally {
    for (const rel of sentinels) fs.rmSync(path.join(root, 'packages/runtime', rel), { force: true });
    for (const dir of created) fs.rmSync(dir, { recursive: true, force: true });
  }
});
test('release.json status reflects source provenance; dirty builds are never packed by default', () => {
  const release = JSON.parse(fs.readFileSync(path.join(root, 'packages/icons/dist/release.json')));
  const schema = JSON.parse(fs.readFileSync(path.join(root, 'schemas/release.schema.json')));
  for (const key of schema.required) assert(key in release, 'release.json lacks ' + key);
  assert.deepEqual(Object.keys(release).filter(key => !(key in schema.properties)), []);
  assert.equal(release.status, release.sourceDirty ? 'prerelease-dev' : 'release');
  const dirty = Boolean(spawnSync('git', ['status', '--porcelain'], { cwd: root, encoding: 'utf8' }).stdout.trim());
  if (!release.sourceDirty && !dirty) return;
  const output = path.join(fs.mkdtempSync(path.join(os.tmpdir(), 'icon-pack-')), 'out');
  try {
    const result = spawnSync(process.execPath, [path.join(root, 'scripts/pack-release.mjs'), output], { encoding: 'utf8' });
    assert.notEqual(result.status, 0); assert.match(result.stderr, /uncommitted/); assert.equal(fs.existsSync(output), false);
  } finally { fs.rmSync(path.dirname(output), { recursive: true, force: true }); }
});

test('any weight from 1 to 3 paints: authored layers at their stops, the stroke-width token in between', () => {
  const player = new Player();
  const handle = connectIcon(stateful, player, { weight: 2.3 });
  assert.equal(player.slots['style.outline.regular'], 100); assert.equal(player.slots['style.outline.bold'], 0);
  assert.equal(player.slots['style.outline.width'], 2.3);
  handle.update({ weight: 2.5 });
  assert.equal(player.slots['style.outline.bold'], 100); assert.equal(player.slots['style.outline.regular'], 0);
  assert.equal(player.slots['style.outline.width'], 2);
  handle.update({ style: 'solid', weight: 1.2 });
  assert.equal(player.slots['style.outline.regular'], 0); assert.equal(player.slots['style.solid'], 100);
  for (const weight of [0.5, 3.5, NaN, '2']) assert.throws(() => handle.update({ weight }), /weight must be a number from 1 to 3/);
});

test('in-between weights are written into the .lottie: every token-bound stroke, slot default and theme rule', async () => {
  const original = fs.readFileSync(path.join(root, 'packages/icons/dist/assets/heart.lottie'));
  const before = await readZip(original), after = await readZip(await rewriteStrokeWidth(original, 2.3));
  assert.deepEqual(after.map(e => e.name), before.map(e => e.name));
  let bound = 0, untouched = 0;
  const walk = shapes => { for (const s of shapes || []) { if (s.ty === 'gr') walk(s.it); else if (s.ty === 'st') { if (s.w.sid === 'style.outline.width') { bound++; assert.equal(s.w.k, 2.3); } else { untouched++; assert.notEqual(s.w.k, 2.3); } } } };
  for (const entry of after) {
    const json = JSON.parse(new TextDecoder().decode(entry.data));
    if (entry.name.startsWith('a/')) { for (const layer of json.layers) walk(layer.shapes); assert.equal(json.slots['style.outline.width'].p.k, 2.3); }
    if (entry.name.startsWith('t/')) assert.equal(json.rules.find(r => r.id === 'style.outline.width').value, 2.3);
    if (entry.name === 'manifest.json') assert.deepEqual(json, JSON.parse(new TextDecoder().decode(before.find(e => e.name === entry.name).data)));
  }
  assert.ok(bound > 0 && untouched > 0, `bound ${bound}, untouched ${untouched}`);
});
