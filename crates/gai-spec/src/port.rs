//! Ports a v1 library icon's drawing to a spec: the Light animation at frame 0, which is
//! the rest pose (a stateful icon's start state). It is the start of a reanimation, so the
//! spec has no motion yet (`check` reports that `motion` is empty until a track has keys).
//!
//! v1 exports one layer per part and style; each paint's opacity slot names its style
//! (`style.outline.regular`, `style.solid`, `style.tint`, …). The Regular outline layers
//! become parts, in draw order. A tint or Solid layer with the same geometry as a part
//! becomes that part's tint or Solid; any other one becomes a part of its own that draws
//! only in that style, so every style keeps v1's exact shapes.

use serde_json::{Map, Value, json};

/// The library facts a port needs beside the animation.
#[derive(Debug, Clone, Default)]
pub struct Icon {
    pub id: String,
    pub label: String,
    pub intent: String,
    /// `(from, to, labels)` of a stateful icon.
    pub state: Option<(String, String, [String; 2])>,
    /// A preview reset (`fade`) instead of a reverse.
    pub preview_reset: bool,
}

/// A ported spec and what to check by hand.
#[derive(Debug, Clone)]
pub struct Port {
    pub spec: Value,
    pub notes: Vec<String>,
}

/// One layer's drawing at frame 0.
#[derive(Debug, Clone)]
struct Layer {
    /// The layer name without the icon id.
    name: String,
    /// Index in the file (0 is the front).
    index: usize,
    paths: Vec<String>,
    visible: bool,
    /// Rest opacity (0 to 1) of a visible layer.
    opacity: f64,
    styles: Vec<(String, Paint)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Paint {
    Stroke(f64),
    Fill { even_odd: bool },
}

/// A part being assembled.
#[derive(Debug, Clone)]
struct Part {
    name: String,
    index: usize,
    paths: Vec<String>,
    role: &'static str,
    /// `none` for a part that draws only in tint or Solid.
    outline_none: bool,
    tint: bool,
    solid: Option<Value>,
    hidden: bool,
    opacity: f64,
}

/// Ports `lottie` (the icon's Light animation) to a spec.
pub fn port(lottie: &Value, icon: &Icon) -> Result<Port, String> {
    let mut notes = Vec::new();
    let (w, h) = (lottie.get("w").and_then(Value::as_f64).unwrap_or(32.0), lottie.get("h").and_then(Value::as_f64).unwrap_or(32.0));
    if (w, h) != (32.0, 32.0) {
        return Err(format!("the animation is {w}×{h}, not the 32-unit grid"));
    }
    let layers: Vec<Layer> = lottie
        .get("layers")
        .and_then(Value::as_array)
        .ok_or("the animation has no layers")?
        .iter()
        .enumerate()
        .filter(|(_, l)| l.get("ty").and_then(Value::as_i64) == Some(4))
        .map(|(index, l)| read_layer(l, index, &icon.id, &mut notes))
        .collect::<Result<_, _>>()?;

    let style_of = |l: &Layer, style: &str| l.styles.iter().find(|(s, _)| s == style).map(|(_, p)| *p);
    let mut parts: Vec<Part> = Vec::new();
    for l in &layers {
        let Some(paint) = style_of(l, "style.outline.regular") else { continue };
        let (name, frame) = match l.name.strip_suffix("-regular") {
            Some(base) => match base.strip_suffix("-frame") {
                Some(part) => (part.to_string(), true),
                None => (base.to_string(), false),
            },
            None => (l.name.clone(), false),
        };
        let role = match (paint, frame) {
            (Paint::Fill { .. }, _) => "fill",
            (Paint::Stroke(_), true) if all_closed(&l.paths) => "surface",
            (Paint::Stroke(_), _) => "stroke",
        };
        parts.push(Part { name, index: l.index, paths: l.paths.clone(), role, outline_none: false, tint: false, solid: None, hidden: !l.visible, opacity: l.opacity });
    }
    for weight in ["light", "bold"] {
        let style = format!("style.outline.{weight}");
        for l in layers.iter().filter(|l| style_of(l, &style).is_some()) {
            if !parts.iter().any(|p| p.paths == l.paths) {
                notes.push(format!("{} ({weight}) has geometry no Regular layer has; specs draw one geometry at every weight", l.name));
            }
        }
    }

    // Tint, then Solid layers (a tint makes a part a surface, which changes what its Solid
    // means): a part's own when the geometry matches, else parts of their own.
    let mut extras: Vec<Part> = Vec::new();
    for l in &layers {
        if let Some(Paint::Fill { .. }) = style_of(l, "style.tint") {
            // A surface's frame is closed: an open part's tint is a part of its own, closed
            // (a fill draws the same either way).
            match parts.iter_mut().find(|p| p.paths == l.paths && !p.tint) {
                Some(p) if p.role != "fill" && all_closed(&p.paths) => {
                    p.role = "surface";
                    p.tint = true;
                }
                Some(p) if p.role == "fill" => {}
                _ => extras.push(Part { name: format!("{}-tint", l.name), index: l.index, paths: closed(&l.paths), role: "surface", outline_none: true, tint: true, solid: Some(json!("none")), hidden: !l.visible, opacity: l.opacity }),
            }
        }
    }
    for l in &layers {
        if let Some(paint) = style_of(l, "style.solid") {
            let both = l.styles.iter().filter(|(s, _)| s == "style.solid").count() > 1;
            let stroke_w = l.styles.iter().find_map(|(s, p)| match (s.as_str(), p) {
                ("style.solid", Paint::Stroke(w)) => Some(*w),
                _ => None,
            });
            if let Some(w) = stroke_w.filter(|w| (w - 2.6).abs() > 1e-6) {
                notes.push(format!("{}'s Solid stroke is {w}, not 2.6", l.name));
            }
            if let Paint::Fill { even_odd: true } = paint {
                notes.push(format!("{}'s Solid fill is even-odd; check its openings in Solid", l.name));
            }
            let fill = matches!(paint, Paint::Fill { .. });
            match parts.iter_mut().find(|p| p.paths == l.paths && p.solid.is_none() && (!fill || p.role == "fill" || all_closed(&p.paths))) {
                Some(p) => {
                    p.solid = match (p.role, paint, both) {
                        ("surface", _, true) => Some(json!("auto")),
                        ("surface", Paint::Fill { .. }, false) | ("stroke", Paint::Fill { .. }, _) => Some(json!("fill")),
                        ("surface", Paint::Stroke(_), false) => Some(json!("ring")),
                        // A stroke's own Solid is its centerline at 2.6; a fill's is filled.
                        _ => Some(json!("auto")),
                    };
                }
                None => {
                    // A part of its own that draws only in Solid: a fill as a closed surface,
                    // a stroke as a stroke with no outline (its Solid is the 2.6 centerline).
                    let (role, solid, paths) = match paint {
                        Paint::Fill { .. } if both => ("surface", "auto", closed(&l.paths)),
                        Paint::Fill { .. } => ("surface", "fill", closed(&l.paths)),
                        Paint::Stroke(_) => ("stroke", "auto", l.paths.clone()),
                    };
                    extras.push(Part { name: format!("{}-solid", l.name), index: l.index, paths, role, outline_none: true, tint: false, solid: Some(json!(solid)), hidden: !l.visible, opacity: l.opacity });
                }
            }
        }
    }
    for p in parts.iter_mut().filter(|p| p.solid.is_none()) {
        p.solid = Some(json!("none"));
    }
    parts.extend(extras);
    // Back to front: the file lists the front layer first.
    parts.sort_by(|a, b| b.index.cmp(&a.index));

    let mut used: Vec<String> = Vec::new();
    let mut out = Map::new();
    for p in &parts {
        let name = unique(&kebab(&p.name), &used);
        used.push(name.clone());
        let shapes: Vec<Value> = p.paths.iter().map(|d| json!({"path": d})).collect();
        let mut part = Map::new();
        part.insert("shape".into(), if shapes.len() == 1 { shapes[0].clone() } else { Value::Array(shapes) });
        if p.role != "stroke" {
            part.insert("role".into(), json!(p.role));
        }
        if p.outline_none {
            part.insert("outline".into(), json!("none"));
        }
        if p.role == "surface" && !p.tint {
            part.insert("tint".into(), json!(false));
        }
        if let Some(solid) = p.solid.as_ref().filter(|s| *s != "auto") {
            part.insert("solid".into(), solid.clone());
        }
        if p.hidden {
            part.insert("hidden".into(), json!(true));
        } else if p.opacity < 1.0 {
            part.insert("opacity".into(), json!((p.opacity * 1000.0).round() / 1000.0));
        }
        out.insert(name, Value::Object(part));
    }
    if out.is_empty() {
        return Err("the animation has no Regular outline layers".into());
    }

    let mut spec = Map::new();
    spec.insert("$schema".into(), json!("https://gen-animated-icons.lottiefiles.com/schema/icon-v2.json"));
    spec.insert("format".into(), json!(crate::FORMAT));
    spec.insert("id".into(), json!(icon.id));
    spec.insert("label".into(), json!(icon.label));
    spec.insert("intent".into(), json!(icon.intent));
    spec.insert(
        "//".into(),
        json!("Ported from the library's v1 drawing (its Light animation at frame 0). Keep the drawing, write the motion: check reports that motion is empty until a track has keys."),
    );
    if let Some((from, to, labels)) = &icon.state {
        let (from, to, labels, reverse) = match icon.preview_reset {
            true => ("prepared".to_string(), "result".to_string(), ["Play action".to_string(), "Reset preview".to_string()], "fade"),
            false => (kebab(from).trim_start_matches("part-").to_string(), kebab(to).trim_start_matches("part-").to_string(), labels.clone(), "auto"),
        };
        spec.insert("state".into(), json!({"from": from, "to": to, "labels": labels, "reverse": reverse}));
    }
    spec.insert("parts".into(), Value::Object(out));
    spec.insert("motion".into(), json!({}));
    Ok(Port { spec: Value::Object(spec), notes })
}

/// A layer's name, paths and styles at frame 0.
fn read_layer(l: &Value, index: usize, id: &str, notes: &mut Vec<String>) -> Result<Layer, String> {
    let full = l.get("nm").and_then(Value::as_str).unwrap_or_default();
    let name = full.strip_prefix(&format!("{id}-")).unwrap_or(full).to_string();
    let ks = l.get("ks").cloned().unwrap_or(Value::Null);
    let opacity = ks.get("o").map_or(100.0, |o| scalar(o).unwrap_or(100.0));
    let matrix = transform(&ks);
    let (mut paths, mut styles) = (Vec::new(), Vec::new());
    collect(l.get("shapes").and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default(), &matrix, &name, &mut paths, &mut styles, notes)?;
    for key in ["masksProperties", "tt", "td", "parent"] {
        if l.get(key).is_some_and(|v| !v.is_null() && v != &json!([])) {
            notes.push(format!("{name} has `{key}`, which the port leaves out"));
        }
    }
    Ok(Layer { name, index, paths, visible: opacity > 0.0, opacity: (opacity / 100.0).clamp(0.0, 1.0), styles })
}

/// Paths and paints of a shape list (groups included, each with its transform).
fn collect(shapes: &[Value], m: &[f64; 6], layer: &str, paths: &mut Vec<String>, styles: &mut Vec<(String, Paint)>, notes: &mut Vec<String>) -> Result<(), String> {
    for s in shapes {
        match s.get("ty").and_then(Value::as_str) {
            Some("sh") => {
                let shape = s.get("ks").and_then(at0).ok_or_else(|| format!("{layer}: a path without data"))?;
                let shape = shape.as_array().and_then(|a| a.first()).unwrap_or(shape);
                paths.push(svg_path(shape, m).ok_or_else(|| format!("{layer}: an unreadable path"))?);
            }
            Some(kind @ ("st" | "fl")) => {
                let style = s.get("o").and_then(|o| o.get("sid")).and_then(Value::as_str).unwrap_or_default().to_string();
                let paint = match kind {
                    "st" => Paint::Stroke(s.get("w").and_then(scalar).unwrap_or(2.0)),
                    _ => Paint::Fill { even_odd: s.get("r").and_then(Value::as_i64) == Some(2) },
                };
                if style.is_empty() {
                    notes.push(format!("{layer} has a paint without a style slot"));
                }
                styles.push((style, paint));
            }
            Some("gr") => {
                let items = s.get("it").and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default();
                let local = items.iter().find(|i| i.get("ty").and_then(Value::as_str) == Some("tr")).map_or(IDENTITY, transform);
                collect(items, &multiply(m, &local), layer, paths, styles, notes)?;
            }
            Some("tr") => {}
            Some("tm") => {
                let full = s.get("s").and_then(scalar) == Some(0.0) && s.get("e").and_then(scalar) == Some(100.0);
                if !full {
                    notes.push(format!("{layer} is trimmed at frame 0; the port draws it whole"));
                }
            }
            Some(other) => notes.push(format!("{layer} has a `{other}` shape item the port leaves out")),
            None => {}
        }
    }
    Ok(())
}

const IDENTITY: [f64; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// A property's value at frame 0: the static value, or the first key's start value.
fn at0(prop: &Value) -> Option<&Value> {
    let k = prop.get("k")?;
    match (prop.get("a").and_then(Value::as_i64), k.as_array().and_then(|a| a.first())) {
        (Some(1), Some(first)) if first.get("s").is_some() => first.get("s"),
        _ => Some(k),
    }
}

fn scalar(prop: &Value) -> Option<f64> {
    let v = at0(prop)?;
    v.as_f64().or_else(|| v.as_array().and_then(|a| a.first()).and_then(Value::as_f64))
}

fn pair(prop: Option<&Value>, default: [f64; 2]) -> [f64; 2] {
    let Some(prop) = prop else { return default };
    if prop.get("s").and_then(Value::as_bool) == Some(true) {
        return [prop.get("x").and_then(scalar).unwrap_or(default[0]), prop.get("y").and_then(scalar).unwrap_or(default[1])];
    }
    match at0(prop).and_then(Value::as_array) {
        Some(a) if a.len() >= 2 => [a[0].as_f64().unwrap_or(default[0]), a[1].as_f64().unwrap_or(default[1])],
        _ => default,
    }
}

/// `[a, b, c, d, e, f]`: x' = a·x + c·y + e, y' = b·x + d·y + f, from anchor, position,
/// scale and rotation at frame 0.
fn transform(ks: &Value) -> [f64; 6] {
    let a = pair(ks.get("a"), [0.0, 0.0]);
    let p = pair(ks.get("p"), [0.0, 0.0]);
    let s = pair(ks.get("s"), [100.0, 100.0]);
    let r = ks.get("r").and_then(scalar).unwrap_or(0.0).to_radians();
    let (sx, sy) = (s[0] / 100.0, s[1] / 100.0);
    let (cos, sin) = (r.cos(), r.sin());
    let (ma, mb, mc, md) = (cos * sx, sin * sx, -sin * sy, cos * sy);
    [ma, mb, mc, md, p[0] - (ma * a[0] + mc * a[1]), p[1] - (mb * a[0] + md * a[1])]
}

fn multiply(outer: &[f64; 6], inner: &[f64; 6]) -> [f64; 6] {
    let [a, b, c, d, e, f] = *outer;
    let [ia, ib, ic, id, ie, iff] = *inner;
    [a * ia + c * ib, b * ia + d * ib, a * ic + c * id, b * ic + d * id, a * ie + c * iff + e, b * ie + d * iff + f]
}

fn apply(m: &[f64; 6], x: f64, y: f64) -> (f64, f64) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}

/// SVG path data of a Lottie bezier (`v`, `i`, `o`, `c`), absolute, one subpath.
fn svg_path(shape: &Value, m: &[f64; 6]) -> Option<String> {
    let points = |key: &str| -> Option<Vec<(f64, f64)>> {
        shape.get(key)?.as_array()?.iter().map(|p| Some((p.get(0)?.as_f64()?, p.get(1)?.as_f64()?))).collect()
    };
    let (v, i, o) = (points("v")?, points("i")?, points("o")?);
    if v.is_empty() || i.len() != v.len() || o.len() != v.len() {
        return None;
    }
    let closed = shape.get("c").and_then(Value::as_bool).unwrap_or(false);
    let at = |x: f64, y: f64| {
        let (x, y) = apply(m, x, y);
        format!("{} {}", num(x), num(y))
    };
    let mut d = format!("M{}", at(v[0].0, v[0].1));
    let n = v.len();
    let segments = if closed { n } else { n - 1 };
    for k in 0..segments {
        let (a, b) = (k, (k + 1) % n);
        let straight = o[a] == (0.0, 0.0) && i[b] == (0.0, 0.0);
        if straight {
            d.push_str(&format!(" L{}", at(v[b].0, v[b].1)));
        } else {
            d.push_str(&format!(
                " C{} {} {}",
                at(v[a].0 + o[a].0, v[a].1 + o[a].1),
                at(v[b].0 + i[b].0, v[b].1 + i[b].1),
                at(v[b].0, v[b].1)
            ));
        }
    }
    if closed {
        d.push_str(" Z");
    }
    Some(d)
}

/// Whether every path is closed.
fn all_closed(paths: &[String]) -> bool {
    paths.iter().all(|d| d.ends_with('Z'))
}

/// The paths, each closed.
fn closed(paths: &[String]) -> Vec<String> {
    paths.iter().map(|d| if d.ends_with('Z') { d.clone() } else { format!("{d} Z") }).collect()
}

/// A coordinate with at most 3 decimals (a thousandth of a unit) and no trailing zeros.
fn num(x: f64) -> String {
    let r = (x * 1_000.0).round() / 1_000.0;
    let r = if r == 0.0 { 0.0 } else { r };
    let s = format!("{r:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// A layer name as a part name: lowercase letters, digits and single hyphens, starting
/// with a letter.
fn kebab(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let out = out.trim_end_matches('-').to_string();
    let out = if out.starts_with(|c: char| c.is_ascii_lowercase()) { out } else { format!("part-{out}") };
    out.chars().take(32).collect::<String>().trim_end_matches('-').to_string()
}

fn unique(name: &str, used: &[String]) -> String {
    if !used.iter().any(|u| u == name) {
        return name.to_string();
    }
    (2..).map(|n| format!("{}-{n}", name.chars().take(29).collect::<String>())).find(|c| !used.contains(c)).expect("a free name")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_keep_lines_and_curves() {
        let shape = json!({"c": true, "v": [[0, 0], [10, 0], [10, 10]], "i": [[0, 0], [0, 0], [0, -2]], "o": [[0, 0], [0, 2], [0, 0]]});
        assert_eq!(svg_path(&shape, &IDENTITY).unwrap(), "M0 0 L10 0 C10 2 10 8 10 10 L0 0 Z");
        let moved = transform(&json!({"p": {"a": 0, "k": [16, 16]}, "a": {"a": 0, "k": [16, 16]}, "r": {"a": 0, "k": 90}}));
        let (x, y) = apply(&moved, 20.0, 16.0);
        assert!((x - 16.0).abs() < 1e-9 && (y - 20.0).abs() < 1e-9);
    }

    #[test]
    fn names_become_part_names() {
        assert_eq!(kebab("solid-handle-1"), "solid-handle-1");
        assert_eq!(kebab("1-rail"), "part-1-rail");
        assert_eq!(unique("body", &["body".into()]), "body-2");
    }
}
