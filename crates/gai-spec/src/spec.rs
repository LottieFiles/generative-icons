//! The v2 spec (`gai-icon/2`) as typed data, parsed from JSON with every structural rule of
//! `assets/icon-v2.schema.json` checked by hand. Each kind of shape, key item, ease and motion
//! value is recognised by its kind key first and only that branch is validated, so an error
//! says "polygon: 30 vertices, at most 96" or "unknown shape `cirle`; did you mean `circle`?"
//! instead of a generic `oneOf` failure.

use crate::diag::{Diagnostic, Diagnostics, at, did_you_mean};
use indexmap::IndexMap;
use serde_json::{Map, Value};

/// The `format` of every v2 spec.
pub const FORMAT: &str = "gai-icon/2";
/// The timeline, in seconds.
pub const DURATION: f64 = 2.4;
/// The longest action: rest and its one-frame marker must fit in the timeline.
pub const MAX_ACTIVE: f64 = 2.3;
/// Coordinates: the 32-unit grid with a margin for parts that move in from outside.
const COORD: (f64, f64) = (-16.0, 48.0);

/// A point: literal, a named or derived anchor, or polar from another point.
#[derive(Debug, Clone, PartialEq)]
pub enum Point {
    Xy(f64, f64),
    /// `name` or `part.c|top|bottom|left|right|tl|tr|bl|br|start|end`.
    Anchor(String),
    /// `angle` in degrees, 0 = +x (3 o'clock), clockwise on screen; `len` units from `from`.
    Polar { from: Box<Point>, angle: f64, len: f64 },
}

/// A polygon vertex.
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub p: Point,
    pub r: Option<f64>,
    /// Control points bending the edge to the next vertex.
    pub curve: Option<[Point; 2]>,
}

/// Corner radii of a rect: one for all, or `[tl, tr, br, bl]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Radii {
    One(f64),
    Four([f64; 4]),
}

impl Radii {
    pub fn corners(self) -> [f64; 4] {
        match self {
            Radii::One(r) => [r; 4],
            Radii::Four(r) => r,
        }
    }
}

/// One shape of a part.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Line([Point; 2]),
    Polyline { points: Vec<Point>, r: f64 },
    Rect { x: f64, y: f64, w: f64, h: f64, r: Radii },
    Circle { c: Point, r: f64 },
    Ellipse { c: Point, rx: f64, ry: f64 },
    /// Degrees, 0 = +x, clockwise on screen.
    Arc { c: Point, r: f64, from: f64, to: f64, closed: bool },
    Polygon { vertices: Vec<Vertex>, r: f64 },
    Wave { x: [f64; 2], y: f64, cycles: f64, amp: f64, phase: f64 },
    /// SVG path data with one subpath.
    Path(String),
    Minus(Vec<Shape>),
    Morph { a: Box<Shape>, b: Box<Shape>, by: String },
}

impl Shape {
    pub fn kind(&self) -> &'static str {
        match self {
            Shape::Line(_) => "line",
            Shape::Polyline { .. } => "polyline",
            Shape::Rect { .. } => "rect",
            Shape::Circle { .. } => "circle",
            Shape::Ellipse { .. } => "ellipse",
            Shape::Arc { .. } => "arc",
            Shape::Polygon { .. } => "polygon",
            Shape::Wave { .. } => "wave",
            Shape::Path(_) => "path",
            Shape::Minus(_) => "minus",
            Shape::Morph { .. } => "morph",
        }
    }
}

const SHAPE_KINDS: [&str; 11] =
    ["line", "polyline", "rect", "circle", "ellipse", "arc", "polygon", "wave", "path", "minus", "morph"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Stroke,
    Surface,
    Fill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outline {
    Closed,
    Open,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Solid {
    Auto,
    Ring,
    Fill,
    None,
    Grow(f64),
    Shape(Shape),
}

/// A knockout: a part cut out of this surface in Solid, with its clearance.
#[derive(Debug, Clone, PartialEq)]
pub struct Knockout {
    pub part: String,
    pub gap: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PartBody {
    Shapes(Vec<Shape>),
    Mirror { of: String, x: Option<f64>, y: Option<f64> },
}

/// A part as written. Options are `None` when not given, so a mirror inherits them.
#[derive(Debug, Clone, PartialEq)]
pub struct PartSrc {
    pub name: String,
    pub path: String,
    pub body: PartBody,
    pub role: Option<Role>,
    pub outline: Option<Outline>,
    pub tint: Option<bool>,
    pub solid: Option<Solid>,
    pub holes: Option<Vec<Shape>>,
    pub knockout: Option<Vec<Knockout>>,
    /// `gap` as a number: the default knockout clearance.
    pub knockout_gap: Option<f64>,
    /// `gap` as an object: cut this part within `d` of another part at rest.
    pub gaps: Option<Vec<(String, f64)>>,
    pub hidden: Option<bool>,
    /// The opacity the part rests at (0 to 1, not 0: that is `hidden`).
    pub opacity: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pivot {
    Point(Point),
    /// Tip about the first point for negative angles and the second for positive ones.
    Contact(Point, Point),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RigSrc {
    pub name: String,
    pub path: String,
    pub pivot: Option<Pivot>,
    pub carries: Vec<String>,
    /// `swing`, `flip`, `squash`: the deformer kinds present.
    pub swing: bool,
    pub flip: bool,
    pub squash: Option<Squash>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Squash {
    pub base: f64,
    pub widen: f64,
}

/// How the motion arrives at a key.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ease {
    Auto,
    Linear,
    In,
    Out,
    InOut,
    Sine,
    Hit,
    Launch,
    Pass,
    Step,
    Bezier([f64; 4]),
}

const EASES: [&str; 11] = ["auto", "linear", "in", "out", "inOut", "smooth", "sine", "hit", "launch", "pass", "step"];

/// One item of a key list.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyItem {
    Key { t: f64, v: f64, ease: Ease },
    Hold { t: f64 },
    Settle { t: f64, to: Option<f64>, amp: f64, decay: f64, swings: u32 },
    Bounce { t: f64, amp: f64 },
    Wobble { at: f64, amp: f64, until: f64, swings: u32 },
}

/// A key list item and its pointer.
pub type Items = Vec<(KeyItem, String)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Prop {
    X,
    Y,
    Rotate,
    Scale,
    Opacity,
    Draw,
    Swing,
    Flip,
    Squash,
    R,
    Grow,
    Phase,
    Amp,
}

impl Prop {
    pub fn parse(s: &str) -> Option<Prop> {
        Some(match s {
            "x" => Prop::X,
            "y" => Prop::Y,
            "rotate" => Prop::Rotate,
            "scale" => Prop::Scale,
            "opacity" => Prop::Opacity,
            "draw" => Prop::Draw,
            "swing" => Prop::Swing,
            "flip" => Prop::Flip,
            "squash" => Prop::Squash,
            "r" => Prop::R,
            "grow" => Prop::Grow,
            "phase" => Prop::Phase,
            "amp" => Prop::Amp,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Prop::X => "x",
            Prop::Y => "y",
            Prop::Rotate => "rotate",
            Prop::Scale => "scale",
            Prop::Opacity => "opacity",
            Prop::Draw => "draw",
            Prop::Swing => "swing",
            Prop::Flip => "flip",
            Prop::Squash => "squash",
            Prop::R => "r",
            Prop::Grow => "grow",
            Prop::Phase => "phase",
            Prop::Amp => "amp",
        }
    }
}

const PROPS: [&str; 13] =
    ["x", "y", "rotate", "scale", "opacity", "draw", "swing", "flip", "squash", "r", "grow", "phase", "amp"];

/// `<part or rig>.<prop>`, or a bare morph driver.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Target {
    pub name: String,
    pub prop: Option<Prop>,
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.prop {
            Some(prop) => write!(f, "{}.{}", self.name, prop.as_str()),
            None => write!(f, "{}", self.name),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Follow {
    pub target: Target,
    pub lag: f64,
    pub scale: f64,
    pub offset: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MotionValue {
    Keys(Items),
    Pulse { start: f64, dur: f64, r: Option<f64>, grow: Option<f64> },
    Loop { keys: Items, period: f64, until: Option<f64> },
    Follow(Follow),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrackSrc {
    PerMotion { light: Option<(MotionValue, String)>, strong: Option<(MotionValue, String)> },
    Follow(Follow),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionEntry {
    pub target: Target,
    pub path: String,
    pub track: TrackSrc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReverseMode {
    Auto,
    Mirror,
    Fade,
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub from: String,
    pub to: String,
    pub labels: [String; 2],
    pub reverse: ReverseMode,
}

/// Explicit keys replacing one track of a reverse motion.
#[derive(Debug, Clone, PartialEq)]
pub struct Override {
    pub target: Target,
    pub path: String,
    pub keys: Items,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Timing {
    pub light: Option<f64>,
    pub strong: Option<f64>,
    pub reverse: Option<f64>,
    pub reverse_strong: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Preview {
    pub light: Option<Vec<f64>>,
    pub strong: Option<Vec<f64>>,
}

/// A parsed, structurally valid spec. Names are resolved later, by the compiler.
#[derive(Debug, Clone, PartialEq)]
pub struct Spec {
    pub id: String,
    pub label: String,
    pub intent: String,
    pub state: Option<State>,
    pub anchors: IndexMap<String, (f64, f64)>,
    pub parts: Vec<PartSrc>,
    pub rigs: Vec<RigSrc>,
    pub motion: Vec<MotionEntry>,
    pub reverse_light: Vec<Override>,
    pub reverse_strong: Vec<Override>,
    pub timing: Timing,
    pub preview: Preview,
    /// Rotational symmetry in degrees: a rotation may end this many degrees (or a multiple)
    /// from where it started (`cycleDegrees`). Whole turns are always allowed.
    pub cycle: Option<f64>,
}

/// Parses and structurally validates a spec. Every error is collected, not just the first.
pub fn parse(text: &str) -> Result<Spec, Diagnostics> {
    let value: Value = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(error) => {
            let mut d = Diagnostics::default();
            d.push(
                Diagnostic::error(
                    "V2_JSON",
                    "",
                    format!("not valid JSON at line {}, column {}: {error}", error.line(), error.column()),
                )
                .hint("v2 specs are plain JSON: no comments (use a \"//\" key), no trailing commas, no functions"),
            );
            return Err(d);
        }
    };
    parse_value(&value)
}

/// [`parse`] for an already-parsed JSON value.
pub fn parse_value(value: &Value) -> Result<Spec, Diagnostics> {
    let mut p = Parser { d: Diagnostics::default() };
    let spec = p.spec(value);
    match (spec, p.d.has_errors()) {
        (Some(spec), false) => Ok(spec),
        _ => Err(p.d),
    }
}

struct Parser {
    d: Diagnostics,
}

const TOP: [&str; 15] = [
    "$schema", "//", "format", "id", "label", "intent", "state", "anchors", "parts", "rigs", "motion", "reverse",
    "timing", "preview", "cycle",
];

impl Parser {
    fn err(&mut self, path: &str, message: impl Into<String>) {
        self.d.push(Diagnostic::error("V2_SCHEMA", path, message));
    }

    fn err_hint(&mut self, path: &str, message: impl Into<String>, hint: impl Into<String>) {
        self.d.push(Diagnostic::error("V2_SCHEMA", path, message).hint(hint));
    }

    /// An object with only `allowed` keys and every `required` key.
    fn object<'a>(&mut self, v: &'a Value, path: &str, what: &str, allowed: &[&str], required: &[&str]) -> Option<&'a Map<String, Value>> {
        let Some(map) = v.as_object() else {
            self.err(path, format!("{what} must be an object, got {}", kind(v)));
            return None;
        };
        for key in map.keys() {
            if !allowed.contains(&key.as_str()) {
                let hint = match did_you_mean(key, allowed) {
                    Some(close) => format!("did you mean `{close}`?"),
                    None => format!("{what} takes {}", list(allowed)),
                };
                self.err_hint(&at(path, key), format!("unknown field `{key}` in {what}"), hint);
            }
        }
        for key in required {
            if !map.contains_key(*key) {
                self.err(path, format!("{what} needs `{key}`"));
            }
        }
        Some(map)
    }

    fn number(&mut self, v: &Value, path: &str, what: &str, lo: f64, hi: f64) -> Option<f64> {
        match v.as_f64() {
            Some(n) if n >= lo && n <= hi => Some(n),
            Some(n) => {
                self.err(path, format!("{what} must be from {} to {}, got {}", fmt(lo), fmt(hi), fmt(n)));
                None
            }
            None => {
                self.err(path, format!("{what} must be a number, got {}", kind(v)));
                None
            }
        }
    }

    /// A number in `(lo, hi]`.
    fn positive(&mut self, v: &Value, path: &str, what: &str, hi: f64) -> Option<f64> {
        match v.as_f64() {
            Some(n) if n > 0.0 && n <= hi => Some(n),
            Some(n) => {
                self.err(path, format!("{what} must be above 0 and at most {}, got {}", fmt(hi), fmt(n)));
                None
            }
            None => {
                self.err(path, format!("{what} must be a number, got {}", kind(v)));
                None
            }
        }
    }

    fn coord(&mut self, v: &Value, path: &str) -> Option<f64> {
        self.number(v, path, "a coordinate", COORD.0, COORD.1)
    }

    fn time(&mut self, v: &Value, path: &str) -> Option<f64> {
        self.number(v, path, "a time (seconds)", 0.0, DURATION)
    }

    fn boolean(&mut self, v: &Value, path: &str) -> Option<bool> {
        match v.as_bool() {
            Some(b) => Some(b),
            None => {
                self.err(path, format!("expected true or false, got {}", kind(v)));
                None
            }
        }
    }

    fn string(&mut self, v: &Value, path: &str, what: &str) -> Option<String> {
        match v.as_str() {
            Some(s) => Some(s.to_string()),
            None => {
                self.err(path, format!("{what} must be a string, got {}", kind(v)));
                None
            }
        }
    }

    fn name(&mut self, v: &Value, path: &str, what: &str) -> Option<String> {
        let s = self.string(v, path, what)?;
        if !is_name(&s) {
            self.err_hint(path, format!("{what} `{s}` is not a kebab-case name"), "use lowercase letters, digits and single hyphens, starting with a letter, at most 32 characters");
            return None;
        }
        Some(s)
    }

    fn array<'a>(&mut self, v: &'a Value, path: &str, what: &str, min: usize, max: usize) -> Option<&'a Vec<Value>> {
        let Some(items) = v.as_array() else {
            self.err(path, format!("{what} must be an array, got {}", kind(v)));
            return None;
        };
        if items.len() < min || items.len() > max {
            let range = if min == max { format!("exactly {min}") } else if max == usize::MAX { format!("at least {min}") } else { format!("{min} to {max}") };
            self.err(path, format!("{what}: {} items, expected {range}", items.len()));
            return None;
        }
        Some(items)
    }

    fn spec(&mut self, v: &Value) -> Option<Spec> {
        let map = self.object(v, "", "the spec", &TOP, &["format", "id", "label", "intent", "parts", "motion"])?;
        match map.get("format") {
            Some(Value::String(f)) if f == FORMAT => {}
            Some(other) => self.err_hint("/format", format!("format must be \"{FORMAT}\", got {other}"), "v1 icons are .mjs batches; a .icon.json file is always gai-icon/2"),
            None => {}
        }
        let id = map.get("id").and_then(|v| self.name(v, "/id", "id"));
        let label = map.get("label").and_then(|v| self.string(v, "/label", "label"));
        if let Some(label) = &label
            && (label.is_empty() || label.chars().count() > 40)
        {
            self.err("/label", format!("label must be 1 to 40 characters, got {}", label.chars().count()));
        }
        let intent = map.get("intent").and_then(|v| self.string(v, "/intent", "intent"));
        if let Some(intent) = &intent
            && intent.chars().count() < 20
        {
            self.err_hint("/intent", "intent is too short", "say what Light and Strong (and the reverse) show, in plain words; at least 20 characters");
        }
        let state = map.get("state").and_then(|v| self.state(v));
        let anchors = match map.get("anchors") {
            Some(v) => self.anchors(v),
            None => IndexMap::new(),
        };
        let parts = map.get("parts").map(|v| self.parts(v)).unwrap_or_default();
        let rigs = map.get("rigs").map(|v| self.rigs(v)).unwrap_or_default();
        let motion = map.get("motion").map(|v| self.motion(v)).unwrap_or_default();
        let (reverse_light, reverse_strong) = match map.get("reverse") {
            Some(v) => self.reverse(v),
            None => (Vec::new(), Vec::new()),
        };
        let timing = map.get("timing").map(|v| self.timing(v)).unwrap_or_default();
        let preview = map.get("preview").map(|v| self.preview(v)).unwrap_or_default();
        let cycle = map.get("cycle").and_then(|v| self.positive(v, "/cycle", "cycle (degrees)", 360.0));
        if let Some(comment) = map.get("//")
            && !comment.is_string()
        {
            self.err("/~1~1", "a \"//\" comment must be a string");
        }
        Some(Spec {
            id: id?,
            label: label?,
            intent: intent?,
            state,
            anchors,
            parts,
            rigs,
            motion,
            reverse_light,
            reverse_strong,
            timing,
            preview,
            cycle,
        })
    }

    fn state(&mut self, v: &Value) -> Option<State> {
        let map = self.object(v, "/state", "state", &["from", "to", "labels", "reverse"], &["from", "to", "labels"])?;
        let from = map.get("from").and_then(|v| self.name(v, "/state/from", "state.from"));
        let to = map.get("to").and_then(|v| self.name(v, "/state/to", "state.to"));
        let labels = map.get("labels").and_then(|v| {
            let items = self.array(v, "/state/labels", "state.labels", 2, 2)?;
            let mut out = Vec::new();
            for (i, item) in items.iter().enumerate() {
                let path = at("/state/labels", i);
                let s = self.string(item, &path, "a label")?;
                if s.is_empty() || s.chars().count() > 24 {
                    self.err(&path, "a label must be 1 to 24 characters");
                    return None;
                }
                out.push(s);
            }
            Some([out[0].clone(), out[1].clone()])
        });
        let reverse = match map.get("reverse") {
            None => Some(ReverseMode::Auto),
            Some(Value::String(s)) if s == "auto" => Some(ReverseMode::Auto),
            Some(Value::String(s)) if s == "mirror" => Some(ReverseMode::Mirror),
            Some(Value::String(s)) if s == "fade" => Some(ReverseMode::Fade),
            Some(other) => {
                self.err_hint("/state/reverse", format!("unknown reverse mode {other}"), "use auto, mirror or fade");
                None
            }
        };
        Some(State { from: from?, to: to?, labels: labels?, reverse: reverse? })
    }

    fn anchors(&mut self, v: &Value) -> IndexMap<String, (f64, f64)> {
        let mut out = IndexMap::new();
        let Some(map) = v.as_object() else {
            self.err("/anchors", format!("anchors must be an object of name: [x, y], got {}", kind(v)));
            return out;
        };
        for (name, value) in map {
            let path = at("/anchors", name);
            if !is_name(name) {
                self.err(&path, format!("anchor name `{name}` is not kebab-case"));
                continue;
            }
            if let Some((x, y)) = self.xy(value, &path) {
                out.insert(name.clone(), (x, y));
            }
        }
        out
    }

    fn xy(&mut self, v: &Value, path: &str) -> Option<(f64, f64)> {
        let items = self.array(v, path, "a point", 2, 2)?;
        let x = self.coord(&items[0], &at(path, 0));
        let y = self.coord(&items[1], &at(path, 1));
        Some((x?, y?))
    }

    fn point(&mut self, v: &Value, path: &str) -> Option<Point> {
        match v {
            Value::Array(_) => self.xy(v, path).map(|(x, y)| Point::Xy(x, y)),
            Value::String(s) => {
                if is_anchor_ref(s) {
                    Some(Point::Anchor(s.clone()))
                } else {
                    self.err_hint(path, format!("`{s}` is not an anchor reference"), "use a name from anchors, or <part>.c|top|bottom|left|right|tl|tr|bl|br|start|end");
                    None
                }
            }
            Value::Object(_) => {
                let map = self.object(v, path, "a polar point", &["from", "angle", "len"], &["from", "angle", "len"])?;
                let from = self.point(&map["from"], &at(path, "from"));
                let angle = self.number(&map["angle"], &at(path, "angle"), "angle (degrees)", -720.0, 720.0);
                let len = self.number(&map["len"], &at(path, "len"), "len", 0.0, 48.0);
                Some(Point::Polar { from: Box::new(from?), angle: angle?, len: len? })
            }
            other => {
                self.err(path, format!("a point is [x, y], an anchor name or {{\"from\", \"angle\", \"len\"}}, got {}", kind(other)));
                None
            }
        }
    }

    fn points(&mut self, v: &Value, path: &str, what: &str, min: usize, max: usize) -> Option<Vec<Point>> {
        let items = self.array(v, path, what, min, max)?;
        let mut out = Vec::new();
        let mut ok = true;
        for (i, item) in items.iter().enumerate() {
            match self.point(item, &at(path, i)) {
                Some(p) => out.push(p),
                None => ok = false,
            }
        }
        ok.then_some(out)
    }

    fn radius(&mut self, v: &Value, path: &str) -> Option<f64> {
        self.number(v, path, "a radius", 0.0, 16.0)
    }

    fn length(&mut self, v: &Value, path: &str, what: &str) -> Option<f64> {
        self.positive(v, path, what, 48.0)
    }

    /// A tuple field: `[a, b, …]` with exactly `n` items.
    fn tuple<'a>(&mut self, v: &'a Value, path: &str, what: &str, n: usize, form: &str) -> Option<&'a Vec<Value>> {
        match v.as_array() {
            Some(items) if items.len() == n => Some(items),
            Some(items) => {
                self.err_hint(path, format!("{what}: {} items, expected {n}", items.len()), format!("write {form}"));
                None
            }
            None => {
                self.err_hint(path, format!("{what} must be an array, got {}", kind(v)), format!("write {form}"));
                None
            }
        }
    }

    fn shape(&mut self, v: &Value, path: &str) -> Option<Shape> {
        let Some(map) = v.as_object() else {
            self.err_hint(path, format!("a shape must be an object, got {}", kind(v)), "for example {\"line\": [[4, 4], [28, 28]]}");
            return None;
        };
        let kinds: Vec<&str> = map.keys().map(String::as_str).filter(|k| SHAPE_KINDS.contains(k)).collect();
        let kind_key = match kinds.as_slice() {
            [one] => *one,
            [] => {
                let unknown = map.keys().find(|k| !matches!(k.as_str(), "r" | "closed" | "by"));
                match unknown.and_then(|k| did_you_mean(k, &SHAPE_KINDS).map(|close| (k, close))) {
                    Some((k, close)) => self.err_hint(&at(path, k), format!("unknown shape `{k}`"), format!("did you mean `{close}`?")),
                    None => self.err_hint(path, "no shape kind", format!("a shape has exactly one of {}", list(&SHAPE_KINDS))),
                }
                return None;
            }
            many => {
                self.err(path, format!("a shape has one kind, found {}", many.join(" and ")));
                return None;
            }
        };
        let field = |k: &str| at(path, k);
        match kind_key {
            "line" => {
                self.object(v, path, "line", &["line"], &["line"])?;
                let pts = self.points(&map["line"], &field("line"), "line", 2, 2)?;
                Some(Shape::Line([pts[0].clone(), pts[1].clone()]))
            }
            "polyline" => {
                self.object(v, path, "polyline", &["polyline", "r"], &["polyline"])?;
                let points = self.points(&map["polyline"], &field("polyline"), "polyline", 2, 96)?;
                let r = match map.get("r") {
                    Some(r) => self.radius(r, &field("r"))?,
                    None => 0.0,
                };
                Some(Shape::Polyline { points, r })
            }
            "rect" => {
                self.object(v, path, "rect", &["rect", "r"], &["rect"])?;
                let items = self.tuple(&map["rect"], &field("rect"), "rect", 4, "[x, y, w, h] from the top-left corner")?;
                let x = self.coord(&items[0], &at(&field("rect"), 0));
                let y = self.coord(&items[1], &at(&field("rect"), 1));
                let w = self.length(&items[2], &at(&field("rect"), 2), "rect width");
                let h = self.length(&items[3], &at(&field("rect"), 3), "rect height");
                let r = match map.get("r") {
                    None => Some(Radii::One(0.0)),
                    Some(Value::Array(items)) => {
                        let rp = field("r");
                        if items.len() != 4 {
                            self.err_hint(&rp, format!("rect r: {} radii, expected 4", items.len()), "write one radius, or [tl, tr, br, bl]");
                            None
                        } else {
                            let radii: Vec<Option<f64>> = items.iter().enumerate().map(|(i, r)| self.radius(r, &at(&rp, i))).collect();
                            radii.iter().all(Option::is_some).then(|| Radii::Four([radii[0].unwrap(), radii[1].unwrap(), radii[2].unwrap(), radii[3].unwrap()]))
                        }
                    }
                    Some(r) => self.radius(r, &field("r")).map(Radii::One),
                };
                Some(Shape::Rect { x: x?, y: y?, w: w?, h: h?, r: r? })
            }
            "circle" => {
                self.object(v, path, "circle", &["circle"], &["circle"])?;
                let items = self.tuple(&map["circle"], &field("circle"), "circle", 2, "[centre, r]")?;
                let c = self.point(&items[0], &at(&field("circle"), 0));
                let r = self.length(&items[1], &at(&field("circle"), 1), "circle radius");
                Some(Shape::Circle { c: c?, r: r? })
            }
            "ellipse" => {
                self.object(v, path, "ellipse", &["ellipse"], &["ellipse"])?;
                let items = self.tuple(&map["ellipse"], &field("ellipse"), "ellipse", 3, "[centre, rx, ry]")?;
                let c = self.point(&items[0], &at(&field("ellipse"), 0));
                let rx = self.length(&items[1], &at(&field("ellipse"), 1), "ellipse rx");
                let ry = self.length(&items[2], &at(&field("ellipse"), 2), "ellipse ry");
                Some(Shape::Ellipse { c: c?, rx: rx?, ry: ry? })
            }
            "arc" => {
                self.object(v, path, "arc", &["arc", "closed"], &["arc"])?;
                let items = self.tuple(&map["arc"], &field("arc"), "arc", 4, "[centre, r, from, to] in degrees, 0 = +x, clockwise")?;
                let c = self.point(&items[0], &at(&field("arc"), 0));
                let r = self.length(&items[1], &at(&field("arc"), 1), "arc radius");
                let from = self.number(&items[2], &at(&field("arc"), 2), "arc from (degrees)", -720.0, 720.0);
                let to = self.number(&items[3], &at(&field("arc"), 3), "arc to (degrees)", -720.0, 720.0);
                let closed = match map.get("closed") {
                    Some(b) => self.boolean(b, &field("closed"))?,
                    None => false,
                };
                if let (Some(a), Some(b)) = (from, to)
                    && ((b - a).abs() < 1e-9 || (b - a).abs() > 360.0)
                {
                    self.err(&field("arc"), "an arc sweeps more than 0 and at most 360 degrees");
                    return None;
                }
                Some(Shape::Arc { c: c?, r: r?, from: from?, to: to?, closed })
            }
            "polygon" => {
                self.object(v, path, "polygon", &["polygon", "r"], &["polygon"])?;
                let pp = field("polygon");
                let items = self.array(&map["polygon"], &pp, "polygon", 3, 96)?;
                let mut vertices = Vec::new();
                let mut ok = true;
                for (i, item) in items.iter().enumerate() {
                    let vp = at(&pp, i);
                    let vertex = if item.is_object() && item.get("from").is_none() {
                        let vm = self.object(item, &vp, "a polygon vertex", &["p", "r", "curve"], &["p"]);
                        vm.and_then(|vm| {
                            let p = self.point(&vm["p"], &at(&vp, "p"));
                            let r = match vm.get("r") {
                                Some(r) => Some(self.radius(r, &at(&vp, "r"))),
                                None => Some(None),
                            };
                            let curve = match vm.get("curve") {
                                Some(c) => self.points(c, &at(&vp, "curve"), "curve", 2, 2).map(|c| Some([c[0].clone(), c[1].clone()])),
                                None => Some(None),
                            };
                            Some(Vertex { p: p?, r: r?, curve: curve? })
                        })
                    } else {
                        self.point(item, &vp).map(|p| Vertex { p, r: None, curve: None })
                    };
                    match vertex {
                        Some(vx) => vertices.push(vx),
                        None => ok = false,
                    }
                }
                let r = match map.get("r") {
                    Some(r) => self.radius(r, &field("r"))?,
                    None => 0.0,
                };
                ok.then_some(Shape::Polygon { vertices, r })
            }
            "wave" => {
                self.object(v, path, "wave", &["wave"], &["wave"])?;
                let wp = field("wave");
                let wm = self.object(&map["wave"], &wp, "wave", &["x", "y", "cycles", "amp", "phase"], &["x", "y", "cycles", "amp"])?;
                let xs = self.tuple(&wm["x"], &at(&wp, "x"), "wave x", 2, "[x0, x1]")?;
                let x0 = self.coord(&xs[0], &at(&at(&wp, "x"), 0));
                let x1 = self.coord(&xs[1], &at(&at(&wp, "x"), 1));
                let y = self.coord(&wm["y"], &at(&wp, "y"));
                let cycles = self.positive(&wm["cycles"], &at(&wp, "cycles"), "cycles", 6.0);
                let amp = self.number(&wm["amp"], &at(&wp, "amp"), "amp", 0.0, 8.0);
                let phase = match wm.get("phase") {
                    Some(p) => self.number(p, &at(&wp, "phase"), "phase (cycles)", -8.0, 8.0),
                    None => Some(0.0),
                };
                Some(Shape::Wave { x: [x0?, x1?], y: y?, cycles: cycles?, amp: amp?, phase: phase? })
            }
            "path" => {
                self.object(v, path, "path", &["path"], &["path"])?;
                let d = self.string(&map["path"], &field("path"), "path")?;
                if d.len() > 8000 {
                    self.err(&field("path"), "path data is longer than 8000 characters");
                    return None;
                }
                if let Err(problem) = check_one_subpath(&d) {
                    self.d.push(
                        Diagnostic::error("ONE_SUBPATH", field("path"), problem)
                            .hint("the engine rejects a second MoveTo; write \"shape\": [ {...}, {...} ] (a list of shapes) instead"),
                    );
                    return None;
                }
                Some(Shape::Path(d))
            }
            "minus" => {
                self.object(v, path, "minus", &["minus"], &["minus"])?;
                let mp = field("minus");
                let items = self.array(&map["minus"], &mp, "minus", 2, 6)?;
                let shapes: Vec<Option<Shape>> = items.iter().enumerate().map(|(i, s)| self.shape(s, &at(&mp, i))).collect();
                shapes.into_iter().collect::<Option<Vec<_>>>().map(Shape::Minus)
            }
            "morph" => {
                self.object(v, path, "morph", &["morph", "by"], &["morph", "by"])?;
                let mp = field("morph");
                let items = self.array(&map["morph"], &mp, "morph", 2, 2)?;
                let a = self.shape(&items[0], &at(&mp, 0));
                let b = self.shape(&items[1], &at(&mp, 1));
                let by = map.get("by").and_then(|b| self.name(b, &field("by"), "morph driver"));
                if let (Some(a), Some(b)) = (&a, &b)
                    && a.kind() != b.kind()
                {
                    self.d.push(Diagnostic::error("MORPH_TOPOLOGY", mp.clone(), format!("a morph needs two shapes of the same kind, got {} and {}", a.kind(), b.kind())));
                    return None;
                }
                Some(Shape::Morph { a: Box::new(a?), b: Box::new(b?), by: by? })
            }
            _ => unreachable!(),
        }
    }

    fn shapes(&mut self, v: &Value, path: &str) -> Option<Vec<Shape>> {
        match v {
            Value::Array(items) => {
                if items.len() < 2 || items.len() > 8 {
                    self.err(path, format!("a list of shapes has 2 to 8 items, got {}", items.len()));
                    return None;
                }
                let shapes: Vec<Option<Shape>> = items.iter().enumerate().map(|(i, s)| self.shape(s, &at(path, i))).collect();
                shapes.into_iter().collect()
            }
            other => self.shape(other, path).map(|s| vec![s]),
        }
    }

    fn parts(&mut self, v: &Value) -> Vec<PartSrc> {
        let mut out = Vec::new();
        let Some(map) = v.as_object() else {
            self.err("/parts", format!("parts must be an object of name: part (back to front), got {}", kind(v)));
            return out;
        };
        if map.is_empty() || map.len() > 40 {
            self.err("/parts", format!("parts: {} parts, expected 1 to 40", map.len()));
        }
        for (name, value) in map {
            let path = at("/parts", name);
            if !is_name(name) {
                self.err(&path, format!("part name `{name}` is not kebab-case"));
                continue;
            }
            if let Some(part) = self.part(name, value, &path) {
                out.push(part);
            }
        }
        out
    }

    fn part(&mut self, name: &str, v: &Value, path: &str) -> Option<PartSrc> {
        const FIELDS: [&str; 12] = ["//", "shape", "mirror", "role", "outline", "tint", "solid", "holes", "knockout", "gap", "hidden", "opacity"];
        let map = self.object(v, path, "a part", &FIELDS, &[])?;
        let body = match (map.get("shape"), map.get("mirror")) {
            (Some(s), None) => PartBody::Shapes(self.shapes(s, &at(path, "shape"))?),
            (None, Some(m)) => {
                let mp = at(path, "mirror");
                let mm = self.object(m, &mp, "mirror", &["of", "x", "y"], &["of"])?;
                let of = self.name(&mm["of"], &at(&mp, "of"), "mirror.of")?;
                let x = match mm.get("x") {
                    Some(x) => Some(self.coord(x, &at(&mp, "x"))?),
                    None => None,
                };
                let y = match mm.get("y") {
                    Some(y) => Some(self.coord(y, &at(&mp, "y"))?),
                    None => None,
                };
                if x.is_some() && y.is_some() {
                    self.err(&mp, "mirror across x or y, not both");
                }
                PartBody::Mirror { of, x, y }
            }
            (Some(_), Some(_)) => {
                self.err(path, "a part has either shape or mirror, not both");
                return None;
            }
            (None, None) => {
                self.err_hint(path, "a part needs a shape", "add \"shape\": {...} or \"mirror\": {\"of\": \"<part>\"}");
                return None;
            }
        };
        let role = match map.get("role") {
            None => None,
            Some(Value::String(s)) => match s.as_str() {
                "stroke" => Some(Role::Stroke),
                "surface" => Some(Role::Surface),
                "fill" => Some(Role::Fill),
                other => {
                    let hint = did_you_mean(other, &["stroke", "surface", "fill"]).map_or("use stroke, surface or fill".to_string(), |c| format!("did you mean `{c}`?"));
                    self.err_hint(&at(path, "role"), format!("unknown role `{other}`"), hint);
                    return None;
                }
            },
            Some(other) => {
                self.err(&at(path, "role"), format!("role must be a string, got {}", kind(other)));
                return None;
            }
        };
        let outline = match map.get("outline") {
            None => None,
            Some(Value::String(s)) if s == "closed" => Some(Outline::Closed),
            Some(Value::String(s)) if s == "open" => Some(Outline::Open),
            Some(Value::String(s)) if s == "none" => Some(Outline::None),
            Some(other) => {
                self.err_hint(&at(path, "outline"), format!("unknown outline {other}"), "use closed, open or none");
                return None;
            }
        };
        let tint = match map.get("tint") {
            Some(b) => Some(self.boolean(b, &at(path, "tint"))?),
            None => None,
        };
        let solid = match map.get("solid") {
            None => None,
            Some(Value::String(s)) => match s.as_str() {
                "auto" => Some(Solid::Auto),
                "ring" => Some(Solid::Ring),
                "fill" => Some(Solid::Fill),
                "none" => Some(Solid::None),
                other => {
                    self.err_hint(&at(path, "solid"), format!("unknown solid `{other}`"), "use auto, ring, fill, none, {\"grow\": d} or a shape");
                    return None;
                }
            },
            Some(Value::Object(o)) if o.contains_key("grow") => {
                let sp = at(path, "solid");
                let om = self.object(&map["solid"], &sp, "solid", &["grow"], &["grow"])?;
                Some(Solid::Grow(self.number(&om["grow"], &at(&sp, "grow"), "grow", -2.0, 3.0)?))
            }
            Some(other) => Some(Solid::Shape(self.shape(other, &at(path, "solid"))?)),
        };
        let holes = match map.get("holes") {
            Some(h) => {
                let hp = at(path, "holes");
                let items = self.array(h, &hp, "holes", 1, 6)?;
                let shapes: Vec<Option<Shape>> = items.iter().enumerate().map(|(i, s)| self.shape(s, &at(&hp, i))).collect();
                Some(shapes.into_iter().collect::<Option<Vec<_>>>()?)
            }
            None => None,
        };
        let knockout = match map.get("knockout") {
            Some(k) => {
                let kp = at(path, "knockout");
                let items = self.array(k, &kp, "knockout", 1, 6)?;
                let mut out = Vec::new();
                for (i, item) in items.iter().enumerate() {
                    let ip = at(&kp, i);
                    if item.is_object() {
                        let km = self.object(item, &ip, "a knockout", &["part", "gap"], &["part"])?;
                        let part = self.name(&km["part"], &at(&ip, "part"), "knockout part")?;
                        let gap = match km.get("gap") {
                            Some(g) => Some(self.number(g, &at(&ip, "gap"), "gap", 0.0, 4.0)?),
                            None => None,
                        };
                        out.push(Knockout { part, gap });
                    } else {
                        out.push(Knockout { part: self.name(item, &ip, "knockout part")?, gap: None });
                    }
                }
                Some(out)
            }
            None => None,
        };
        let (knockout_gap, gaps) = match map.get("gap") {
            None => (None, None),
            Some(Value::Number(_)) => (Some(self.number(&map["gap"], &at(path, "gap"), "gap", 0.0, 4.0)?), None),
            Some(Value::Object(o)) => {
                let gp = at(path, "gap");
                let mut gaps = Vec::new();
                for (other, d) in o {
                    if !is_name(other) {
                        self.err(&at(&gp, other), format!("`{other}` is not a part name"));
                        continue;
                    }
                    if let Some(d) = self.number(d, &at(&gp, other), "gap distance", 0.0, 6.0) {
                        gaps.push((other.clone(), d));
                    }
                }
                (None, Some(gaps))
            }
            Some(other) => {
                self.err_hint(&at(path, "gap"), format!("gap must be a number or an object, got {}", kind(other)), "a number is the knockout clearance; {\"<part>\": d} cuts this part within d of that part");
                return None;
            }
        };
        let hidden = match map.get("hidden") {
            Some(b) => Some(self.boolean(b, &at(path, "hidden"))?),
            None => None,
        };
        let opacity = match map.get("opacity") {
            Some(v) => {
                let o = self.number(v, &at(path, "opacity"), "opacity (the part's rest opacity)", 0.0, 1.0)?;
                if o == 0.0 || hidden == Some(true) {
                    self.err_hint(&at(path, "opacity"), "a part resting at opacity 0 is hidden", "use \"hidden\": true, and leave out opacity");
                    return None;
                }
                Some(o)
            }
            None => None,
        };
        Some(PartSrc { name: name.to_string(), path: path.to_string(), body, role, outline, tint, solid, holes, knockout, knockout_gap, gaps, hidden, opacity })
    }

    fn rigs(&mut self, v: &Value) -> Vec<RigSrc> {
        let mut out = Vec::new();
        let Some(map) = v.as_object() else {
            self.err("/rigs", format!("rigs must be an object of name: rig, got {}", kind(v)));
            return out;
        };
        for (name, value) in map {
            let path = at("/rigs", name);
            if !is_name(name) {
                self.err(&path, format!("rig name `{name}` is not kebab-case"));
                continue;
            }
            let Some(rm) = self.object(value, &path, "a rig", &["//", "pivot", "carries", "swing", "flip", "squash"], &[]) else { continue };
            let pivot = match rm.get("pivot") {
                None => None,
                Some(Value::Object(o)) if o.contains_key("contact") => {
                    let pp = at(&path, "pivot");
                    let Some(om) = self.object(&rm["pivot"], &pp, "a contact pivot", &["contact"], &["contact"]) else { continue };
                    let Some(pts) = self.points(&om["contact"], &at(&pp, "contact"), "contact", 2, 2) else { continue };
                    Some(Pivot::Contact(pts[0].clone(), pts[1].clone()))
                }
                Some(p) => match self.point(p, &at(&path, "pivot")) {
                    Some(p) => Some(Pivot::Point(p)),
                    None => continue,
                },
            };
            let carries = match rm.get("carries") {
                Some(c) => {
                    let cp = at(&path, "carries");
                    let Some(items) = self.array(c, &cp, "carries", 1, usize::MAX) else { continue };
                    let mut names = Vec::new();
                    for (i, item) in items.iter().enumerate() {
                        if let Some(n) = self.name(item, &at(&cp, i), "a carried name") {
                            if names.contains(&n) {
                                self.err(&at(&cp, i), format!("`{n}` is listed twice"));
                            }
                            names.push(n);
                        }
                    }
                    names
                }
                None => Vec::new(),
            };
            let swing = rm.contains_key("swing");
            if let Some(s) = rm.get("swing") {
                let sp = at(&path, "swing");
                if let Some(sm) = self.object(s, &sp, "swing", &["hinge", "front", "occludes", "fade"], &["hinge"]) {
                    self.point(&sm["hinge"], &at(&sp, "hinge"));
                }
            }
            let flip = rm.contains_key("flip");
            if let Some(f) = rm.get("flip") {
                let fp = at(&path, "flip");
                if let Some(fm) = self.object(f, &fp, "flip", &["axis", "back"], &["axis"]) {
                    self.coord(&fm["axis"], &at(&fp, "axis"));
                }
            }
            let squash = match rm.get("squash") {
                Some(s) => {
                    let sp = at(&path, "squash");
                    let Some(sm) = self.object(s, &sp, "squash", &["base", "widen"], &["base"]) else { continue };
                    let base = self.coord(&sm["base"], &at(&sp, "base"));
                    let widen = match sm.get("widen") {
                        Some(w) => self.number(w, &at(&sp, "widen"), "widen", 0.0, 2.0),
                        None => Some(0.5),
                    };
                    match (base, widen) {
                        (Some(base), Some(widen)) => Some(Squash { base, widen }),
                        _ => continue,
                    }
                }
                None => None,
            };
            out.push(RigSrc { name: name.clone(), path, pivot, carries, swing, flip, squash });
        }
        out
    }

    fn target(&mut self, s: &str, path: &str) -> Option<Target> {
        let (name, prop) = match s.split_once('.') {
            Some((name, prop)) => (name, Some(prop)),
            None => (s, None),
        };
        if !is_name(name) {
            self.err_hint(path, format!("`{s}` is not a motion target"), "write <part or rig>.<property>, e.g. bell.rotate, or a bare morph driver name");
            return None;
        }
        let prop = match prop {
            None => None,
            Some(p) => match Prop::parse(p) {
                Some(p) => Some(p),
                None => {
                    let hint = did_you_mean(p, &PROPS).map_or(format!("properties: {}", list(&PROPS)), |c| format!("did you mean `{name}.{c}`?"));
                    self.err_hint(path, format!("unknown motion property `{p}`"), hint);
                    return None;
                }
            },
        };
        Some(Target { name: name.to_string(), prop })
    }

    fn motion(&mut self, v: &Value) -> Vec<MotionEntry> {
        let mut out = Vec::new();
        let Some(map) = v.as_object() else {
            self.err("/motion", format!("motion must be an object of target: track, got {}", kind(v)));
            return out;
        };
        if map.is_empty() {
            self.err_hint("/motion", "motion is empty", "key at least one track, e.g. \"bell.rotate\": {\"light\": [[0, 0], [0.2, -8], [0.5, 0]]}");
        }
        for (key, value) in map {
            if key == "//" {
                continue;
            }
            let path = at("/motion", key);
            let Some(target) = self.target(key, &path) else { continue };
            let track = if value.get("follow").is_some() {
                match self.follow(value, &path) {
                    Some(f) => TrackSrc::Follow(f),
                    None => continue,
                }
            } else {
                let Some(tm) = self.object(value, &path, "a track", &["//", "light", "strong"], &[]) else { continue };
                if !tm.contains_key("light") && !tm.contains_key("strong") {
                    self.err_hint(&path, "a track needs light or strong keys", "write {\"light\": [[t, v], …], \"strong\": [[t, v], …]}, or {\"follow\": \"<target>\"}");
                    continue;
                }
                let light = tm.get("light").and_then(|v| {
                    let p = at(&path, "light");
                    self.motion_value(v, &p).map(|m| (m, p))
                });
                let strong = tm.get("strong").and_then(|v| {
                    let p = at(&path, "strong");
                    self.motion_value(v, &p).map(|m| (m, p))
                });
                TrackSrc::PerMotion { light, strong }
            };
            out.push(MotionEntry { target, path, track });
        }
        out
    }

    fn follow(&mut self, v: &Value, path: &str) -> Option<Follow> {
        let map = self.object(v, path, "follow", &["//", "follow", "lag", "scale", "offset"], &["follow"])?;
        let raw = self.string(&map["follow"], &at(path, "follow"), "follow")?;
        let target = self.target(&raw, &at(path, "follow"))?;
        let lag = match map.get("lag") {
            Some(l) => self.number(l, &at(path, "lag"), "lag", 0.0, 1.0)?,
            None => 0.0,
        };
        let scale = match map.get("scale") {
            Some(s) => self.number(s, &at(path, "scale"), "scale", -4.0, 4.0)?,
            None => 1.0,
        };
        let offset = match map.get("offset") {
            Some(o) => self.number(o, &at(path, "offset"), "offset", -1080.0, 1080.0)?,
            None => 0.0,
        };
        Some(Follow { target, lag, scale, offset })
    }

    fn motion_value(&mut self, v: &Value, path: &str) -> Option<MotionValue> {
        match v {
            Value::Array(_) => self.items(v, path).map(MotionValue::Keys),
            Value::Object(o) if o.contains_key("pulse") => {
                let map = self.object(v, path, "pulse", &["pulse", "r", "grow"], &["pulse"])?;
                let pp = at(path, "pulse");
                let items = self.tuple(&map["pulse"], &pp, "pulse", 2, "[start, duration]")?;
                let start = self.time(&items[0], &at(&pp, 0));
                let dur = self.positive(&items[1], &at(&pp, 1), "pulse duration", DURATION);
                let r = match map.get("r") {
                    Some(r) => Some(self.number(r, &at(path, "r"), "r", -8.0, 8.0)?),
                    None => None,
                };
                let grow = match map.get("grow") {
                    Some(g) => Some(self.number(g, &at(path, "grow"), "grow", -8.0, 8.0)?),
                    None => None,
                };
                Some(MotionValue::Pulse { start: start?, dur: dur?, r, grow })
            }
            Value::Object(o) if o.contains_key("loop") => {
                let map = self.object(v, path, "loop", &["loop", "period", "until"], &["loop", "period"])?;
                let keys = self.items(&map["loop"], &at(path, "loop"));
                let period = self.positive(&map["period"], &at(path, "period"), "period", DURATION);
                let until = match map.get("until") {
                    Some(u) => Some(self.time(u, &at(path, "until"))?),
                    None => None,
                };
                Some(MotionValue::Loop { keys: keys?, period: period?, until })
            }
            Value::Object(o) if o.contains_key("follow") => self.follow(v, path).map(MotionValue::Follow),
            other => {
                self.err_hint(path, format!("a motion value is a key list, pulse, loop or follow; got {}", kind(other)), "write [[t, value], [t, value, ease], …]");
                None
            }
        }
    }

    fn items(&mut self, v: &Value, path: &str) -> Option<Items> {
        let items = self.array(v, path, "a key list", 1, 40)?;
        let mut out = Vec::new();
        let mut ok = true;
        for (i, item) in items.iter().enumerate() {
            let ip = at(path, i);
            match self.item(item, &ip) {
                Some(k) => out.push((k, ip)),
                None => ok = false,
            }
        }
        ok.then_some(out)
    }

    fn value(&mut self, v: &Value, path: &str) -> Option<f64> {
        self.number(v, path, "a value", -1080.0, 1080.0)
    }

    fn item(&mut self, v: &Value, path: &str) -> Option<KeyItem> {
        match v {
            Value::Array(items) => {
                if items.len() < 2 || items.len() > 3 {
                    self.err_hint(path, format!("a key has 2 or 3 items, got {}", items.len()), "write [t, value] or [t, value, ease]");
                    return None;
                }
                let t = self.time(&items[0], &at(path, 0));
                let value = self.value(&items[1], &at(path, 1));
                let ease = match items.get(2) {
                    Some(e) => self.ease(e, &at(path, 2)),
                    None => Some(Ease::Auto),
                };
                Some(KeyItem::Key { t: t?, v: value?, ease: ease? })
            }
            Value::Object(o) => {
                const PRESETS: [&str; 4] = ["hold", "settle", "bounce", "wobble"];
                let kinds: Vec<&str> = o.keys().map(String::as_str).filter(|k| PRESETS.contains(k)).collect();
                match kinds.as_slice() {
                    ["hold"] => {
                        let map = self.object(v, path, "hold", &["hold"], &["hold"])?;
                        Some(KeyItem::Hold { t: self.time(&map["hold"], &at(path, "hold"))? })
                    }
                    ["settle"] => {
                        let map = self.object(v, path, "settle", &["settle", "to", "amp", "decay", "swings"], &["settle"])?;
                        let t = self.time(&map["settle"], &at(path, "settle"));
                        let to = match map.get("to") {
                            Some(to) => Some(self.value(to, &at(path, "to"))?),
                            None => None,
                        };
                        let amp = match map.get("amp") {
                            Some(a) => self.positive(a, &at(path, "amp"), "amp", 1.0)?,
                            None => 0.3,
                        };
                        let decay = match map.get("decay") {
                            Some(d) => match d.as_f64() {
                                Some(x) if x > 0.0 && x < 1.0 => x,
                                _ => {
                                    self.err(&at(path, "decay"), "decay must be above 0 and below 1");
                                    return None;
                                }
                            },
                            None => 0.5,
                        };
                        let swings = match map.get("swings") {
                            Some(s) => self.integer(s, &at(path, "swings"), 1, 4)?,
                            None => 2,
                        };
                        Some(KeyItem::Settle { t: t?, to, amp, decay, swings })
                    }
                    ["bounce"] => {
                        let map = self.object(v, path, "bounce", &["bounce", "amp"], &["bounce"])?;
                        let t = self.time(&map["bounce"], &at(path, "bounce"));
                        let amp = match map.get("amp") {
                            Some(a) => self.positive(a, &at(path, "amp"), "amp", 1.0)?,
                            None => 0.12,
                        };
                        Some(KeyItem::Bounce { t: t?, amp })
                    }
                    ["wobble"] => {
                        let map = self.object(v, path, "wobble", &["wobble", "amp", "until", "swings"], &["wobble", "amp", "until"])?;
                        let at_t = self.time(&map["wobble"], &at(path, "wobble"));
                        let amp = self.positive(&map["amp"], &at(path, "amp"), "amp", 90.0);
                        let until = self.time(&map["until"], &at(path, "until"));
                        let swings = match map.get("swings") {
                            Some(s) => self.integer(s, &at(path, "swings"), 1, 6)?,
                            None => 3,
                        };
                        Some(KeyItem::Wobble { at: at_t?, amp: amp?, until: until?, swings })
                    }
                    [] => {
                        let first = o.keys().next().cloned().unwrap_or_default();
                        let hint = did_you_mean(&first, &PRESETS).map_or(format!("presets: {}", list(&PRESETS)), |c| format!("did you mean `{c}`?"));
                        self.err_hint(path, format!("unknown key preset `{first}`"), hint);
                        None
                    }
                    many => {
                        self.err(path, format!("one preset per item, found {}", many.join(" and ")));
                        None
                    }
                }
            }
            other => {
                self.err_hint(path, format!("a key is [t, value, ease?] or a preset object, got {}", kind(other)), "for example [0.2, -8, \"hit\"] or {\"settle\": 0.6}");
                None
            }
        }
    }

    fn integer(&mut self, v: &Value, path: &str, lo: u64, hi: u64) -> Option<u32> {
        match v.as_u64() {
            Some(n) if n >= lo && n <= hi => Some(n as u32),
            _ => {
                self.err(path, format!("expected a whole number from {lo} to {hi}, got {v}"));
                None
            }
        }
    }

    fn ease(&mut self, v: &Value, path: &str) -> Option<Ease> {
        match v {
            Value::String(s) => Some(match s.as_str() {
                "auto" => Ease::Auto,
                "linear" => Ease::Linear,
                "in" => Ease::In,
                "out" => Ease::Out,
                "inOut" | "smooth" => Ease::InOut,
                "sine" => Ease::Sine,
                "hit" => Ease::Hit,
                "launch" => Ease::Launch,
                "pass" => Ease::Pass,
                "step" => Ease::Step,
                other => {
                    let hint = did_you_mean(other, &EASES).map_or(format!("eases: {}", list(&EASES)), |c| format!("did you mean `{c}`?"));
                    self.err_hint(path, format!("unknown ease `{other}`"), hint);
                    return None;
                }
            }),
            Value::Array(items) => {
                if items.len() != 4 {
                    self.err_hint(path, format!("a bezier ease has 4 numbers, got {}", items.len()), "write [x1, y1, x2, y2] with x in 0..1");
                    return None;
                }
                let x1 = self.number(&items[0], &at(path, 0), "bezier x1", 0.0, 1.0);
                let y1 = self.number(&items[1], &at(path, 1), "bezier y1", -2.0, 3.0);
                let x2 = self.number(&items[2], &at(path, 2), "bezier x2", 0.0, 1.0);
                let y2 = self.number(&items[3], &at(path, 3), "bezier y2", -2.0, 3.0);
                Some(Ease::Bezier([x1?, y1?, x2?, y2?]))
            }
            other => {
                self.err(path, format!("an ease is a name or [x1, y1, x2, y2], got {}", kind(other)));
                None
            }
        }
    }

    fn reverse(&mut self, v: &Value) -> (Vec<Override>, Vec<Override>) {
        let mut light = Vec::new();
        let mut strong = Vec::new();
        let Some(map) = self.object(v, "/reverse", "reverse", &["//", "light", "strong"], &[]) else { return (light, strong) };
        for (which, out) in [("light", &mut light), ("strong", &mut strong)] {
            let Some(o) = map.get(which) else { continue };
            let path = at("/reverse", which);
            let Some(om) = o.as_object() else {
                self.err(&path, "overrides must be an object of target: keys");
                continue;
            };
            for (key, keys) in om {
                if key == "//" {
                    continue;
                }
                let kp = at(&path, key);
                let Some(target) = self.target(key, &kp) else { continue };
                if let Some(items) = self.items(keys, &kp) {
                    out.push(Override { target, path: kp, keys: items });
                }
            }
        }
        (light, strong)
    }

    fn timing(&mut self, v: &Value) -> Timing {
        let mut t = Timing::default();
        let Some(map) = self.object(v, "/timing", "timing", &["light", "strong", "reverse", "reverseStrong"], &[]) else { return t };
        for (key, slot) in [("light", &mut t.light), ("strong", &mut t.strong), ("reverse", &mut t.reverse), ("reverseStrong", &mut t.reverse_strong)] {
            if let Some(v) = map.get(key) {
                *slot = self.positive(v, &at("/timing", key), "an action length (seconds)", MAX_ACTIVE);
            }
        }
        t
    }

    fn preview(&mut self, v: &Value) -> Preview {
        let mut p = Preview::default();
        let Some(map) = self.object(v, "/preview", "preview", &["light", "strong"], &[]) else { return p };
        for (key, slot) in [("light", &mut p.light), ("strong", &mut p.strong)] {
            if let Some(v) = map.get(key) {
                let path = at("/preview", key);
                if let Some(items) = self.array(v, &path, "preview times", 2, 8) {
                    let times: Vec<Option<f64>> = items.iter().enumerate().map(|(i, t)| self.time(t, &at(&path, i))).collect();
                    *slot = times.into_iter().collect();
                }
            }
        }
        p
    }
}

/// `[a-z][a-z0-9]*(-[a-z0-9]+)*`, at most 32 characters.
pub fn is_name(s: &str) -> bool {
    s.len() <= 32
        && s.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && s.split('-').all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()))
}

/// Derived anchor suffixes.
pub const DERIVED: [&str; 11] = ["c", "top", "bottom", "left", "right", "tl", "tr", "bl", "br", "start", "end"];

fn is_anchor_ref(s: &str) -> bool {
    match s.split_once('.') {
        Some((name, suffix)) => is_name(name) && DERIVED.contains(&suffix),
        None => is_name(s),
    }
}

/// One subpath: a single M/m at the start and Z/z only at the end.
fn check_one_subpath(d: &str) -> Result<(), String> {
    let trimmed = d.trim();
    if !trimmed.starts_with(['M', 'm']) {
        return Err("path data must start with M".into());
    }
    let moves = trimmed.matches(['M', 'm']).count();
    if moves > 1 {
        return Err(format!("path has {moves} subpaths (MoveTo commands); one path holds one subpath"));
    }
    if let Some(z) = trimmed.find(['Z', 'z'])
        && !trimmed[z + 1..].trim().is_empty()
    {
        return Err("Z must come last: a Z before the end starts a second subpath".into());
    }
    Ok(())
}

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

fn list(items: &[&str]) -> String {
    items.iter().map(|i| format!("`{i}`")).collect::<Vec<_>>().join(", ")
}

/// A number as JSON prints it.
pub fn fmt(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1e15 { format!("{}", x as i64) } else { format!("{x}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn errors(text: &str) -> Vec<String> {
        match parse(text) {
            Ok(_) => Vec::new(),
            Err(d) => d.0.iter().map(|d| format!("{} {}: {}{}", d.code, d.path, d.message, d.hint.as_ref().map(|h| format!(" ({h})")).unwrap_or_default())).collect(),
        }
    }

    const HEAD: &str = r#""format": "gai-icon/2", "id": "x", "label": "X", "intent": "Light: nothing much. Strong: a bit more.""#;

    fn spec(parts: &str, motion: &str) -> String {
        format!(r#"{{{HEAD}, "parts": {parts}, "motion": {motion}}}"#)
    }

    #[test]
    fn examples_parse() {
        for name in ["alarm-clock", "late-checkout", "in-room-safe"] {
            let text = std::fs::read_to_string(format!("{}/tests/fixtures/{name}.icon.json", env!("CARGO_MANIFEST_DIR"))).unwrap();
            let spec = parse(&text).unwrap_or_else(|d| panic!("{name}: {d:?}"));
            assert_eq!(spec.id, name);
        }
    }

    #[test]
    fn typos_get_suggestions() {
        let e = errors(&spec(r#"{"a": {"shape": {"cirle": [[16, 16], 4]}}}"#, r#"{"a.opacity": {"light": [[0, 1]]}}"#));
        assert_eq!(e, ["V2_SCHEMA /parts/a/shape/cirle: unknown shape `cirle` (did you mean `circle`?)"]);
        let e = errors(&spec(r#"{"a": {"shape": {"line": [[1, 1], [2, 2]]}}}"#, r#"{"a.rotat": {"light": [[0, 0]]}}"#));
        assert_eq!(e, ["V2_SCHEMA /motion/a.rotat: unknown motion property `rotat` (did you mean `a.rotate`?)"]);
        let e = errors(&spec(r#"{"a": {"shape": {"line": [[1, 1], [2, 2]]}}}"#, r#"{"a.x": {"light": [[0, 0], [0.2, 1, "hti"]]}}"#));
        assert_eq!(e, ["V2_SCHEMA /motion/a.x/light/1/2: unknown ease `hti` (did you mean `hit`?)"]);
    }

    #[test]
    fn rejects_what_the_schema_rejects() {
        let two = errors(&spec(r#"{"feet": {"shape": {"path": "M10 23 L10 26 M21 23 L21 26"}}}"#, r#"{"feet.y": {"light": [[0, 0]]}}"#));
        assert_eq!(two.len(), 1);
        assert!(two[0].starts_with("ONE_SUBPATH /parts/feet/shape/path: path has 2 subpaths"), "{two:?}");
        let early = errors(&spec(r#"{"a": {"shape": {"path": "M1 1 L2 2 Z L3 3"}}}"#, r#"{"a.y": {"light": [[0, 0]]}}"#));
        assert!(early[0].starts_with("ONE_SUBPATH"), "{early:?}");
        let handle = errors(&spec(r#"{"a": {"shape": {"line": [[1, 1], [2, 2]]}}}"#, r#"{"a.x": {"light": [[0, 0], [0.2, 1, [1.4, 0, 0.6, 1]]]}}"#));
        assert_eq!(handle, ["V2_SCHEMA /motion/a.x/light/1/2/0: bezier x1 must be from 0 to 1, got 1.4"]);
        let late = errors(&spec(r#"{"a": {"shape": {"line": [[1, 1], [2, 2]]}}}"#, r#"{"a.x": {"light": [[0, 0], [2.5, 1]]}}"#));
        assert_eq!(late, ["V2_SCHEMA /motion/a.x/light/1/0: a time (seconds) must be from 0 to 2.4, got 2.5"]);
        let field = errors(&spec(r#"{"a": {"shape": {"line": [[1, 1], [2, 2]]}, "pose": "t => t"}}"#, r#"{"a.x": {"light": [[0, 0]]}}"#));
        assert!(field[0].contains("unknown field `pose` in a part"), "{field:?}");
        let role = errors(&spec(r#"{"a": {"shape": {"line": [[1, 1], [2, 2]]}, "role": "outline"}}"#, r#"{"a.x": {"light": [[0, 0]]}}"#));
        assert!(role[0].contains("unknown role `outline`"), "{role:?}");
    }

    #[test]
    fn polygons_take_96_vertices_and_polar_points() {
        let gear: Vec<String> = (0..30).map(|i| format!(r#"{{"from": [16, 16], "angle": {}, "len": 10}}"#, i * 12)).collect();
        let text = spec(&format!(r#"{{"gear": {{"shape": {{"polygon": [{}]}}, "role": "surface"}}}}"#, gear.join(", ")), r#"{"gear.rotate": {"light": [[0, 0]]}}"#);
        assert!(errors(&text).is_empty(), "{:?}", errors(&text));
    }
}
