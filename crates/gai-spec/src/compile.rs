//! Spec → plan. Resolves anchors, mirrors and rigs, expands each part's role into style
//! layers (with masks and mattes for holes, gaps and knockouts), resolves Light, Strong and
//! the reverses, and turns every track into engine keys: rig rotation and position, group
//! opacity, trim, and path keys for shape parameters, morphs and scale/squash.

use crate::diag::{Diagnostic, Diagnostics, at, did_you_mean};
use crate::ease::{self, CKey};
use crate::geom::{self, BBox, Contour, Geo, PolyVertex, Pt};
use crate::motion::{self, CTrack, Motion, ReverseInput, Track, Which};
use crate::plan::{self, KeyValue, Layer, Mask, PKey, Paint, Plan};
use crate::spec::{Ease, Follow, Knockout, MotionValue, Outline, PartBody, PartSrc, Pivot, Point, Prop, ReverseMode, Role, Shape, Solid, Spec, Squash, Target, TrackSrc};
use indexmap::{IndexMap, IndexSet};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;

/// Outline weights: variant, style, stroke width.
pub const WEIGHTS: [(&str, &str, f64); 3] =
    [("light", "outline.light", 1.5), ("regular", "outline.regular", 2.0), ("bold", "outline.bold", 2.5)];
/// The Solid stroke: v1's `tube` width.
pub const SOLID_WIDTH: f64 = 2.6;
/// Default clearance around a knockout.
pub const KNOCKOUT_GAP: f64 = 0.6;

/// A shape with every point resolved.
#[derive(Debug, Clone, PartialEq)]
pub enum RShape {
    Line(Pt, Pt),
    Polyline(Vec<Pt>, f64),
    Rect { x: f64, y: f64, w: f64, h: f64, r: [f64; 4] },
    Circle { c: Pt, r: f64 },
    Ellipse { c: Pt, rx: f64, ry: f64 },
    Arc { c: Pt, r: f64, from: f64, to: f64, closed: bool },
    Polygon(Vec<(Pt, f64, Option<[Pt; 2]>)>),
    Wave { x0: f64, x1: f64, y: f64, cycles: f64, amp: f64, phase: f64 },
    Path(Contour),
    Morph { a: Box<RShape>, b: Box<RShape>, by: String },
    Mirrored { inner: Box<RShape>, axis: f64, vertical: bool },
}

/// Offsets of the keyable shape parameters.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Params {
    pub r: f64,
    pub grow: f64,
    pub amp: f64,
}

/// How a shape is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Form {
    /// An engine primitive where one fits.
    Native,
    /// Always a path, with a topology that every parameter value shares.
    Path,
    /// The outline without its closing edge.
    Open,
}

impl RShape {
    pub fn geo(&self, p: Params, form: Form) -> Geo {
        match self {
            RShape::Line(a, b) => Geo::Path(geom::line(*a, *b)),
            RShape::Polyline(pts, r) => Geo::Path(geom::polyline(pts, *r)),
            RShape::Rect { x, y, w, h, r } => {
                let g = p.grow;
                let radii = r.map(|c| (c + p.r + g).max(0.0));
                let (x, y, w, h) = (x - g, y - g, w + 2.0 * g, h + 2.0 * g);
                match form {
                    Form::Native if radii.iter().all(|c| (c - radii[0]).abs() < 1e-12) => {
                        Geo::Rect { c: Pt::new(x + w / 2.0, y + h / 2.0), w, h, r: radii[0] }
                    }
                    Form::Native => Geo::Path(geom::rect(x, y, w, h, radii, true, false)),
                    Form::Path => Geo::Path(geom::rect(x, y, w, h, radii, true, true)),
                    Form::Open => Geo::Path(geom::rect(x, y, w, h, radii, false, false)),
                }
            }
            RShape::Circle { c, r } => {
                let rr = r + p.r + p.grow;
                if form == Form::Native { Geo::Ellipse { c: *c, rx: rr, ry: rr } } else { Geo::Path(geom::ellipse(*c, rr, rr)) }
            }
            RShape::Ellipse { c, rx, ry } => {
                let (a, b) = (rx + p.r + p.grow, ry + p.r + p.grow);
                if form == Form::Native { Geo::Ellipse { c: *c, rx: a, ry: b } } else { Geo::Path(geom::ellipse(*c, a, b)) }
            }
            RShape::Arc { c, r, from, to, closed } => {
                Geo::Path(geom::arc(*c, r + p.r + p.grow, *from, *to, *closed && form != Form::Open, geom::arc_segments(to - from)))
            }
            RShape::Polygon(vs) => {
                if form == Form::Open {
                    let pts: Vec<Pt> = vs.iter().map(|v| v.0).collect();
                    let r = vs.iter().map(|v| v.1).fold(0.0, f64::max);
                    Geo::Path(geom::polyline(&pts, r))
                } else {
                    let vertices: Vec<PolyVertex> = vs.iter().map(|(p, r, curve)| PolyVertex { p: *p, r: *r, curve: *curve }).collect();
                    Geo::Path(geom::polygon(&vertices))
                }
            }
            RShape::Wave { x0, x1, y, cycles, amp, phase } => Geo::Path(geom::wave(*x0, *x1, *y, *cycles, amp + p.amp, *phase)),
            RShape::Path(c) => {
                if form == Form::Open { Geo::Path(Contour { closed: false, ..c.clone() }) } else { Geo::Path(c.clone()) }
            }
            RShape::Morph { a, .. } => a.geo(p, if form == Form::Native { Form::Path } else { form }),
            RShape::Mirrored { inner, axis, vertical } => inner.geo(p, form).mirror(*axis, *vertical),
        }
    }

    /// A morph's geometry with its driver at `k` (0 = the first shape, 1 = the second).
    pub fn morph_at(&self, k: f64, form: Form) -> Option<Geo> {
        match self {
            RShape::Morph { a, b, .. } => {
                let form = if form == Form::Native { Form::Path } else { form };
                let (ca, cb) = morph_pair(a, b, form)?;
                let pa = ca.flat_points();
                let pb = cb.flat_points();
                let mixed: Vec<f64> = pa.iter().zip(&pb).map(|(x, y)| x + (y - x) * k).collect();
                Some(Geo::Path(ca.with_points(&mixed)))
            }
            RShape::Mirrored { inner, axis, vertical } => inner.morph_at(k, form).map(|g| g.mirror(*axis, *vertical)),
            _ => None,
        }
    }

    pub fn driver(&self) -> Option<&str> {
        match self {
            RShape::Morph { by, .. } => Some(by),
            RShape::Mirrored { inner, .. } => inner.driver(),
            _ => None,
        }
    }

    /// Whether a keyed `r`, `grow` or `amp` changes this shape (exactly: the points are
    /// linear in each of them).
    pub fn takes(&self, prop: Prop) -> bool {
        match self {
            RShape::Mirrored { inner, .. } => inner.takes(prop),
            RShape::Rect { .. } | RShape::Circle { .. } | RShape::Ellipse { .. } | RShape::Arc { .. } => matches!(prop, Prop::R | Prop::Grow),
            RShape::Wave { .. } => prop == Prop::Amp,
            _ => false,
        }
    }

    fn closed(&self) -> bool {
        self.geo(Params::default(), Form::Native).closed()
    }
}

/// The two shapes of a morph as contours with one topology.
fn morph_pair(a: &RShape, b: &RShape, form: Form) -> Option<(Contour, Contour)> {
    let (ca, cb) = match (a, b) {
        (RShape::Arc { c: c1, r: r1, from: f1, to: t1, closed }, RShape::Arc { c: c2, r: r2, from: f2, to: t2, .. }) => {
            let n = geom::arc_segments(t1 - f1).max(geom::arc_segments(t2 - f2));
            let closed = *closed && form != Form::Open;
            (geom::arc(*c1, *r1, *f1, *t1, closed, n), geom::arc(*c2, *r2, *f2, *t2, closed, n))
        }
        _ => (a.geo(Params::default(), form).contour(), b.geo(Params::default(), form).contour()),
    };
    (ca.commands() == cb.commands()).then_some((ca, cb))
}

#[derive(Debug, Clone, PartialEq)]
pub enum SolidR {
    Auto,
    Ring,
    Fill,
    None,
    Grow(f64),
    Shape(RShape),
}

/// A part with its options settled (mirrors inherit their source's) and shapes resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub name: String,
    pub path: String,
    pub role: Role,
    pub outline: Outline,
    pub tint: bool,
    pub solid: SolidR,
    pub holes: Vec<RShape>,
    pub knockouts: Vec<(String, f64)>,
    pub gaps: Vec<(String, f64)>,
    pub hidden: bool,
    /// Rest opacity when not hidden (1 unless the spec sets `opacity`).
    pub opacity: f64,
    /// Light and Bold outline shapes of their own (`weights`).
    pub weights: Vec<(String, Vec<RShape>)>,
    pub shapes: Vec<RShape>,
}

impl Part {
    pub fn bbox(&self) -> BBox {
        self.shapes.iter().map(|s| s.geo(Params::default(), Form::Native).bbox()).reduce(BBox::union).unwrap_or(BBox::point(Pt::new(16.0, 16.0)))
    }
}

struct Resolver<'a> {
    spec: &'a Spec,
    src: IndexMap<&'a str, &'a PartSrc>,
    cache: HashMap<String, Option<Part>>,
    visiting: Vec<String>,
    d: Diagnostics,
    /// `(owner part, referenced part, pointer)` for points on another part's derived anchor.
    refs: Vec<(String, String, String)>,
}

impl<'a> Resolver<'a> {
    fn new(spec: &'a Spec) -> Self {
        Resolver { spec, src: spec.parts.iter().map(|p| (p.name.as_str(), p)).collect(), cache: HashMap::new(), visiting: Vec::new(), d: Diagnostics::default(), refs: Vec::new() }
    }

    fn names(&self) -> Vec<&str> {
        self.src.keys().copied().chain(self.spec.anchors.keys().map(String::as_str)).collect()
    }

    fn part(&mut self, name: &str) -> Option<Part> {
        if let Some(done) = self.cache.get(name) {
            return done.clone();
        }
        let src = *self.src.get(name)?;
        if self.visiting.iter().any(|v| v == name) {
            self.d.push(Diagnostic::error("ANCHOR_CYCLE", src.path.clone(), format!("{name} is placed relative to itself through {}", self.visiting.join(" → "))));
            return None;
        }
        self.visiting.push(name.to_string());
        let part = self.build(src);
        self.visiting.pop();
        self.cache.insert(name.to_string(), part.clone());
        part
    }

    fn build(&mut self, src: &PartSrc) -> Option<Part> {
        let name = src.name.as_str();
        let (shapes, base): (Vec<RShape>, Option<(Part, &PartSrc)>) = match &src.body {
            PartBody::Shapes(list) => {
                let single = list.len() == 1;
                let mut out = Vec::new();
                for (i, s) in list.iter().enumerate() {
                    let path = if single { at(&src.path, "shape") } else { at(&at(&src.path, "shape"), i) };
                    out.push(self.shape(s, name, &path)?);
                }
                (out, None)
            }
            PartBody::Mirror { of, x, y } => {
                let path = at(&at(&src.path, "mirror"), "of");
                if of == name {
                    self.d.push(Diagnostic::error("UNKNOWN_NAME", path, "a part cannot mirror itself"));
                    return None;
                }
                let Some(base_src) = self.src.get(of.as_str()).copied() else {
                    self.unknown(&path, of, "part");
                    return None;
                };
                let base = self.part(of)?;
                let (axis, vertical) = match (x, y) {
                    (_, Some(y)) => (*y, false),
                    (Some(x), None) => (*x, true),
                    (None, None) => (16.0, true),
                };
                let m = |s: &RShape| RShape::Mirrored { inner: Box::new(s.clone()), axis, vertical };
                let shapes = base.shapes.iter().map(m).collect();
                let mirrored = Part {
                    holes: base.holes.iter().map(m).collect(),
                    weights: base.weights.iter().map(|(w, list)| (w.clone(), list.iter().map(m).collect())).collect(),
                    solid: match &base.solid {
                        SolidR::Shape(s) => SolidR::Shape(m(s)),
                        other => other.clone(),
                    },
                    ..base.clone()
                };
                (shapes, Some((mirrored, base_src)))
            }
        };
        let inherit = base.as_ref().map(|(p, _)| p);
        let role = src.role.or(inherit.map(|p| p.role)).unwrap_or(Role::Stroke);
        let outline = src.outline.or(inherit.map(|p| p.outline)).unwrap_or(Outline::Closed);
        let tint = src.tint.or(inherit.map(|p| p.tint)).unwrap_or(role == Role::Surface);
        let solid = match &src.solid {
            Some(Solid::Auto) => SolidR::Auto,
            Some(Solid::Ring) => SolidR::Ring,
            Some(Solid::Fill) => SolidR::Fill,
            Some(Solid::None) => SolidR::None,
            Some(Solid::Grow(d)) => SolidR::Grow(*d),
            Some(Solid::Shape(s)) => SolidR::Shape(self.shape(s, name, &at(&src.path, "solid"))?),
            None => inherit.map_or(SolidR::Auto, |p| p.solid.clone()),
        };
        let holes = match &src.holes {
            Some(list) => {
                let mut out = Vec::new();
                for (i, s) in list.iter().enumerate() {
                    out.push(self.shape(s, name, &at(&at(&src.path, "holes"), i))?);
                }
                out
            }
            None => inherit.map(|p| p.holes.clone()).unwrap_or_default(),
        };
        let default_gap = src.knockout_gap.unwrap_or(KNOCKOUT_GAP);
        let knockouts = match &src.knockout {
            Some(list) => list.iter().map(|Knockout { part, gap }| (part.clone(), gap.unwrap_or(default_gap))).collect(),
            None => inherit.map(|p| p.knockouts.clone()).unwrap_or_default(),
        };
        let gaps = match &src.gaps {
            Some(g) => g.clone(),
            None => inherit.map(|p| p.gaps.clone()).unwrap_or_default(),
        };
        let hidden = src.hidden.or(inherit.map(|p| p.hidden)).unwrap_or(false);
        let opacity = src.opacity.or(inherit.map(|p| p.opacity)).unwrap_or(1.0);
        let weights = match &src.weights {
            Some(list) => {
                let mut out = Vec::new();
                for (weight, shapes) in list {
                    let mut resolved = Vec::new();
                    for (i, s) in shapes.iter().enumerate() {
                        resolved.push(self.shape(s, name, &at(&at(&at(&src.path, "weights"), weight), i))?);
                    }
                    out.push((weight.clone(), resolved));
                }
                out
            }
            None => inherit.map(|p| p.weights.clone()).unwrap_or_default(),
        };
        if !weights.is_empty() && outline == Outline::Open {
            self.d.push(Diagnostic::error("NOT_IMPLEMENTED", at(&src.path, "weights"), "weights on an open outline are not implemented").hint("close the outline, or drop weights"));
        }
        // A stroke with no outline draws only in Solid (its centerline at 2.6).
        if (outline == Outline::Open && role != Role::Surface) || (outline == Outline::None && role == Role::Fill) {
            self.d.push(Diagnostic::error("V2_SCHEMA", at(&src.path, "outline"), "outline \"open\" is for surfaces; \"none\" for surfaces and strokes").hint("set \"role\": \"surface\", or drop outline"));
        }
        if role == Role::Surface && !shapes.iter().all(RShape::closed) && outline != Outline::None {
            self.d.push(
                Diagnostic::error("OPEN_SURFACE", src.path.clone(), "a surface needs closed shapes: its tint and Solid are filled")
                    .hint("close the shape (arc \"closed\": true, path ending in Z), or use role stroke"),
            );
        }
        if outline == Outline::Open && shapes.iter().any(|s| matches!(s, RShape::Circle { .. } | RShape::Ellipse { .. })) {
            self.d.push(Diagnostic::error("V2_SCHEMA", at(&src.path, "outline"), "a circle or ellipse has no closing edge to leave out"));
        }
        Some(Part { name: name.to_string(), path: src.path.clone(), role, outline, tint, solid, holes, knockouts, gaps, hidden, opacity, weights, shapes })
    }

    fn unknown(&mut self, path: &str, name: &str, what: &str) {
        let names = self.names();
        let hint = did_you_mean(name, &names).map_or_else(|| format!("known names: {}", names.join(", ")), |c| format!("did you mean `{c}`?"));
        self.d.push(Diagnostic::error("UNKNOWN_NAME", path, format!("no {what} named `{name}`")).hint(hint));
    }

    fn point(&mut self, p: &Point, owner: &str, path: &str) -> Option<Pt> {
        match p {
            Point::Xy(x, y) => Some(Pt::new(*x, *y)),
            Point::Polar { from, angle, len } => Some(self.point(from, owner, path)?.polar(*angle, *len)),
            Point::Anchor(s) => {
                if let Some((x, y)) = self.spec.anchors.get(s) {
                    return Some(Pt::new(*x, *y));
                }
                let Some((name, suffix)) = s.split_once('.') else {
                    let hint = if self.src.contains_key(s.as_str()) { format!("a part's point is derived: write `{s}.c`, `{s}.top`, `{s}.end`, …") } else { String::new() };
                    self.unknown(path, s, "anchor");
                    if !hint.is_empty()
                        && let Some(last) = self.d.0.last_mut()
                    {
                        last.hint = Some(hint);
                    }
                    return None;
                };
                if name == owner {
                    self.d.push(Diagnostic::error("ANCHOR_CYCLE", path, format!("{owner} cannot be placed relative to its own anchor `{s}`")));
                    return None;
                }
                if !self.src.contains_key(name) {
                    self.unknown(path, name, "part");
                    return None;
                }
                let part = self.part(name)?;
                self.refs.push((owner.to_string(), name.to_string(), path.to_string()));
                let b = part.bbox();
                let c = b.center();
                Some(match suffix {
                    "c" => c,
                    "top" => Pt::new(c.x, b.y0),
                    "bottom" => Pt::new(c.x, b.y1),
                    "left" => Pt::new(b.x0, c.y),
                    "right" => Pt::new(b.x1, c.y),
                    "tl" => Pt::new(b.x0, b.y0),
                    "tr" => Pt::new(b.x1, b.y0),
                    "bl" => Pt::new(b.x0, b.y1),
                    "br" => Pt::new(b.x1, b.y1),
                    "start" | "end" => {
                        let first = part.shapes.first()?.geo(Params::default(), Form::Native).contour();
                        let last = part.shapes.last()?.geo(Params::default(), Form::Native).contour();
                        if first.closed {
                            self.d.push(Diagnostic::error("UNKNOWN_NAME", path, format!("`{s}`: {name} is closed, so it has no {suffix}")).hint("use .c, .top, .left, … for a closed shape"));
                            return None;
                        }
                        if suffix == "start" { first.start } else { last.end() }
                    }
                    _ => return None,
                })
            }
        }
    }

    fn shape(&mut self, s: &Shape, owner: &str, path: &str) -> Option<RShape> {
        let k = |key: &str| at(path, key);
        Some(match s {
            Shape::Line([a, b]) => RShape::Line(self.point(a, owner, &at(&k("line"), 0))?, self.point(b, owner, &at(&k("line"), 1))?),
            Shape::Polyline { points, r } => {
                let mut pts = Vec::new();
                for (i, p) in points.iter().enumerate() {
                    pts.push(self.point(p, owner, &at(&k("polyline"), i))?);
                }
                RShape::Polyline(pts, *r)
            }
            Shape::Rect { x, y, w, h, r } => RShape::Rect { x: *x, y: *y, w: *w, h: *h, r: r.corners() },
            Shape::Circle { c, r } => RShape::Circle { c: self.point(c, owner, &at(&k("circle"), 0))?, r: *r },
            Shape::Ellipse { c, rx, ry } => RShape::Ellipse { c: self.point(c, owner, &at(&k("ellipse"), 0))?, rx: *rx, ry: *ry },
            Shape::Arc { c, r, from, to, closed } => RShape::Arc { c: self.point(c, owner, &at(&k("arc"), 0))?, r: *r, from: *from, to: *to, closed: *closed },
            Shape::Polygon { vertices, r } => {
                let mut out = Vec::new();
                for (i, v) in vertices.iter().enumerate() {
                    let vp = at(&k("polygon"), i);
                    let p = self.point(&v.p, owner, &vp)?;
                    let curve = match &v.curve {
                        Some([c1, c2]) => Some([self.point(c1, owner, &vp)?, self.point(c2, owner, &vp)?]),
                        None => None,
                    };
                    out.push((p, v.r.unwrap_or(*r), curve));
                }
                RShape::Polygon(out)
            }
            Shape::Wave { x, y, cycles, amp, phase } => RShape::Wave { x0: x[0], x1: x[1], y: *y, cycles: *cycles, amp: *amp, phase: *phase },
            Shape::Path(d) => match geom::svg_path(d) {
                Ok(c) => RShape::Path(c),
                Err(e) => {
                    self.d.push(Diagnostic::error("V2_SCHEMA", k("path"), e));
                    return None;
                }
            },
            Shape::Minus(_) => {
                self.d.push(
                    Diagnostic::error("NOT_IMPLEMENTED", k("minus"), "minus (booleans) is not implemented in this prototype")
                        .hint("draw the result as one path, or use holes (subtract masks)"),
                );
                return None;
            }
            Shape::Morph { a, b, by } => {
                let ra = self.shape(a, owner, &at(&k("morph"), 0))?;
                let rb = self.shape(b, owner, &at(&k("morph"), 1))?;
                if morph_pair(&ra, &rb, Form::Path).is_none() {
                    self.d.push(
                        Diagnostic::error("MORPH_TOPOLOGY", k("morph"), format!("the two {} shapes compile to different command lists", a.kind()))
                            .hint("give both the same number of points, corners and rounded corners"),
                    );
                    return None;
                }
                RShape::Morph { a: Box::new(ra), b: Box::new(rb), by: by.clone() }
            }
        })
    }
}

/// A transform node of the spec.
#[derive(Debug, Clone, PartialEq)]
struct RigNode {
    name: String,
    path: String,
    pivot: Option<PivotR>,
    parent: Option<String>,
    squash: Option<Squash>,
    /// Made for a part keyed by a rig property, with no rig of its own declared.
    implicit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PivotR {
    Point(Pt),
    Contact(Pt, Pt),
}

impl RigNode {
    /// The plan rig carried parts and child rigs hang from.
    fn inner(&self) -> String {
        match self.pivot {
            Some(PivotR::Contact(..)) => format!("rig-{}-a", self.name),
            _ => format!("rig-{}", self.name),
        }
    }
    /// The plan rig that takes translation and the positive rotation.
    fn outer(&self) -> String {
        match self.pivot {
            Some(PivotR::Contact(..)) => format!("rig-{}-b", self.name),
            _ => format!("rig-{}", self.name),
        }
    }
    fn pivot_point(&self) -> Pt {
        match self.pivot {
            Some(PivotR::Point(p)) => p,
            Some(PivotR::Contact(_, b)) => b,
            None => Pt::new(16.0, 16.0),
        }
    }
}

/// Where a geometry comes from, for path keys.
#[derive(Debug, Clone, PartialEq)]
struct GeoDef {
    part: String,
    shape: RShape,
    form: Form,
    hole: bool,
}

/// One layer's drawing before naming.
struct Drawing {
    geos: Vec<String>,
    paints: Vec<Paint>,
}

/// Everything a compile produces.
#[derive(Debug, Clone)]
pub struct Compiled {
    pub spec: Spec,
    pub parts: Vec<Part>,
    pub plan: Plan,
    /// Light, Strong, then the reverses, in plan order.
    pub motions: Vec<Motion>,
    pub duotone_fill: bool,
    /// Warnings and notes; errors fail the compile.
    pub diagnostics: Diagnostics,
    /// Draft sheet rows: `{motion, look, times}`.
    pub sheet_rows: Vec<Value>,
    /// Part name → its plan rig chain, innermost first.
    pub chains: HashMap<String, Vec<String>>,
}

impl Compiled {
    pub fn stateful(&self) -> bool {
        self.spec.state.is_some()
    }

    /// The draft sheet: 8 columns, 4 px gaps, at 4× and real 24 px size.
    pub fn sheet(&self) -> Value {
        json!({"cols": 8, "gap": 4, "scales": [4, 0.75], "rows": self.sheet_rows})
    }
}

/// Compiles a parsed spec; errors come back as diagnostics with JSON Pointers.
pub fn compile(spec: &Spec) -> Result<Compiled, Diagnostics> {
    let mut d = Diagnostics::default();
    let mut resolver = Resolver::new(spec);
    let parts: Vec<Part> = spec.parts.iter().filter_map(|p| resolver.part(&p.name)).collect();
    let refs = std::mem::take(&mut resolver.refs);
    d.extend(std::mem::take(&mut resolver.d));
    if d.has_errors() || parts.len() != spec.parts.len() {
        return Err(d);
    }
    let part_index: HashMap<&str, usize> = parts.iter().enumerate().map(|(i, p)| (p.name.as_str(), i)).collect();
    let part_names: Vec<&str> = parts.iter().map(|p| p.name.as_str()).collect();
    let unknown = |d: &mut Diagnostics, path: String, name: &str, what: &str, names: &[&str]| {
        let hint = did_you_mean(name, names).map_or_else(|| format!("known: {}", names.join(", ")), |c| format!("did you mean `{c}`?"));
        d.push(Diagnostic::error("UNKNOWN_NAME", path, format!("no {what} named `{name}`")).hint(hint));
    };
    for p in &parts {
        for (i, (k, _)) in p.knockouts.iter().enumerate() {
            if !part_index.contains_key(k.as_str()) || k == &p.name {
                unknown(&mut d, at(&at(&p.path, "knockout"), i), k, "other part", &part_names);
            }
        }
        for (other, _) in &p.gaps {
            if !part_index.contains_key(other.as_str()) || other == &p.name {
                unknown(&mut d, at(&at(&p.path, "gap"), other), other, "other part", &part_names);
            }
        }
    }

    // Morph drivers.
    let mut drivers: IndexSet<String> = IndexSet::new();
    for p in &parts {
        for s in &p.shapes {
            if let Some(by) = s.driver() {
                drivers.insert(by.to_string());
            }
        }
    }

    // Every motion target, for implicit rigs and name checks.
    let mut targets: Vec<(Target, String)> = Vec::new();
    for e in &spec.motion {
        targets.push((e.target.clone(), e.path.clone()));
        let follows = match &e.track {
            TrackSrc::Follow(f) => vec![f.clone()],
            TrackSrc::PerMotion { light, strong } => [light, strong]
                .into_iter()
                .flatten()
                .filter_map(|(v, _)| match v {
                    crate::spec::MotionValue::Follow(f) => Some(f.clone()),
                    _ => None,
                })
                .collect(),
        };
        for f in follows {
            targets.push((f.target.clone(), at(&e.path, "follow")));
        }
    }
    for o in spec.reverse_light.iter().chain(&spec.reverse_strong) {
        targets.push((o.target.clone(), o.path.clone()));
    }

    // Rigs.
    let mut rigs: IndexMap<String, RigNode> = IndexMap::new();
    let point = |d: &mut Diagnostics, p: &Point, path: &str| -> Option<Pt> {
        let mut r = Resolver::new(spec);
        let out = r.point(p, "", path);
        d.extend(r.d);
        out
    };
    for r in &spec.rigs {
        let pivot = match &r.pivot {
            None => None,
            Some(Pivot::Point(p)) => match point(&mut d, p, &at(&r.path, "pivot")) {
                Some(p) => Some(PivotR::Point(p)),
                None => return Err(d),
            },
            Some(Pivot::Contact(a, b)) => {
                let cp = at(&at(&r.path, "pivot"), "contact");
                let a = point(&mut d, a, &at(&cp, 0));
                let b = point(&mut d, b, &at(&cp, 1));
                match (a, b) {
                    (Some(a), Some(b)) => Some(PivotR::Contact(a, b)),
                    _ => return Err(d),
                }
            }
        };
        if r.swing || r.flip {
            let kind = if r.swing { "swing" } else { "flip" };
            d.push(
                Diagnostic::error("NOT_IMPLEMENTED", at(&r.path, kind), format!("{kind} (a deformer fitted to path keys) is not implemented in this prototype"))
                    .hint("rotate or move the door with a pivot rig for now; swing, flip and occlusion need the sampling fitter"),
            );
        }
        rigs.insert(r.name.clone(), RigNode { name: r.name.clone(), path: r.path.clone(), pivot, parent: None, squash: r.squash, implicit: false });
    }
    let mut part_rig: HashMap<String, String> = HashMap::new();
    for r in &spec.rigs {
        for (i, name) in r.carries.iter().enumerate() {
            let path = at(&at(&r.path, "carries"), i);
            if name == &r.name {
                // A rig named like a part moves that part; listing it again is harmless.
                if !part_index.contains_key(name.as_str()) {
                    d.push(Diagnostic::error("RIG_CYCLE", path, format!("rig {} carries itself", r.name)));
                }
                continue;
            }
            if rigs.contains_key(name) {
                let child = rigs.get_mut(name).expect("rig");
                if let Some(other) = &child.parent {
                    d.push(Diagnostic::error("CARRIED_TWICE", path, format!("{name} is carried by both {other} and {}", r.name)));
                } else {
                    child.parent = Some(r.name.clone());
                }
            } else if part_index.contains_key(name.as_str()) {
                if let Some(other) = part_rig.insert(name.clone(), r.name.clone()) {
                    d.push(Diagnostic::error("CARRIED_TWICE", path, format!("{name} is carried by both {other} and {}", r.name)));
                }
            } else {
                let names: Vec<&str> = part_names.iter().copied().chain(rigs.keys().map(String::as_str)).collect();
                unknown(&mut d, path, name, "part or rig", &names);
            }
        }
    }
    // A rig named like a part moves it; implicit rigs for parts keyed by rig properties.
    let rig_props = [Prop::X, Prop::Y, Prop::Rotate, Prop::Scale, Prop::Squash, Prop::Swing, Prop::Flip];
    for (t, path) in &targets {
        if t.prop.is_some_and(|p| rig_props.contains(&p)) && !rigs.contains_key(&t.name) && part_index.contains_key(t.name.as_str()) {
            let center = parts[part_index[t.name.as_str()]].bbox().center();
            rigs.insert(t.name.clone(), RigNode { name: t.name.clone(), path: path.clone(), pivot: Some(PivotR::Point(center)), parent: None, squash: None, implicit: true });
        }
    }
    let rig_names: Vec<String> = rigs.keys().cloned().collect();
    for name in &rig_names {
        if part_index.contains_key(name.as_str())
            && let Some(carrier) = part_rig.insert(name.clone(), name.clone())
        {
            let node = rigs.get_mut(name).expect("rig");
            if node.parent.is_none() {
                node.parent = Some(carrier);
            }
        }
    }
    // Cycles, then parent-first order.
    let mut order: Vec<String> = Vec::new();
    for name in &rig_names {
        let mut chain = vec![name.clone()];
        let mut cur = rigs[name].parent.clone();
        while let Some(p) = cur {
            if chain.contains(&p) {
                d.push(Diagnostic::error("RIG_CYCLE", rigs[name].path.clone(), format!("rigs carry each other: {} → {p}", chain.join(" → "))));
                return Err(d);
            }
            chain.push(p.clone());
            cur = rigs[&p].parent.clone();
        }
        for r in chain.into_iter().rev() {
            if !order.contains(&r) {
                order.push(r);
            }
        }
    }
    let chain_of = |rig: &str| -> Vec<String> {
        let mut out = Vec::new();
        let mut cur = Some(rig.to_string());
        while let Some(r) = cur {
            cur = rigs[&r].parent.clone();
            out.push(r);
        }
        out
    };
    let children = |rig: &str| rigs.values().any(|r| r.parent.as_deref() == Some(rig));

    // Name and property checks on every target.
    for (t, path) in &targets {
        let is_rig = rigs.contains_key(&t.name);
        let is_part = part_index.contains_key(t.name.as_str());
        let known: Vec<&str> = part_names.iter().copied().chain(rigs.keys().map(String::as_str)).chain(drivers.iter().map(String::as_str)).collect();
        match t.prop {
            None => {
                if !drivers.contains(&t.name) {
                    let hint = if is_rig || is_part { format!("add a property, e.g. `{}.rotate` or `{}.opacity`", t.name, t.name) } else { "a bare name keys a morph's driver (its `by`)".into() };
                    d.push(Diagnostic::error("UNKNOWN_NAME", path.clone(), format!("no morph is driven by `{}`", t.name)).hint(hint));
                }
            }
            Some(prop) if !is_rig && !is_part => {
                let _ = prop;
                unknown(&mut d, path.clone(), &t.name, "part or rig", &known);
            }
            Some(Prop::Swing | Prop::Flip) => {
                d.push(Diagnostic::error("NOT_IMPLEMENTED", path.clone(), "swing and flip are not implemented in this prototype").hint("rotate about a pivot instead"));
            }
            Some(Prop::Phase) => {
                d.push(Diagnostic::error("NOT_IMPLEMENTED", path.clone(), "a keyed wave phase is not implemented in this prototype (it is not linear, so it needs fitting)").hint("key amp, or move the wave with x"));
            }
            Some(Prop::Rotate) => {
                // A turn about a guessed centre is the classic wrong pivot: name it.
                if rigs[&t.name].pivot.is_none() || rigs[&t.name].implicit {
                    d.push(Diagnostic::error("PIVOT", path.clone(), format!("{} turns but has no pivot", t.name)).hint(format!("add \"pivot\": [x, y] (or an anchor) to rigs.{}", t.name)));
                }
            }
            Some(Prop::Scale) => {
                if rigs[&t.name].pivot.is_none() {
                    d.push(Diagnostic::error("PIVOT", path.clone(), format!("{} scales but has no pivot", t.name)).hint(format!("add \"pivot\" to rigs.{}", t.name)));
                } else if children(&t.name) {
                    d.push(Diagnostic::error("NOT_IMPLEMENTED", path.clone(), "scale on a rig that carries other rigs is not implemented in this prototype").hint("scale a rig that carries parts only"));
                }
            }
            Some(Prop::Squash) => {
                if rigs[&t.name].squash.is_none() {
                    d.push(Diagnostic::error("UNKNOWN_NAME", path.clone(), format!("{} has no squash", t.name)).hint(format!("add \"squash\": {{\"base\": y}} to rigs.{}", t.name)));
                } else if children(&t.name) {
                    d.push(Diagnostic::error("NOT_IMPLEMENTED", path.clone(), "squash on a rig that carries other rigs is not implemented in this prototype"));
                }
            }
            Some(Prop::Draw | Prop::R | Prop::Grow | Prop::Amp) if !is_part => {
                d.push(Diagnostic::error("UNKNOWN_NAME", path.clone(), format!("{} is a rig; .{} keys a part", t.name, t.prop.map_or("", Prop::as_str))));
            }
            Some(prop @ (Prop::R | Prop::Grow | Prop::Amp)) => {
                let part = &parts[part_index[t.name.as_str()]];
                if !part.shapes.iter().any(|s| s.takes(prop)) {
                    d.push(
                        Diagnostic::error("PARAM_UNSUPPORTED", path.clone(), format!("no shape of {} takes {}", t.name, prop.as_str()))
                            .hint("r and grow key circles, ellipses, arcs and rects; amp keys a wave"),
                    );
                }
            }
            _ => {}
        }
    }
    if d.has_errors() {
        return Err(d);
    }

    // Parts' rigs and live anchors.
    let plan_rig_of = |part: &str| part_rig.get(part).map(|r| rigs[r].inner());
    for (owner, other, path) in &refs {
        if !owner.is_empty() && plan_rig_of(owner) != plan_rig_of(other) {
            d.push(
                Diagnostic::warning("LIVE_ANCHOR", path.clone(), format!("{owner} uses an anchor of {other}, which moves on another rig; it is resolved at rest and will not follow"))
                    .hint("connectors are not implemented in this prototype: carry both parts on one rig"),
            );
        }
    }

    // Motions.
    let pulsed: HashSet<String> = spec
        .motion
        .iter()
        .filter(|e| match &e.track {
            TrackSrc::PerMotion { light, strong } => [light, strong].iter().any(|v| matches!(v, Some((crate::spec::MotionValue::Pulse { .. }, _)))),
            TrackSrc::Follow(_) => false,
        })
        .map(|e| e.target.name.clone())
        .collect();
    let hidden = |name: &str| part_index.get(name).is_some_and(|&i| parts[i].hidden) || pulsed.contains(name);
    // A part's rest opacity: 0 when hidden, else its `opacity` (1 by default); a rig's is 1.
    let rest_opacity = |name: &str| match hidden(name) {
        true => 0.0,
        false => part_index.get(name).map_or(1.0, |&i| parts[i].opacity),
    };
    let mut motions: Vec<Motion> = Vec::new();
    for which in [Which::Light, Which::Strong] {
        let errors_before = d.errors().count();
        let mut raw = motion::resolve(spec, which, &rest_opacity, &mut d);
        let name = match which {
            Which::Light => spec.id.clone(),
            Which::Strong => format!("{}-strong", spec.id),
        };
        let timing = match which {
            Which::Light => spec.timing.light,
            Which::Strong => spec.timing.strong,
        };
        // A gesture's turn that ends a whole turn (or a `cycle`) from rest draws the same
        // pose, but rasterises a little differently, and the players' checks compare the rest
        // frame with the first pixel for pixel: step back to rest, invisibly, a frame later.
        if spec.state.is_none() {
            for t in raw.iter_mut().filter(|t| t.target.prop == Some(Prop::Rotate)) {
                let Some(&last) = t.keys.last() else { continue };
                let turn = |period: f64| (last.v / period - (last.v / period).round()).abs() < 1e-9;
                if last.v.abs() < 1e-9 || !(turn(360.0) || spec.cycle.is_some_and(turn)) {
                    continue;
                }
                let step = last.t + 1.0 / 60.0;
                if timing.is_some_and(|end| step > end + 1e-9) {
                    d.push(
                        Diagnostic::error("MOVES_AFTER_REST", t.path.clone(), format!("{} ends a whole turn at {} s; its step back to rest comes a frame later, after {name}'s action ends", t.target, motion::r4(last.t)))
                            .hint(format!("end the turn by {} s, or lengthen timing", motion::r4(timing.unwrap_or(step) - 1.0 / 60.0))),
                    );
                    continue;
                }
                t.keys.push(ease::Key::new(step, 0.0, Ease::Step));
            }
        }
        let tracks = motion::compile_tracks(&raw);
        if tracks.is_empty() {
            if d.errors().count() == errors_before {
                d.push(Diagnostic::error("NO_MOTION", "/motion", format!("{} has no keys", which.id())).hint(format!("every icon has Light and Strong: add \"{}\" keys to a track", which.id())));
            }
            continue;
        }
        let active = timing.unwrap_or_else(|| motion::last_key(&tracks));
        motion::check_timing(&name, active, &tracks, &at("/timing", which.id()), &mut d);
        // Rest values: offsets start at 0, factors at 1; a gesture also ends there.
        for t in &raw {
            let rest = motion::rest_value(t.target.prop, rest_opacity(&t.target.name));
            let (Some(first), Some(last)) = (t.keys.first(), t.keys.last()) else { continue };
            let starts = !matches!(t.target.prop, Some(Prop::Opacity | Prop::Draw));
            let what = match t.target.prop {
                Some(Prop::Scale | Prop::Squash) => " (a factor: rest is 1)".to_string(),
                Some(Prop::Opacity) if rest == 0.0 => " (the part is hidden at rest)".to_string(),
                Some(Prop::Opacity) if rest < 1.0 => format!(" (the part rests at opacity {rest})"),
                Some(Prop::Opacity | Prop::Draw) => " (absolute: rest is 1)".to_string(),
                _ => " (an offset from the drawn pose: rest is 0)".to_string(),
            };
            if starts && (first.v - rest).abs() > 1e-9 {
                d.push(Diagnostic::error("REST_VALUE", t.path.clone(), format!("{} starts at {}, not its rest value {}{what}", t.target, first.v, rest)).hint("the drawing is the rest pose; key offsets from it"));
            }
            let turn = |v: f64, period: f64| ((v - rest) / period - ((v - rest) / period).round()).abs() < 1e-9;
            let same_pose = t.target.prop == Some(Prop::Rotate) && (turn(last.v, 360.0) || spec.cycle.is_some_and(|c| turn(last.v, c)));
            if spec.state.is_none() && (last.v - rest).abs() > 1e-9 && !same_pose {
                d.push(
                    Diagnostic::error("REST_VALUE", t.path.clone(), format!("{} ends at {}, not its rest value {}{what}", t.target, last.v, rest))
                        .hint("a gesture ends where it starts; make the icon stateful with \"state\" if it should stay changed"),
                );
            }
            if t.keys.len() >= 2 && (last.v - t.keys[t.keys.len() - 2].v).abs() < 1e-12 && timing.is_none() && (last.t - active).abs() < 1e-9 {
                d.push(Diagnostic::warning("TRAILING_HOLD", t.path.clone(), format!("{}'s last key repeats the value before it and stretches {} to {} s", t.target, which.id(), last.t)).hint("drop the last key, or set timing"));
            }
        }
        motions.push(Motion { name, which, reverse: false, active, rest: motion::rest_of(active), raw, tracks });
    }
    if d.has_errors() {
        return Err(d);
    }

    // Parts each track changes (rig tracks: every part their rig carries).
    let parts_under = |rig: &str| -> Vec<String> { parts.iter().filter(|p| part_rig.get(&p.name).is_some_and(|r| chain_of(r).iter().any(|c| c == rig))).map(|p| p.name.clone()).collect() };
    let changed_parts = |t: &Target| -> Vec<String> {
        match t.prop {
            None => parts.iter().filter(|p| p.shapes.iter().any(|s| s.driver() == Some(t.name.as_str()))).map(|p| p.name.clone()).collect(),
            Some(p) if rig_props.contains(&p) => parts_under(&t.name),
            Some(Prop::Opacity) if !part_index.contains_key(t.name.as_str()) => parts_under(&t.name),
            Some(_) => vec![t.name.clone()],
        }
    };

    // Reverses.
    if let Some(state) = &spec.state {
        let forwards = motions.clone();
        for forward in &forwards {
            let (overrides_src, active) = match forward.which {
                Which::Light => (&spec.reverse_light, spec.timing.reverse),
                Which::Strong => (&spec.reverse_strong, spec.timing.reverse_strong),
            };
            let mut overrides: Vec<Track> = Vec::new();
            for o in overrides_src {
                let rest = motion::rest_value(o.target.prop, rest_opacity(&o.target.name));
                if let Some(keys) = motion::expand(&o.keys, rest, &mut d) {
                    overrides.push(Track { target: o.target.clone(), keys, path: o.path.clone() });
                }
            }
            // A track that follows an overridden one follows the override, as it follows its
            // source forward, unless the reverse overrides it too.
            let follows: Vec<(&Target, &Follow)> = spec
                .motion
                .iter()
                .filter_map(|e| match (&e.track, forward.which) {
                    (TrackSrc::Follow(f), _)
                    | (TrackSrc::PerMotion { light: Some((MotionValue::Follow(f), _)), .. }, Which::Light)
                    | (TrackSrc::PerMotion { strong: Some((MotionValue::Follow(f), _)), .. }, Which::Strong) => Some((&e.target, f)),
                    _ => None,
                })
                .collect();
            loop {
                let derived: Vec<Track> = follows
                    .iter()
                    .filter(|(target, _)| !overrides.iter().any(|o| o.target == **target))
                    .filter_map(|(target, f)| {
                        let source = overrides.iter().find(|o| o.target == f.target)?;
                        let keys = source.keys.iter().map(|k| ease::Key::new(k.t + f.lag, f.scale * k.v + f.offset, k.ease)).collect();
                        Some(Track { target: (*target).clone(), keys, path: source.path.clone() })
                    })
                    .collect();
                if derived.is_empty() {
                    break;
                }
                overrides.extend(derived);
            }
            let mut fading = Vec::new();
            if state.reverse == ReverseMode::Fade {
                // Only parts the action leaves changed fade: a part that moves and comes back
                // (a dip, a pop, a transient shown and hidden) is already at its start.
                let mut seen: IndexSet<String> = IndexSet::new();
                for t in &forward.tracks {
                    if let (Some(first), Some(last)) = (t.keys.first(), t.keys.last())
                        && (first.v - last.v).abs() > 1e-9
                    {
                        seen.extend(changed_parts(&t.target));
                    }
                }
                for name in seen {
                    let target = Target { name: name.clone(), prop: Some(Prop::Opacity) };
                    let rest = motion::rest_value(Some(Prop::Opacity), rest_opacity(&name));
                    let own = forward.tracks.iter().find(|t| t.target == target);
                    let start = own.and_then(|t| t.keys.first()).map_or(rest, |k| k.v);
                    let stop = own.and_then(|t| t.keys.last()).map_or(rest, |k| k.v);
                    fading.push((target, start, stop));
                }
            }
            let reverse = motion::reverse(&spec.id, ReverseInput { mode: state.reverse, forward, overrides, active, fading, name: state.reverse_name.clone() }, &mut d);
            motions.push(reverse);
        }
        if let Some(name) = &state.reverse_name
            && (!name.starts_with(&format!("{}-", spec.id)) || name.ends_with("-strong") || *name == format!("{}-strong", spec.id))
        {
            d.push(
                Diagnostic::error("V2_SCHEMA", "/state/reverseName", format!("state.reverseName `{name}` must start with `{}-` and not end with -strong", spec.id))
                    .hint(format!("name the Light reverse, such as {}-undo; Strong's adds -strong", spec.id)),
            );
        }
        if state.reverse == ReverseMode::Fade && (state.from != "prepared" || state.to != "result" || state.labels != ["Play action".to_string(), "Reset preview".to_string()]) {
            d.push(
                Diagnostic::error("FADE_STATE", "/state", "a fade reverse is the preview reset: states prepared → result, labels [\"Play action\", \"Reset preview\"]")
                    .hint("use those values, or reverse auto or mirror"),
            );
        }
    } else if !spec.reverse_light.is_empty() || !spec.reverse_strong.is_empty() {
        d.push(Diagnostic::error("V2_SCHEMA", "/reverse", "reverse overrides need a stateful icon").hint("add \"state\""));
    }
    if d.has_errors() {
        return Err(d);
    }

    // Which parts are drawn as paths with keys.
    let mut deform_rigs: HashMap<String, Option<Squash>> = HashMap::new();
    let mut param_parts: HashSet<String> = HashSet::new();
    for m in &motions {
        for t in &m.tracks {
            match t.target.prop {
                Some(Prop::Scale) => {
                    deform_rigs.insert(t.target.name.clone(), None);
                }
                Some(Prop::Squash) => {
                    deform_rigs.insert(t.target.name.clone(), rigs[&t.target.name].squash);
                }
                Some(Prop::R | Prop::Grow | Prop::Amp) => {
                    param_parts.insert(t.target.name.clone());
                }
                _ => {}
            }
        }
    }
    let deformed = |part: &str| part_rig.get(part).is_some_and(|r| deform_rigs.contains_key(r));
    let path_form = |part: &str| deformed(part) || param_parts.contains(part);

    // Geometry registry.
    let mut geo_defs: IndexMap<String, GeoDef> = IndexMap::new();
    let mut shape_ids: HashMap<String, Vec<String>> = HashMap::new();
    let mut open_ids: HashMap<String, Vec<String>> = HashMap::new();
    let mut hole_ids: HashMap<String, Vec<String>> = HashMap::new();
    let mut solid_ids: HashMap<String, String> = HashMap::new();
    let mut weight_ids: HashMap<(String, String), Vec<String>> = HashMap::new();
    for p in &parts {
        let form = if path_form(&p.name) { Form::Path } else { Form::Native };
        let ids: Vec<String> = p
            .shapes
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let id = format!("{}/{i}", p.name);
                geo_defs.insert(id.clone(), GeoDef { part: p.name.clone(), shape: s.clone(), form, hole: false });
                id
            })
            .collect();
        shape_ids.insert(p.name.clone(), ids);
        // Light and Bold shapes of their own: static outlines on the part's rig.
        for (weight, shapes) in &p.weights {
            let reshaped = form == Form::Path || p.shapes.iter().any(|s| s.driver().is_some());
            if reshaped {
                d.push(Diagnostic::error("NOT_IMPLEMENTED", at(&p.path, "weights"), format!("{} has weights and is reshaped by motion; per-weight shapes are static", p.name)).hint("move it with its rig (x, y, rotate), or drop weights"));
                continue;
            }
            let ids: Vec<String> = shapes
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let id = format!("{}/{weight}/{i}", p.name);
                    geo_defs.insert(id.clone(), GeoDef { part: p.name.clone(), shape: s.clone(), form: Form::Native, hole: false });
                    id
                })
                .collect();
            weight_ids.insert((p.name.clone(), weight.clone()), ids);
        }
        if p.role == Role::Surface && p.outline == Outline::Open {
            let ids = p
                .shapes
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let id = format!("{}/{i}/open", p.name);
                    geo_defs.insert(id.clone(), GeoDef { part: p.name.clone(), shape: s.clone(), form: Form::Open, hole: false });
                    id
                })
                .collect();
            open_ids.insert(p.name.clone(), ids);
        }
        let holes = p
            .holes
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let id = format!("{}/h{i}", p.name);
                let hform = if deformed(&p.name) { Form::Path } else { Form::Native };
                geo_defs.insert(id.clone(), GeoDef { part: p.name.clone(), shape: s.clone(), form: hform, hole: true });
                id
            })
            .collect();
        hole_ids.insert(p.name.clone(), holes);
        if let SolidR::Shape(s) = &p.solid {
            let id = format!("{}/solid", p.name);
            let sform = if deformed(&p.name) { Form::Path } else { Form::Native };
            geo_defs.insert(id.clone(), GeoDef { part: p.name.clone(), shape: s.clone(), form: sform, hole: true });
            solid_ids.insert(p.name.clone(), id);
        }
    }
    let rest_geo = |id: &str| -> Geo {
        let g = &geo_defs[id];
        g.shape.geo(Params::default(), g.form)
    };

    // Solid drawing of each part (before knockouts).
    let solid_drawing = |p: &Part| -> Option<(Drawing, f64, bool)> {
        let shapes = shape_ids[&p.name].clone();
        let (geos, paints) = match (&p.solid, p.role) {
            (SolidR::None, _) => return None,
            (SolidR::Shape(_), _) => (vec![solid_ids[&p.name].clone()], vec![Paint::Fill { even_odd: false }]),
            (SolidR::Auto | SolidR::Ring, Role::Stroke) | (SolidR::Ring, _) => (shapes, vec![Paint::Stroke(SOLID_WIDTH)]),
            (SolidR::Fill, _) | (SolidR::Auto, Role::Fill) => (shapes, vec![Paint::Fill { even_odd: false }]),
            (SolidR::Auto, Role::Surface) => (shapes, vec![Paint::Fill { even_odd: false }, Paint::Stroke(SOLID_WIDTH)]),
            (SolidR::Grow(g), Role::Stroke) => (shapes, vec![Paint::Stroke(SOLID_WIDTH + 2.0 * g.max(-1.3))]),
            (SolidR::Grow(g), _) if *g > 0.0 => (shapes, vec![Paint::Fill { even_odd: false }, Paint::Stroke(2.0 * g)]),
            (SolidR::Grow(_), _) => (shapes, vec![Paint::Fill { even_odd: false }]),
        };
        let half = paints.iter().map(|p| if let Paint::Stroke(w) = p { w / 2.0 } else { 0.0 }).fold(0.0, f64::max);
        let filled = paints.iter().any(|p| matches!(p, Paint::Fill { .. }));
        Some((Drawing { geos, paints }, half, filled))
    };
    if parts.iter().any(|p| matches!(p.solid, SolidR::Grow(g) if g < 0.0) && p.role != Role::Stroke) {
        let p = parts.iter().find(|p| matches!(p.solid, SolidR::Grow(g) if g < 0.0)).expect("found");
        d.push(Diagnostic::error("NOT_IMPLEMENTED", at(&p.path, "solid"), "a negative grow (an inset) is not implemented in this prototype").hint("use \"fill\", or give an explicit solid shape"));
        return Err(d);
    }
    let knocked: HashSet<String> = parts.iter().flat_map(|p| p.knockouts.iter().map(|(k, _)| k.clone())).collect();

    // Layers.
    let mut layers: Vec<Layer> = Vec::new();
    let mut duotone_fill = false;
    let mut opacity_targets: HashSet<String> = HashSet::new();
    let mut draw_parts: HashSet<String> = HashSet::new();
    let mut morph_parts: HashSet<String> = HashSet::new();
    for m in &motions {
        for t in &m.tracks {
            match t.target.prop {
                Some(Prop::Draw) => {
                    draw_parts.insert(t.target.name.clone());
                }
                Some(Prop::Opacity) => {
                    opacity_targets.extend(changed_parts(&t.target));
                }
                None => {
                    morph_parts.extend(changed_parts(&t.target));
                }
                _ => {}
            }
        }
    }
    // A knockout is a static mask only when nothing but its surface's rig moves it: a part
    // that is reshaped (a morph included), drawn on, faded or hidden at rest needs a matte
    // that follows it.
    let dynamic = |k: &str| path_form(k) || morph_parts.contains(k) || draw_parts.contains(k) || opacity_targets.contains(k) || rest_opacity(k) < 1.0;
    for p in &parts {
        let rig = plan_rig_of(&p.name);
        let rest = rest_opacity(&p.name);
        let gap_masks: Vec<Mask> = p
            .gaps
            .iter()
            .flat_map(|(other, dist)| {
                let o = &parts[part_index[other.as_str()]];
                let filled = o.role != Role::Stroke;
                shape_ids[&o.name].iter().flat_map(|id| geom::grown(&rest_geo(id), filled, *dist)).map(Mask::fixed).collect::<Vec<_>>()
            })
            .collect();
        // A hole of a part that scale or squash reshapes follows the hole's path keys.
        let hole_masks: Vec<Mask> = hole_ids[&p.name]
            .iter()
            .map(|id| Mask { contour: Contour { closed: true, ..rest_geo(id).contour() }, geo: deformed(&p.name).then(|| id.clone()) })
            .collect();
        let layer = |variant: &str, style: Option<&'static str>, drawing: Drawing, masks: Vec<Mask>| Layer {
            name: format!("{}-{}-{variant}", spec.id, p.name),
            part: p.name.clone(),
            variant: variant.to_string(),
            style,
            rig: rig.clone(),
            geos: drawing.geos,
            paints: drawing.paints,
            masks,
            vis: Some(format!("{}.opacity", p.name)),
            trim: draw_parts.contains(&p.name).then(|| format!("{}.draw", p.name)),
            opacity: (rest < 1.0).then_some(rest),
            matte: None,
        };
        let shapes = shape_ids[&p.name].clone();
        let holes = hole_ids[&p.name].clone();
        // Tint (back), then the three weights, then Solid (front).
        if p.role == Role::Surface && p.tint {
            duotone_fill = true;
            let masks = [gap_masks.clone(), hole_masks.clone()].concat();
            layers.push(layer("tint", Some("tint"), Drawing { geos: shapes.clone(), paints: vec![Paint::Fill { even_odd: false }] }, masks));
        }
        for (variant, style, width) in WEIGHTS {
            let shapes = weight_ids.get(&(p.name.clone(), variant.to_string())).cloned().unwrap_or_else(|| shapes.clone());
            let drawing = match (p.role, p.outline) {
                (Role::Stroke, Outline::None) => continue,
                (Role::Stroke, _) => Drawing { geos: [shapes.clone(), holes.clone()].concat(), paints: vec![Paint::Stroke(width)] },
                (Role::Surface, Outline::None) => continue,
                (Role::Surface, Outline::Open) => Drawing { geos: [open_ids[&p.name].clone(), holes.clone()].concat(), paints: vec![Paint::Stroke(width)] },
                (Role::Surface, Outline::Closed) => Drawing { geos: [shapes.clone(), holes.clone()].concat(), paints: vec![Paint::Stroke(width)] },
                (Role::Fill, _) => Drawing { geos: shapes.clone(), paints: vec![Paint::Fill { even_odd: false }] },
            };
            let masks = if p.role == Role::Fill { [gap_masks.clone(), hole_masks.clone()].concat() } else { gap_masks.clone() };
            layers.push(layer(variant, Some(style), drawing, masks));
        }
        if knocked.contains(&p.name) {
            continue;
        }
        let Some((drawing, _, _)) = solid_drawing(p) else { continue };
        let mut masks = [gap_masks.clone(), hole_masks.clone()].concat();
        let mut moving: Vec<(&Part, f64)> = Vec::new();
        for (k, gap) in &p.knockouts {
            let kp = &parts[part_index[k.as_str()]];
            let fixed = plan_rig_of(k) == rig && !dynamic(k);
            let Some((kd, half, filled)) = solid_drawing(kp) else { continue };
            if fixed {
                for id in &kd.geos {
                    masks.extend(geom::grown(&rest_geo(id), filled, half + gap).into_iter().map(Mask::fixed));
                }
            } else {
                moving.push((kp, *gap));
            }
        }
        // A morph's cut-out follows it only while the surface's one matte is free for it;
        // otherwise it is cut at its rest shape, as a static mask.
        let morph_only = |k: &str| plan_rig_of(k) == rig && morph_parts.contains(k) && !(path_form(k) || draw_parts.contains(k) || opacity_targets.contains(k) || hidden(k));
        let clash = |moving: &[(&Part, f64)]| {
            let rigs_used: IndexSet<Option<String>> = moving.iter().map(|(kp, _)| plan_rig_of(&kp.name)).collect();
            let own = moving.iter().any(|(kp, _)| draw_parts.contains(&kp.name) || opacity_targets.contains(&kp.name) || hidden(&kp.name));
            rigs_used.len() > 1 || (moving.len() > 1 && own)
        };
        if clash(&moving) && moving.iter().any(|(kp, _)| morph_only(&kp.name)) {
            let (still, keep): (Vec<_>, Vec<_>) = moving.into_iter().partition(|(kp, _)| morph_only(&kp.name));
            let followed: Vec<&str> = keep.iter().map(|(kp, _)| kp.name.as_str()).collect();
            for (kp, gap) in &still {
                let (kd, half, filled) = solid_drawing(kp).expect("checked");
                for id in &kd.geos {
                    masks.extend(geom::grown(&rest_geo(id), filled, half + gap).into_iter().map(Mask::fixed));
                }
                d.push(
                    Diagnostic::warning("MORPH_KNOCKOUT", at(&p.path, "knockout"), format!("in Solid, {} is cut out of {} at its rest shape: the surface's one matte follows {}", kp.name, p.name, followed.join(" and ")))
                        .hint("knock out one moving part per surface, or give the surface \"solid\": \"ring\" so nothing needs cutting"),
                );
            }
            moving = keep;
        }
        let mut solid = layer("solid", Some("solid"), drawing, masks);
        if !moving.is_empty() {
            let rigs_used: IndexSet<Option<String>> = moving.iter().map(|(kp, _)| plan_rig_of(&kp.name)).collect();
            if rigs_used.len() > 1 {
                d.push(
                    Diagnostic::error("KNOCKOUT_RIGS", at(&p.path, "knockout"), format!("the moving knockouts of {} ride {} rigs; a layer has one matte, and its source one parent", p.name, rigs_used.len()))
                        .hint("knock out parts that move together (one rig), and draw the others over the Solid"),
                );
                continue;
            }
            // The matte carries one trim and one visibility: those of a single part.
            let own: Vec<&str> = moving.iter().filter(|(kp, _)| draw_parts.contains(&kp.name) || opacity_targets.contains(&kp.name) || hidden(&kp.name)).map(|(kp, _)| kp.name.as_str()).collect();
            if moving.len() > 1 && !own.is_empty() {
                d.push(
                    Diagnostic::error("KNOCKOUT_TRACKS", at(&p.path, "knockout"), format!("{} is knocked out of {} with other parts, but draws on, fades or starts hidden; one matte follows one part's trim and visibility", own.join(" and "), p.name))
                        .hint("knock out only that part from this surface, and draw the others over the Solid"),
                );
                continue;
            }
            let mut geos = Vec::new();
            let mut any_fill = false;
            let mut width: f64 = 0.0;
            for (kp, gap) in &moving {
                let (kd, half, filled) = solid_drawing(kp).expect("checked");
                geos.extend(kd.geos);
                any_fill |= filled;
                width = width.max(2.0 * (half + gap));
            }
            let paints = if any_fill { vec![Paint::Fill { even_odd: false }, Paint::Stroke(width)] } else { vec![Paint::Stroke(width)] };
            let first = moving[0].0;
            solid.matte = Some(Box::new(Layer {
                name: format!("{}-{}-solid-matte", spec.id, p.name),
                part: first.name.clone(),
                variant: "matte".into(),
                style: None,
                rig: plan_rig_of(&first.name),
                geos,
                paints,
                masks: Vec::new(),
                vis: Some(format!("{}.opacity", first.name)),
                trim: draw_parts.contains(&first.name).then(|| format!("{}.draw", first.name)),
                opacity: Some(rest_opacity(&first.name)).filter(|o| *o < 1.0),
                matte: None,
            }));
        }
        layers.push(solid);
    }
    if d.has_errors() {
        return Err(d);
    }

    // Plan rigs, parent first.
    let mut split_rigs: HashSet<String> = HashSet::new();
    for m in &motions {
        for t in &m.tracks {
            if matches!(t.target.prop, Some(Prop::X | Prop::Y)) {
                split_rigs.insert(rigs[&t.target.name].outer());
            }
        }
    }
    let mut plan_rigs = Vec::new();
    for name in &order {
        let r = &rigs[name];
        let parent = r.parent.as_ref().map(|p| rigs[p].inner());
        match r.pivot {
            Some(PivotR::Contact(a, b)) => {
                plan_rigs.push(plan::Rig { name: r.outer(), pivot: (b.x, b.y), parent, split: split_rigs.contains(&r.outer()) });
                plan_rigs.push(plan::Rig { name: r.inner(), pivot: (a.x, a.y), parent: Some(r.outer()), split: false });
            }
            _ => {
                let p = r.pivot_point();
                plan_rigs.push(plan::Rig { name: r.outer(), pivot: (p.x, p.y), parent, split: split_rigs.contains(&r.outer()) });
            }
        }
    }

    // Tracks.
    let mut plan_motions = Vec::new();
    for m in &motions {
        let mut tracks: Vec<plan::Track> = Vec::new();
        let mut path_drivers: IndexMap<String, (String, Vec<PKey>)> = IndexMap::new();
        let mut add_path = |d: &mut Diagnostics, id: &str, source: &CTrack, f: &dyn Fn(f64) -> Geo| {
            let keys: Vec<PKey> = source.keys.iter().map(|k| PKey { t: k.t, v: KeyValue::Points(f(k.v).contour().flat_points()), out: k.out }).collect();
            if let Some((other, _)) = path_drivers.get(id) {
                d.push(
                    Diagnostic::error("COMBINED_PATH_TRACKS", source.path.clone(), format!("{} and {other} both reshape {id} in {}", source.target, m.name))
                        .hint("one path-changing track per shape in this prototype (morph, r/grow/amp, or a rig's scale/squash)"),
                );
                return;
            }
            path_drivers.insert(id.to_string(), (source.target.to_string(), keys));
        };
        for t in &m.tracks {
            let scalar = |keys: &[CKey], f: &dyn Fn(f64) -> f64| -> Vec<PKey> { keys.iter().map(|k| PKey { t: k.t, v: KeyValue::Scalar(f(k.v)), out: k.out }).collect() };
            let deg = |v: f64| -v * PI / 180.0;
            match t.target.prop {
                Some(Prop::Rotate) => {
                    let r = &rigs[&t.target.name];
                    match r.pivot {
                        Some(PivotR::Contact(..)) => {
                            let (pos, neg) = ease::split_sign(&t.keys);
                            if !ease::is_constant(&pos, 0.0) {
                                tracks.push(plan::Track { target: r.outer(), prop: plan::Prop::Rotation, keys: scalar(&pos, &deg) });
                            }
                            if !ease::is_constant(&neg, 0.0) {
                                tracks.push(plan::Track { target: r.inner(), prop: plan::Prop::Rotation, keys: scalar(&neg, &deg) });
                            }
                        }
                        _ => tracks.push(plan::Track { target: r.outer(), prop: plan::Prop::Rotation, keys: scalar(&t.keys, &deg) }),
                    }
                }
                Some(Prop::X) => {
                    let r = &rigs[&t.target.name];
                    let px = r.pivot_point().x;
                    tracks.push(plan::Track { target: r.outer(), prop: plan::Prop::PositionX, keys: scalar(&t.keys, &|v| px + v) });
                }
                Some(Prop::Y) => {
                    let r = &rigs[&t.target.name];
                    let py = r.pivot_point().y;
                    tracks.push(plan::Track { target: r.outer(), prop: plan::Prop::PositionY, keys: scalar(&t.keys, &|v| py + v) });
                }
                Some(Prop::Opacity) => {
                    for name in changed_parts(&t.target) {
                        tracks.push(plan::Track { target: format!("{name}.opacity"), prop: plan::Prop::Opacity, keys: scalar(&t.keys, &|v| v) });
                    }
                }
                Some(Prop::Draw) => tracks.push(plan::Track { target: format!("{}.draw", t.target.name), prop: plan::Prop::TrimEnd, keys: scalar(&t.keys, &|v| v) }),
                Some(prop @ (Prop::R | Prop::Grow | Prop::Amp)) => {
                    for id in &shape_ids[&t.target.name] {
                        let def = geo_defs[id].clone();
                        if !def.shape.takes(prop) {
                            continue;
                        }
                        let f = |v: f64| {
                            let params = match prop {
                                Prop::R => Params { r: v, ..Params::default() },
                                Prop::Grow => Params { grow: v, ..Params::default() },
                                _ => Params { amp: v, ..Params::default() },
                            };
                            def.shape.geo(params, def.form)
                        };
                        add_path(&mut d, id, t, &f);
                    }
                    for id in open_ids.get(&t.target.name).into_iter().flatten() {
                        let def = geo_defs[id].clone();
                        if def.shape.takes(prop) {
                            let f = |v: f64| def.shape.geo(if prop == Prop::R { Params { r: v, ..Params::default() } } else { Params { grow: v, ..Params::default() } }, Form::Open);
                            add_path(&mut d, id, t, &f);
                        }
                    }
                }
                Some(Prop::Scale | Prop::Squash) => {
                    let r = &rigs[&t.target.name];
                    let pivot = r.pivot_point();
                    let squash = deform_rigs.get(&t.target.name).copied().flatten();
                    let map = move |k: f64| {
                        move |p: Pt| match squash {
                            None => Pt::new(pivot.x + k * (p.x - pivot.x), pivot.y + k * (p.y - pivot.y)),
                            Some(Squash { base, widen }) => Pt::new(pivot.x + (p.x - pivot.x) * (1.0 + widen * (1.0 - k)), base + (p.y - base) * k),
                        }
                    };
                    let ids: Vec<String> = geo_defs.iter().filter(|(_, g)| part_rig.get(&g.part) == Some(&t.target.name)).map(|(id, _)| id.clone()).collect();
                    for id in ids {
                        let rest = rest_geo(&id).contour();
                        let f = |k: f64| Geo::Path(rest.map(map(k)));
                        add_path(&mut d, &id, t, &f);
                    }
                }
                None => {
                    let ids: Vec<String> = geo_defs.iter().filter(|(_, g)| !g.hole && g.shape.driver() == Some(t.target.name.as_str())).map(|(id, _)| id.clone()).collect();
                    for id in ids {
                        let def = geo_defs[&id].clone();
                        let f = |k: f64| def.shape.morph_at(k, def.form).expect("a morph");
                        add_path(&mut d, &id, t, &f);
                    }
                }
                Some(Prop::Swing | Prop::Flip | Prop::Phase) => {}
            }
        }
        for (id, (_, keys)) in path_drivers {
            tracks.push(plan::Track { target: id, prop: plan::Prop::PathPoints, keys });
        }
        plan_motions.push(plan::Motion { name: m.name.clone(), active: m.active, rest: m.rest, tracks });
    }
    if d.has_errors() {
        return Err(d);
    }

    // Rest geometry: keyed shapes are paths.
    let geos: IndexMap<String, Geo> = geo_defs.keys().map(|id| (id.clone(), rest_geo(id))).collect();
    let plan = Plan { id: spec.id.clone(), rigs: plan_rigs, geos, layers, motions: plan_motions };

    // Draft rows.
    let mut sheet_rows = Vec::new();
    let times_for = |m: &Motion| -> Vec<f64> {
        let custom = match (m.reverse, m.which) {
            (false, Which::Light) => spec.preview.light.clone(),
            (false, Which::Strong) => spec.preview.strong.clone(),
            _ => None,
        };
        custom.unwrap_or_else(|| motion::beat_times(m, 8))
    };
    for m in &motions {
        let looks: &[&str] = match (m.reverse, m.which) {
            (false, Which::Light) => &["regular"],
            (false, Which::Strong) => &["regular", "bold", "solid", "duotone"],
            (true, _) => &["regular"],
        };
        for look in looks {
            sheet_rows.push(json!({"motion": m.name, "look": look, "times": times_for(m)}));
        }
    }
    let chains = parts.iter().map(|p| (p.name.clone(), part_rig.get(&p.name).map(|r| {
        let mut out = Vec::new();
        for c in chain_of(r) {
            let node = &rigs[&c];
            out.push(node.inner());
            if node.inner() != node.outer() {
                out.push(node.outer());
            }
        }
        out
    }).unwrap_or_default())).collect();

    let compiled = Compiled { spec: spec.clone(), parts, plan, motions, duotone_fill, diagnostics: d, sheet_rows, chains };
    let mut lints = crate::lint::lint(&compiled);
    let mut compiled = compiled;
    compiled.diagnostics.0.append(&mut lints.0);
    if compiled.diagnostics.has_errors() {
        return Err(compiled.diagnostics);
    }
    Ok(compiled)
}
