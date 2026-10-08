// The runtime's SVG export against the golden files gai-svg is also held to.
// UPDATE_SVG_GOLDENS=1 node --test tests/svg.test.mjs rewrites them.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { iconToSvg, dotLottieToSvg, defaultTrigger, lottieToSvg } from '../packages/runtime/src/svg.mjs';

const root = new URL('../', import.meta.url);
const golden = new URL('crates/gai-svg/tests/golden/', root);
const { cases } = JSON.parse(fs.readFileSync(new URL('cases.json', golden)));
const animations = set => name => JSON.parse(fs.readFileSync(new URL(`prototype/icons/${set}/assets/${name}.json`, root)));

test('exports match the golden files', () => {
  for (const c of cases) {
    const svg = iconToSvg(c.icon, animations(c.set), c.options);
    const file = new URL(c.file, golden);
    if (process.env.UPDATE_SVG_GOLDENS) fs.writeFileSync(file, svg);
    assert.equal(svg, fs.readFileSync(file, 'utf8'), `${c.file}: ${c.why}`);
  }
});

test('the rest pose is the static fallback and every keyTimes list is valid', () => {
  for (const c of cases) {
    const svg = fs.readFileSync(new URL(c.file, golden), 'utf8');
    for (const [element] of svg.matchAll(/<animate(Transform)? [^>]*>/g)) {
      const values = element.match(/values="([^"]*)"/)[1].split(';');
      const keyTimes = element.match(/keyTimes="([^"]*)"/)?.[1].split(';').map(Number);
      if (!keyTimes) { assert.equal(values.length, 1, element); continue; }
      assert.equal(keyTimes.length, values.length, element);
      assert.equal(keyTimes[0], 0, element);
      assert.ok(keyTimes.every((t, i) => i === 0 || t >= keyTimes[i - 1]), `keyTimes decrease: ${element}`);
      if (!element.includes('discrete')) assert.equal(keyTimes.at(-1), 1, element);
      const splines = element.match(/keySplines="([^"]*)"/)?.[1].split(';');
      if (splines) {
        assert.equal(splines.length, values.length - 1, element);
        assert.ok(splines.every(s => s.split(' ').map(Number).every(v => v >= 0 && v <= 1)), element);
      }
    }
    // SMIL references break on '-' or '.' inside ids
    for (const [, id] of svg.matchAll(/ id="([^"]*)"/g)) assert.match(id, /^[A-Za-z][A-Za-z0-9_]*$/);
  }
});

test('triggers follow the icon: toggle when stateful, loop for loaders, hover otherwise', () => {
  const byId = id => cases.find(c => c.icon.id === id).icon;
  assert.equal(defaultTrigger(byId('sort-order')), 'toggle');
  assert.equal(defaultTrigger(byId('searching')), 'loop');
  assert.equal(defaultTrigger(byId('bell')), 'hover');
  const bell = cases.find(c => c.icon.id === 'bell');
  assert.throws(() => iconToSvg(bell.icon, animations(bell.set), { trigger: 'toggle' }), /not stateful/);
  assert.throws(() => iconToSvg(byId('sort-order'), animations('28'), { trigger: 'hover' }), /stateful/);
  assert.throws(() => iconToSvg(bell.icon, animations(bell.set), { color: 'red' }), /color/);
  assert.throws(() => iconToSvg(bell.icon, animations(bell.set), { weight: 4 }), /weight/);
  assert.throws(() => iconToSvg(bell.icon, animations(bell.set), { id: 'a.b' }), /id/);
  assert.throws(() => lottieToSvg({ layers: [{ ty: 0, nm: 'precomp', ks: {} }], w: 32, h: 32, fr: 60, ip: 0, op: 10 }), /Unsupported Lottie/);
});

test('a .lottie converts the same as its animations', async () => {
  const c = cases.find(c => c.file === 'sort-order.svg');
  const bytes = fs.readFileSync(new URL(`prototype/icons/${c.set}/assets/${c.icon.id}.lottie`, root));
  // The per-action .lottie in a set holds every animation of the icon.
  assert.equal(await dotLottieToSvg(c.icon, bytes, c.options), fs.readFileSync(new URL(c.file, golden), 'utf8'));
});
