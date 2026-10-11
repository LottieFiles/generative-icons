// Original source checks inspired by Regen's early compiler validation.
// This imports trusted local definitions, never exports or writes assets.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import svgpath from 'svgpath';
import crypto from 'node:crypto';
import { loadCatalog } from './generation-context.mjs';

export const contractPath = fileURLToPath(new URL('./family-contract.json', import.meta.url));
export const contract = JSON.parse(fs.readFileSync(contractPath, 'utf8'));
export const projectRoot = fileURLToPath(new URL('../../../', import.meta.url));
const epsilon = 1e-8;
const pair = p => Array.isArray(p) && p.length === 2 && p.every(Number.isFinite);

// Normalize the commands accepted by the current native builder. svgpath can
// return an empty/partial path on malformed input, so its error must be checked.
export function parseGeometry(d) {
  if (typeof d !== 'string' || !d.trim()) throw Error('Path must be a nonempty SVG path string');
  const parsed = svgpath(d);
  if (parsed.err) throw Error(parsed.err);
  const segments = parsed.abs().unarc().unshort().segments;
  const commands = [], points = [];
  let last = [0, 0];
  for (const [cmd, ...args] of segments) {
    if (!args.every(Number.isFinite)) throw Error('Nonfinite path coordinates');
    if (cmd === 'M' || cmd === 'L' || cmd === 'C') {
      commands.push(cmd); points.push(...args); last = args.slice(-2);
    } else if (cmd === 'H' || cmd === 'V') {
      last = cmd === 'H' ? [args[0], last[1]] : [last[0], args[0]];
      commands.push('L'); points.push(...last);
    } else if (cmd === 'Z') commands.push(cmd);
    else throw Error(`Unsupported native command ${cmd}`);
  }
  if (commands[0] !== 'M' || !commands.some(c => c === 'L' || c === 'C')) throw Error('Path has no drawable segment');
  return { topology: commands.join(''), points };
}

export function checkDefinitions(icons) {
  const diagnostics = [], seen = new Set(), ids = new Set();
  let samples = 0, actions = 0;
  const add = (severity, code, icon, part, message, detail = {}) => {
    const id = JSON.stringify([severity, code, icon, part, detail.profile, detail.reverse, detail.weight]);
    if (seen.has(id)) return;
    seen.add(id); diagnostics.push({ severity, code, icon, ...(part ? { part } : {}), message, ...detail });
  };
  if (!Array.isArray(icons) || !icons.length) return { status: 'FAIL', icons: 0, actions, samples, diagnostics: [{ severity: 'error', code: 'EMPTY_SET', message: 'Export a nonempty icons array' }] };
  for (const icon of icons) {
    const name = icon?.name;
    if (!icon || !/^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/.test(name)) {
      add('error', 'ICON_NAME', name, null, 'Use a stable kebab-case name'); continue;
    }
    if (ids.has(name)) add('error', 'DUPLICATE_ICON', name, null, 'Icon names must be unique within the set');
    ids.add(name);
    if (typeof icon.label !== 'string' || !icon.label.trim()) add('error', 'LABEL', name, null, 'A readable label is required');
    if (!Array.isArray(icon.parts) || !icon.parts.length) {
      add('error', 'EMPTY_PARTS', name, null, 'Named vector parts are required'); continue;
    }
    const names = new Set();
    for (const p of icon.parts) {
      if (!p || typeof p.name !== 'string' || !p.name.trim()) {
        add('error', 'PART_NAME', name, null, 'Every part needs a name'); continue;
      }
      if (names.has(p.name)) add('warning', 'AMBIGUOUS_LAYER_NAME', name, p.name, 'Separate paints share a layer name; use unique names in the next drawing revision');
      names.add(p.name);
      if (!contract.sourceStyles.includes(p.style)) add('error', 'PAINT_STYLE', name, p.name, `Expected ${contract.sourceStyles.join(', ')}`);
      if (p.type !== undefined && !['fill', 'stroke'].includes(p.type)) add('error', 'PAINT_TYPE', name, p.name, 'Expected fill or stroke');
      if (p.type === 'stroke' && (!Number.isFinite(p.width) || p.width <= 0)) add('error', 'STROKE_WIDTH', name, p.name, 'Stroke width must be positive and finite');
      if (p.type === 'stroke' && p.style === 'outline' && p.width !== contract.outlineWeights.regular) add('error', 'OUTLINE_WIDTH', name, p.name, 'Author the regular centerline at width 2; the builder creates weight presets');
      for (const fn of ['pose', 'visibility']) if (p[fn] !== undefined && typeof p[fn] !== 'function') add('error', 'CALLBACK', name, p.name, `${fn} must be a function`);
    }
    if (icon.duotoneFill !== undefined && (typeof icon.duotoneFill !== 'boolean' || icon.duotoneFill !== icon.parts.some(p => p?.style === 'tint'))) add('error', 'DUOTONE_POLICY', name, null, 'Declared Duotone policy disagrees with tint parts');
    if (icon.duotoneFill === undefined) add('info', 'DUOTONE_UNDECLARED', name, null, 'Legacy source: native paint policy remains authoritative; explicitly declare surface eligibility for new icons');
    if (icon.stateful && (!icon.initialState || !icon.finalState)) add('error', 'STATE_POLICY', name, null, 'Stateful icons need names for both states');
    if (icon.stateful && !icon.reverseSuffix) add('info', 'LEGACY_REVERSE_NAME', name, null, 'Verify the set-specific builder names its reverse/reset; the shared builder requires reverseSuffix');
    for (const reverse of icon.stateful ? [false, true] : [false]) for (const profile of contract.profiles) {
      actions++;
      const strong = profile === 'strong', timing = (reverse ? icon.reverseTimings : icon.timings)?.[profile];
      const context = { profile, reverse };
      if (!timing || !Number.isFinite(timing.active) || timing.active <= 0 || timing.active > contract.durationSeconds || !Number.isFinite(timing.rest) || timing.rest < 0 || timing.rest > contract.durationSeconds || (!icon.continuous && timing.rest < timing.active)) {
        add('error', 'TIMING', name, null, 'Action/rest must fit the current timeline; gesture rest follows active motion', context); continue;
      }
      const keys = timing.keyPoses;
      if (!Array.isArray(keys) || !keys.length || keys.some((t, i) => !Number.isFinite(t) || t < 0 || t > contract.durationSeconds || (i > 0 && t < keys[i - 1]))) {
        add('error', 'KEY_POSES', name, null, 'Key poses must be ordered finite seconds inside the timeline', context); continue;
      }
      const times = [...new Set([...Array.from({ length: Math.round(contract.durationSeconds * contract.fps) + 1 }, (_, f) => f / contract.fps), ...keys, timing.active, timing.rest])].sort((a, b) => a - b);
      for (const part of icon.parts.filter(p => p && typeof p.name === 'string')) {
        // Outline dots can be fills. They still receive the builder's width argument.
        const weights = part.style === 'outline' ? Object.entries(contract.outlineWeights) : [['authored', part.width]];
        for (const [weight, width] of weights) {
          let topology, pivot;
          for (const time of times) {
            const detail = { ...context, weight, time };
            try {
              const geo = parseGeometry(typeof part.d === 'function' ? part.d(time, strong, reverse, width) : part.d);
              if (topology !== undefined && topology !== geo.topology) add('error', 'MORPH_TOPOLOGY', name, part.name, 'Normalized path commands change during this action', detail);
              topology ??= geo.topology;
              const visibility = part.visibility ? part.visibility(time, strong, reverse) : 1;
              if (!Number.isFinite(visibility) || visibility < -epsilon || visibility > 1 + epsilon) add('error', 'VISIBILITY', name, part.name, 'Visibility must remain between 0 and 1', detail);
              const world = [];
              for (let i = 0; i < geo.points.length; i += 2) {
                const point = part.pose ? part.pose(geo.points[i], geo.points[i + 1], time, strong, reverse) : geo.points.slice(i, i + 2);
                if (!pair(point)) throw Error('Pose must return two finite coordinates');
                world.push(point);
              }
              if (part.pose?.transform && typeof part.d === 'string') {
                const tr = part.pose.transform(time, strong, reverse), p = tr.pivot ?? [16, 16];
                if (!pair(p) || ![tr.dx ?? 0, tr.dy ?? 0, tr.angle ?? 0].every(Number.isFinite)) throw Error('Rigid transform must have a finite pivot, translation and angle');
                const current = JSON.stringify(p);
                if (pivot !== undefined && pivot !== current) add('error', 'MOVING_PIVOT', name, part.name, 'Native rigid transforms require a constant pivot', detail);
                pivot ??= current;
              }
              const radius = part.type === 'stroke' ? width / 2 : 0;
              if (visibility > 0 && world.some(p => p.some(v => v - radius < -epsilon || v + radius > contract.grid + epsilon))) add('warning', 'CONTROL_HULL_MARGIN', name, part.name, 'Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry', detail);
              samples++;
            } catch (error) {
              add('error', 'INVALID_GEOMETRY', name, part.name, error.message, detail); break;
            }
          }
        }
      }
    }
  }
  return { status: diagnostics.some(d => d.severity === 'error') ? 'FAIL' : 'PASS', icons: icons.length, actions, samples, diagnostics };
}

// A set made only of specs (`library adopt --add`): its committed manifest lists icons, and
// every one is adopted from the set's specs/<id>.icon.json.
function specOnly(root) {
  const manifest = path.join(projectRoot, 'prototype/icons', path.basename(root).split('-')[0], 'assets/set.json');
  let icons;
  try { icons = JSON.parse(fs.readFileSync(manifest, 'utf8')).icons; } catch { return false; }
  const isFile = file => fs.statSync(file, { throwIfNoEntry: false })?.isFile() ?? false;
  return Array.isArray(icons) && icons.length > 0 && icons.every(icon => icon?.source === 'gai-icon/2' && typeof icon.icon === 'string' && isFile(path.join(root, 'specs', `${icon.icon}.icon.json`)));
}

export async function checkSet(root) {
  const file = path.join(root, 'definitions.mjs');
  if (!fs.existsSync(file)) throw Error(`No separate definitions.mjs in ${root}; legacy inline builders require their existing checks`);
  const { icons } = await import(pathToFileURL(file).href);
  let report = checkDefinitions(icons);
  // A spec-only set defines no icons; `build` and `library verify` check them instead.
  if (report.diagnostics.some(d => d.code === 'EMPTY_SET') && specOnly(root)) report = { status: 'PASS', icons: 0, actions: 0, samples: 0, diagnostics: [] };
  // Existing names may be revised inside their own source set, but a new set
  // must not introduce a second card/download under an already public ID.
  const { icons: catalogIcons, inputs: catalogInputs } = loadCatalog();
  const setNumber = path.basename(root).split('-')[0];
  for (const icon of Array.isArray(icons) ? icons : []) {
    const existing = catalogIcons.find(i => i.id === icon?.name);
    if (existing && existing.manifest.split('/')[2] !== setNumber) {
      report.diagnostics.push({ severity: 'error', code: 'CATALOG_ID_COLLISION', icon: icon.name, message: `ID already belongs to ${existing.set}; revise there or choose a distinct concept/ID` });
      report.status = 'FAIL';
    }
  }
  // Conservative input inventory: include shared authoring modules as well as
  // local batches. Extra hashes are preferable to omitting a helper dependency.
  const library = path.dirname(fileURLToPath(import.meta.url));
  const candidates = [file, path.join(root, 'set-config.json'),
    ...fs.readdirSync(library).filter(n => /\.(mjs|json)$/.test(n)).map(n => path.join(library, n))];
  const batches = path.join(root, 'batches');
  if (fs.existsSync(batches)) candidates.push(...fs.readdirSync(batches).filter(n => /\.(mjs|json)$/.test(n)).map(n => path.join(batches, n)));
  const inputs = [...new Set(candidates)].filter(f => fs.existsSync(f)).sort().map(f => ({ path: f, sha256: crypto.createHash('sha256').update(fs.readFileSync(f)).digest('hex') }));
  return { set: path.basename(root), ...report, inputs: [...inputs, ...catalogInputs] };
}

if (process.argv[1] && fs.realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const arg = process.argv[2];
    if (!arg) throw Error('Usage: node preflight.mjs <source-set-directory> | --all');
    const setsRoot = path.join(projectRoot, 'motion/icon-sets');
    const directories = arg === '--all' ? fs.readdirSync(setsRoot).filter(d => /^\d\d-/.test(d)).sort().map(d => path.join(setsRoot, d)) : [path.resolve(arg)];
    const reports = [], skipped = [];
    for (const dir of directories) {
      if (arg === '--all' && !fs.existsSync(path.join(dir, 'definitions.mjs'))) { skipped.push({ set: path.basename(dir), reason: 'Legacy inline builder; existing native/source checks required' }); continue; }
      reports.push(await checkSet(dir));
    }
    const status = reports.some(r => r.status !== 'PASS') ? 'FAIL' : 'PASS';
    console.log(JSON.stringify({ status, scope: 'Source screening only; no native export or visual review', reports, skipped }, null, 2));
    if (status !== 'PASS') process.exitCode = 1;
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
