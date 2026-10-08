// Animated SVG export: a Generative Icons Lottie animation becomes an SVG that the
// browser animates with SMIL, without a player or script.
//
// It converts exactly the Lottie the icon compiler emits (crates/gai-spec/src/plan.rs):
// shape layers with sh/rc/el paths, stroke/fill/trim, parenting, layer opacity,
// position (split or not), rotation about the anchor, path keyframes, trim end,
// subtract masks and alpha-inverted mattes, bezier and hold keys, and the family's
// colour/style/weight slots. Anything else throws instead of degrading.
//
// Static attributes hold the rest pose, so renderers without SMIL show the icon at rest.
// crates/gai-svg ports this file and must stay byte-identical with it.
import { readZip } from './weight.mjs';

export const SVG_TRIGGERS = ['auto', 'hover', 'click', 'toggle', 'loop', 'load', 'js'];
const STYLES = ['outline', 'solid', 'duotone'];
const KAPPA = 0.5519150244935105;

// ---------- numbers and paths ----------
function num(n, dp = 3) {
  let s = (+n.toFixed(dp)).toString();
  if (s === '-0') s = '0';
  return s.replace(/^(-?)0\./, '$1.');
}
const pt = (x, y) => `${num(x)} ${num(y)}`.replace(' -', '-');

// {v,i,o} -> [vx, vy, ix, iy, ox, oy, ...]: linear in the key, like Lottie's interpolation
function flatShape(sh) {
  const out = [];
  for (let k = 0; k < sh.v.length; k++) out.push(sh.v[k][0], sh.v[k][1], sh.i[k][0], sh.i[k][1], sh.o[k][0], sh.o[k][1]);
  return out;
}
const shapeConv = s => flatShape(Array.isArray(s) ? s[0] : s);
// SMIL interpolates a path only between keys with one command list. Pad shorter keys
// with a zero-length vertex (a copy of the last one that takes over its out-tangent).
function padMorph(prop) {
  if (prop?.a !== 1) return prop;
  const n = Math.max(...prop.k.filter(k => k.s).map(k => k.s[0].v.length));
  if (prop.k.every(k => !k.s || k.s[0].v.length === n)) return prop;
  const pad = sh => {
    sh = { ...sh, v: [...sh.v], i: [...sh.i], o: [...sh.o] };
    while (sh.v.length < n) { const j = sh.v.length - 1; sh.v.push(sh.v[j]); sh.i.push([0, 0]); sh.o.push(sh.o[j]); sh.o[j] = [0, 0]; }
    return sh;
  };
  return { ...prop, k: prop.k.map(k => k.s ? { ...k, s: [pad(k.s[0])] } : k) };
}
// Segments whose ends meet in any key (the round end of a collapsed pill). WebKit drops
// such a cubic together with its joins, so these are written as two halves.
function hairpins(keys, closed) {
  const out = new Set();
  for (const flat of keys) {
    const n = flat.length / 6;
    for (let a = 0; a < (closed ? n : n - 1); a++) {
      const b = (a + 1) % n;
      if (Math.hypot(flat[6 * a] - flat[6 * b], flat[6 * a + 1] - flat[6 * b + 1]) < 0.01) out.add(a);
    }
  }
  return out;
}
const tlKeys = tl => tl.constant ? [tl.constant] : [tl.first.v, ...tl.segs.map(s => s.v1), tl.last.v];
// Every segment of a keyed path is a C so all keys share one command list.
function shapeD(flat, closed, forceCurves, split) {
  const n = flat.length / 6;
  if (!n) return '';
  const V = k => [flat[6 * k], flat[6 * k + 1]];
  const I = k => [flat[6 * k + 2], flat[6 * k + 3]];
  const O = k => [flat[6 * k + 4], flat[6 * k + 5]];
  let d = 'M' + pt(...V(0));
  const seg = (a, b) => {
    const [ax, ay] = V(a), [bx, by] = V(b), [ox, oy] = O(a), [ix, iy] = I(b);
    if (split?.has(a)) {
      // de Casteljau at 0.5: exact, and linear in the control points, so morphs stay exact
      const p1 = [ax + ox, ay + oy], p2 = [bx + ix, by + iy];
      const m = (u, v) => [(u[0] + v[0]) / 2, (u[1] + v[1]) / 2];
      const q0 = m([ax, ay], p1), q1 = m(p1, p2), q2 = m(p2, [bx, by]), r0 = m(q0, q1), r1 = m(q1, q2), c = m(r0, r1);
      return 'C' + pt(...q0) + ' ' + pt(...r0) + ' ' + pt(...c) + 'C' + pt(...r1) + ' ' + pt(...q2) + ' ' + pt(bx, by);
    }
    if (!forceCurves && !ox && !oy && !ix && !iy) return 'L' + pt(bx, by);
    return 'C' + pt(ax + ox, ay + oy) + ' ' + pt(bx + ix, by + iy) + ' ' + pt(bx, by);
  };
  for (let k = 0; k < n - 1; k++) d += seg(k, k + 1);
  if (closed) d += seg(n - 1, 0) + 'Z';
  return d;
}
// lottie-web's rect and ellipse paths (direction 1), so trims start where Lottie's do
function rectShape(p, s, r) {
  const [p0, p1] = p, v0 = s[0] / 2, v1 = s[1] / 2;
  const round = Math.min(v0, v1, r), c = round * (1 - KAPPA);
  const T = [];
  const add = (vx, vy, ox, oy, ix, iy) => T.push([vx, vy, ix - vx, iy - vy, ox - vx, oy - vy]);
  if (round) {
    add(p0 + v0, p1 - v1 + round, p0 + v0, p1 - v1 + round, p0 + v0, p1 - v1 + c);
    add(p0 + v0, p1 + v1 - round, p0 + v0, p1 + v1 - c, p0 + v0, p1 + v1 - round);
    add(p0 + v0 - round, p1 + v1, p0 + v0 - round, p1 + v1, p0 + v0 - c, p1 + v1);
    add(p0 - v0 + round, p1 + v1, p0 - v0 + c, p1 + v1, p0 - v0 + round, p1 + v1);
    add(p0 - v0, p1 + v1 - round, p0 - v0, p1 + v1 - round, p0 - v0, p1 + v1 - c);
    add(p0 - v0, p1 - v1 + round, p0 - v0, p1 - v1 + c, p0 - v0, p1 - v1 + round);
    add(p0 - v0 + round, p1 - v1, p0 - v0 + round, p1 - v1, p0 - v0 + c, p1 - v1);
    add(p0 + v0 - round, p1 - v1, p0 + v0 - c, p1 - v1, p0 + v0 - round, p1 - v1);
  } else {
    add(p0 + v0, p1 - v1, p0 + v0, p1 - v1, p0 + v0, p1 - v1);
    add(p0 + v0, p1 + v1, p0 + v0, p1 + v1, p0 + v0, p1 + v1);
    add(p0 - v0, p1 + v1, p0 - v0, p1 + v1, p0 - v0, p1 + v1);
    add(p0 - v0, p1 - v1, p0 - v0, p1 - v1, p0 - v0, p1 - v1);
  }
  return T.flat();
}
function ellipseShape(p, s) {
  const [x, y] = p, rx = s[0] / 2, ry = s[1] / 2, cx = rx * KAPPA, cy = ry * KAPPA;
  return [x, y - ry, -cx, 0, cx, 0, x + rx, y, 0, -cy, 0, cy, x, y + ry, cx, 0, -cx, 0, x - rx, y, 0, cy, 0, -cy];
}

// ---------- easing ----------
const bz = (p0, p1, p2, p3, u) => { const m = 1 - u; return m * m * m * p0 + 3 * m * m * u * p1 + 3 * m * u * u * p2 + u * u * u * p3; };
const dbz = (p0, p1, p2, p3, u) => { const m = 1 - u; return 3 * m * m * (p1 - p0) + 6 * m * u * (p2 - p1) + 3 * u * u * (p3 - p2); };
// The curve parameter where x(u) = x; c = [x1, y1, x2, y2], x monotonic
function solveU(c, x) {
  if (x <= 0) return 0; if (x >= 1) return 1;
  let u = x;
  for (let k = 0; k < 8; k++) { const f = bz(0, c[0], c[2], 1, u) - x, d = dbz(0, c[0], c[2], 1, u); if (Math.abs(f) < 1e-9) return u; if (Math.abs(d) < 1e-6) break; u -= f / d; if (u < 0 || u > 1) break; }
  let lo = 0, hi = 1; u = x;
  for (let k = 0; k < 60; k++) { const f = bz(0, c[0], c[2], 1, u); if (Math.abs(f - x) < 1e-10) break; if (f < x) lo = u; else hi = u; u = (lo + hi) / 2; }
  return u;
}
// The part of the easing curve between parameters ua and ub (de Casteljau)
function subCurve(c, ua, ub) {
  const split = (Q, u) => {
    const l = (a, b) => [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u];
    const a = l(Q[0], Q[1]), b = l(Q[1], Q[2]), cc = l(Q[2], Q[3]), d = l(a, b), e = l(b, cc), f = l(d, e);
    return [[Q[0], a, d, f], [f, e, cc, Q[3]]];
  };
  let Q = [[0, 0], [c[0], c[1]], [c[2], c[3]], [1, 1]];
  if (ub < 1) Q = split(Q, ub)[0];
  if (ua > 0) Q = split(Q, ua / ub)[1];
  return Q;
}
function yExtrema(c) {
  const [p0, p1, p2, p3] = [0, c[1], c[3], 1];
  const a = -p0 + 3 * p1 - 3 * p2 + p3, b = 2 * (p0 - 2 * p1 + p2), cc = p1 - p0;
  const r = [];
  if (Math.abs(a) < 1e-12) { if (Math.abs(b) > 1e-12) r.push(-cc / b); }
  else { const disc = b * b - 4 * a * cc; if (disc >= 0) { const s = Math.sqrt(disc); r.push((-b + s) / (2 * a), (-b - s) / (2 * a)); } }
  return r.filter(u => u > 1e-6 && u < 1 - 1e-6).sort((x, y) => x - y);
}
// keySplines must lie in 0-1. Overshooting curves are cut at their value extrema, then
// halved until every piece's normalized control points fit; each piece is exact.
function validPieces(c, u0, u1, depth) {
  const Q = subCurve(c, u0, u1);
  const dx = Q[3][0] - Q[0][0], dy = Q[3][1] - Q[0][1];
  if (dx <= 1e-12 || Math.abs(dy) < 1e-12) return [[u0, u1, [0, 0, 1, 1]]];
  const n = q => [(q[0] - Q[0][0]) / dx, (q[1] - Q[0][1]) / dy];
  const [a, b] = [n(Q[1]), n(Q[2])];
  const ok = [a[0], a[1], b[0], b[1]].every(v => v >= -1e-9 && v <= 1 + 1e-9);
  if (ok || depth >= 6) {
    const spl = [a[0], a[1], b[0], b[1]].map(v => Math.min(1, Math.max(0, v)));
    return [[u0, u1, Math.abs(spl[0] - spl[1]) < 1e-6 && Math.abs(spl[2] - spl[3]) < 1e-6 ? [0, 0, 1, 1] : spl]];
  }
  const ext = depth === 0 ? yExtrema(c).filter(u => u > u0 + 1e-9 && u < u1 - 1e-9) : [];
  if (ext.length) { const cuts = [u0, ...ext, u1], out = []; for (let j = 0; j < cuts.length - 1; j++) out.push(...validPieces(c, cuts[j], cuts[j + 1], 1)); return out; }
  const m = (u0 + u1) / 2;
  return [...validPieces(c, u0, m, depth + 1), ...validPieces(c, m, u1, depth + 1)];
}

// ---------- timelines ----------
// {constant} or {first, last, segs: [{t0, t1, v0, v1, ease: [x1, y1, x2, y2] | 'hold'}]}
function timeline(prop, conv) {
  if (!prop) return null;
  if (prop.a !== 1) return { constant: conv(prop.k) };
  // engine exports carry f32 noise in key times (28.00002, 4.7999997): snap to 1e-4 frame
  const k = prop.k.map(kf => ({ ...kf, t: Math.round(kf.t * 1e4) / 1e4 })), segs = [];
  for (let j = 0; j < k.length - 1; j++) {
    const a = k[j], b = k[j + 1];
    const v0 = conv(a.s), v1 = conv(a.e ?? b.s);
    if (a.h === 1) segs.push({ t0: a.t, t1: b.t, v0, v1: v0, ease: 'hold' });
    else {
      const sc = x => Array.isArray(x) ? x[0] : x;
      segs.push({ t0: a.t, t1: b.t, v0, v1, ease: [sc(a.o.x), sc(a.o.y), sc(a.i.x), sc(a.i.y)] });
    }
  }
  return { first: { t: k[0].t, v: conv(k[0].s) }, last: { t: k[k.length - 1].t, v: conv(k[k.length - 1].s ?? k[k.length - 2].e) }, segs };
}
const lerpV = (a, b, y) => a.map((x, i) => x + (b[i] - x) * y);
function valueAt(tl, t) {
  if (tl.constant) return tl.constant;
  if (t <= tl.first.t) return tl.first.v;
  for (const s of tl.segs) {
    if (t < s.t1) {
      if (s.ease === 'hold') return s.v0;
      const u = solveU(s.ease, (t - s.t0) / (s.t1 - s.t0));
      return lerpV(s.v0, s.v1, bz(0, s.ease[1], s.ease[3], 1, u));
    }
  }
  return tl.last.v;
}
const sameV = (a, b, eps = 1e-6) => a.length === b.length && a.every((x, i) => Math.abs(x - b[i]) <= eps);
function clampPts(pts, [lo, hi]) {
  const c = v => [Math.min(hi, Math.max(lo, v[0]))];
  const out = [];
  for (let j = 0; j < pts.length; j++) {
    const p = pts[j], q = pts[j - 1];
    if (q) {
      for (const bnd of [lo, hi]) {
        if ((q.v[0] - bnd) * (p.v[0] - bnd) < 0) {
          const f = (bnd - q.v[0]) / (p.v[0] - q.v[0]);
          out.push({ t: q.t + (p.t - q.t) * f, v: [bnd], spline: [0, 0, 1, 1] });
        }
      }
    }
    out.push({ ...p, v: c(p.v) });
  }
  return out;
}
// SMIL keys of a timeline over frames [W0, W1]: null when constant, else {keyTimes,
// times, values, splines}. `clamp` bounds scalar values (trim).
function smilKeys(tl, W0, W1, clamp) {
  if (tl.constant) return null;
  const D = W1 - W0, kt = t => (t - W0) / D;
  const pts = []; // {t, v, spline of the interval ending here}
  const push = (t, v, spline) => {
    const last = pts[pts.length - 1];
    if (last && Math.abs(last.t - t) < 1e-9 && sameV(last.v, v)) return;
    pts.push({ t, v, spline });
  };
  push(W0, valueAt(tl, W0));
  for (const s of tl.segs) {
    const a = Math.max(s.t0, W0), b = Math.min(s.t1, W1);
    if (b <= a) continue;
    if (s.t0 > W0 && pts[pts.length - 1].t < s.t0) push(s.t0, s.v0, [0, 0, 1, 1]);
    if (s.ease === 'hold') {
      // Lottie: v0 while t < t1, the next value from t1 on. The hold ends in a 1e-6 ramp
      // whose ends round down, so the new value shows on the key's own frame and no two
      // keyTimes are equal (engines read equal keyTimes differently).
      const next = valueAt(tl, s.t1);
      if (b === s.t1 && b < W1 && !sameV(next, s.v0)) {
        push(Math.max(a, b - 1e-6 * D), s.v0, [0, 0, 1, 1]); pts[pts.length - 1].floor = true;
        push(b, next, [0, 0, 1, 1]); pts[pts.length - 1].floor = true;
      } else push(b, s.v0, [0, 0, 1, 1]);
      continue;
    }
    const D0 = s.t1 - s.t0, xa = (a - s.t0) / D0, xb = (b - s.t0) / D0;
    for (const [, u1, spl] of validPieces(s.ease, solveU(s.ease, xa), solveU(s.ease, xb), 0)) {
      const x1 = bz(0, s.ease[0], s.ease[2], 1, u1), y1 = bz(0, s.ease[1], s.ease[3], 1, u1);
      push(s.t0 + x1 * D0, lerpV(s.v0, s.v1, y1), spl);
    }
  }
  if (pts[pts.length - 1].t < W1 - 1e-9) push(W1, valueAt(tl, W1), [0, 0, 1, 1]);
  const out = clamp ? clampPts(pts, clamp) : pts;
  if (out.every(p => sameV(p.v, out[0].v))) return null;
  const merged = [out[0]];
  for (let j = 1; j < out.length; j++) {
    const p = out[j], prev = merged[merged.length - 1], prev2 = merged[merged.length - 2];
    if (prev2 && sameV(prev2.v, prev.v) && sameV(prev.v, p.v) && p.spline && prev.spline) { prev.t = p.t; prev.floor = p.floor; continue; }
    merged.push(p);
  }
  return {
    keyTimes: merged.map(p => p.floor ? { f: Math.floor(kt(p.t) * 1e7 + 1e-6) / 1e7 } : kt(p.t)),
    times: merged.map(p => p.t),
    values: merged.map(p => p.v),
    splines: merged.slice(1).map(p => p.spline || [0, 0, 1, 1]),
  };
}

// ---------- slots ----------
// The runtime's paintIcon, as static values
function slotValues(style, weight) {
  const authored = [1.5, 2, 2.5].includes(weight);
  const s = {};
  for (const [name, w] of [['light', 1.5], ['regular', 2], ['bold', 2.5]]) s[`style.outline.${name}`] = style !== 'solid' && (authored ? weight === w : name === 'regular') ? 100 : 0;
  s['style.outline.width'] = authored ? 2 : weight;
  s['style.solid'] = style === 'solid' ? 100 : 0;
  s['style.tint'] = style === 'duotone' ? 18 : 0;
  return s;
}
const resolve = (prop, slots) => prop && prop.sid && prop.sid in slots && prop.a !== 1 ? { a: 0, k: slots[prop.sid] } : prop;
const scalar = k => [Array.isArray(k) ? k[0] : k];
const vec = k => [k[0], k[1]];
const unsupported = what => Error(`Unsupported Lottie for SVG export: ${what}`);

// ---------- renderer ----------
// variants: [{anim, window: [f0, f1], begin, extra}] share one element tree; the static
// attributes are the pose at frame `rest` of variant `restVariant`.
function render({ variants, rest, restVariant, id, style, weight, color, size, ui }) {
  const base = variants[0].anim, fps = base.fr;
  const slots = slotValues(style, weight);
  const V = variants.map(v => {
    const [W0, W1] = v.window;
    if (v.anim.layers.length !== base.layers.length || v.anim.layers.some((l, i) => l.nm !== base.layers[i].nm)) throw unsupported('the animations of one icon differ in layers');
    return { ...v, W0, W1, timing: `dur="${num((W1 - W0) / fps, 4)}s" begin="${v.begin}"${v.extra ?? ''}` };
  });
  const byInd = new Map(base.layers.map((l, i) => [l.ind, i]));
  const defs = [];
  let maskN = 0;
  const many = V.length > 1;

  const kAttr = k => {
    // SMIL drops an animation whose keyTimes decrease: keep the rounded list non-decreasing
    let last = 0;
    const ks = k.keyTimes.map(x => { let s = typeof x === 'object' ? num(x.f, 7) : num(x, 5); if (+s < last) s = num(last, 7); last = +s; return s; }).join(';');
    const sp = k.splines.map(s => s.map(x => num(x, 4)).join(' ')).join(';');
    const linear = k.splines.every(s => s[0] === 0 && s[1] === 0 && s[2] === 1 && s[3] === 1);
    return `keyTimes="${ks}"` + (linear ? '' : ` calcMode="spline" keySplines="${sp}"`);
  };
  const track = (get, conv) => V.map(v => timeline(get(v.anim), conv));
  const restOf = tls => valueAt(tls[restVariant], rest);
  // Per variant: keys, a constant (when it differs from rest, or when variants share the
  // element and must override each other's frozen values), or null.
  function keysOf(tls, clamp, restVal = restOf(tls)) {
    const ks = tls.map((tl, k) => smilKeys(tl, V[k].W0, V[k].W1, clamp));
    const cl = v => clamp ? [Math.min(clamp[1], Math.max(clamp[0], v[0]))] : v;
    const consts = tls.map((tl, k) => ks[k] ? null : cl(valueAt(tl, V[k].W0)));
    if (!(ks.some(Boolean) || consts.some(c => c && !sameV(c, cl(restVal))))) return null;
    return ks.map((k, j) => k ?? (many || !sameV(consts[j], cl(restVal)) ? { constant: consts[j] } : null));
  }
  function emit(tag, attr, ks, fmt) {
    if (!ks) return '';
    return ks.map((k, j) => {
      if (!k) return '';
      const what = tag === 'animateTransform' ? `attributeName="transform" type="${attr}"` : `attributeName="${attr}"`;
      if (k.constant) return `<${tag} ${what} values="${fmt(k.constant)}" ${V[j].timing}/>`;
      return `<${tag} ${what} values="${k.values.map(fmt).join(';')}" ${kAttr(k)} ${V[j].timing}/>`;
    }).join('');
  }

  // A layer's own transform, outermost first: translate(p - a) then rotate(r, a)
  function layerTransform(li) {
    const ks = base.layers[li].ks, out = [];
    const a = ks.a ? vec(ks.a.k) : [0, 0];
    if (ks.s && (ks.s.a === 1 || ks.s.k[0] !== 100 || ks.s.k[1] !== 100)) throw unsupported('layer scale');
    if (V.some(v => v.anim.layers[li].ks.a?.a === 1)) throw unsupported('an animated anchor');
    if (ks.sk && (ks.sk.a === 1 || ks.sk.k !== 0)) throw unsupported('skew');
    const T = (x, y) => `${num(x)},${num(y)}`;
    let pRest;
    const pEls = [];
    // One animation may key a layer's position per axis and another as a pair.
    if (V.some(v => v.anim.layers[li].ks.p.s)) {
      const axis = i => V.map(v => { const p = v.anim.layers[li].ks.p; return p.s ? timeline(i ? p.y : p.x, scalar) : timeline(p, k => [k[i]]); });
      const tx = axis(0), ty = axis(1);
      const rx = restOf(tx)[0], ry = restOf(ty)[0];
      pRest = [rx - a[0], ry - a[1]];
      const kx = keysOf(tx), ky = keysOf(ty);
      if (kx && ky) {
        pEls.push(`<g transform="translate(${num(pRest[0])} 0)">${emit('animateTransform', 'translate', kx, v => T(v[0] - a[0], 0))}`);
        pEls.push(`<g transform="translate(0 ${num(pRest[1])})">${emit('animateTransform', 'translate', ky, v => T(0, v[0] - a[1]))}`);
      } else if (kx) pEls.push(`<g transform="translate(${num(pRest[0])} ${num(pRest[1])})">${emit('animateTransform', 'translate', kx, v => T(v[0] - a[0], ry - a[1]))}`);
      else if (ky) pEls.push(`<g transform="translate(${num(pRest[0])} ${num(pRest[1])})">${emit('animateTransform', 'translate', ky, v => T(rx - a[0], v[0] - a[1]))}`);
    } else {
      const tp = track(an => an.layers[li].ks.p, vec), r = restOf(tp), kp = keysOf(tp);
      pRest = [r[0] - a[0], r[1] - a[1]];
      if (kp) pEls.push(`<g transform="translate(${num(pRest[0])} ${num(pRest[1])})">${emit('animateTransform', 'translate', kp, v => T(v[0] - a[0], v[1] - a[1]))}`);
    }
    const tr = track(an => an.layers[li].ks.r ?? { a: 0, k: 0 }, scalar), kr = keysOf(tr), rRest = restOf(tr)[0];
    const rot = r => `${num(r)} ${num(a[0])} ${num(a[1])}`;
    const staticT = [];
    for (const open of pEls) out.push(open);
    if (!pEls.length && (Math.abs(pRest[0]) > 1e-9 || Math.abs(pRest[1]) > 1e-9)) staticT.push(`translate(${num(pRest[0])} ${num(pRest[1])})`);
    if (kr) {
      if (staticT.length) { out.push(`<g transform="${staticT.join(' ')}">`); staticT.length = 0; }
      out.push(`<g transform="rotate(${rot(rRest)})">${emit('animateTransform', 'rotate', kr, v => rot(v[0]))}`);
    } else if (Math.abs(rRest) > 1e-9) staticT.push(`rotate(${rot(rRest)})`);
    if (staticT.length) out.push(`<g transform="${staticT.join(' ')}">`);
    return out;
  }
  const wrap = (opens, inner) => opens.join('') + inner + '</g>'.repeat(opens.length);
  const chain = li => { const c = []; let p = base.layers[li].parent; while (p != null) { const pi = byInd.get(p); if (pi === undefined) throw unsupported('a missing parent layer'); c.unshift(pi); p = base.layers[pi].parent; } return c; };

  function opacityWrap(li) {
    const t = track(an => an.layers[li].ks.o ?? { a: 0, k: 100 }, scalar), k = keysOf(t), r = restOf(t)[0];
    if (!k) return r >= 100 ? null : { open: `<g opacity="${num(r / 100)}">`, zero: r <= 0 };
    return { open: `<g opacity="${num(r / 100)}">${emit('animate', 'opacity', k, v => num(v[0] / 100))}` };
  }

  // per-variant keys of several paths drawn as one compound path (fill winding)
  function multiKeys(geoms) {
    const restD = geoms.map(g => shapeD(restOf(g.tls), g.closed, true, g.split)).join('');
    let any = false;
    const ks = V.map((v, j) => {
      const per = geoms.map(g => smilKeys(g.tls[j], v.W0, v.W1)), anim = per.filter(Boolean);
      const at = t => geoms.map(g => shapeD(valueAt(g.tls[j], t), g.closed, true, g.split)).join('');
      if (!anim.length) { const c = at(v.W0); if (c !== restD) any = true; return many || c !== restD ? { constant: c } : null; }
      any = true;
      const same = anim.every(k => k.times.length === anim[0].times.length && k.times.every((x, i) => Math.abs(x - anim[0].times[i]) < 1e-6) && k.splines.every((s, i) => s.every((x, q) => Math.abs(x - anim[0].splines[i][q]) < 1e-4)));
      if (same) return { keyTimes: anim[0].keyTimes, splines: anim[0].splines, values: anim[0].times.map(at) };
      // paths keyed at different times: one key per frame
      const times = []; for (let f = v.W0; f < v.W1; f += 1) times.push(f); times.push(v.W1);
      return { keyTimes: times.map(t => (t - v.W0) / (v.W1 - v.W0)), splines: times.slice(1).map(() => [0, 0, 1, 1]), values: times.map(at) };
    });
    return { restD, ks: any ? ks : null };
  }

  // Lottie draws nothing while a trim's end is 0; SVG would draw a round-cap dot. The
  // state only changes at key times.
  function visibilityKeys(te, j, restHidden) {
    const { W0, W1 } = V[j], D = W1 - W0;
    const hid = t => valueAt(te, t)[0] <= 1e-6;
    const times = [W0, ...(te.segs ?? []).flatMap(s => [s.t0, s.t1]).filter(t => t > W0 && t < W1), W1].sort((x, y) => x - y).filter((t, i, arr) => i === 0 || t - arr[i - 1] > 1e-9);
    const fl = x => Math.floor(x * 1e7 + 1e-6) / 1e7;
    const changes = [];
    const set = (kt, h) => { const last = changes[changes.length - 1]; if (last && last[1] === h) return; if (last && Math.abs(last[0] - kt) < 1e-9) { last[1] = h; return; } changes.push([kt, h]); };
    for (let q = 0; q < times.length - 1; q++) {
      const t = times[q], mid = hid((t + times[q + 1]) / 2), here = hid(t);
      if (q === 0) set(0, here && mid ? true : mid);
      else if (here && !mid) { set(fl((t - W0) / D), true); set(fl((t - W0) / D) + 1e-6, false); }
      else set(fl((t - W0) / D), mid);
    }
    if (changes.length === 1 && changes[0][1] === restHidden && !many) return '';
    if (changes.length === 1) return `<set attributeName="visibility" to="${changes[0][1] ? 'hidden' : 'visible'}" ${V[j].timing}/>`;
    return `<animate attributeName="visibility" calcMode="discrete" values="${changes.map(c => c[1] ? 'hidden' : 'visible').join(';')}" keyTimes="${changes.map(c => num(c[0], 7)).join(';')}" ${V[j].timing}/>`;
  }

  // a layer's shapes; `paintOverride` paints a matte source
  function layerContent(li, paintOverride) {
    const items = base.layers[li].shapes ?? [];
    const kinds = l => (l.shapes ?? []).map(x => x.ty).join() + '|' + (l.masksProperties?.length ?? 0);
    if (V.some(v => kinds(v.anim.layers[li]) !== kinds(base.layers[li]))) throw unsupported(`the animations of one icon differ in the shapes of ${base.layers[li].nm}`);
    const geoms = [], styles = [];
    let trimIdx = -1;
    items.forEach((it, si) => {
      if (it.ty === 'sh') { const tls = track(an => padMorph(an.layers[li].shapes[si].ks), shapeConv), closed = (it.ks.a ? it.ks.k[0].s[0] : it.ks.k).c; geoms.push({ tls, closed, split: hairpins(tls.flatMap(tlKeys), closed) }); }
      else if (it.ty === 'rc' || it.ty === 'el') {
        if (['p', 's', 'r'].some(p => it[p]?.a === 1)) throw unsupported(`an animated ${it.ty === 'rc' ? 'rectangle' : 'ellipse'}`);
        const flat = it.ty === 'rc' ? rectShape(it.p.k, it.s.k, it.r.k) : ellipseShape(it.p.k, it.s.k);
        geoms.push({ tls: V.map(() => ({ constant: flat })), closed: true, prim: it });
      }
      else if (it.ty === 'tm') trimIdx = si;
      else if (it.ty === 'st' || it.ty === 'fl') styles.push(it);
      else throw unsupported(`shape item ${it.ty}`);
    });
    let stroke = null, fill = null;
    for (const st of styles) {
      const o = resolve(st.o, slots);
      if (o.a === 1) throw unsupported('animated paint opacity');
      const op = scalar(o.k)[0];
      if (op <= 0) continue;
      if (st.ty === 'st') {
        const w = resolve(st.w, slots); if (w.a === 1) throw unsupported('animated stroke width');
        if (st.d?.length) throw unsupported('dashed strokes');
        stroke = { o: op, w: scalar(w.k)[0], before: !fill };
      } else fill = { o: op, rule: st.r === 2 ? 'evenodd' : null, before: !stroke };
    }
    if (!stroke && !fill) return '';
    const c = paintOverride ?? color;
    let paint = '';
    if (fill) { paint += ` fill="${c}"`; if (fill.o < 100) paint += ` fill-opacity="${num(fill.o / 100)}"`; if (fill.rule) paint += ` fill-rule="${fill.rule}"`; }
    else paint += ' fill="none"';
    if (stroke) { paint += ` stroke="${c}" stroke-width="${num(stroke.w)}"`; if (stroke.o < 100) paint += ` stroke-opacity="${num(stroke.o / 100)}"`; }
    // Lottie paints earlier items on top: a fill listed before its stroke covers it
    if (fill && stroke && fill.before) paint += ' paint-order="stroke"';
    const geomEl = g => {
      const k = keysOf(g.tls);
      return { d: shapeD(restOf(g.tls), g.closed, !!k, g.split), anim: emit('animate', 'd', k, v => shapeD(v, g.closed, true, g.split)) };
    };
    const close = (open, inner) => open + (inner ? `>${inner}</path>` : '/>');
    if (trimIdx >= 0) {
      // trim end only (start and offset are 0 in every plan); "simultaneous" trims each
      // path. A fill under a trim stays whole: SVG can't trim a fill (team-roles, lasso-select).
      if (V.some(v => { const t = v.anim.layers[li].shapes[trimIdx]; return t.s.a === 1 || t.o.a === 1 || scalar(t.s.k)[0] !== 0 || scalar(t.o.k)[0] !== 0 || (t.m ?? 1) !== 1; })) throw unsupported('trim start, offset or individual trims');
      const te = track(an => an.layers[li].shapes[trimIdx].e, scalar);
      const ke = keysOf(te, [0, 100]), eRest = Math.min(100, Math.max(0, restOf(te)[0]));
      if (!ke && eRest <= 0) return '';
      const els = [];
      for (const g of geoms) {
        const { d, anim } = geomEl(g);
        let dash = '', extra = '';
        if (ke || eRest < 100) {
          // Renderers without SMIL often ignore pathLength too, so a stroke fully drawn or
          // hidden at rest carries no dash; the dash pattern is set only while animating.
          const partial = eRest > 0 && eRest < 100;
          dash = ' pathLength="1"' + (partial ? ` stroke-dasharray="1 2" stroke-dashoffset="${num(1 - eRest / 100, 4)}"` : '');
          if (ke) {
            extra = V.map((v, j) => ke[j] && !partial ? `<set attributeName="stroke-dasharray" to="1 2" ${v.timing}/>` : '').join('');
            extra += emit('animate', 'stroke-dashoffset', ke, v => num(1 - v[0] / 100, 4));
            extra += V.map((v, j) => ke[j] ? visibilityKeys(te[j], j, eRest <= 0) : '').join('');
          }
        }
        els.push(close(`<path d="${d}"${paint}${dash}${eRest <= 0 ? ' visibility="hidden"' : ''}`, anim + extra));
      }
      return els.join('');
    }
    if (geoms.length === 1 && geoms[0].prim) {
      const pr = geoms[0].prim;
      if (pr.ty === 'el') { const [x, y] = pr.p.k, [w, h] = pr.s.k; return w === h ? `<circle cx="${num(x)}" cy="${num(y)}" r="${num(w / 2)}"${paint}/>` : `<ellipse cx="${num(x)}" cy="${num(y)}" rx="${num(w / 2)}" ry="${num(h / 2)}"${paint}/>`; }
      const [x, y] = pr.p.k, [w, h] = pr.s.k, r = Math.min(w / 2, h / 2, pr.r.k);
      return `<rect x="${num(x - w / 2)}" y="${num(y - h / 2)}" width="${num(w)}" height="${num(h)}"${r ? ` rx="${num(r)}"` : ''}${paint}/>`;
    }
    if (geoms.length === 1) { const { d, anim } = geomEl(geoms[0]); return close(`<path d="${d}"${paint}`, anim); }
    const { restD, ks } = multiKeys(geoms);
    if (!ks) return `<path d="${geoms.map(g => shapeD(restOf(g.tls), g.closed, false, g.split)).join('')}"${paint}/>`;
    return close(`<path d="${restD}"${paint}`, emit('animate', 'd', ks, v => v));
  }

  function maskDef(li) { // layer masks (subtract), in layer space
    const mid = `${id}_k${maskN++}`;
    let s = `<mask id="${mid}" maskUnits="userSpaceOnUse" x="-64" y="-64" width="160" height="160"><rect x="-64" y="-64" width="160" height="160" fill="#fff"/>`;
    base.layers[li].masksProperties.forEach((m, mi) => {
      if (m.mode !== 's' || m.inv || (m.o && scalar(m.o.k)[0] !== 100) || (m.x && scalar(m.x.k)[0] !== 0)) throw unsupported(`mask mode ${m.mode}`);
      const tls = track(an => padMorph(an.layers[li].masksProperties[mi].pt), shapeConv), closed = (m.pt.a ? m.pt.k[0].s[0] : m.pt.k).c;
      const k = keysOf(tls), split = hairpins(tls.flatMap(tlKeys), closed);
      s += `<path d="${shapeD(restOf(tls), closed, !!k, split)}" fill="#000"` + (k ? `>${emit('animate', 'd', k, v => shapeD(v, closed, true, split))}</path>` : '/>');
    });
    defs.push(s + '</mask>');
    return mid;
  }

  // one layer: its own transform, opacity, masks and content (not its parents)
  function layerBody(li, paintOverride) {
    const l = base.layers[li];
    if (l.ty !== 4 || l.tm || l.ef?.length || l.bm || (l.ip ?? 0) > base.ip || (l.op ?? base.op) < base.op) throw unsupported(`layer ${l.nm} (type ${l.ty}, time remap, effects, blend or in/out points)`);
    const content = layerContent(li, paintOverride);
    if (!content) return '';
    const op = opacityWrap(li);
    if (op?.zero && !many) return '';
    let s = content;
    if (l.masksProperties?.length) s = `<g mask="url(#${maskDef(li)})">${s}</g>`;
    if (op) s = op.open + s + '</g>';
    return wrap(layerTransform(li), s);
  }
  const wrapChain = (anc, inner) => { let s = inner; for (let j = anc.length - 1; j >= 0; j--) s = wrap(layerTransform(anc[j]), s); return s; };

  // bottom to top; neighbours with the same parents share their parents' groups
  let body = '', run = null;
  const flush = () => { if (run && run.inner) body += wrapChain(run.anc, run.inner); run = null; };
  for (let li = base.layers.length - 1; li >= 0; li--) {
    const l = base.layers[li];
    if (l.td) continue;
    const anc = chain(li);
    if (l.tt) {
      if (l.tt !== 2) throw unsupported(`matte mode ${l.tt}`);
      flush();
      const inner = layerBody(li);
      if (!inner) continue;
      const si = l.tp != null ? byInd.get(l.tp) : li - 1;
      if (si === undefined || !base.layers[si]?.td) throw unsupported('a missing matte source');
      const mid = `${id}_t${maskN++}`;
      defs.push(`<mask id="${mid}" maskUnits="userSpaceOnUse" x="-64" y="-64" width="160" height="160"><rect x="-64" y="-64" width="160" height="160" fill="#fff"/>${wrapChain(chain(si), layerBody(si, '#000'))}</mask>`);
      body += `<g mask="url(#${mid})">${wrapChain(anc, inner)}</g>`;
      continue;
    }
    const inner = layerBody(li);
    if (!inner) continue;
    const key = anc.join(',');
    if (run && run.key === key) run.inner += inner;
    else { flush(); run = { key, anc, inner }; }
  }
  flush();

  const W = base.w, H = base.h, longest = num(Math.max(...V.map(v => v.W1 - v.W0)) / fps, 4) + 's';
  const hit = (hid, extra = '') => `<rect id="${hid}" width="${W}" height="${H}" fill="none" pointer-events="all"${extra}>`;
  let controls = '';
  if (ui === 'hover' || ui === 'click') controls = `${hit(`${id}_h`)}<set id="${id}_m" attributeName="x" to="0" begin="${id}_h.${ui === 'hover' ? 'mouseenter' : 'click'}" dur="${longest}" restart="whenNotActive"/></rect>`;
  if (ui === 'js') controls = `<rect width="0" height="0"><set id="${id}_m" attributeName="x" to="0" begin="indefinite" dur="${longest}"/></rect>`;
  if (ui === 'toggle') controls = `${hit(`${id}_a`)}<set attributeName="display" to="none" begin="${id}_a.click" end="${id}_b.click"/></rect>` +
    `${hit(`${id}_b`, ' display="none"')}<set attributeName="display" to="inline" begin="${id}_a.click" end="${id}_b.click"/></rect>`;
  if (ui === 'js-states') controls = `<rect width="0" height="0"><set id="${id}_a" attributeName="x" to="0" begin="indefinite" dur="${longest}"/><set id="${id}_b" attributeName="x" to="0" begin="indefinite" dur="${longest}"/></rect>`;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${W} ${H}" width="${num(size ?? W)}" height="${num((size ?? W) * H / W)}" stroke-linecap="round" stroke-linejoin="round">` +
    (defs.length ? `<defs>${defs.join('')}</defs>` : '') + body + controls + '</svg>';
}

// ---------- options ----------
function looks(options) {
  const { style = 'outline', weight = 2, color = 'currentColor', size } = options;
  if (!STYLES.includes(style)) throw Error(`style must be one of ${STYLES.join(', ')}`);
  if (!Number.isFinite(weight) || weight < 1 || weight > 3) throw Error('weight must be a number from 1 to 3');
  if (color !== 'currentColor' && !/^#([0-9a-f]{3}|[0-9a-f]{6})$/i.test(color)) throw Error('color must be currentColor or a hex value such as #008c76');
  if (size !== undefined && !(Number.isFinite(size) && size > 0 && size <= 4096)) throw Error('size must be greater than 0 and at most 4096');
  return { style, weight, color, size };
}
// SMIL timing reads '-' and '.' inside an id as an offset or an event: ids stay [A-Za-z0-9_]
function svgId(id) {
  if (typeof id !== 'string' || !/^[A-Za-z][A-Za-z0-9_-]{0,63}$/.test(id)) throw Error('id must start with a letter and hold at most 64 letters, digits, - and _');
  return id.replace(/-/g, '_');
}

/** The trigger an icon gets by default: toggle when stateful, loop for loaders, else hover. */
export function defaultTrigger(icon, motion = 'light') {
  const action = icon.motions?.find(m => m.motionId === motion) ?? icon.motions?.[0];
  return icon.stateful ? 'toggle' : action?.continuous ? 'loop' : 'hover';
}

/**
 * One Lottie animation as an animated SVG (no stateful toggle).
 * `window` is the frame range played, `rest` the frame shown before and after.
 */
export function lottieToSvg(animation, options = {}) {
  const { trigger = 'loop', window = [animation.ip, animation.op], rest = window[0] } = options;
  if (!['hover', 'click', 'loop', 'load', 'js'].includes(trigger)) throw Error('lottieToSvg plays one animation: use hover, click, loop, load or js');
  const id = svgId(options.id ?? 'icon');
  const begin = trigger === 'loop' || trigger === 'load' ? '0s' : `${id}_m.begin`;
  return render({ variants: [{ anim: animation, window, begin, extra: trigger === 'loop' ? ' repeatCount="indefinite"' : '' }], rest, restVariant: 0, id, ...looks(options), ui: trigger });
}

/**
 * An icon as an animated SVG, the way the runtime plays it.
 * `animations` maps animation names (`motions[].name`) to Lottie JSON.
 * trigger `auto`: toggle for stateful icons, loop for loaders, hover otherwise.
 */
export function iconToSvg(icon, animations, options = {}) {
  const motionId = options.motion ?? 'light';
  const motion = icon.motions?.find(m => m.motionId === motionId);
  if (!motion) throw Error(`${icon.id} has no ${motionId} motion`);
  const trigger = !options.trigger || options.trigger === 'auto' ? defaultTrigger(icon, motionId) : options.trigger;
  if (!SVG_TRIGGERS.includes(trigger)) throw Error(`trigger must be one of ${SVG_TRIGGERS.join(', ')}`);
  const anim = name => {
    const a = typeof animations === 'function' ? animations(name) : animations?.[name];
    if (!a) throw Error(`Animation ${name} is missing`);
    return a;
  };
  const id = svgId(options.id ?? icon.id);
  // The runtime plays frame 0 to ceil(active * fps) and then rests on `rest`; loaders run
  // their whole timeline.
  const action = m => m.continuous ? [0, m.frames] : [0, Math.max(Math.min(m.frames - 1, Math.ceil(m.active * m.fps)), m.rest)];
  if (icon.stateful) {
    if (trigger !== 'toggle' && trigger !== 'js') throw Error(`${icon.id} is stateful: use the toggle or js trigger`);
    if (!motion.reverse) throw Error(`${icon.id} has no reverse action`);
    const state = options.state ?? icon.endState;
    if (state !== icon.startState && state !== icon.endState) throw Error(`Unknown state: ${state}`);
    // The first click leaves the initial state; the second comes back.
    const [leave, back] = state === icon.endState ? [motion.reverse, motion] : [motion, motion.reverse];
    const begin = trigger === 'toggle' ? ['_a.click', '_b.click'] : ['_a.begin', '_b.begin'];
    return render({
      variants: [leave, back].map((m, k) => ({ anim: anim(m.name), window: action(m), begin: id + begin[k], extra: ' fill="freeze"' })),
      rest: back.rest, restVariant: 1, id, ...looks(options), ui: trigger === 'toggle' ? 'toggle' : 'js-states',
    });
  }
  if (trigger === 'toggle') throw Error(`${icon.id} is not stateful: use hover, click, loop, load or js`);
  const window = trigger === 'loop' ? [0, motion.frames] : action(motion);
  return lottieToSvg(anim(motion.name), { ...options, id, trigger, window, rest: motion.rest });
}

/** iconToSvg from the bytes of the icon's `.lottie`. */
export async function dotLottieToSvg(icon, bytes, options = {}) {
  const decoder = new TextDecoder(), entries = await readZip(bytes), animations = {};
  for (const entry of entries) if (/^a\/[^/]+\.json$/.test(entry.name)) animations[entry.name.slice(2, -5)] = JSON.parse(decoder.decode(entry.data));
  return iconToSvg(icon, animations, options);
}
