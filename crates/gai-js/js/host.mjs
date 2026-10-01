// gai:host, the bootstrap gai-js drives. bind() takes a set's icons, install() applies a
// sampling profile, describe() reports the set, sampleVariant() samples one animation and
// probe() evaluates one unexpanded part at chosen times.
//
// Sampling follows the old builders (library/build-set.mjs:86-124, 01/02 build.mjs)
// call for call, and rounding stays in JS. Numbers leave through typed arrays or tape(),
// never as text: QuickJS's Number::toString differs from V8's at powers of two.
import svgpath from 'svgpath';

// The builtins the host calls, captured before any definitions load, so authoring code
// that replaces one cannot change what the host computes or hands to Rust. The host's own
// arrays avoid species and iteration. Values the definitions own (icons, parts, callback
// results) keep their JS semantics, and svgpath uses the builtins it finds, so this is no
// sandbox: definitions can still behave differently under preflight than under a build.
// Review trusts the Rust checks of the exported files, not this.
const uncurry = Function.prototype.bind.bind(Function.prototype.call); // uncurry(f)(self, ...args)
const { Array, Boolean, Error, Float64Array, Int32Array, JSON, Map, Math, Number, Object, Set, String, Uint8Array } =
  globalThis;
const { isArray } = Array, { isFinite: finite } = Number, { stringify } = JSON, { ceil, PI, round: mathRound } = Math;
const { assign, defineProperty, entries, getOwnPropertyDescriptor, getPrototypeOf } = Object;
const [push, every, some, forEach, join, indexOf, includes, fill] =
  ['push', 'every', 'some', 'forEach', 'join', 'indexOf', 'includes', 'fill'].map(name => uncurry(Array.prototype[name]));
const TypedArray = getPrototypeOf(Int32Array.prototype);
const typedFill = uncurry(TypedArray.fill), typedSet = uncurry(TypedArray.set), copyWithin = uncurry(TypedArray.copyWithin);
const typedLength = uncurry(getOwnPropertyDescriptor(TypedArray, 'length').get);
const mapGet = uncurry(Map.prototype.get), mapSet = uncurry(Map.prototype.set);
const setAdd = uncurry(Set.prototype.add), setSize = uncurry(getOwnPropertyDescriptor(Set.prototype, 'size').get);
const trim = uncurry(String.prototype.trim), charCodeAt = uncurry(String.prototype.charCodeAt);
const isPrototypeOf = uncurry(Object.prototype.isPrototypeOf), NumberPrototype = Number.prototype;

// array.map(f) on the host's own arrays.
function mapped(array, f) {
  const out = [];
  for (let i = 0; i < array.length; i++) push(out, f(array[i], i));
  return out;
}

// Type.from(array) without the iteration protocol.
function typed(Type, array) {
  const out = new Type(array.length);
  for (let i = 0; i < array.length; i++) out[i] = array[i];
  return out;
}

// Object.fromEntries(pairs) without the iteration protocol.
function fromEntries(pairs) {
  const out = {};
  for (let i = 0; i < pairs.length; i++) {
    defineProperty(out, pairs[i][0], { __proto__: null, value: pairs[i][1], writable: true, enumerable: true, configurable: true });
  }
  return out;
}

const FPS = 60, DURATION = 2.4, FRAMES = mathRound(DURATION * FPS) + 1; // 145
const MOTIONS = ['light', 'strong'];
const WEIGHTS = [['light', 1.5], ['regular', 2], ['bold', 2.5]];
const round = (value) => mathRound(value * 1e5) / 1e5;
const pair = p => isArray(p) && p.length === 2 && every(p, finite);

// library/build-set.mjs:26-51. svgpath's segments are [cmd, ...args] with 2 args for M
// and L, 6 for C and 1 for H and V.
function geometry(d) {
  const commands = [],
    points = [];
  let last = [0, 0];
  const segments = svgpath(d).abs().unarc().unshort().segments;
  for (let s = 0; s < segments.length; s++) {
    const segment = segments[s], cmd = segment[0], n = segment.length;
    if (cmd === "M" || cmd === "L" || cmd === "C") {
      push(commands, cmd === "M" ? 0 : cmd === "L" ? 1 : 2);
      for (let i = 1; i < n; i++) push(points, segment[i]);
      last = [segment[n - 2], segment[n - 1]];
    } else if (cmd === "H") {
      push(commands, 1);
      last = [segment[1], last[1]];
      push(points, last[0], last[1]);
    } else if (cmd === "V") {
      push(commands, 1);
      last = [last[0], segment[1]];
      push(points, last[0], last[1]);
    } else if (cmd === "Z") push(commands, 3);
    else throw new Error(`Unsupported command ${cmd}`);
  }
  return { commands, points };
}

// Parsing is pure, so each static path is parsed once.
const staticGeometries = new Map();
function staticGeometry(d) {
  let geo = mapGet(staticGeometries, d);
  if (!geo) {
    geo = geometry(d);
    mapSet(staticGeometries, d, geo);
  }
  return geo;
}

// library/preflight.mjs:18-37, with preflight's own wording.
function parseGeometry(d) {
  if (typeof d !== 'string' || !trim(d)) throw Error('Path must be a nonempty SVG path string');
  const parsed = svgpath(d);
  if (parsed.err) throw Error(parsed.err);
  const segments = parsed.abs().unarc().unshort().segments;
  const commands = [], points = [];
  let last = [0, 0];
  for (let s = 0; s < segments.length; s++) {
    const segment = segments[s], cmd = segment[0], n = segment.length;
    for (let i = 1; i < n; i++) if (!finite(segment[i])) throw Error('Nonfinite path coordinates');
    if (cmd === 'M' || cmd === 'L' || cmd === 'C') {
      push(commands, cmd);
      for (let i = 1; i < n; i++) push(points, segment[i]);
      last = [segment[n - 2], segment[n - 1]];
    } else if (cmd === 'H' || cmd === 'V') {
      last = cmd === 'H' ? [segment[1], last[1]] : [last[0], segment[1]];
      push(commands, 'L'); push(points, last[0], last[1]);
    } else if (cmd === 'Z') push(commands, cmd);
    else throw Error(`Unsupported native command ${cmd}`);
  }
  if (commands[0] !== 'M' || !some(commands, c => c === 'L' || c === 'C')) throw Error('Path has no drawable segment');
  return { topology: join(commands, ''), points };
}

// probe() parses each static path once. Parsing is pure, so a failure is rethrown as is.
const staticParses = new Map();
function parseStatic(d) {
  let parsed = mapGet(staticParses, d);
  if (!parsed) {
    try {
      parsed = { __proto__: null, geo: parseGeometry(d) };
    } catch (error) {
      parsed = { __proto__: null, error };
    }
    mapSet(staticParses, d, parsed);
  }
  if ('error' in parsed) throw parsed.error;
  return parsed.geo;
}

// Reverse animation names: 03-essentials/build.mjs:93 hardcodes them, later builders read reverseSuffix.
const reverseNames = {
  suffix: base => base.reverseSuffix,
  essentials: base => base.name === 'check' ? '-reset' : '-unlock',
};

let S = null;

// The part being evaluated, for error reports.
export function currentPart() {
  return S?.part;
}

// Keeps the icons and each icon's unexpanded parts. probe() works from here on.
export function bind(icons) {
  if (S) throw Error('gai:host: bind() called twice in one runtime');
  S = {
    icons,
    sourceParts: isArray(icons) ? icons.map(icon => icon?.parts) : [],
    profile: null,
    variants: new Map(),
    part: undefined,
  };
}

// profile: { kind: 'modern', passWidth, checkDuotone, reverseName: 'suffix' | 'essentials' }
//        | { kind: 'legacy', shim: <gai:legacy/NN namespace> }
export function install(profile) {
  if (!S) throw Error('gai:host: bind() must come before install()');
  if (S.profile) throw Error('gai:host: install() called twice in one runtime');
  const definitions = S.icons;
  if (!isArray(definitions)) throw Error('Definitions must export an icons array');
  if (profile.kind === 'legacy') {
    S.profile = { kind: 'legacy', strongOverrides: profile.shim.strongOverrides, pose: profile.shim.pose };
  } else {
    S.profile = { kind: 'modern', passWidth: profile.passWidth, reverseName: reverseNames[profile.reverseName] };
    // build-set.mjs:14; the 03-06 builders lack it, which changes nothing there.
    const names = definitions.map(i=>i.name), unique = new Set();
    for (let k = 0; k < names.length; k++) setAdd(unique, names[k]);
    if (setSize(unique) !== definitions.length) throw Error("Duplicate icon name");
    // build-set.mjs:86 (07+ builders), before the weight expansion.
    if (profile.checkDuotone) for(const icon of definitions)if(icon.duotoneFill!==undefined && icon.duotoneFill!==icon.parts.some(p=>p.style==='tint'))throw Error('Declared Duotone policy disagrees with drawing: '+icon.name);
  }
  // build-set.mjs:91-93
  for (const icon of definitions) icon.parts = icon.parts.flatMap(part =>
    part.style === "outline" ? mapped(WEIGHTS, weight =>
      ({ ...part, name: `${part.name}-${weight[0]}`, style: `outline.${weight[0]}`, width: weight[1] })) : [part]);
  S.sourceIndex = mapped(S.sourceParts, parts => parts.flatMap((part, j) => part.style === 'outline' ? [j, j, j] : [j]));
}

function installed(what) {
  if (!S?.profile) throw Error(`gai:host: install() must come before ${what}()`);
}

// The animations of one icon in builder order: light, light reverse, strong, strong reverse.
function variantsOf(iconIndex) {
  const known = mapGet(S.variants, iconIndex);
  if (known) return known;
  const base = S.icons[iconIndex], out = [];
  if (S.profile.kind === 'legacy') {
    // 01-heart-bell/build.mjs:172-184, 02-search/build.mjs:131-138
    for (let m = 0; m < MOTIONS.length; m++) {
      const motionId = MOTIONS[m], strong = motionId === 'strong';
      const icon = { ...base, icon: base.name, motionId, motionLabel: strong ? 'Strong motion' : 'Light motion', name: base.name + (strong ? '-strong' : '') };
      assign(icon, S.profile.strongOverrides(base, strong));
      push(out, { icon, strong, reverse: false });
    }
  } else {
    // build-set.mjs:95-99
    for (let m = 0; m < MOTIONS.length; m++) {
      const motionId = MOTIONS[m], reverses = base.stateful ? [false,true] : [false];
      for (let r = 0; r < reverses.length; r++) {
        const reverse = reverses[r], strong=motionId==='strong', timing=(reverse?base.reverseTimings:base.timings)[motionId];
        const icon={...base,...timing,icon:base.name,motionId,motionLabel:strong?'Strong motion':'Light motion',direction:reverse?'reverse':'forward',name:base.name+(reverse?S.profile.reverseName(base):'')+(strong?'-strong':'')};
        icon.rest=ceil(icon.rest*FPS)/FPS;
        push(out, { icon, strong, reverse });
      }
    }
  }
  mapSet(S.variants, iconIndex, out);
  return out;
}

function withoutKeys(value, keys) {
  const pairs = entries(value), kept = [];
  for (let i = 0; i < pairs.length; i++) if (!includes(keys, pairs[i][0])) push(kept, pairs[i]);
  return fromEntries(kept);
}
const callbacks = part => ({
  d: typeof part.d,
  pose: typeof part.pose,
  visibility: typeof part.visibility,
  transform: Boolean(part.pose && part.pose.transform),
});

// Plain data for Rust's project(): nothing here is formatted by QuickJS.
export function describe() {
  installed('describe');
  const legacy = S.profile.kind === 'legacy';
  return {
    kind: S.profile.kind,
    frames: FRAMES,
    icons: S.icons.map((base, i) => ({
      meta: withoutKeys(base, ['parts']),
      sourceParts: S.sourceParts[i].map(part => ({ callbacks: callbacks(part), meta: withoutKeys(part, ['d', 'pose', 'visibility']) })),
      parts: base.parts.map((part, k) => ({
        sourceIndex: S.sourceIndex[i][k],
        callbacks: callbacks(part),
        meta: withoutKeys(part, ['d', 'pose', 'visibility']),
        // build-set.mjs:124, 01-heart-bell/build.mjs:205-217
        node: {
          name: legacy ? `${part.name}__${part.slot}__${part.style}` : `${part.name}__color.primary__${part.style}`,
          type: part.type || 'fill',
          opacity: part.style === 'outline.regular' ? 1 : 0,
          width: part.width || 1,
        },
      })),
      variants: mapped(variantsOf(i), ({ icon, strong, reverse }) => ({ name: icon.name, strong, reverse, meta: withoutKeys(icon, ['parts']) })),
    })),
  };
}

// One animation, packed for Rust:
//   header   Int32Array  [nParts, then per part: nCommands, nPointFloats, flags (bit0 rigid), cmdOffset, dataOffset]
//   commands Uint8Array  each part's command codes (0 M, 1 L, 2 C, 3 Z)
//   data     Float64Array per part: frames[FRAMES*n] posed and rounded, frame-major
//                         | visibility[FRAMES] rounded (modern only)
//                         | if rigid: anchor[2] | basePoints[n] rounded | position[2*FRAMES] | rotation[FRAMES]
export function sampleVariant(iconIndex, variantIndex) {
  installed('sampleVariant');
  const { icon, strong, reverse } = variantsOf(iconIndex)[variantIndex];
  const legacy = S.profile.kind === 'legacy', passWidth = !legacy && S.profile.passWidth;
  const header = [icon.parts.length], commands = [], chunks = [];
  let dataLength = 0;
  for (const part of icon.parts) {
    S.part = part.name;
    const dynamic = typeof part.d === 'function';
    const sourceAt = time => dynamic ? geometry(passWidth ? part.d(time, strong, reverse, part.width) : part.d(time, strong, reverse)) : staticGeometry(part.d);
    // Legacy builders parse d once and never check topology.
    const first = legacy ? staticGeometry(part.d) : sourceAt(0);
    const n = first.points.length;
    const frames = new Float64Array(FRAMES * n);
    // A static, unposed part has the same points on every frame and calls no author code.
    const still = !legacy && !dynamic && !part.pose;
    for (let frame = 0; frame < FRAMES; frame++) {
      if (still && frame > 0) {
        copyWithin(frames, frame * n, 0, n);
        continue;
      }
      const time = frame / FPS, geo = legacy ? first : sourceAt(time);
      if (!legacy && !sameCommands(geo.commands, first.commands)) throw new Error('Changing topology: ' + part.name);
      if (geo.points.length !== n) throw new Error('Changing point count: ' + part.name);
      for (let i = 0; i < n; i += 2) {
        const p = legacy ? S.profile.pose(icon, part, geo.points[i], geo.points[i + 1], time, strong)
          : part.pose ? part.pose(geo.points[i], geo.points[i + 1], time, strong, reverse) : [geo.points[i], geo.points[i + 1]];
        if (!p || p.length !== 2) throw new Error('Pose must return [x, y]: ' + part.name);
        frames[frame * n + i] = round(p[0]);
        frames[frame * n + i + 1] = round(p[1]);
      }
    }
    // build-set.mjs:112-123: a rigid pose on a static path becomes a native transform.
    let rigid = null;
    if (!legacy && part.pose?.transform && typeof part.d === 'string') {
      const samples = [];
      for (let frame = 0; frame < FRAMES; frame++) {
        const time = frame / FPS, { dx = 0, dy = 0, angle = 0, pivot = [16, 16] } = part.pose.transform(time, strong, reverse);
        push(samples, { dx, dy, angle, pivot });
      }
      const anchor = samples[0].pivot;
      if (some(samples, s => stringify(s.pivot) !== stringify(anchor))) throw Error('Moving rigid pivot');
      if (!anchor || anchor.length !== 2 || some([anchor[0], anchor[1]], v => typeof v !== 'number')) throw Error('Rigid pivot must be [x, y]: ' + part.name);
      if (some(samples, s => some([s.dx, s.dy, s.angle], v => typeof v !== 'number'))) throw Error('Rigid transform must be numeric: ' + part.name);
      rigid = new Float64Array(2 + n + 3 * FRAMES);
      rigid[0] = anchor[0];
      rigid[1] = anchor[1];
      for (let i = 0; i < n; i++) rigid[2 + i] = round(first.points[i]);
      forEach(samples, (s, f) => {
        rigid[2 + n + 2 * f] = anchor[0] + s.dx;
        rigid[2 + n + 2 * f + 1] = anchor[1] + s.dy;
        rigid[2 + n + 2 * FRAMES + f] = -s.angle * PI / 180;
      });
    }
    let visibility = null;
    if (!legacy) {
      visibility = new Float64Array(FRAMES);
      for (let frame = 0; frame < FRAMES; frame++) visibility[frame] = round(part.visibility ? part.visibility(frame / FPS, strong, reverse) : 1);
    }
    push(header, first.commands.length, n, rigid ? 1 : 0, commands.length, dataLength);
    for (let i = 0; i < first.commands.length; i++) push(commands, first.commands[i]);
    const parts = [frames, visibility, rigid];
    for (let c = 0; c < parts.length; c++) if (parts[c]) {
      push(chunks, parts[c]);
      dataLength += typedLength(parts[c]);
    }
  }
  S.part = undefined;
  const data = new Float64Array(dataLength);
  let offset = 0;
  for (let c = 0; c < chunks.length; c++) {
    typedSet(data, chunks[c], offset);
    offset += typedLength(chunks[c]);
  }
  return { header: typed(Int32Array, header), commands: typed(Uint8Array, commands), data };
}

function sameCommands(a, b) {
  return a.length === b.length && every(a, (c, i) => c === b[i]);
}

// Per probed time, status is 0 when every step completed, else why it stopped; stage is
// the step it stopped at.
const THREW = 1, THREW_NULLISH = 2, TRANSFORM_UNDEFINED = 3, TRANSFORM_NULL = 4, BAD_GEOMETRY = 5;
const D = 1, GEOMETRY = 2, VISIBILITY = 3, POSE = 4, TRANSFORM = 5;

// Evaluates unexpanded part `partIndex` of icon `iconIndex` at each time, unrounded, in
// preflight's order (preflight.mjs:96-113): d, geometry, visibility, pose per pair, then
// the rigid transform. d always receives 4 arguments; width is the part's authored width
// when authoredWidth is true. Evaluation of a time stops at the first failure. Values
// that must not be coerced (posedOk, trOk, visibility flags) are judged here.
export function probe(iconIndex, partIndex, strong, reverse, width, authoredWidth, times) {
  if (!S) throw Error('gai:host: bind() must come before probe()');
  const parts = S.sourceParts[iconIndex];
  if (!isArray(parts) || parts[partIndex] == null) throw Error(`gai:host: icon ${iconIndex} has no part ${partIndex}`);
  const part = parts[partIndex], w = authoredWidth ? part.width : width, count = typedLength(times);
  const status = new Int32Array(count), stage = new Uint8Array(count), messages = fill(new Array(count), undefined);
  const geoError = fill(new Array(count), ''), svgErr = fill(new Array(count), '');
  const topology = typedFill(new Int32Array(count), -1), topologies = [];
  const pointOffsets = new Int32Array(count + 1), points = [], posed = [], posedOk = [], posedPairs = new Int32Array(count);
  const visibility = new Float64Array(count), visFlags = new Uint8Array(count);
  const trFlags = new Uint8Array(count), trNullish = new Float64Array(3 * count), pivot = new Float64Array(2 * count);
  const pivotJson = fill(new Array(count), ''), trOr = new Float64Array(3 * count);
  const number = v => typeof v === 'bigint' || typeof v === 'symbol' ? NaN : v;
  for (let k = 0; k < count; k++) {
    const t = times[k];
    S.part = part.name;
    pointOffsets[k + 1] = pointOffsets[k];
    let at = D;
    try {
      const dynamic = typeof part.d === 'function';
      const src = dynamic ? part.d(t, strong, reverse, w) : part.d;
      at = GEOMETRY;
      let geo;
      try {
        geo = !dynamic && typeof src === 'string' ? parseStatic(src) : parseGeometry(src);
      } catch (error) {
        geoError[k] = String(error.message);
        if (typeof src === 'string' && trim(src)) svgErr[k] = svgpath(src).err;
        status[k] = BAD_GEOMETRY;
        stage[k] = GEOMETRY;
        continue;
      }
      let index = indexOf(topologies, geo.topology);
      if (index < 0) index = push(topologies, geo.topology) - 1;
      topology[k] = index;
      for (let i = 0; i < geo.points.length; i++) push(points, geo.points[i]);
      pointOffsets[k + 1] = points.length;
      at = VISIBILITY;
      const v = part.visibility ? part.visibility(t, strong, reverse) : 1;
      visibility[k] = number(v);
      visFlags[k] = 1 | (typeof v === 'number' ? 2 : 0) | (v == null ? 4 : 0);
      at = POSE;
      for (let i = 0; i < geo.points.length; i += 2) {
        const p = part.pose ? part.pose(geo.points[i], geo.points[i + 1], t, strong, reverse) : [geo.points[i], geo.points[i + 1]];
        const ok = pair(p);
        push(posed, ok ? p[0] : NaN, ok ? p[1] : NaN);
        push(posedOk, ok ? 1 : 0);
        posedPairs[k]++;
      }
      at = TRANSFORM;
      if (part.pose?.transform) {
        trFlags[k] = 1;
        const tr = part.pose.transform(t, strong, reverse);
        if (tr == null) {
          status[k] = tr === null ? TRANSFORM_NULL : TRANSFORM_UNDEFINED;
          stage[k] = TRANSFORM;
          continue;
        }
        // preflight.mjs:108-110 (?? defaults) and measure-motion.mjs (|| defaults)
        const nullish = [tr.dx ?? 0, tr.dy ?? 0, tr.angle ?? 0], p = tr.pivot ?? [16, 16];
        forEach(nullish, (v, j) => { trNullish[3 * k + j] = number(v); });
        if (pair(p)) {
          pivot[2 * k] = p[0];
          pivot[2 * k + 1] = p[1];
        } else {
          pivot[2 * k] = pivot[2 * k + 1] = NaN;
        }
        if (pair(p) && every(nullish, finite)) trFlags[k] |= 4;
        pivotJson[k] = stringify(p);
        forEach([tr.angle || 0, tr.dx || 0, tr.dy || 0], (v, j) => { trOr[3 * k + j] = number(v); });
        trFlags[k] |= 2;
      }
    } catch (error) {
      stage[k] = at;
      if (error == null) status[k] = THREW_NULLISH;
      else {
        status[k] = THREW;
        messages[k] = error.message;
      }
    } finally {
      // Keep posed aligned with points when a pose call failed part way.
      while (posed.length < points.length) {
        push(posed, NaN);
        if (posed.length % 2 === 0) push(posedOk, 0);
      }
    }
  }
  S.part = undefined;
  return {
    callbacks: callbacks(part), staticD: typeof part.d === 'string',
    status, stage, messages, geoError, svgErr, topology, topologies,
    pointOffsets, points: typed(Float64Array, points), posed: typed(Float64Array, posed), posedOk: typed(Uint8Array, posedOk), posedPairs,
    visibility, visFlags, trFlags, trNullish, pivot, pivotJson, trOr,
  };
}

// JSON.stringify(value) with each number replaced by "\0<index>" and each string that
// starts with "\0" escaped as "\0" + string, so Rust can put the exact doubles back.
export function tape(value) {
  const numbers = [];
  const text = stringify(value, function (key, v) {
    if (isPrototypeOf(NumberPrototype, v)) v = Number(v); // v instanceof Number
    if (typeof v === 'number') return '\0' + (push(numbers, v) - 1);
    if (typeof v === 'string' && charCodeAt(v, 0) === 0) return '\0' + v;
    return v;
  });
  return { text, numbers: typed(Float64Array, numbers) };
}
