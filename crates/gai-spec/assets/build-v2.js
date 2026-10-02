// build-v2.js: gen-animated-icons spec v2. ONE creator_exec per icon builds every motion
// from a compiled plan (gai-plan/1) and either exports them as one bundle or renders a
// draft sheet. params: {plan, file?, sheet?}
//   plan   rigs (parent first), geometry, layers (back to front), motions with eased keys.
//   file   every motion comp becomes a precomp of one bundle comp, exported once.
//   sheet  {cols, gap, scales, rows: [{motion, look, times}]}: time-offset precomps rendered
//          with renderFilmstrip; the PNGs are attached to the result, nothing is stored.
const T0 = Date.now();
const P = params.plan;
const PT = PropertyType;
const COLOR = { red: P.color[0], green: P.color[1], blue: P.color[2], alpha: 1 };
// Paint opacity per style for one look: the draft's stand-in for slots (18 % tint, as
// gai-render's Duotone look).
const LOOKS = {
  regular: { 'outline.regular': 1 }, light: { 'outline.light': 1 }, bold: { 'outline.bold': 1 },
  solid: { solid: 1 }, duotone: { 'outline.regular': 1, tint: 0.18 },
};
const clamp01 = (x) => Math.max(0, Math.min(1, x));
// Engine errors name the plan entry that failed, not a script line.
function at(where, f) {
  try { return f(); } catch (e) {
    const hint = e && e.hint ? ' (hint: ' + e.hint + ')' : '';
    throw new Error(where + ': ' + (e && e.message ? e.message : String(e)) + hint);
  }
}

// Clear: a comp a precomp still references cannot be removed, and describeComps() lists a
// referenced comp before its hosts, so the reverse removes hosts first. A comp that still
// fails waits for the next pass. Never deleteNodes on comps ("snapshot serialization failed").
at('clear', () => {
  for (let pass = 0; pass < 6; pass++) {
    const comps = creator.describeComps();
    if (!comps.length) return;
    let removed = 0;
    for (const c of comps.slice().reverse()) {
      try { creator.removeComp(c.id); removed++; } catch (e) { /* referenced: next pass */ }
    }
    if (!removed) throw new Error(comps.length + ' compositions could not be removed');
  }
});

function geoNode(comp, g) {
  if (g.ellipse) { const [cx, cy, rx, ry] = g.ellipse; return creator.insertEllipse(comp, cx, cy, 2 * rx, 2 * ry, COLOR); }
  if (g.rect) {
    const [cx, cy, w, h, r] = g.rect;
    const n = creator.insertRect(comp, cx, cy, w, h, COLOR);
    if (r) creator.setStaticF32(n, PT.CornerRadius, r);
    return n;
  }
  return creator.insertPath(comp, 0, 0, g.path.c, g.path.p, COLOR);   // one subpath per node
}

// keys: [[t, value, ease]]; ease [ox, oy, ix, iy] or null (hold); the last key has none.
// Tangents once the next key exists; x clamped (the engine exports x outside 0..1 as is).
function setKeys(node, prop, keys) {
  const track = creator.insertTrack(node, PT[prop], keys[0][0]);
  const kfs = keys.map((k) => {
    const kf = creator.addKeyFrame(node, PT[prop], track, k[0]);
    if (prop === 'PathPoints') creator.setKeyFrameVec2Slice(kf, k[1]); else creator.setKeyFrameF32(kf, k[1]);
    return kf;
  });
  for (let i = 0; i < kfs.length - 1; i++) {
    const e = keys[i][2];
    if (e === null) creator.setTangents(kfs[i], null);
    else if (e) creator.setTangents(kfs[i], [clamp01(e[0]), e[1], clamp01(e[2]), e[3]]);
  }
}

function build(motion, look) {
  const comp = creator.createComp(P.w, P.h, P.fps, 0, P.duration);
  creator.focusOn(comp);
  creator.setName(comp, motion.name + (look === 'regular' ? '' : '@' + look));
  const nodes = {};                        // track address -> node ids
  const add = (k, n) => (nodes[k] = nodes[k] || []).push(n);
  const rigs = {};
  for (const r of P.rigs) at('rig ' + r.name, () => {
    const g = creator.createGroup(comp, r.pivot[0], r.pivot[1]);
    creator.setName(g, r.name);
    creator.setStaticVec2(g, PT.AnchorPoint, { x: r.pivot[0], y: r.pivot[1] });
    if (r.parent) creator.setTransformParent(g, rigs[r.parent]);
    if (r.split) creator.splitPosition(g);
    rigs[r.name] = g; add(r.name, g);
  });
  const layer = (L) => at('layer ' + L.name, () => {
    const g = creator.createGroup(comp, 0, 0);
    creator.setName(g, L.name);
    if (L.rig) creator.setTransformParent(g, rigs[L.rig]);
    const made = L.geos.map((id) => {
      const n = geoNode(comp, P.geos[id]);
      creator.setParent(n, g); creator.setName(n, L.name); add(id, n);
      return n;
    });
    // Several shapes or paints go on the group: a compound path (no stroke ever carries two
    // MoveTos), or fill and stroke on one layer (the Solid surface).
    const holder = made.length > 1 || L.paints.length > 1 ? g : made[0];
    for (const n of made) for (const p of creator.getPaints(n)) creator.removePaint(p);
    for (const spec of L.paints) {
      let paint;
      if (spec[0] === 'stroke') {
        paint = creator.addStroke(holder, COLOR, spec[1]);
        creator.setStrokeCap(paint, 2); creator.setStrokeJoin(paint, 2);
      } else {
        paint = creator.addFill(holder, COLOR);
        if (spec[1] === 1) creator.setFillRule(paint, 1);
      }
      if (L.style) {
        creator.setName(paint, 'color.primary__' + L.style);
        creator.setStaticF32(paint, PT.Opacity, LOOKS[look][L.style] || 0);
      }
    }
    // A mask with a geometry id follows that geometry's path keys (a reshaped part's hole).
    for (const m of L.masks || []) {
      const mask = creator.addMask(g, m.c, m.p, MaskMode.Subtract);
      if (m.geo) add(m.geo, mask);
    }
    if (L.trim) add(L.trim, creator.createTrimShape(holder));
    if (L.opacity !== undefined) creator.setStaticF32(g, PT.Opacity, L.opacity);
    if (L.vis) add(L.vis, g);
    return g;
  });
  for (const L of P.layers) {
    const g = layer(L);
    // A matte source goes directly above its target (the newest node is on top), so `tp`
    // and the older "matte is the layer above" rule name the same layer.
    if (L.matte) {
      const src = layer(L.matte);
      at('matte ' + L.matte.name, () => creator.setMatte(g, src, MaskMode.InvAlpha));
    }
  }
  for (const t of motion.tracks) at('track ' + t.target + ' ' + t.prop + ' in ' + motion.name, () => {
    for (const node of nodes[t.target] || []) setKeys(node, t.prop, t.keys);
  });
  // For engine-side previews; post-export rewrites markers exactly (a bundle drops them).
  creator.addMarker(comp, 0, motion.active, 'action');
  creator.addMarker(comp, motion.rest, motion.rest + 1 / P.fps, 'rest');
  creator.addMarker(comp, 0, P.duration, 'showcase');
  return comp;
}

const comps = {};
for (const m of P.motions) comps[m.name] = build(m, 'regular');
const T1 = Date.now();
const out = { stored: [], sheets: [] };
if (params.file) at('export', () => {     // one stored file per icon
  const bundle = creator.createComp(P.w, P.h, P.fps, 0, P.duration);
  creator.setName(bundle, P.id + '@bundle');
  for (const m of P.motions) creator.setName(creator.createPrecomp(bundle, comps[m.name], 0, 0), m.name);
  creator.focusOn(bundle);
  out.stored.push(creator.export(params.file, 'lottie'));
});
const T2 = Date.now();
if (params.sheet) at('sheet', () => {     // rows = motion x look, columns = beats
  const S = params.sheet, CELL = P.w, GAP = S.gap, lookComps = {};
  const compFor = (name, look) => {
    if (look === 'regular') return comps[name];
    const key = name + '@' + look;
    if (!lookComps[key]) lookComps[key] = build(P.motions.find((m) => m.name === name), look);
    return lookComps[key];
  };
  creator.setBackgroundColor({ red: 0.063, green: 0.09, blue: 0.114, alpha: 1 });
  const W = S.cols * CELL + (S.cols + 1) * GAP, H = S.rows.length * CELL + (S.rows.length + 1) * GAP;
  const sheet = creator.createComp(W, H, P.fps, 0, P.duration);
  creator.setName(sheet, P.id + '@sheet');
  S.rows.forEach((row, r) => {
    const src = compFor(row.motion, row.look);
    row.times.forEach((t, c) => {
      const x = GAP + c * (CELL + GAP), y = GAP + r * (CELL + GAP);
      creator.insertRect(sheet, x + CELL / 2, y + CELL / 2, CELL, CELL, { red: 0.12, green: 0.16, blue: 0.2, alpha: 1 });
      creator.setTimelineOffset(creator.createPrecomp(sheet, src, x, y), t);
    });
  });
  creator.focusOn(sheet);
  for (const scale of S.scales) out.sheets.push(creator.renderFilmstrip({ tiles: 1, width: Math.round(W * scale) }));
});
const T3 = Date.now();
return {
  timing: { startedAt: T0, buildMs: T1 - T0, bundleMs: T2 - T1, sheetMs: T3 - T2, totalMs: T3 - T0 },
  ...out,
  comps: creator.describeComps().length,
};
