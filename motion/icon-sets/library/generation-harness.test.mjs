import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { checkDefinitions, checkSet, contract, parseGeometry } from './preflight.mjs';
import { loadCatalog, searchIcons, buildPacket, projectRoot } from './generation-context.mjs';
import { buildChecked } from './build-checked.mjs';

const fixture = () => ({ name: 'test-arrow', label: 'Test arrow', duotoneFill: false,
  parts: [{ name: 'shaft', d: 'M8 16 L24 16', type: 'stroke', width: 2, style: 'outline' }],
  timings: { light: { active: .4, rest: .5, keyPoses: [0, .4] }, strong: { active: .6, rest: .7, keyPoses: [0, .6] } } });
const has = (report, code) => report.diagnostics.some(d => d.code === code);

test('screening accepts current geometry without snapping fractional control points', () => {
  const icon = fixture(); icon.parts[0].d = 'M8.17 16 C11.31 12.12 20.39 16 24 16';
  const report = checkDefinitions([icon]);
  assert.equal(report.status, 'PASS'); assert.ok(report.samples >= 145 * 3 * 2);
});
test('malformed SVG cannot silently become an empty or partial native shape', () => {
  for (const d of ['', 'M4 4', 'M4 4 LNaN 8', 'M4 4 L9']) assert.throws(() => parseGeometry(d));
});
test('diagnostics identify profile, time and weight of topology failure', () => {
  const icon = fixture(); icon.parts[0].d = (t, strong, reverse, width) => strong && width === 2.5 && t > .2 ? 'M8 16 C12 12 20 12 24 16' : 'M8 16 L24 16';
  const report = checkDefinitions([icon]); const d = report.diagnostics.find(d => d.code === 'MORPH_TOPOLOGY');
  assert.equal(report.status, 'FAIL'); assert.equal(d.profile, 'strong'); assert.equal(d.weight, 'bold'); assert.ok(d.time > .2); assert.equal(d.part, 'shaft');
});
test('reverse and invisible geometry are still checked for finite coordinates', () => {
  const icon = fixture(); Object.assign(icon, { stateful: true, initialState: 'open', finalState: 'closed', reverseSuffix: '-reset', reverseTimings: icon.timings });
  icon.parts[0].visibility = () => 0;
  icon.parts[0].pose = (x, y, t, s, reverse) => reverse && t > .1 ? [NaN, y] : [x, y];
  const report = checkDefinitions([icon]);
  assert.equal(report.actions, 4); assert.ok(report.diagnostics.some(d => d.code === 'INVALID_GEOMETRY' && d.reverse));
});
test('source opacity overshoot is an error even if a player clamps it', () => {
  const icon = fixture(); icon.parts[0].visibility = t => t > .2 && t < .5 ? 1.06 : 1;
  assert.ok(has(checkDefinitions([icon]), 'VISIBILITY'));
});
test('invalid paint, Duotone policy and duplicate IDs fail independently', () => {
  const icon = fixture(); icon.parts[0].width = 1.7; icon.duotoneFill = true;
  const report = checkDefinitions([icon, icon]);
  for (const code of ['OUTLINE_WIDTH', 'DUOTONE_POLICY', 'DUPLICATE_ICON']) assert.ok(has(report, code));
});
test('native rigid pivots must be constant across the action', () => {
  const icon = fixture(), pose = (x, y) => [x, y]; pose.transform = t => ({ pivot: [16 + t, 16] }); icon.parts[0].pose = pose;
  assert.ok(has(checkDefinitions([icon]), 'MOVING_PIVOT'));
});
test('control hull bounds and ambiguous labels are advisories, not optical verdicts', () => {
  const icon = fixture(); icon.parts[0].d = 'M8 16 C-5 10 40 22 24 16'; icon.parts.push({ ...icon.parts[0], style: 'solid' });
  const report = checkDefinitions([icon]);
  assert.equal(report.status, 'PASS'); assert.ok(has(report, 'CONTROL_HULL_MARGIN')); assert.ok(has(report, 'AMBIGUOUS_LAYER_NAME'));
});
test('continuous sources allow rest marker zero without pretending to prove wrap velocity', () => {
  const icon = fixture(); icon.continuous = true;
  for (const timing of Object.values(icon.timings)) Object.assign(timing, { active: 2.4, rest: 0 });
  assert.equal(checkDefinitions([icon]).status, 'PASS');
});
test('checked build prevents the writer from running when preflight fails', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'icon-harness-fail-'));
  try {
    const icon = fixture(); icon.parts[0].width = 9;
    fs.writeFileSync(path.join(dir, 'definitions.mjs'), `export const icons=${JSON.stringify([icon])};`);
    let calls = 0;
    const report = await buildChecked(dir, () => { calls++; return { status: 0 }; });
    assert.equal(report.built, false); assert.equal(calls, 0); assert.deepEqual(fs.readdirSync(dir), ['definitions.mjs']);
  } finally { fs.rmSync(dir, { recursive: true }); }
});
test('checked build calls the existing builder only after clean source screening', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'icon-harness-pass-'));
  try {
    fs.writeFileSync(path.join(dir, 'definitions.mjs'), `export const icons=${JSON.stringify([fixture()])};`);
    const report = await buildChecked(dir, file => { assert.equal(file, path.join(dir, 'build.mjs')); return { status: 0 }; });
    assert.equal(report.built, true);
  } finally { fs.rmSync(dir, { recursive: true }); }
});
test('legacy inline source is disclosed, never imported as if it were a read-only definition', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'icon-harness-legacy-'));
  try { await assert.rejects(checkSet(dir), /legacy inline builders/i); }
  finally { fs.rmSync(dir, { recursive: true }); }
});
test('retrieval ranks exact names before aliases, stays deterministic and permits no match', () => {
  const icons = [{ id: 'bell', label: 'Bell', category: 'Essentials', intent: 'Alert' }, { id: 'notification', label: 'Notification', category: 'Feedback' }];
  assert.equal(searchIcons(icons, 'bell')[0].id, 'bell');
  assert.equal(searchIcons(icons, 'notification')[0].id, 'notification');
  assert.deepEqual(searchIcons(icons, 'unfindablezzz'), []);
  assert.deepEqual(searchIcons(icons, 'alert'), searchIcons([...icons].reverse(), 'alert'));
});
test('generation contract agrees with every existing catalog icon', () => {
  const { icons } = loadCatalog(); assert.ok(icons.length > 0);
  for (const icon of icons) {
    assert.equal(icon.width, contract.grid, icon.id); assert.equal(icon.height, contract.grid, icon.id);
    assert.deepEqual(icon.strokeWidth.presets, Object.values(contract.outlineWeights), icon.id);
    assert.deepEqual(icon.styles, contract.presentationStyles, icon.id);
    assert.equal(icon.fps, contract.fps); assert.equal(icon.duration, contract.durationSeconds);
  }
});
test('real reference packet embeds geometry and matches current packaged bytes', () => {
  const packet = buildPacket('notification feedback', ['bell']);
  assert.equal(packet.references.length, 1); assert.equal(packet.references[0].id, 'bell');
  assert.ok(packet.references[0].initialOutlineGeometry.length > 0);
  assert.ok(packet.inputs.some(i => i.path.endsWith('.lottie')));
  assert.deepEqual(packet.contract, contract);
  assert.throws(() => buildPacket('test', ['unknown-neighbor']), /Unknown neighbor/);
});
test('stale package metadata prevents misleading reference context', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'icon-context-stale-'));
  try {
    for (const relative of ['prototype/icons/catalog.json', 'motion/icon-sets/library/family-contract.json']) {
      fs.mkdirSync(path.dirname(path.join(dir, relative)), { recursive: true });
      fs.copyFileSync(path.join(projectRoot, relative), path.join(dir, relative));
    }
    const catalog = JSON.parse(fs.readFileSync(path.join(dir, 'prototype/icons/catalog.json')));
    catalog.sets = ['01/assets/set.json']; catalog.categories = catalog.categories.filter(c => c.sets.includes(catalog.sets[0]));
    fs.writeFileSync(path.join(dir, 'prototype/icons/catalog.json'), JSON.stringify(catalog));
    const assets = 'prototype/icons/01/assets'; fs.mkdirSync(path.join(dir, assets), { recursive: true });
    for (const file of ['set.json', 'bell-strong.json', 'bell-strong.lottie']) fs.copyFileSync(path.join(projectRoot, assets, file), path.join(dir, assets, file));
    fs.appendFileSync(path.join(dir, assets, 'bell-strong.lottie'), 'changed');
    assert.throws(() => buildPacket('bell', ['bell'], dir), /Stale package hash/);
    fs.copyFileSync(path.join(projectRoot, assets, 'bell-strong.lottie'), path.join(dir, assets, 'bell-strong.lottie'));
    const file = path.join(dir, assets, 'bell-strong.json'), native = JSON.parse(fs.readFileSync(file)); native.w = 31;
    fs.writeFileSync(file, JSON.stringify(native));
    assert.throws(() => buildPacket('bell', ['bell'], dir), /Loose native JSON differs/);
  } finally { fs.rmSync(dir, { recursive: true }); }
});

test('new source sets cannot duplicate a public icon ID', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'icon-catalog-collision-'));
  try {
    const icon = fixture(); icon.name = 'heart';
    fs.writeFileSync(path.join(dir, 'definitions.mjs'), `export const icons=${JSON.stringify([icon])};`);
    assert.ok(has(await checkSet(dir), 'CATALOG_ID_COLLISION'));
  } finally { fs.rmSync(dir, { recursive: true }); }
});
