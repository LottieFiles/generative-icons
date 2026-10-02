//! The compiled plan (`gai-plan/1`): the only input the engine script reads. Engine units:
//! radians counter-clockwise, opacity and trim 0–1, seconds; every number rounded to 6
//! decimals. Rigs come parent first, layers back to front.

use crate::ease::Handle;
use crate::geom::{Contour, Geo, r6};
use indexmap::IndexMap;
use serde_json::{Map, Value, json};

pub const FORMAT: &str = "gai-plan/1";
/// `#00ddb3`, the colour every node is drawn in; slots recolour it.
pub const COLOR: [f64; 3] = [0.0, 0.866667, 0.701961];

/// A composition-level group with a constant pivot, chained by layer parenting.
#[derive(Debug, Clone, PartialEq)]
pub struct Rig {
    pub name: String,
    pub pivot: (f64, f64),
    pub parent: Option<String>,
    /// Position keyed per axis (`splitPosition`).
    pub split: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Paint {
    Stroke(f64),
    Fill { even_odd: bool },
}

/// A subtract mask. `geo` names the geometry whose path keys it follows (a hole of a part
/// that scale or squash reshapes); a mask without one is static.
#[derive(Debug, Clone, PartialEq)]
pub struct Mask {
    pub contour: Contour,
    pub geo: Option<String>,
}

impl Mask {
    pub fn fixed(contour: Contour) -> Mask {
        Mask { contour, geo: None }
    }
}

/// One composition-level group: a part in one style (or a matte source).
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    /// `<id>-<part>-<variant>`; the contract suffix is added for painted layers.
    pub name: String,
    pub part: String,
    pub variant: String,
    /// `outline.light|regular|bold`, `solid`, `tint`; `None` for a matte source.
    pub style: Option<&'static str>,
    pub rig: Option<String>,
    pub geos: Vec<String>,
    pub paints: Vec<Paint>,
    /// Subtract masks, in the layer's own space.
    pub masks: Vec<Mask>,
    /// Track address of the group opacity.
    pub vis: Option<String>,
    /// Track address of a trim on the layer's paths.
    pub trim: Option<String>,
    /// Static group opacity (0 for a part hidden at rest).
    pub opacity: Option<f64>,
    /// Inverted-alpha matte source, created directly above this layer.
    pub matte: Option<Box<Layer>>,
}

impl Layer {
    pub fn full_name(&self) -> String {
        match self.style {
            Some(style) => format!("{}__color.primary__{style}", self.name),
            None => self.name.clone(),
        }
    }
}

/// Engine properties a plan keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Prop {
    Rotation,
    PositionX,
    PositionY,
    Opacity,
    PathPoints,
    TrimEnd,
}

impl Prop {
    pub fn as_str(self) -> &'static str {
        match self {
            Prop::Rotation => "Rotation",
            Prop::PositionX => "PositionX",
            Prop::PositionY => "PositionY",
            Prop::Opacity => "Opacity",
            Prop::PathPoints => "PathPoints",
            Prop::TrimEnd => "TrimEnd",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyValue {
    Scalar(f64),
    Points(Vec<f64>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PKey {
    pub t: f64,
    pub v: KeyValue,
    pub out: Option<Handle>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    /// A rig name, a geometry id, `<part>.opacity` or `<part>.draw`.
    pub target: String,
    pub prop: Prop,
    pub keys: Vec<PKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Motion {
    pub name: String,
    pub active: f64,
    pub rest: f64,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub id: String,
    pub rigs: Vec<Rig>,
    pub geos: IndexMap<String, Geo>,
    pub layers: Vec<Layer>,
    pub motions: Vec<Motion>,
}

fn contour_json(c: &Contour) -> Value {
    json!({"c": c.commands(), "p": c.flat_points().into_iter().map(r6).collect::<Vec<_>>()})
}

fn mask_json(m: &Mask) -> Value {
    let mut v = contour_json(&m.contour);
    if let Some(geo) = &m.geo {
        v["geo"] = json!(geo);
    }
    v
}

fn layer_json(l: &Layer) -> Value {
    let mut m = Map::new();
    m.insert("name".into(), json!(l.full_name()));
    m.insert("style".into(), json!(l.style));
    if let Some(rig) = &l.rig {
        m.insert("rig".into(), json!(rig));
    }
    m.insert("geos".into(), json!(l.geos));
    let paints: Vec<Value> = l
        .paints
        .iter()
        .map(|p| match p {
            Paint::Stroke(w) => json!(["stroke", r6(*w)]),
            Paint::Fill { even_odd: false } => json!(["fill"]),
            Paint::Fill { even_odd: true } => json!(["fill", 1]),
        })
        .collect();
    m.insert("paints".into(), Value::Array(paints));
    if !l.masks.is_empty() {
        m.insert("masks".into(), Value::Array(l.masks.iter().map(mask_json).collect()));
    }
    if let Some(vis) = &l.vis {
        m.insert("vis".into(), json!(vis));
    }
    if let Some(trim) = &l.trim {
        m.insert("trim".into(), json!(trim));
    }
    if let Some(o) = l.opacity {
        m.insert("opacity".into(), json!(r6(o)));
    }
    if let Some(matte) = &l.matte {
        m.insert("matte".into(), layer_json(matte));
    }
    Value::Object(m)
}

fn handle_json(h: Option<Handle>) -> Option<Value> {
    match h? {
        Handle::Hold => Some(Value::Null),
        Handle::Bez(b) => Some(json!(b.map(r6))),
    }
}

impl Plan {
    /// The plan as the engine script's `params.plan`.
    pub fn to_json(&self) -> Value {
        let rigs: Vec<Value> = self
            .rigs
            .iter()
            .map(|r| {
                let mut m = Map::new();
                m.insert("name".into(), json!(r.name));
                m.insert("pivot".into(), json!([r6(r.pivot.0), r6(r.pivot.1)]));
                if let Some(p) = &r.parent {
                    m.insert("parent".into(), json!(p));
                }
                if r.split {
                    m.insert("split".into(), json!(true));
                }
                Value::Object(m)
            })
            .collect();
        let geos: Map<String, Value> = self.geos.iter().map(|(id, g)| (id.clone(), g.to_plan())).collect();
        let motions: Vec<Value> = self
            .motions
            .iter()
            .map(|m| {
                let tracks: Vec<Value> = m
                    .tracks
                    .iter()
                    .map(|t| {
                        let keys: Vec<Value> = t
                            .keys
                            .iter()
                            .map(|k| {
                                let v = match &k.v {
                                    KeyValue::Scalar(x) => json!(r6(*x)),
                                    KeyValue::Points(p) => json!(p.iter().map(|x| r6(*x)).collect::<Vec<_>>()),
                                };
                                match handle_json(k.out) {
                                    Some(h) => json!([r6(k.t), v, h]),
                                    None => json!([r6(k.t), v]),
                                }
                            })
                            .collect();
                        json!({"target": t.target, "prop": t.prop.as_str(), "keys": keys})
                    })
                    .collect();
                json!({"name": m.name, "active": r6(m.active), "rest": r6(m.rest), "tracks": tracks})
            })
            .collect();
        json!({
            "format": FORMAT,
            "id": self.id,
            "w": 32,
            "h": 32,
            "fps": 60,
            "duration": 2.4,
            "color": COLOR,
            "rigs": rigs,
            "geos": Value::Object(geos),
            "layers": self.layers.iter().map(layer_json).collect::<Vec<_>>(),
            "motions": motions,
        })
    }

    /// Layers in the order the engine creates them (each matte source right after its target).
    pub fn created_layers(&self) -> Vec<&Layer> {
        let mut out = Vec::new();
        for l in &self.layers {
            out.push(l);
            if let Some(m) = &l.matte {
                out.push(m);
            }
        }
        out
    }

    /// Exported layer count: rigs plus every created layer.
    pub fn layer_count(&self) -> usize {
        self.rigs.len() + self.created_layers().len()
    }
}
