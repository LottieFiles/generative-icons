//! Geometry: shapes as engine primitives or single cubic contours, their exact bounding
//! boxes, mirrors and point maps, SVG path data, and the grown regions that masks cut
//! (gaps and static knockouts). All coordinates are the 32-unit grid, y down; angles are
//! degrees, clockwise on screen.

use std::f64::consts::PI;

/// Handle length of a quarter circle as a cubic.
const KAPPA: f64 = 0.552_284_749_830_793_4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pt {
    pub x: f64,
    pub y: f64,
}

impl Pt {
    pub const fn new(x: f64, y: f64) -> Pt {
        Pt { x, y }
    }
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, o: Pt) -> Pt {
        Pt::new(self.x + o.x, self.y + o.y)
    }
    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, o: Pt) -> Pt {
        Pt::new(self.x - o.x, self.y - o.y)
    }
    pub fn scale(self, k: f64) -> Pt {
        Pt::new(self.x * k, self.y * k)
    }
    pub fn len(self) -> f64 {
        self.x.hypot(self.y)
    }
    pub fn norm(self) -> Pt {
        let l = self.len();
        if l < 1e-12 { Pt::new(0.0, 0.0) } else { self.scale(1.0 / l) }
    }
    pub fn dot(self, o: Pt) -> f64 {
        self.x * o.x + self.y * o.y
    }
    pub fn lerp(self, o: Pt, t: f64) -> Pt {
        Pt::new(self.x + (o.x - self.x) * t, self.y + (o.y - self.y) * t)
    }
    /// The point at `angle` degrees (0 = +x, clockwise on screen) and distance `len`.
    pub fn polar(self, angle: f64, len: f64) -> Pt {
        let a = angle * PI / 180.0;
        Pt::new(self.x + len * a.cos(), self.y + len * a.sin())
    }
}

/// A segment ending at its last point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Seg {
    Line(Pt),
    Cubic(Pt, Pt, Pt),
}

impl Seg {
    pub fn end(self) -> Pt {
        match self {
            Seg::Line(p) | Seg::Cubic(_, _, p) => p,
        }
    }
    fn map(self, f: &impl Fn(Pt) -> Pt) -> Seg {
        match self {
            Seg::Line(p) => Seg::Line(f(p)),
            Seg::Cubic(a, b, c) => Seg::Cubic(f(a), f(b), f(c)),
        }
    }
}

/// One subpath.
#[derive(Debug, Clone, PartialEq)]
pub struct Contour {
    pub start: Pt,
    pub segs: Vec<Seg>,
    pub closed: bool,
}

/// Engine path commands: MoveTo 0, LineTo 1, CubicTo 2, Close 3.
pub type Commands = Vec<u8>;

impl Contour {
    pub fn map(&self, f: impl Fn(Pt) -> Pt) -> Contour {
        Contour { start: f(self.start), segs: self.segs.iter().map(|s| s.map(&f)).collect(), closed: self.closed }
    }

    pub fn end(&self) -> Pt {
        self.segs.last().map_or(self.start, |s| s.end())
    }

    /// The command list: same commands means the same topology (morphs, path keys).
    pub fn commands(&self) -> Commands {
        let mut c = vec![0];
        for s in &self.segs {
            c.push(match s {
                Seg::Line(_) => 1,
                Seg::Cubic(..) => 2,
            });
        }
        if self.closed {
            c.push(3);
        }
        c
    }

    /// Every point in command order, flattened `[x, y, …]`.
    pub fn flat_points(&self) -> Vec<f64> {
        let mut p = vec![self.start.x, self.start.y];
        for s in &self.segs {
            match s {
                Seg::Line(a) => p.extend([a.x, a.y]),
                Seg::Cubic(a, b, c) => p.extend([a.x, a.y, b.x, b.y, c.x, c.y]),
            }
        }
        p
    }

    /// A closed contour whose last straight edge returns to the start: the Close draws it.
    fn tidy(mut self) -> Contour {
        if self.closed
            && self.segs.len() > 1
            && let Some(Seg::Line(p)) = self.segs.last()
            && p.sub(self.start).len() < 1e-9
        {
            self.segs.pop();
        }
        self
    }

    /// The same commands with new points (`[x, y, …]` in command order).
    pub fn with_points(&self, flat: &[f64]) -> Contour {
        let mut it = flat.chunks(2).map(|c| Pt::new(c[0], c.get(1).copied().unwrap_or(0.0)));
        let fallback = self.start;
        let mut next = move || it.next().unwrap_or(fallback);
        let start = next();
        let segs = self
            .segs
            .iter()
            .map(|s| match s {
                Seg::Line(_) => Seg::Line(next()),
                Seg::Cubic(..) => {
                    let (a, b, c) = (next(), next(), next());
                    Seg::Cubic(a, b, c)
                }
            })
            .collect();
        Contour { start, segs, closed: self.closed }
    }

    /// The exact bounding box (cubic extrema included).
    pub fn bbox(&self) -> BBox {
        let mut b = BBox::point(self.start);
        let mut from = self.start;
        for s in &self.segs {
            match *s {
                Seg::Line(p) => b.include(p),
                Seg::Cubic(c1, c2, p) => {
                    b.include(p);
                    for t in cubic_extrema(from.x, c1.x, c2.x, p.x).into_iter().chain(cubic_extrema(from.y, c1.y, c2.y, p.y)) {
                        b.include(cubic_at(from, c1, c2, p, t));
                    }
                }
            }
            from = s.end();
        }
        b
    }

    /// The contour as straight pieces: lines as they are, each cubic in about one-unit steps.
    pub fn flatten(&self) -> Vec<(Pt, Pt)> {
        let mut out = Vec::new();
        let mut from = self.start;
        let mut push = |a: Pt, b: Pt| {
            if b.sub(a).len() > 1e-9 {
                out.push((a, b));
            }
        };
        for s in &self.segs {
            match *s {
                Seg::Line(p) => push(from, p),
                Seg::Cubic(c1, c2, p) => {
                    let rough = c1.sub(from).len() + c2.sub(c1).len() + p.sub(c2).len();
                    let n = (rough / 1.0).ceil().clamp(2.0, 16.0) as usize;
                    let mut a = from;
                    for i in 1..=n {
                        let b = cubic_at(from, c1, c2, p, i as f64 / n as f64);
                        push(a, b);
                        a = b;
                    }
                }
            }
            from = s.end();
        }
        if self.closed {
            push(from, self.start);
        }
        out
    }
}

pub fn cubic_at(p0: Pt, p1: Pt, p2: Pt, p3: Pt, t: f64) -> Pt {
    let u = 1.0 - t;
    let a = u * u * u;
    let b = 3.0 * u * u * t;
    let c = 3.0 * u * t * t;
    let d = t * t * t;
    Pt::new(a * p0.x + b * p1.x + c * p2.x + d * p3.x, a * p0.y + b * p1.y + c * p2.y + d * p3.y)
}

/// Parameters in (0, 1) where one coordinate of a cubic has a zero derivative.
fn cubic_extrema(p0: f64, p1: f64, p2: f64, p3: f64) -> Vec<f64> {
    let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
    let b = 2.0 * (p0 - 2.0 * p1 + p2);
    let c = p1 - p0;
    let mut roots = Vec::new();
    if a.abs() < 1e-12 {
        if b.abs() > 1e-12 {
            roots.push(-c / b);
        }
    } else {
        let disc = b * b - 4.0 * a * c;
        if disc >= 0.0 {
            let s = disc.sqrt();
            roots.push((-b + s) / (2.0 * a));
            roots.push((-b - s) / (2.0 * a));
        }
    }
    roots.into_iter().filter(|t| *t > 0.0 && *t < 1.0).collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl BBox {
    pub fn point(p: Pt) -> BBox {
        BBox { x0: p.x, y0: p.y, x1: p.x, y1: p.y }
    }
    pub fn include(&mut self, p: Pt) {
        self.x0 = self.x0.min(p.x);
        self.y0 = self.y0.min(p.y);
        self.x1 = self.x1.max(p.x);
        self.y1 = self.y1.max(p.y);
    }
    pub fn union(mut self, o: BBox) -> BBox {
        self.include(Pt::new(o.x0, o.y0));
        self.include(Pt::new(o.x1, o.y1));
        self
    }
    pub fn grow(self, d: f64) -> BBox {
        BBox { x0: self.x0 - d, y0: self.y0 - d, x1: self.x1 + d, y1: self.y1 + d }
    }
    pub fn w(&self) -> f64 {
        self.x1 - self.x0
    }
    pub fn h(&self) -> f64 {
        self.y1 - self.y0
    }
    pub fn center(&self) -> Pt {
        Pt::new((self.x0 + self.x1) / 2.0, (self.y0 + self.y1) / 2.0)
    }
}

/// A part's geometry: an engine primitive where one fits, otherwise a cubic contour.
#[derive(Debug, Clone, PartialEq)]
pub enum Geo {
    Ellipse { c: Pt, rx: f64, ry: f64 },
    /// A rect with one corner radius (`insertRect` plus `CornerRadius`).
    Rect { c: Pt, w: f64, h: f64, r: f64 },
    Path(Contour),
}

impl Geo {
    /// The geometry as a contour (primitives drawn as the engine would).
    pub fn contour(&self) -> Contour {
        match self {
            Geo::Ellipse { c, rx, ry } => ellipse(*c, *rx, *ry),
            Geo::Rect { c, w, h, r } => rect(c.x - w / 2.0, c.y - h / 2.0, *w, *h, [*r; 4], true, false),
            Geo::Path(contour) => contour.clone(),
        }
    }

    pub fn closed(&self) -> bool {
        match self {
            Geo::Path(c) => c.closed,
            _ => true,
        }
    }

    pub fn bbox(&self) -> BBox {
        match self {
            Geo::Ellipse { c, rx, ry } => BBox { x0: c.x - rx, y0: c.y - ry, x1: c.x + rx, y1: c.y + ry },
            Geo::Rect { c, w, h, .. } => BBox { x0: c.x - w / 2.0, y0: c.y - h / 2.0, x1: c.x + w / 2.0, y1: c.y + h / 2.0 },
            Geo::Path(c) => c.bbox(),
        }
    }

    /// Mirrored across the vertical line `x = axis` or, with `vertical: false`, the
    /// horizontal line `y = axis`.
    pub fn mirror(&self, axis: f64, vertical: bool) -> Geo {
        let m = |p: Pt| if vertical { Pt::new(2.0 * axis - p.x, p.y) } else { Pt::new(p.x, 2.0 * axis - p.y) };
        match self {
            Geo::Ellipse { c, rx, ry } => Geo::Ellipse { c: m(*c), rx: *rx, ry: *ry },
            Geo::Rect { c, w, h, r } => Geo::Rect { c: m(*c), w: *w, h: *h, r: *r },
            Geo::Path(c) => Geo::Path(c.map(m)),
        }
    }

    /// `[cmds, flat points]` for a path, the primitive's parameters otherwise.
    pub fn to_plan(&self) -> serde_json::Value {
        use serde_json::json;
        match self {
            Geo::Ellipse { c, rx, ry } => json!({"ellipse": [r6(c.x), r6(c.y), r6(*rx), r6(*ry)]}),
            Geo::Rect { c, w, h, r } => json!({"rect": [r6(c.x), r6(c.y), r6(*w), r6(*h), r6(*r)]}),
            Geo::Path(c) => json!({"path": {"c": c.commands(), "p": c.flat_points().into_iter().map(r6).collect::<Vec<_>>()}}),
        }
    }
}

/// Rounded to 6 decimals, the precision of every number in a plan.
pub fn r6(x: f64) -> f64 {
    let r = (x * 1e6).round() / 1e6;
    if r == 0.0 { 0.0 } else { r }
}

/// A straight line.
pub fn line(a: Pt, b: Pt) -> Contour {
    Contour { start: a, segs: vec![Seg::Line(b)], closed: false }
}

/// A fillet at `v` between the edges from `p` and to `n`: `(t1, cubic to t2)`, or `None`
/// when the corner stays sharp.
fn fillet(p: Pt, v: Pt, n: Pt, r: f64) -> Option<(Pt, Seg)> {
    if r <= 0.0 {
        return None;
    }
    let u1 = p.sub(v).norm();
    let u2 = n.sub(v).norm();
    let cos = u1.dot(u2).clamp(-1.0, 1.0);
    let theta = cos.acos();
    if theta < 1e-6 || (PI - theta) < 1e-6 {
        return None;
    }
    let mut d = r / (theta / 2.0).tan();
    let limit = (p.sub(v).len().min(n.sub(v).len())) / 2.0;
    d = d.min(limit);
    let reff = d * (theta / 2.0).tan();
    let phi = PI - theta;
    let h = 4.0 / 3.0 * (phi / 4.0).tan() * reff;
    let t1 = v.add(u1.scale(d));
    let t2 = v.add(u2.scale(d));
    Some((t1, Seg::Cubic(t1.sub(u1.scale(h)), t2.sub(u2.scale(h)), t2)))
}

/// An open polyline with rounded inner corners.
pub fn polyline(points: &[Pt], r: f64) -> Contour {
    let mut c = Contour { start: points[0], segs: Vec::new(), closed: false };
    for i in 1..points.len() - 1 {
        match fillet(points[i - 1], points[i], points[i + 1], r) {
            Some((t1, arc)) => {
                c.segs.push(Seg::Line(t1));
                c.segs.push(arc);
            }
            None => c.segs.push(Seg::Line(points[i])),
        }
    }
    c.segs.push(Seg::Line(*points.last().expect("two points")));
    c
}

/// A polygon vertex: point, corner radius, and the controls of a curved edge to the next.
pub struct PolyVertex {
    pub p: Pt,
    pub r: f64,
    pub curve: Option<[Pt; 2]>,
}

/// A closed polygon with per-vertex rounding; a corner next to a curved edge stays sharp.
pub fn polygon(vertices: &[PolyVertex]) -> Contour {
    let n = vertices.len();
    let curved_in = |i: usize| vertices[(i + n - 1) % n].curve.is_some();
    let corner = |i: usize| -> Option<(Pt, Seg)> {
        if vertices[i].curve.is_some() || curved_in(i) {
            return None;
        }
        fillet(vertices[(i + n - 1) % n].p, vertices[i].p, vertices[(i + 1) % n].p, vertices[i].r)
    };
    let corners: Vec<Option<(Pt, Seg)>> = (0..n).map(corner).collect();
    let start = match &corners[0] {
        Some((_, arc)) => arc.end(),
        None => vertices[0].p,
    };
    let mut c = Contour { start, segs: Vec::new(), closed: true };
    for step in 1..=n {
        let i = step % n;
        let prev = &vertices[(i + n - 1) % n];
        match (&prev.curve, &corners[i]) {
            (Some([c1, c2]), _) => c.segs.push(Seg::Cubic(*c1, *c2, vertices[i].p)),
            (None, Some((t1, _))) => c.segs.push(Seg::Line(*t1)),
            (None, None) => c.segs.push(Seg::Line(vertices[i].p)),
        }
        if let Some((_, arc)) = &corners[i] {
            c.segs.push(*arc);
        }
    }
    c.tidy()
}

/// A rect from its top-left corner with radii `[tl, tr, br, bl]`. The contour starts at the
/// bottom-left corner and runs clockwise, so the closing edge is the bottom one (`open`
/// leaves it out). `full` keeps every corner and edge even when it has no length, so two
/// rects always share a topology.
pub fn rect(x: f64, y: f64, w: f64, h: f64, radii: [f64; 4], closed: bool, full: bool) -> Contour {
    let limit = w.min(h) / 2.0;
    let [tl, tr, br, bl] = radii.map(|r| r.min(limit).max(0.0));
    let k = KAPPA;
    let mut segs = Vec::new();
    let corner = |segs: &mut Vec<Seg>, from: Pt, c1: Pt, c2: Pt, to: Pt, r: f64| {
        if r > 0.0 || full {
            segs.push(Seg::Cubic(c1, c2, to));
        }
        let _ = from;
    };
    let start = Pt::new(x + bl, y + h);
    // BL corner, up the left edge, TL, the top, TR, down the right edge, BR.
    corner(&mut segs, start, Pt::new(x + bl - k * bl, y + h), Pt::new(x, y + h - bl + k * bl), Pt::new(x, y + h - bl), bl);
    let edge = |segs: &mut Vec<Seg>, to: Pt, length: f64| {
        if length > 1e-9 || full {
            segs.push(Seg::Line(to));
        }
    };
    edge(&mut segs, Pt::new(x, y + tl), h - bl - tl);
    corner(&mut segs, Pt::new(x, y + tl), Pt::new(x, y + tl - k * tl), Pt::new(x + tl - k * tl, y), Pt::new(x + tl, y), tl);
    edge(&mut segs, Pt::new(x + w - tr, y), w - tl - tr);
    corner(&mut segs, Pt::new(x + w - tr, y), Pt::new(x + w - tr + k * tr, y), Pt::new(x + w, y + tr - k * tr), Pt::new(x + w, y + tr), tr);
    edge(&mut segs, Pt::new(x + w, y + h - br), h - tr - br);
    corner(&mut segs, Pt::new(x + w, y + h - br), Pt::new(x + w, y + h - br + k * br), Pt::new(x + w - br + k * br, y + h), Pt::new(x + w - br, y + h), br);
    if !closed {
        return Contour { start, segs, closed: false };
    }
    if full {
        segs.push(Seg::Line(start));
        return Contour { start, segs, closed: true };
    }
    Contour { start, segs, closed: true }
}

/// An ellipse from the top, clockwise, in four cubics.
pub fn ellipse(c: Pt, rx: f64, ry: f64) -> Contour {
    let k = KAPPA;
    let top = Pt::new(c.x, c.y - ry);
    let right = Pt::new(c.x + rx, c.y);
    let bottom = Pt::new(c.x, c.y + ry);
    let left = Pt::new(c.x - rx, c.y);
    Contour {
        start: top,
        segs: vec![
            Seg::Cubic(Pt::new(c.x + k * rx, top.y), Pt::new(right.x, c.y - k * ry), right),
            Seg::Cubic(Pt::new(right.x, c.y + k * ry), Pt::new(c.x + k * rx, bottom.y), bottom),
            Seg::Cubic(Pt::new(c.x - k * rx, bottom.y), Pt::new(left.x, c.y + k * ry), left),
            Seg::Cubic(Pt::new(left.x, c.y - k * ry), Pt::new(c.x - k * rx, top.y), top),
        ],
        closed: true,
    }
}

/// Segments an arc of `sweep` degrees uses unless a topology is imposed.
pub fn arc_segments(sweep: f64) -> usize {
    ((sweep.abs() / 90.0) - 1e-9).ceil().max(1.0) as usize
}

/// An arc about `c` from `from` to `to` degrees (0 = +x, clockwise on screen), in `n` cubics;
/// `closed` adds the chord.
pub fn arc(c: Pt, r: f64, from: f64, to: f64, closed: bool, n: usize) -> Contour {
    let sweep = to - from;
    let delta = sweep / n as f64 * PI / 180.0;
    let h = 4.0 / 3.0 * (delta / 4.0).tan() * r;
    let at = |deg: f64| c.polar(deg, r);
    let tangent = |deg: f64| {
        let a = deg * PI / 180.0;
        Pt::new(-a.sin(), a.cos())
    };
    let mut segs = Vec::new();
    for i in 0..n {
        let a0 = from + sweep * i as f64 / n as f64;
        let a1 = from + sweep * (i + 1) as f64 / n as f64;
        let p0 = at(a0);
        let p3 = at(a1);
        segs.push(Seg::Cubic(p0.add(tangent(a0).scale(h)), p3.sub(tangent(a1).scale(h)), p3));
    }
    Contour { start: at(from), segs, closed }
}

/// A sine wave from `x0` to `x1` about `y`: `cycles` periods, rising first, `phase` in
/// cycles. One cubic per quarter period; the points are linear in `amp`.
pub fn wave(x0: f64, x1: f64, y: f64, cycles: f64, amp: f64, phase: f64) -> Contour {
    let n = ((cycles * 4.0).ceil() as usize).clamp(1, 24);
    let f = |u: f64| y - amp * (2.0 * PI * (cycles * u + phase)).sin();
    let df = |u: f64| -amp * 2.0 * PI * cycles * (2.0 * PI * (cycles * u + phase)).cos();
    let dx = x1 - x0;
    let point = |u: f64| Pt::new(x0 + dx * u, f(u));
    let mut segs = Vec::new();
    for i in 0..n {
        let (ua, ub) = (i as f64 / n as f64, (i + 1) as f64 / n as f64);
        let du = (ub - ua) / 3.0;
        let pa = point(ua);
        let pb = point(ub);
        segs.push(Seg::Cubic(pa.add(Pt::new(dx * du, df(ua) * du)), pb.sub(Pt::new(dx * du, df(ub) * du)), pb));
    }
    Contour { start: point(0.0), segs, closed: false }
}

/// A capsule (stadium) around the segment `a`–`b` with radius `rad`: a closed contour.
pub fn capsule(a: Pt, b: Pt, rad: f64) -> Contour {
    let d = b.sub(a);
    if d.len() < 1e-9 {
        return ellipse(a, rad, rad);
    }
    let n = Pt::new(-d.y, d.x).norm();
    let theta = n.y.atan2(n.x) * 180.0 / PI;
    let around_b = arc(b, rad, theta, theta - 180.0, false, 2);
    let around_a = arc(a, rad, theta + 180.0, theta, false, 2);
    let mut segs = vec![Seg::Line(b.add(n.scale(rad)))];
    segs.extend(around_b.segs);
    segs.push(Seg::Line(a.sub(n.scale(rad))));
    segs.extend(around_a.segs);
    Contour { start: a.add(n.scale(rad)), segs, closed: true }
}

/// The region within `d` of a geometry: its interior when `filled`, plus a capsule around
/// every straight piece of its outline. Subtract masks unite, so this is exact up to the
/// flattening of curves.
pub fn grown(geo: &Geo, filled: bool, d: f64) -> Vec<Contour> {
    match (geo, filled) {
        (Geo::Ellipse { c, rx, ry }, true) => vec![ellipse(*c, rx + d, ry + d)],
        (Geo::Rect { c, w, h, r }, true) => {
            vec![rect(c.x - w / 2.0 - d, c.y - h / 2.0 - d, w + 2.0 * d, h + 2.0 * d, [r + d; 4], true, false)]
        }
        _ => {
            let contour = geo.contour();
            let mut out = Vec::new();
            if filled && contour.closed {
                out.push(contour.clone());
            }
            if d > 0.0 {
                out.extend(contour.flatten().into_iter().map(|(a, b)| capsule(a, b, d)));
            }
            out
        }
    }
}

/// Parses SVG path data with one subpath into a contour: M, L, H, V, C, S, Q, T, A and Z,
/// absolute or relative.
pub fn svg_path(d: &str) -> Result<Contour, String> {
    let tokens = tokenize(d)?;
    let mut i = 0;
    let mut cur = Pt::new(0.0, 0.0);
    let mut start = None;
    let mut segs = Vec::new();
    let mut closed = false;
    let mut last_ctrl: Option<(char, Pt)> = None;
    let mut cmd = ' ';
    let num = |i: &mut usize| -> Result<f64, String> {
        match tokens.get(*i) {
            Some(Tok::Num(n)) => {
                *i += 1;
                Ok(*n)
            }
            _ => Err(format!("expected a number at token {} of the path data", *i + 1)),
        }
    };
    while i < tokens.len() {
        if let Tok::Cmd(c) = tokens[i] {
            cmd = c;
            i += 1;
        } else if cmd == ' ' {
            return Err("path data must start with M".into());
        }
        let rel = cmd.is_ascii_lowercase();
        let base = if rel { cur } else { Pt::new(0.0, 0.0) };
        let pt = |i: &mut usize| -> Result<Pt, String> {
            let x = num(i)?;
            let y = num(i)?;
            Ok(Pt::new(base.x + x, base.y + y))
        };
        match cmd.to_ascii_uppercase() {
            'M' => {
                let p = pt(&mut i)?;
                if start.is_some() {
                    return Err("a path holds one subpath (one M)".into());
                }
                start = Some(p);
                cur = p;
                // Further pairs are LineTo.
                cmd = if rel { 'l' } else { 'L' };
                last_ctrl = None;
                continue;
            }
            'L' => {
                let p = pt(&mut i)?;
                segs.push(Seg::Line(p));
                cur = p;
                last_ctrl = None;
            }
            'H' => {
                let x = num(&mut i)?;
                let p = Pt::new(if rel { cur.x + x } else { x }, cur.y);
                segs.push(Seg::Line(p));
                cur = p;
                last_ctrl = None;
            }
            'V' => {
                let y = num(&mut i)?;
                let p = Pt::new(cur.x, if rel { cur.y + y } else { y });
                segs.push(Seg::Line(p));
                cur = p;
                last_ctrl = None;
            }
            'C' => {
                let c1 = pt(&mut i)?;
                let c2 = pt(&mut i)?;
                let p = pt(&mut i)?;
                segs.push(Seg::Cubic(c1, c2, p));
                cur = p;
                last_ctrl = Some(('C', c2));
            }
            'S' => {
                let c1 = match last_ctrl {
                    Some(('C', c)) => cur.add(cur.sub(c)),
                    _ => cur,
                };
                let c2 = pt(&mut i)?;
                let p = pt(&mut i)?;
                segs.push(Seg::Cubic(c1, c2, p));
                cur = p;
                last_ctrl = Some(('C', c2));
            }
            'Q' => {
                let q = pt(&mut i)?;
                let p = pt(&mut i)?;
                segs.push(quad(cur, q, p));
                cur = p;
                last_ctrl = Some(('Q', q));
            }
            'T' => {
                let q = match last_ctrl {
                    Some(('Q', c)) => cur.add(cur.sub(c)),
                    _ => cur,
                };
                let p = pt(&mut i)?;
                segs.push(quad(cur, q, p));
                cur = p;
                last_ctrl = Some(('Q', q));
            }
            'A' => {
                let rx = num(&mut i)?;
                let ry = num(&mut i)?;
                let rot = num(&mut i)?;
                let large = num(&mut i)? != 0.0;
                let sweep = num(&mut i)? != 0.0;
                let p = pt(&mut i)?;
                segs.extend(svg_arc(cur, rx, ry, rot, large, sweep, p));
                cur = p;
                last_ctrl = None;
            }
            'Z' => {
                closed = true;
                if i < tokens.len() {
                    return Err("Z must come last".into());
                }
                break;
            }
            other => return Err(format!("unknown path command {other}")),
        }
    }
    let start = start.ok_or("path data must start with M")?;
    if segs.is_empty() {
        return Err("path data draws nothing".into());
    }
    Ok(Contour { start, segs, closed }.tidy())
}

fn quad(p0: Pt, q: Pt, p: Pt) -> Seg {
    Seg::Cubic(p0.add(q.sub(p0).scale(2.0 / 3.0)), p.add(q.sub(p).scale(2.0 / 3.0)), p)
}

/// An SVG elliptical arc as cubics (endpoint to centre parameterisation, SVG 1.1 F.6).
fn svg_arc(p0: Pt, rx: f64, ry: f64, rot: f64, large: bool, sweep: bool, p: Pt) -> Vec<Seg> {
    if p0.sub(p).len() < 1e-12 {
        return Vec::new();
    }
    let (mut rx, mut ry) = (rx.abs(), ry.abs());
    if rx < 1e-12 || ry < 1e-12 {
        return vec![Seg::Line(p)];
    }
    let phi = rot * PI / 180.0;
    let (cs, sn) = (phi.cos(), phi.sin());
    let dx = (p0.x - p.x) / 2.0;
    let dy = (p0.y - p.y) / 2.0;
    let x1 = cs * dx + sn * dy;
    let y1 = -sn * dx + cs * dy;
    let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
    if lambda > 1.0 {
        rx *= lambda.sqrt();
        ry *= lambda.sqrt();
    }
    let num = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1;
    let den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let mut coef = (num / den).max(0.0).sqrt();
    if large == sweep {
        coef = -coef;
    }
    let cx1 = coef * rx * y1 / ry;
    let cy1 = -coef * ry * x1 / rx;
    let cx = cs * cx1 - sn * cy1 + (p0.x + p.x) / 2.0;
    let cy = sn * cx1 + cs * cy1 + (p0.y + p.y) / 2.0;
    let angle = |ux: f64, uy: f64, vx: f64, vy: f64| (ux * vy - uy * vx).atan2(ux * vx + uy * vy);
    let t1 = angle(1.0, 0.0, (x1 - cx1) / rx, (y1 - cy1) / ry);
    let mut dt = angle((x1 - cx1) / rx, (y1 - cy1) / ry, (-x1 - cx1) / rx, (-y1 - cy1) / ry);
    if !sweep && dt > 0.0 {
        dt -= 2.0 * PI;
    } else if sweep && dt < 0.0 {
        dt += 2.0 * PI;
    }
    let n = ((dt.abs() / (PI / 2.0)) - 1e-9).ceil().max(1.0) as usize;
    let delta = dt / n as f64;
    let h = 4.0 / 3.0 * (delta / 4.0).tan();
    let point = |t: f64| Pt::new(cx + rx * t.cos() * cs - ry * t.sin() * sn, cy + rx * t.cos() * sn + ry * t.sin() * cs);
    let deriv = |t: f64| Pt::new(-rx * t.sin() * cs - ry * t.cos() * sn, -rx * t.sin() * sn + ry * t.cos() * cs);
    let mut segs = Vec::new();
    for i in 0..n {
        let a = t1 + delta * i as f64;
        let b = a + delta;
        let pa = point(a);
        let pb = if i + 1 == n { p } else { point(b) };
        segs.push(Seg::Cubic(pa.add(deriv(a).scale(h)), pb.sub(deriv(b).scale(h)), pb));
    }
    segs
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tok {
    Cmd(char),
    Num(f64),
}

fn tokenize(d: &str) -> Result<Vec<Tok>, String> {
    let bytes = d.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_whitespace() || c == ',' {
            i += 1;
        } else if "MmLlHhVvCcSsQqTtAaZz".contains(c) {
            out.push(Tok::Cmd(c));
            i += 1;
        } else if c == '-' || c == '+' || c == '.' || c.is_ascii_digit() {
            let start = i;
            if c == '-' || c == '+' {
                i += 1;
            }
            let mut dot = false;
            while i < bytes.len() {
                let ch = bytes[i] as char;
                if ch.is_ascii_digit() {
                    i += 1;
                } else if ch == '.' && !dot {
                    dot = true;
                    i += 1;
                } else if (ch == 'e' || ch == 'E') && i + 1 < bytes.len() {
                    i += 1;
                    if bytes[i] == b'-' || bytes[i] == b'+' {
                        i += 1;
                    }
                } else {
                    break;
                }
            }
            let text = &d[start..i];
            out.push(Tok::Num(text.parse().map_err(|_| format!("`{text}` is not a number in the path data"))?));
        } else {
            return Err(format!("unexpected `{c}` in the path data"));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn rect_starts_bottom_left_clockwise() {
        let c = rect(5.0, 4.0, 11.5, 23.5, [1.5, 1.5, 0.0, 0.0], false, false);
        assert_eq!(c.start, Pt::new(5.0, 27.5));
        // Up the left edge first: the first segment ends above the start.
        assert!(c.segs[0].end().y < c.start.y);
        assert!(!c.closed);
        assert_eq!(c.commands(), vec![0, 1, 2, 1, 2, 1]);
        assert_eq!(c.end(), Pt::new(16.5, 27.5));
        let full = rect(0.0, 0.0, 4.0, 4.0, [0.0; 4], true, true);
        assert_eq!(full.commands(), vec![0, 2, 1, 2, 1, 2, 1, 2, 1, 3]);
    }

    #[test]
    fn arc_matches_the_probe_bell() {
        let c = arc(Pt::new(9.57, 11.07), 2.2, 135.0, 315.0, true, 2);
        assert_eq!(c.commands(), vec![0, 2, 2, 3]);
        let p = c.flat_points();
        assert!(close(p[0], 8.014365) && close(p[1], 12.625635), "{p:?}");
        assert!(close(p[6], 7.155212) || close(p[2], 7.155212), "{p:?}");
    }

    #[test]
    fn bbox_is_tight() {
        let b = ellipse(Pt::new(16.0, 16.0), 4.0, 2.0).bbox();
        assert!(close(b.x0, 12.0) && close(b.x1, 20.0) && close(b.y0, 14.0) && close(b.y1, 18.0), "{b:?}");
        let b = arc(Pt::new(16.0, 16.0), 4.0, 180.0, 360.0, false, 2).bbox();
        assert!(close(b.y0, 12.0) && close(b.y1, 16.0), "{b:?}");
    }

    #[test]
    fn svg_paths_parse() {
        let c = svg_path("M4 4 h10 v6 q 2 2 4 0 a2 2 0 0 1 -4 0 Z").unwrap();
        assert!(c.closed);
        assert_eq!(c.start, Pt::new(4.0, 4.0));
        assert_eq!(c.segs[0], Seg::Line(Pt::new(14.0, 4.0)));
        assert!(svg_path("M1 1 L2 2 M3 3 L4 4").is_err());
        let blob = svg_path("M1.5.5L2-1").unwrap();
        assert_eq!(blob.start, Pt::new(1.5, 0.5));
        assert_eq!(blob.segs[0], Seg::Line(Pt::new(2.0, -1.0)));
    }

    #[test]
    fn polygon_rounds_corners() {
        let v = |x, y| PolyVertex { p: Pt::new(x, y), r: 1.0, curve: None };
        let c = polygon(&[v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0)]);
        assert!(c.closed);
        assert_eq!(c.commands().iter().filter(|&&k| k == 2).count(), 3);
        let b = c.bbox();
        assert!(b.x0 > 0.0 && b.y1 < 10.0);
    }

    #[test]
    fn capsules_cover_their_segment() {
        let c = capsule(Pt::new(0.0, 0.0), Pt::new(10.0, 0.0), 2.0);
        let b = c.bbox();
        assert!(close(b.x0, -2.0) && close(b.x1, 12.0) && close(b.y0, -2.0) && close(b.y1, 2.0), "{b:?}");
    }
}
