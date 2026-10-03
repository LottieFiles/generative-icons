//! Lints on the compiled plan, evaluated as a Lottie player would at 145 frames per motion:
//! paint near or past the canvas edge, parts too small at 24 px, Strong tracks that are not
//! stronger than Light, holes and knockouts that miss their surface, and parts that vanish
//! into another part's Solid fill. They need no engine and no rendering.

use crate::compile::{Compiled, Form, Params, RShape};
use crate::diag::{Diagnostic, Diagnostics, at};
use crate::ease::{self, Handle};
use crate::geom::{BBox, Contour, Geo, Pt};
use crate::motion::{self, Which};
use crate::plan::{KeyValue, Layer, PKey, Paint, Plan, Prop, Track};
use std::collections::HashMap;

/// Frames per motion: 0 to 144 at 60 fps.
pub const FRAMES: usize = 145;
/// Painted outlines closer than this to the canvas edge warn: at 24 px the outer pixel is
/// 1.33 units wide, and ink on it fails the build's visual check ("Clipped icon").
pub const MARGIN: f64 = 1.4;
/// Opacity under which a part's edge cannot reach the visual check's visibility threshold
/// on the outer pixel, so `CANVAS_MARGIN` does not warn about it.
pub const FAINT: f64 = 0.3;
/// Parts whose painted box at Regular is under this many pixels at 24 px warn.
pub const SMALL_PX: f64 = 3.0;
/// Parts hidden at rest are seen in motion, as a flick: under this many pixels at 24 px
/// they read as a speck.
pub const SPECK_PX: f64 = 2.0;

/// `x' = a·x + c·y + e`, `y' = b·x + d·y + f`.
#[derive(Debug, Clone, Copy)]
pub struct Affine([f64; 6]);

impl Affine {
    pub const ID: Affine = Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    pub fn apply(&self, p: Pt) -> Pt {
        let [a, b, c, d, e, f] = self.0;
        Pt::new(a * p.x + c * p.y + e, b * p.x + d * p.y + f)
    }

    /// `self ∘ o`: apply `o` first.
    pub fn then_after(&self, o: &Affine) -> Affine {
        let [a, b, c, d, e, f] = self.0;
        let [a2, b2, c2, d2, e2, f2] = o.0;
        Affine([a * a2 + c * b2, b * a2 + d * b2, a * c2 + c * d2, b * c2 + d * d2, a * e2 + c * f2 + e, b * e2 + d * f2 + f])
    }

    /// A layer transform: rotate by `theta` engine radians (counter-clockwise on screen)
    /// about `anchor`, then put the anchor at `pos`.
    pub fn layer(anchor: (f64, f64), pos: (f64, f64), theta: f64) -> Affine {
        let (s, c) = theta.sin_cos();
        let (a, b, cc, d) = (c, -s, s, c);
        Affine([a, b, cc, d, pos.0 - (a * anchor.0 + cc * anchor.1), pos.1 - (b * anchor.0 + d * anchor.1)])
    }
}

fn scalar_at(keys: &[PKey], t: f64) -> f64 {
    let ck: Vec<ease::CKey> = keys
        .iter()
        .map(|k| ease::CKey { t: k.t, v: if let KeyValue::Scalar(v) = k.v { v } else { 0.0 }, out: k.out })
        .collect();
    ease::eval(&ck, t)
}

fn points_at(keys: &[PKey], t: f64) -> Vec<f64> {
    let get = |k: &PKey| if let KeyValue::Points(p) = &k.v { p.clone() } else { Vec::new() };
    let Some(first) = keys.first() else { return Vec::new() };
    if t <= first.t {
        return get(first);
    }
    for w in keys.windows(2) {
        if t < w[1].t {
            let u = (t - w[0].t) / (w[1].t - w[0].t);
            let p = ease::progress(w[0].out.unwrap_or(Handle::Bez(ease::LINEAR)), u);
            let (a, b) = (get(&w[0]), get(&w[1]));
            return a.iter().zip(&b).map(|(x, y)| x + (y - x) * p).collect();
        }
    }
    keys.last().map(get).unwrap_or_default()
}

/// A motion of a plan evaluated at one time.
pub struct Frame<'a> {
    plan: &'a Plan,
    rigs: HashMap<&'a str, Affine>,
    tracks: HashMap<(&'a str, Prop), &'a Track>,
    t: f64,
}

impl<'a> Frame<'a> {
    pub fn new(plan: &'a Plan, motion: usize, t: f64) -> Frame<'a> {
        let m = &plan.motions[motion];
        let tracks: HashMap<(&str, Prop), &Track> = m.tracks.iter().map(|tr| ((tr.target.as_str(), tr.prop), tr)).collect();
        let mut rigs: HashMap<&str, Affine> = HashMap::new();
        for r in &plan.rigs {
            let get = |prop: Prop, rest: f64| tracks.get(&(r.name.as_str(), prop)).map_or(rest, |tr| scalar_at(&tr.keys, t));
            let theta = get(Prop::Rotation, 0.0);
            let pos = (get(Prop::PositionX, r.pivot.0), get(Prop::PositionY, r.pivot.1));
            let local = Affine::layer(r.pivot, pos, theta);
            let world = match r.parent.as_deref().and_then(|p| rigs.get(p)) {
                Some(parent) => parent.then_after(&local),
                None => local,
            };
            rigs.insert(r.name.as_str(), world);
        }
        Frame { plan, rigs, tracks, t }
    }

    pub fn opacity(&self, layer: &Layer) -> f64 {
        let base = layer.opacity.unwrap_or(1.0);
        match layer.vis.as_deref().and_then(|v| self.tracks.get(&(v, Prop::Opacity))) {
            Some(tr) => scalar_at(&tr.keys, self.t),
            None => base,
        }
    }

    /// The geometry's contour at this time, in composition space.
    pub fn geo_bbox(&self, id: &str, rig: Option<&str>) -> Option<BBox> {
        let rest = self.plan.geos.get(id)?;
        let contour = match self.tracks.get(&(id, Prop::PathPoints)) {
            Some(tr) => {
                let pts = points_at(&tr.keys, self.t);
                rest.contour().with_points(&pts)
            }
            None => rest.contour(),
        };
        let world = rig.and_then(|r| self.rigs.get(r)).copied().unwrap_or(Affine::ID);
        Some(contour.map(|p| world.apply(p)).bbox())
    }

    /// The painted box of a layer (stroke half-width included), or `None` when invisible.
    pub fn painted(&self, layer: &Layer) -> Option<BBox> {
        if self.opacity(layer) < 0.01 {
            return None;
        }
        let half = layer.paints.iter().map(|p| if let Paint::Stroke(w) = p { w / 2.0 } else { 0.0 }).fold(0.0, f64::max);
        let b = layer.geos.iter().filter_map(|g| self.geo_bbox(g, layer.rig.as_deref())).reduce(BBox::union)?;
        Some(b.grow(half))
    }
}

/// Every lint, in a stable order.
pub fn lint(c: &Compiled) -> Diagnostics {
    let mut d = Diagnostics::default();
    light_strong(c, &mut d);
    canvas(c, &mut d);
    small(c, &mut d);
    cuts_that_miss(c, &mut d);
    solid_hidden(c, &mut d);
    d
}

/// Even-odd point in a closed contour, on its flattened outline.
fn inside(contour: &Contour, p: Pt) -> bool {
    if !contour.closed {
        return false;
    }
    let mut odd = false;
    for (a, b) in contour.flatten() {
        if (a.y > p.y) != (b.y > p.y) && a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x) > p.x {
            odd = !odd;
        }
    }
    odd
}

/// Points along a contour: the ends and middle of each flattened piece.
fn samples(contour: &Contour) -> Vec<Pt> {
    let pieces = contour.flatten();
    if pieces.is_empty() {
        return vec![contour.start];
    }
    pieces.iter().flat_map(|&(a, b)| [a, a.lerp(b, 0.5), b]).collect()
}

fn rest_contours(shapes: &[RShape]) -> Vec<Contour> {
    shapes.iter().map(|s| s.geo(Params::default(), Form::Native).contour()).collect()
}

/// `HOLE_OUTSIDE`: a hole or knockout entirely outside its surface at rest cuts nothing; a
/// knocked-out part is not drawn in Solid at all, so it then simply vanishes there.
fn cuts_that_miss(c: &Compiled, d: &mut Diagnostics) {
    for p in &c.parts {
        let surface: Vec<Contour> = rest_contours(&p.shapes).into_iter().filter(|s| s.closed).collect();
        if surface.is_empty() {
            continue;
        }
        let own: Vec<Pt> = surface.iter().flat_map(samples).collect();
        // Overlap either way round: a cut's outline inside the surface, or the surface's inside the cut.
        let touches = |cut: &[Contour]| {
            cut.iter().flat_map(samples).any(|q| surface.iter().any(|s| inside(s, q))) || own.iter().any(|&q| cut.iter().any(|h| inside(h, q)))
        };
        for (i, hole) in p.holes.iter().enumerate() {
            if !touches(&[hole.geo(Params::default(), Form::Native).contour()]) {
                d.push(
                    Diagnostic::warning("HOLE_OUTSIDE", at(&at(&p.path, "holes"), i), format!("this hole lies outside {}'s shape at rest, so it cuts nothing", p.name))
                        .hint("move the hole onto the surface, or remove it"),
                );
            }
        }
        for (i, (k, _)) in p.knockouts.iter().enumerate() {
            let Some(kp) = c.parts.iter().find(|x| &x.name == k) else { continue };
            if !touches(&rest_contours(&kp.shapes)) {
                d.push(
                    Diagnostic::warning("HOLE_OUTSIDE", at(&at(&p.path, "knockout"), i), format!("{k} does not overlap {} at rest, so in Solid it is neither cut out nor drawn: it vanishes", p.name))
                        .hint(format!("remove {k} from {}'s knockout list", p.name)),
                );
            }
        }
    }
}

/// `SOLID_HIDDEN`: in Solid every part is the same colour, so a part drawn entirely inside
/// another part's Solid fill (and not knocked out of it) disappears. v1 agents hit this as
/// details that merged into a filled body.
fn solid_hidden(c: &Compiled, d: &mut Diagnostics) {
    let solids: Vec<&Layer> = c.plan.layers.iter().filter(|l| l.style == Some("solid") && l.opacity != Some(0.0)).collect();
    let mut warned: Vec<&str> = Vec::new();
    for s in &solids {
        if !s.paints.iter().any(|p| matches!(p, Paint::Fill { .. })) {
            continue;
        }
        let fills: Vec<Contour> = s.geos.iter().filter_map(|g| c.plan.geos.get(g)).map(Geo::contour).filter(|f| f.closed).collect();
        let covered = |q: Pt| fills.iter().any(|f| inside(f, q)) && !s.masks.iter().any(|m| inside(&m.contour, q));
        for o in &solids {
            if o.part == s.part || warned.contains(&o.part.as_str()) {
                continue;
            }
            let pts: Vec<Pt> = o.geos.iter().filter_map(|g| c.plan.geos.get(g)).flat_map(|g| samples(&g.contour())).collect();
            if !pts.is_empty() && pts.iter().all(|&q| covered(q)) {
                warned.push(&o.part);
                d.push(
                    Diagnostic::warning("SOLID_HIDDEN", part_path(c, &o.part), format!("in Solid, {} lies inside {}'s fill in the same colour and disappears", o.part, s.part))
                        .hint(format!("add \"knockout\": [\"{}\"] to {} so it reads as a cut-out, or give {} \"solid\": \"ring\"", o.part, s.part, s.part)),
                );
            }
        }
    }
}

fn part_path(c: &Compiled, part: &str) -> String {
    c.parts.iter().find(|p| p.name == part).map_or_else(|| at("/parts", part), |p| p.path.clone())
}

fn canvas(c: &Compiled, d: &mut Diagnostics) {
    for (mi, m) in c.plan.motions.iter().enumerate() {
        // part -> (overflow, time); positive overflow is paint past the edge. `near` counts
        // only frames where the part is at least FAINT opaque: a fading edge near the canvas
        // stays under the visual check's visibility threshold.
        let mut worst: HashMap<String, (f64, f64)> = HashMap::new();
        let mut near: HashMap<String, (f64, f64)> = HashMap::new();
        for f in 0..FRAMES {
            let t = f as f64 / 60.0;
            let frame = Frame::new(&c.plan, mi, t);
            for layer in &c.plan.layers {
                let Some(b) = frame.painted(layer) else { continue };
                let over = [-b.x0, -b.y0, b.x1 - 32.0, b.y1 - 32.0].into_iter().fold(f64::MIN, f64::max);
                let keep = |map: &mut HashMap<String, (f64, f64)>| {
                    let entry = map.entry(layer.part.clone()).or_insert((f64::MIN, 0.0));
                    if over > entry.0 {
                        *entry = (over, t);
                    }
                };
                keep(&mut worst);
                if frame.opacity(layer) >= FAINT {
                    keep(&mut near);
                }
            }
        }
        let mut parts: Vec<(&String, &(f64, f64))> = worst.iter().collect();
        parts.sort_by(|a, b| a.0.cmp(b.0));
        for (part, &(over, t)) in parts {
            let path = part_path(c, part);
            if over > 0.01 {
                d.push(
                    Diagnostic::error("CANVAS_MARGIN", path, format!("{part}'s paint leaves the 32-unit canvas by {:.2} at {:.3} s in {}", over, t, m.name))
                        .hint("move it in, or hide it while it is outside (opacity)"),
                );
            } else if let Some(&(over, t)) = near.get(part.as_str()).filter(|(over, _)| *over > -MARGIN) {
                d.push(
                    Diagnostic::warning("CANVAS_MARGIN", path, format!("{part}'s paint comes within {:.2} of the canvas edge at {:.3} s in {}", -over, t, m.name))
                        .hint(format!("keep paint {MARGIN} clear of the edge at Bold 2.5 in every frame, travel and overshoot included: at 24 px, ink on the outer pixel fails build's visual check (\"Clipped icon\")")),
                );
            }
        }
    }
}

fn small(c: &Compiled, d: &mut Diagnostics) {
    if c.plan.motions.is_empty() {
        return;
    }
    let frame = Frame::new(&c.plan, 0, 0.0);
    // Parts hidden at rest (sparks, drops, arcs) are measured as drawn: a transient that small
    // reads as a speck when it shows.
    for layer in c.plan.layers.iter().filter(|l| l.style == Some("outline.regular")) {
        let transient = layer.opacity == Some(0.0);
        let Some(b) = frame.painted(&Layer { opacity: None, vis: None, ..layer.clone() }) else { continue };
        let size = b.w().max(b.h()) * 24.0 / 32.0;
        if transient && size < SPECK_PX {
            d.push(
                Diagnostic::warning("SMALL_AT_24", part_path(c, &layer.part), format!("{} is {:.1} px across at 24 px when it shows", layer.part, size))
                    .hint(format!("a part hidden at rest under {SPECK_PX} px reads as a speck; make the spark, drop or arc bigger, or show the cause another way")),
            );
        } else if !transient && size < SMALL_PX {
            d.push(
                Diagnostic::warning("SMALL_AT_24", part_path(c, &layer.part), format!("{} is {:.1} px across at 24 px", layer.part, size))
                    .hint(format!("parts under {SMALL_PX} px read as noise at real size; make it bigger or merge it")),
            );
        }
    }
}

fn light_strong(c: &Compiled, d: &mut Diagnostics) {
    let forward = |w: Which| c.motions.iter().find(|m| m.which == w && !m.reverse);
    let (Some(light), Some(strong)) = (forward(Which::Light), forward(Which::Strong)) else { return };
    for s in &strong.raw {
        // Opacity and draw are bounded (0–1): Strong cannot travel further there.
        if matches!(s.target.prop, Some(crate::spec::Prop::Opacity | crate::spec::Prop::Draw)) {
            continue;
        }
        let Some(l) = light.raw.iter().find(|l| l.target == s.target) else { continue };
        let hidden = c.parts.iter().any(|p| p.name == s.target.name && p.hidden);
        let rest = motion::rest_value(s.target.prop, hidden);
        let peak = |keys: &[ease::Key]| keys.iter().map(|k| (k.v - rest).abs()).fold(0.0, f64::max);
        // Moves: runs between direction changes. A Strong with an extra beat (a wind-up, an
        // overshoot) is stronger even when its peak is bounded (a switch's travel).
        let moves = |keys: &[ease::Key]| {
            let mut count = 0;
            let mut last = 0.0;
            for w in keys.windows(2) {
                let d = (w[1].v - w[0].v).signum();
                if d != 0.0 && d != last {
                    count += 1;
                    last = d;
                }
            }
            count
        };
        let (lp, sp) = (peak(&l.keys), peak(&s.keys));
        let same_beats = moves(&l.keys) == moves(&s.keys);
        let same_values = l.keys.len() == s.keys.len() && l.keys.iter().zip(&s.keys).all(|(a, b)| (a.v - b.v).abs() < 1e-9);
        if same_values {
            d.push(
                Diagnostic::warning("LIGHT_STRONG", s.path.clone(), format!("Strong {} repeats Light's values; only the timing differs", s.target))
                    .hint("Strong should add a beat (a wind-up, an overshoot, a second hit) or travel further"),
            );
        } else if same_beats && lp > 0.0 && sp < 1.25 * lp - 1e-9 {
            d.push(
                Diagnostic::warning("LIGHT_STRONG", s.path.clone(), format!("Strong {} peaks at {} against Light's {}: under 1.25×", s.target, round3(sp), round3(lp)))
                    .hint("make Strong clearly bigger than Light, or give it an extra beat"),
            );
        }
    }
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// The geometry of a plan id at rest, for tests.
pub fn rest_geo<'a>(plan: &'a Plan, id: &str) -> Option<&'a Geo> {
    plan.geos.get(id)
}
