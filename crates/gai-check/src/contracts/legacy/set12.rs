//! 12-loading-indicators: moving-point bead clearance, hidden ripple recycling and rigid
//! stack separation, read from fixed scene names with the S_GE sampler.

use super::ensure;
use crate::contracts::ContractError;
use crate::contracts::evidence::js_number;
use crate::contracts::geom::Sampler;
use crate::contracts::jsnum::{hypot2, max_of, max2, min2};
use crate::contracts::scene::{SceneNode, SceneSource};
use serde_json::{Value, json};

/// The Bold paint width both clearances subtract.
const PAINT_WIDTH: f64 = 2.5;

fn name(base: &str, strong: bool) -> String {
    format!("{base}{}", if strong { "-strong" } else { "" })
}

fn coords(node: &SceneNode, t: f64) -> Vec<f64> {
    Sampler::Ge.coords(&node.points, t).into_owned()
}

fn at(p: &[f64], i: usize) -> f64 {
    p.get(i).copied().unwrap_or(f64::NAN)
}

pub(super) fn run(scenes: &dyn SceneSource) -> Result<Value, ContractError> {
    let mut pairs = Vec::new();
    for icon in ["wave-dots", "infinity-run", "bead-relay", "bead-track"] {
        for strong in [false, true] {
            let name = name(icon, strong);
            let scene = scenes.load(&name)?;
            let beads: Vec<&SceneNode> = scene
                .nodes
                .iter()
                .filter(|n| {
                    n.name.contains("-regular__")
                        && n.transform
                        && n.base_points.as_ref().is_some_and(|b| b.len() == 4)
                })
                .collect();
            let mut minimum = f64::INFINITY;
            for f in 0..=144u32 {
                let t = f64::from(f) / 60.0;
                let mut active = Vec::new();
                for n in &beads {
                    if Sampler::Ge.scalar(n.visibility()?, t) > 0.0 {
                        active.push(coords(n, t));
                    }
                }
                for i in 0..active.len() {
                    for j in i + 1..active.len() {
                        let (a, b) = (&active[i], &active[j]);
                        let gap = hypot2(at(a, 0) - at(b, 0), at(a, 1) - at(b, 1)) - PAINT_WIDTH;
                        minimum = min2(minimum, gap);
                        ensure(gap > 0.15, || format!("{name} has colliding Bold beads"))?;
                    }
                }
            }
            pairs.push(json!({"name": name, "minimumBoldGap": js_number(minimum)}));
        }
    }

    let mut guards = Vec::new();
    for strong in [false, true] {
        let name = name("ripple-rings", strong);
        let scene = scenes.load(&name)?;
        let mut resets = 0u32;
        for n in scene
            .nodes
            .iter()
            .filter(|n| n.name.contains("-ring-") && n.name.contains("-regular__"))
        {
            let mut previous = coords(n, 0.0);
            for f in 1..=144u32 {
                let p = coords(n, f64::from(f) / 60.0);
                let jump = max_of(
                    p.iter()
                        .enumerate()
                        .map(|(i, x)| (x - at(&previous, i)).abs()),
                );
                if jump > 3.0 {
                    for k in [-1i32, 0, 1] {
                        let frame = (f as i32 + k + 144) % 144;
                        let v = Sampler::Ge.scalar(n.visibility()?, f64::from(frame) / 60.0);
                        // `assert.equal(v, 0)` is Object.is: -0 fails.
                        ensure(v.to_bits() == 0f64.to_bits(), || {
                            format!("{name} visible radius recycle")
                        })?;
                    }
                    resets += 1;
                }
                previous = p;
            }
        }
        guards.push(json!({"name": name, "resets": resets}));
    }

    let mut stack_gap = f64::INFINITY;
    for strong in [false, true] {
        let scene = scenes.load(&name("stack-cycle", strong))?;
        let bars: Vec<&SceneNode> = scene
            .nodes
            .iter()
            .filter(|n| n.name.contains("-regular__"))
            .collect();
        for f in 0..=144u32 {
            let t = f64::from(f) / 60.0;
            for i in 0..bars.len() {
                for j in i + 1..bars.len() {
                    let (a, b) = (coords(bars[i], t), coords(bars[j], t));
                    let (a0, a1, a2, b0, b1, b2) = (
                        at(&a, 0),
                        at(&a, 1),
                        at(&a, 2),
                        at(&b, 0),
                        at(&b, 1),
                        at(&b, 2),
                    );
                    let overlap_start = max2(min2(a0, a2), min2(b0, b2));
                    let overlap_end = min2(max2(a0, a2), max2(b0, b2));
                    let dx = max2(0.0, overlap_start - overlap_end);
                    let gap = hypot2(dx, (a1 - b1).abs()) - PAINT_WIDTH;
                    ensure(gap > 0.5, || "Stack strokes fuse in Bold".into())?;
                    stack_gap = min2(stack_gap, gap);
                }
            }
        }
    }

    let mut report = serde_json::Map::new();
    report.insert("result".into(), "PASS".into());
    report.insert("paintWidth".into(), js_number(PAINT_WIDTH));
    report.insert("movingPointPairs".into(), Value::Array(pairs));
    report.insert("rippleResetGuards".into(), Value::Array(guards));
    report.insert("stackPaintGap".into(), js_number(stack_gap));
    Ok(Value::Object(report))
}
