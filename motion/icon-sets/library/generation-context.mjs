// Local retrieval, not generative search. Preserve existing IDs and original art.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { spawnSync } from 'node:child_process';

export const projectRoot = fileURLToPath(new URL('../../../', import.meta.url));
const sha = data => crypto.createHash('sha256').update(data).digest('hex');
const stop = new Set('a an and are as at be by can create draw for from icon icons in is it like more new of on or our please the this to use with'.split(' '));
const tokens = text => [...new Set(String(text).toLowerCase().match(/[a-z0-9]+/g) || [])].filter(t => !stop.has(t));
// Vocabulary expansions are transparent suggestions, never semantic equivalence.
const aliases = [
  ['notification', 'notifications', 'alert', 'bell'],
  ['search', 'find', 'magnifier', 'lookup'],
  ['favorite', 'favourite', 'like', 'heart', 'star'],
  ['payment', 'pay', 'bank', 'finance', 'fintech', 'money'],
  ['loading', 'loader', 'spinner', 'progress', 'waiting'],
  ['settings', 'preferences', 'sliders', 'adjust', 'filter'],
  ['delivery', 'shipping', 'parcel', 'package', 'truck'],
];

export function loadCatalog(root = projectRoot) {
  const inputs = [];
  const read = relative => {
    const file = path.join(root, relative), data = fs.readFileSync(file);
    inputs.push({ path: file, sha256: sha(data) });
    return JSON.parse(data);
  };
  const catalog = read('prototype/icons/catalog.json'), icons = [], seen = new Set();
  for (const relative of catalog.sets) {
    const set = read('prototype/icons/' + relative);
    const category = catalog.categories.find(c => c.sets.includes(relative));
    for (const icon of set.icons) {
      const id = icon.icon || icon.name;
      if (seen.has(id)) throw Error(`Duplicate catalog ID ${id}`);
      seen.add(id);
      icons.push({ ...icon, id, category: category?.label || set.category || set.name,
        set: set.id, manifest: 'prototype/icons/' + relative,
        assetDirectory: path.posix.dirname('prototype/icons/' + relative) });
    }
  }
  return { icons, inputs };
}

export function searchIcons(icons, query, limit = 6) {
  const direct = tokens(query), expanded = new Set(direct);
  for (const group of aliases) if (group.some(t => expanded.has(t))) group.forEach(t => expanded.add(t));
  return icons.map(icon => {
    const names = tokens(icon.id + ' ' + icon.label), other = tokens([icon.category, icon.intent, icon.brief, ...(icon.keywords || [])].join(' '));
    const matches = [...expanded].filter(t => names.includes(t) || other.includes(t));
    const score = matches.reduce((sum, t) => sum + (names.includes(t) ? 5 : 1) * (direct.includes(t) ? 3 : 1), 0);
    return { id: icon.id, label: icon.label, category: icon.category, score, matchedTerms: matches };
  }).filter(i => i.score > 0).sort((a, b) => b.score - a.score || a.id.localeCompare(b.id)).slice(0, limit);
}

const initial = property => property?.a === 1 ? property.k[0]?.s : property?.k;
function referenceGeometry(data) {
  // Initial exported pose, including transforms and visibility. It can be an
  // incomplete state; this is explicitly not a rendered/approved rest SVG.
  return data.layers.filter(layer => (layer.shapes || []).some(s => ['st', 'fl'].includes(s.ty) && s.o?.sid === 'style.outline.regular')).map(layer => ({
    name: layer.nm,
    transform: Object.fromEntries(Object.entries(layer.ks).map(([key, value]) => [key, initial(value)])),
    paths: layer.shapes.filter(s => s.ty === 'sh').map(s => s.ks.a === 1 ? initial(s.ks)?.[0] : initial(s.ks)),
    paint: layer.shapes.filter(s => ['st', 'fl'].includes(s.ty)).map(s => ({ type: s.ty, width: initial(s.w), cap: s.lc, join: s.lj })),
  }));
}

export function buildPacket(query, neighbors = [], root = projectRoot) {
  const { icons, inputs } = loadCatalog(root), candidates = searchIcons(icons, query);
  const byId = new Map(icons.map(i => [i.id, i]));
  for (const id of neighbors) if (!byId.has(id)) throw Error(`Unknown neighbor ${id}; choose an existing catalog ID`);
  const selected = neighbors.length ? [...new Set(neighbors)] : candidates.slice(0, 2).map(i => i.id);
  if (selected.length > 4) throw Error('Use at most four focused neighbors; split a large batch into briefs');
  const contractFile = path.join(root, 'motion/icon-sets/library/family-contract.json');
  const contractBytes = fs.readFileSync(contractFile);
  inputs.push({ path: contractFile, sha256: sha(contractBytes) });
  const references = selected.map(id => {
    const icon = byId.get(id), motions = icon.motions || [icon];
    const strong = motions.find(m => m.motionId === 'strong');
    if (!strong) throw Error(`Missing Strong reference for ${id}`);
    const jsonFile = path.join(root, icon.assetDirectory, strong.name + '.json');
    const packageFile = path.join(root, icon.assetDirectory, strong.name + '.lottie');
    const jsonBytes = fs.readFileSync(jsonFile), packageBytes = fs.readFileSync(packageFile);
    if (sha(packageBytes) !== strong.sha256) throw Error(`Stale package hash for ${id}; rebuild/verify before using as a reference`);
    const packed = spawnSync('unzip', ['-p', packageFile, `a/${strong.name}.json`], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
    if (packed.status !== 0) throw Error(`Cannot read native reference from ${packageFile}: ${packed.error?.message || packed.stderr}`);
    if (JSON.stringify(JSON.parse(packed.stdout)) !== JSON.stringify(JSON.parse(jsonBytes))) throw Error(`Loose native JSON differs from package for ${id}; refresh the reference first`);
    inputs.push({ path: jsonFile, sha256: sha(jsonBytes) }, { path: packageFile, sha256: sha(packageBytes) });
    return { id, label: icon.label, category: icon.category, set: icon.set,
      intent: strong.intent || icon.intent, brief: strong.brief || icon.brief,
      sourceManifest: icon.manifest, duotoneFill: icon.duotoneFill ?? 'undeclared in legacy metadata',
      profiles: motions.map(m => ({ profile: m.motionId, activeSeconds: m.active, restFrame: m.rest,
        continuous: !!m.continuous, startState: m.startState, endState: m.endState,
        reverse: m.reverse?.name, packageSha256: m.sha256 })),
      initialOutlineGeometry: referenceGeometry(JSON.parse(jsonBytes)),
      evidenceBoundary: 'Current package hash matches catalog and loose JSON matches packaged animation. Geometry is initial native pose only, with local transforms; this is not a new visual or motion approval.' };
  });
  const self = fileURLToPath(import.meta.url);
  inputs.push({ path: self, sha256: sha(fs.readFileSync(self)) });
  return { version: 1, query, selection: neighbors.length ? 'Agent-selected neighbors' : 'Lexical matches with listed vocabulary expansions; the current coding agent must confirm relevance',
    libraryCount: icons.length, contract: JSON.parse(contractBytes), candidates, references, inputs,
    reviewInstructions: [
      'Check whether the requested meaning already exists. Keep IDs stable; explain a new icon versus a variant. Do not copy external library geometry.',
      'Use selected neighbors for construction consistency, not a shared animation template. Inspect mechanism-specific source and failure records before design.',
      'These are text references. Initial path points do not show playback. The current coding agent reviews real native motion at equal elapsed times; self-review is not independent approval.',
      'Compare changed icons beside these neighbors at 24/32/48 px in both themes and all weights/styles. Also inspect 16/20 px in 32/36 px controls and beside text as stress cases; record limits without silently changing supported sizes.',
      'Keep optical mass/centering observations separate from hard geometry failures. A control-point hull or stroke-length estimate does not measure perceived weight.',
    ] };
}

if (process.argv[1] && fs.realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const { values } = parseArgs({ options: { query: { type: 'string' }, 'query-file': { type: 'string' }, neighbor: { type: 'string', multiple: true, default: [] }, search: { type: 'boolean', default: false } } });
    if ((!values.query && !values['query-file']) || (values.query && values['query-file'])) throw Error('Supply --query text OR --query-file brief.md; optional --neighbor ID or --search');
    const query = values['query-file'] ? fs.readFileSync(path.resolve(values['query-file']), 'utf8') : values.query;
    const result = values.search ? searchIcons(loadCatalog().icons, query) : buildPacket(query, values.neighbor);
    console.log(JSON.stringify(result, null, 2));
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
