//! 14-travel: a 3-unit painted margin plus attachment and negative-space asserts.

use super::{Frame, ensure, flattener20, frame_times};
use crate::contracts::ContractError;
use crate::contracts::evidence::js_number;
use crate::contracts::geom::{inside_poly, poly_dist, seg_dist};
use crate::contracts::jsnum::{hypot2, min2};
use crate::contracts::scene::{SceneSource, SetManifest};
use gai_core::js_number_to_string as js;
use gai_core::parallel::ordered_map;
use serde_json::{Value, json};

#[derive(Default)]
struct Outcome {
    checked: u64,
    margin: f64,
    attachment: u64,
    negative_space: u64,
}

pub(super) fn run(
    manifest: &SetManifest,
    scenes: &dyn SceneSource,
) -> Result<Value, ContractError> {
    let flattener = flattener20();
    let jobs: Vec<_> = manifest
        .icons
        .iter()
        .flat_map(|i| i.animations.iter().map(move |a| (i.name.as_str(), a)))
        .collect();
    let outcomes = ordered_map(jobs.len(), |j| {
        let (icon, action) = jobs[j];
        let scene = scenes.load(&action.name)?;
        let name = action.name.as_str();
        let mut o = Outcome {
            margin: f64::INFINITY,
            ..Outcome::default()
        };
        for (f, t) in frame_times() {
            let fr = Frame {
                scene: &scene,
                flattener: &flattener,
                t,
            };
            fr.margins(|n, m| {
                o.margin = min2(o.margin, m);
                ensure(m >= 3.0 - 0.025, || {
                    format!("{name} {} at frame {f}: painted margin {}", n.name, js(m))
                })?;
                o.checked += 1;
                Ok(())
            })?;
            let part = |p: &str| format!("{icon}-{p}");
            let points = |p: &str| fr.points(&part(p));
            let ends = |p: &str| fr.ends(&part(p));
            let attach = |p: &str, housing: &str, o: &mut Outcome| -> Result<(), ContractError> {
                for e in ends(p)? {
                    ensure(poly_dist(e, &points(housing)?) <= 0.04, || {
                        format!("{name} {p} attachment at frame {f}")
                    })?;
                    o.attachment += 1;
                }
                Ok(())
            };
            match icon {
                "suitcase" => attach("handle", "case", &mut o)?,
                "backpack" => attach("loop", "body", &mut o)?,
                "bus" => {
                    for n in ["tire-left", "tire-right"] {
                        let p = ends(n)?;
                        ensure(poly_dist(p[0], &points("body")?) < 0.03, || {
                            format!("{name} suspension body attachment")
                        })?;
                        ensure((p[1][1] - 27.5).abs() < 0.0001, || {
                            format!("{name} fixed tire contact")
                        })?;
                        o.attachment += 2;
                    }
                }
                "bed" => {
                    for n in ["pillow", "blanket"] {
                        attach(n, "mattress", &mut o)?;
                    }
                }
                "tent" => {
                    for p in ends("door")? {
                        ensure((p[1] - 26.0).abs() < 0.0001, || {
                            format!("{name} tent hem detached")
                        })?;
                        o.attachment += 1;
                    }
                    ensure(!inside_poly([16.0, 23.0], &points("canvas-tint")?), || {
                        format!("{name} tint fills doorway")
                    })?;
                    o.negative_space += 1;
                }
                "beach-umbrella" => {
                    ensure(
                        poly_dist(ends("pole")?[1], &points("sand")?) < 0.0001,
                        || format!("{name} umbrella foot moved"),
                    )?;
                    o.attachment += 1;
                }
                "route" if fr.vis(fr.node(&part("path"))?)? > 0.01 => {
                    let b = fr.painted_box(&part("traveller"))?;
                    let path = points("path")?;
                    let head = path
                        .last()
                        .copied()
                        .ok_or_else(|| ContractError::Geometry("route path is empty".into()))?;
                    let d = hypot2((b[0] + b[2]) / 2.0 - head[0], (b[1] + b[3]) / 2.0 - head[1]);
                    ensure(d < 0.001, || {
                        format!("{name} traveller does not follow path head")
                    })?;
                    o.attachment += 1;
                }
                "bicycle" => {
                    let p = ends("crank")?;
                    for c in [[8.0, 22.0], [24.0, 22.0]] {
                        let gap = seg_dist(c, p[0], p[1]) - 3.2 - 2.6;
                        ensure(gap >= 0.7, || {
                            format!("{name} crank touches wheel: {}", js(gap))
                        })?;
                        o.negative_space += 1;
                    }
                }
                "signpost" => {
                    let (a, b) = (ends("post-between")?, ends("post-lower")?);
                    let attached = poly_dist(a[0], &points("upper-board")?) < 0.03
                        && poly_dist(a[1], &points("lower-board")?) < 0.03
                        && poly_dist(b[0], &points("lower-board")?) < 0.03;
                    ensure(attached, || format!("{name} exposed post attachment"))?;
                    o.attachment += 3;
                    let (board, foot) = (points("lower-board")?, points("foot")?);
                    let gap = board
                        .iter()
                        .fold(f64::INFINITY, |m, &p| min2(m, poly_dist(p, &foot)))
                        - 2.6;
                    ensure(gap >= 0.7, || {
                        format!("{name} board crowds base: {}", js(gap))
                    })?;
                    o.negative_space += 1;
                }
                _ => {}
            }
        }
        Ok(o)
    })?;

    let (mut margin, mut attachment, mut negative_space) = (32.0, 0, 0);
    let mut actions = Vec::new();
    for ((_, action), o) in jobs.iter().zip(&outcomes) {
        actions.push(json!({"name": action.name, "paintedBoundsChecked": o.checked}));
        margin = min2(margin, o.margin);
        attachment += o.attachment;
        negative_space += o.negative_space;
    }
    let mut report = serde_json::Map::new();
    report.insert("result".into(), "PASS".into());
    report.insert("actions".into(), Value::Array(actions));
    report.insert("minimumPaintedMargin".into(), js_number(margin));
    report.insert("attachmentChecks".into(), attachment.into());
    report.insert("negativeSpaceChecks".into(), negative_space.into());
    Ok(Value::Object(report))
}
