//! 15-commerce-delivery: a 3-unit painted margin, attachment, negative-space and paint
//! continuity asserts, and the per-action `returnPackageMotion` summary.

use super::{Frame, ensure, flattener20, frame_times};
use crate::contracts::ContractError;
use crate::contracts::evidence::js_number;
use crate::contracts::geom::Sampler;
use crate::contracts::geom::{Point, bounds, inside_poly, pick_bold, poly_dist, seg_dist};
use crate::contracts::jsnum::{hypot2, max_of, min_of, min2, slice_range};
use crate::contracts::scene::{SceneSource, SetAction, SetManifest};
use gai_core::js_number_to_string as js;
use gai_core::parallel::ordered_map;
use serde_json::{Value, json};

#[derive(Default)]
struct Outcome {
    checked: u64,
    margin: f64,
    attachment: u64,
    negative_space: u64,
    continuity: u64,
    return_package: Option<Value>,
}

/// Which counter a `check()` increments.
#[derive(Clone, Copy)]
enum Counter {
    Attachment,
    NegativeSpace,
    Continuity,
}

impl Outcome {
    /// `check(ok, message, kind)`: assert with the action and frame, then count.
    fn check(
        &mut self,
        ok: bool,
        action: &str,
        f: u32,
        message: impl FnOnce() -> String,
        counter: Counter,
    ) -> Result<(), ContractError> {
        ensure(ok, || format!("{action} frame {f}: {}", message()))?;
        match counter {
            Counter::Attachment => self.attachment += 1,
            Counter::NegativeSpace => self.negative_space += 1,
            Counter::Continuity => self.continuity += 1,
        }
        Ok(())
    }
}

fn pair(p: &[f64], start: i64, end: i64) -> Point {
    let s = &p[slice_range(p.len(), start, Some(end))];
    [
        s.first().copied().unwrap_or(f64::NAN),
        s.get(1).copied().unwrap_or(f64::NAN),
    ]
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
            let center = |p: &str| -> Result<Point, ContractError> {
                let b = fr.painted_box(&part(p))?;
                Ok([(b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0])
            };
            let attach = |p: &str, housing: &str, o: &mut Outcome| -> Result<(), ContractError> {
                for e in ends(p)? {
                    let ok = poly_dist(e, &points(housing)?) <= 0.04;
                    o.check(
                        ok,
                        name,
                        f,
                        || format!("{p} detached from {housing}"),
                        Counter::Attachment,
                    )?;
                }
                Ok(())
            };
            // One-sided: every vertex of a to b's polyline, less a fixed 2.6 paint width.
            let gap = |a: &str, b: &str, min: f64, o: &mut Outcome| -> Result<(), ContractError> {
                let pb = points(b)?;
                let d = points(a)?
                    .iter()
                    .fold(f64::INFINITY, |m, &p| min2(m, poly_dist(p, &pb)))
                    - 2.6;
                o.check(
                    d >= min - 0.025,
                    name,
                    f,
                    || format!("{a}/{b} painted gap {}", js(d)),
                    Counter::NegativeSpace,
                )
            };
            let guides = |pairs: [(&str, &str, &str); 2],
                          suffix: &str,
                          o: &mut Outcome|
             -> Result<(), ContractError> {
                for (n, first, last) in pairs {
                    let e = ends(n)?;
                    let ok = poly_dist(e[0], &points(first)?) < 0.01
                        && poly_dist(e[1], &points(last)?) < 0.01;
                    o.check(ok, name, f, || format!("{n}{suffix}"), Counter::Attachment)?;
                }
                Ok(())
            };
            match icon {
                "storefront" => gap("hem", "facade", 1.0, &mut o)?,
                "shopping-bag" => attach("handle", "body", &mut o)?,
                "basket" => {
                    attach("handle", "rim", &mut o)?;
                    attach("body", "rim", &mut o)?;
                    guides(
                        [
                            ("weave-left", "rim", "body"),
                            ("weave-right", "rim", "body"),
                        ],
                        " detached",
                        &mut o,
                    )?;
                    let y = min_of(points("handle")?.iter().map(|p| p[1]));
                    o.check(
                        14.0 - y - 2.6 >= 0.7,
                        name,
                        f,
                        || "strap apex crowds rim".into(),
                        Counter::NegativeSpace,
                    )?;
                }
                "coupon" => gap("body", "stub", 0.9, &mut o)?,
                "discount" => {
                    gap("slash", "dot-one", 0.7, &mut o)?;
                    gap("slash", "dot-two", 0.7, &mut o)?;
                }
                "checkout" => {
                    guides(
                        [
                            ("guide-left", "console", "drawer"),
                            ("guide-right", "console", "drawer"),
                        ],
                        " guide detached",
                        &mut o,
                    )?;
                    gap("handle", "drawer", 1.0, &mut o)?;
                }
                "open-package" => {
                    for n in ["flap-left", "flap-right"] {
                        let ok = poly_dist(ends(n)?[0], &points("box")?) < 0.01;
                        o.check(
                            ok,
                            name,
                            f,
                            || format!("{n} hinge detached"),
                            Counter::Attachment,
                        )?;
                    }
                }
                "packing-tape" => {
                    let (c, e) = (center("rim")?, ends("strip")?);
                    let tangent = hypot2(e[1][0] - c[0], e[1][1] - c[1] - 6.0) < 0.025;
                    o.check(
                        tangent,
                        name,
                        f,
                        || "tape does not meet roll tangent".into(),
                        Counter::Attachment,
                    )?;
                    let fixed = hypot2(e[0][0] - 5.5, e[0][1] - 23.0) < 0.001;
                    o.check(
                        fixed,
                        name,
                        f,
                        || "free tape end moved".into(),
                        Counter::Attachment,
                    )?;
                    let clear = !inside_poly(c, &points("roll-tint")?);
                    o.check(
                        clear,
                        name,
                        f,
                        || "tape tint fills core".into(),
                        Counter::NegativeSpace,
                    )?;
                }
                "delivery-truck" => {
                    for (support, wheel, housing) in [
                        ("support-left", "wheel-left", "cargo"),
                        ("support-right", "wheel-right", "cab"),
                    ] {
                        let (e, c) = (ends(support)?, center(wheel)?);
                        let body = poly_dist(e[0], &points(housing)?) < 0.01;
                        o.check(
                            body,
                            name,
                            f,
                            || format!("{support} body attachment"),
                            Counter::Attachment,
                        )?;
                        let joint = hypot2(e[1][0] - c[0], e[1][1] - c[1] + 2.5) < 0.001;
                        o.check(
                            joint,
                            name,
                            f,
                            || format!("{support} wheel attachment"),
                            Counter::Attachment,
                        )?;
                        let counter = c[1] - 2.5 + 1.3
                            - (fr.painted_box(&part(housing))?[3] + 0.05)
                            >= 1.0 - 0.025;
                        o.check(
                            counter,
                            name,
                            f,
                            || format!("{wheel} counter crowded"),
                            Counter::NegativeSpace,
                        )?;
                    }
                }
                "order-tracking" => {
                    for n in ["stop-one", "stop-two", "stop-three"] {
                        gap("pin", n, 1.0, &mut o)?;
                    }
                    for (rail, near, far) in [
                        ("track-one", "stop-one", "stop-two"),
                        ("track-two", "stop-two", "stop-three"),
                    ] {
                        let targets = [
                            (ends(rail)?[0], center(near)?),
                            (ends(rail)?[1], center(far)?),
                        ];
                        for (p, c) in targets {
                            let ok = hypot2(p[0] - c[0], p[1] - c[1]) - 1.3 >= 0.7 - 0.001;
                            o.check(
                                ok,
                                name,
                                f,
                                || "rail invades stop counter".into(),
                                Counter::NegativeSpace,
                            )?;
                        }
                    }
                }
                "pickup-locker" => gap("door", "frame-frame", 0.85, &mut o)?,
                "warehouse" => {
                    attach("rail", "jambs", &mut o)?;
                    let r = ends("rail")?[0][1];
                    let open = !inside_poly([16.0, 25.0], &points("walls-tint")?)
                        && !inside_poly([16.0, r + 1.4], &points("curtain-tint")?);
                    o.check(
                        open,
                        name,
                        f,
                        || "warehouse tint fills open aperture".into(),
                        Counter::NegativeSpace,
                    )?;
                }
                "inventory" => {
                    gap("box-left", "box-right", 0.9, &mut o)?;
                    gap("moving-box", "box-left", 0.9, &mut o)?;
                    gap("moving-box", "box-right", 0.9, &mut o)?;
                }
                "return-package" => {
                    for n in ["arrow-body", "arrow-head"] {
                        if fr.vis(fr.node(&part(n))?)? > 0.01 {
                            gap(n, "parcel", 2.5, &mut o)?;
                        }
                    }
                    let head = fr.node(&part("arrow-head"))?;
                    if fr.vis(head)? > 0.01 {
                        let tip = pair(&fr.raw(head), 2, 4);
                        let body = points("arrow-body")?;
                        let end = body
                            .last()
                            .copied()
                            .ok_or_else(|| ContractError::Geometry("arrow body is empty".into()))?;
                        let joined = hypot2(tip[0] - end[0], tip[1] - end[1]) < 0.001;
                        o.check(
                            joined,
                            name,
                            f,
                            || "return head detached from the drawn stem".into(),
                            Counter::Attachment,
                        )?;
                        let stem = pair(&fr.raw(fr.node(&part("arrow-body"))?), -4, -2);
                        let straight =
                            (stem[0] - end[0]).abs() < 0.001 && (stem[1] - end[1]).abs() >= 3.49;
                        o.check(
                            straight,
                            name,
                            f,
                            || "return arrow reaches head without a straight stem".into(),
                            Counter::Attachment,
                        )?;
                    }
                }
                "subscription-box" => gap("hand", "dial", 0.7, &mut o)?,
                "loyalty-points" => {
                    let b = bounds(points("cell-one")?.into_iter());
                    let r = (b[2] - b[0]) / 2.0;
                    let fill = fr.painted_box(&part("earned-one"))?;
                    let fill_radius = (fill[2] - fill[0]) / 2.0;
                    let ok = fill_radius - (r - 0.75) >= 0.099;
                    o.check(
                        ok,
                        name,
                        f,
                        || "earned point meets rather than overlaps the thinnest ring paint".into(),
                        Counter::Continuity,
                    )?;
                }
                _ => {}
            }
        }
        if icon == "return-package" {
            o.return_package = Some(return_package_motion(&scene, action)?);
        }
        Ok(o)
    })?;

    let (mut margin, mut attachment, mut negative_space, mut continuity) = (32.0, 0, 0, 0);
    let (mut actions, mut motion) = (Vec::new(), Vec::new());
    for ((_, action), o) in jobs.iter().zip(outcomes) {
        actions.push(json!({"name": action.name, "paintedBoundsChecked": o.checked}));
        margin = min2(margin, o.margin);
        attachment += o.attachment;
        negative_space += o.negative_space;
        continuity += o.continuity;
        motion.extend(o.return_package);
    }
    let mut report = serde_json::Map::new();
    report.insert("result".into(), "PASS".into());
    report.insert("actions".into(), Value::Array(actions));
    report.insert("minimumPaintedMargin".into(), js_number(margin));
    report.insert("attachmentChecks".into(), attachment.into());
    report.insert("negativeSpaceChecks".into(), negative_space.into());
    report.insert("paintContinuityChecks".into(), continuity.into());
    // Created lazily in the JS, so only sets with return-package carry it.
    if !motion.is_empty() {
        report.insert("returnPackageMotion".into(), Value::Array(motion));
    }
    Ok(Value::Object(report))
}

/// The after-frames summary: parcel rotation over all keys (Strong must rotate, Light
/// must not), the painted barb-to-stem clearance at 2.4 s, and whether the body redraws.
fn return_package_motion(
    scene: &crate::contracts::scene::Scene,
    action: &SetAction,
) -> Result<Value, ContractError> {
    let parcel = pick_bold(scene, "return-package-parcel")?;
    let angles: Vec<f64> = parcel
        .points
        .iter()
        .map(|(_, p)| {
            let at = |i: usize| p.get(i).copied().unwrap_or(f64::NAN);
            libm::atan2(at(3) - at(1), at(2) - at(0)) * 180.0 / std::f64::consts::PI
        })
        .collect();
    let range = max_of(angles.iter().copied()) - min_of(angles.iter().copied());
    if action.direction.as_deref() == Some("forward") {
        let ok = if action.motion_id.as_deref() == Some("strong") {
            range >= 9.9
        } else {
            range < 0.001
        };
        ensure(ok, || {
            format!(
                "{} motion strength must include visible rigid rotation only in Strong",
                action.name
            )
        })?;
    }
    let body = pick_bold(scene, "return-package-arrow-body")?;
    let head = pick_bold(scene, "return-package-arrow-head")?;
    let end = Sampler::Gt.coords(&head.points, 2.4);
    let tail = Sampler::Gt.coords(&body.points, 2.4);
    let (stem, tip, barb) = (pair(&tail, -4, -2), pair(&end, 2, 4), pair(&end, 4, 6));
    let clear = seg_dist(barb, stem, tip) - 2.6;
    ensure(clear >= 0.45, || {
        format!("{} inner arrow barb crowds its straight stem", action.name)
    })?;
    let redraw = min_of(body.visibility()?.iter().map(|k| k.1)) == 0.0;
    Ok(json!({
        "action": action.name,
        "parcelRotationRangeDegrees": js_number(range),
        "headEndPaintedStemClearance": js_number(clear),
        "bodyRedraw": redraw,
    }))
}
