//! Fixed one-off checks of two frozen sets that no rule kind expresses (verify.md §7.1,
//! §7.2): named actions, a 240 Hz clamped sampler, arg-min evidence and line
//! subdivision. They run after the directed report, as their scripts did.

use super::generic::pick_prefix;
use super::{Evidence, SetContext, VerifyError, ensure, evidence_json, read};
use crate::contracts::geom::{Point, Sampler};
use crate::contracts::jsnum::{at_index, hypot2, max_of, max2, min_of, min2};
use crate::contracts::scene::{Scene, SceneNode, Track, cmd};
use serde_json::{Map, Value, json};

/// The one-off checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OneOff {
    /// 10 `verify-approval-clearance.mjs`: token/gate clearance of `approval-step-strong`.
    ApprovalClearance,
    /// 11 `verify-attachments.mjs`: queue, reconnect, offline and cloud-sync attachments.
    Attachments,
}

impl OneOff {
    /// Runs the check; returns its evidence and PASS line.
    pub(super) fn run(self, set: &SetContext<'_>) -> Result<(Evidence, String), VerifyError> {
        match self {
            OneOff::ApprovalClearance => approval(set),
            OneOff::Attachments => attachments(set),
        }
    }
}

/// `at(keys, t)`: clamped interpolation between the two keys around `t` (S_CLAMP).
fn sample_clamped(keys: &Track<Vec<f64>>, t: f64) -> Result<Vec<f64>, VerifyError> {
    if keys.len() < 2 {
        return Err(VerifyError::Geometry(
            "the clamped sampler needs two keys".into(),
        ));
    }
    let mut i = 0;
    while i < keys.len() - 2 && keys[i + 1].0 < t {
        i += 1;
    }
    let ((a, v), (b, w)) = (&keys[i], &keys[i + 1]);
    let q = max2(0.0, min2(1.0, (t - a) / (b - a)));
    Ok(v.iter()
        .enumerate()
        .map(|(j, x)| x + (w.get(j).copied().unwrap_or(f64::NAN) - x) * q)
        .collect())
}

fn find<'s>(scene: &'s Scene, needle: &str) -> Result<&'s SceneNode, VerifyError> {
    scene
        .nodes
        .iter()
        .find(|n| n.name.contains(needle))
        .ok_or_else(|| VerifyError::Assertion(format!("no {needle} node")))
}

fn coord(p: &[f64], i: usize) -> f64 {
    p.get(i).copied().unwrap_or(f64::NAN)
}

/// 10: the Bold token never closes on the gate, sampled at 240 Hz.
fn approval(set: &SetContext<'_>) -> Result<(Evidence, String), VerifyError> {
    let name = "approval-step-strong";
    let bytes = read(&set.scene_path(name))?;
    let scene = Scene::from_slice(&bytes).map_err(|source| VerifyError::Json {
        path: set.scene_path(name),
        source,
    })?;
    let (gate, token) = (
        find(&scene, "gate-bold__")?,
        find(&scene, "token-frame-bold__")?,
    );
    let (mut minimum, mut at_seconds) = (f64::INFINITY, 0.0);
    for k in 0..=240u32 {
        let t = f64::from(k) / 240.0;
        let g = sample_clamped(&gate.points, t)?;
        let (ax, ay, bx, by) = (coord(&g, 0), coord(&g, 1), coord(&g, 2), coord(&g, 3));
        let p = sample_clamped(&token.points, t)?;
        let xs = p.iter().step_by(2).copied();
        let ys = p.iter().skip(1).step_by(2).copied();
        let cx = (min_of(xs.clone()) + max_of(xs)) / 2.0;
        let cy = (min_of(ys.clone()) + max_of(ys)) / 2.0;
        // `x ** 2` is `x * x` in V8.
        let den = (bx - ax) * (bx - ax) + (by - ay) * (by - ay);
        let u = max2(
            0.0,
            min2(1.0, ((cx - ax) * (bx - ax) + (cy - ay) * (by - ay)) / den),
        );
        let gap = hypot2(cx - ax - u * (bx - ax), cy - ay - u * (by - ay)) - 3.25 - 2.5;
        if gap < minimum {
            minimum = gap;
            at_seconds = t;
        }
    }
    ensure(minimum >= 0.75, || {
        format!(
            "Approval token/gate Bold gap closed: {}",
            gai_core::js_number_to_string(minimum)
        )
    })?;
    let proof = json!({
        "result": "PASS", "checks": 241, "minimumBoldGap": minimum, "atSeconds": at_seconds,
        "sceneSha256": gai_core::sha256_hex(&bytes),
    });
    let evidence = Evidence {
        path: "evidence/approval-clearance-verification.json".into(),
        contents: evidence_json(&proof),
    };
    Ok((
        evidence,
        format!(
            "PASS approval clearance: minimum Bold gap {}",
            gai_core::js_number_to_string(minimum)
        ),
    ))
}

/// 11 `contour`: lines subdivided into 16 steps, cubics into 32, `Z` back to the start.
fn contour(node: &SceneNode, t: f64) -> Result<Vec<Point>, VerifyError> {
    let pts = Sampler::Ge.coords(&node.points, t);
    let at = |k: usize| [coord(&pts, k), coord(&pts, k + 1)];
    let (mut out, mut k) = (Vec::new(), 0);
    let (mut last, mut first): (Option<Point>, Option<Point>) = (None, None);
    let missing = || VerifyError::Geometry(format!("{} draws before a move", node.name));
    for &c in &node.commands {
        match c {
            cmd::MOVE => {
                let p = at(k);
                k += 2;
                (last, first) = (Some(p), Some(p));
                out.push(p);
            }
            cmd::LINE => {
                let (from, end) = (last.ok_or_else(missing)?, at(k));
                k += 2;
                out.extend(
                    (1..=16).map(|i| {
                        [0, 1].map(|j| from[j] + (end[j] - from[j]) * f64::from(i) / 16.0)
                    }),
                );
                last = Some(end);
            }
            cmd::CUBIC => {
                let from = last.ok_or_else(missing)?;
                let (c1, c2, end) = (at(k), at(k + 2), at(k + 4));
                k += 6;
                out.extend((1..=32).map(|i| {
                    let u = f64::from(i) / 32.0;
                    let q = 1.0 - u;
                    [0, 1].map(|j| {
                        q * q * q * from[j]
                            + 3.0 * q * q * u * c1[j]
                            + 3.0 * q * u * u * c2[j]
                            + u * u * u * end[j]
                    })
                }));
                last = Some(end);
            }
            cmd::CLOSE => {
                let f = first.ok_or_else(missing)?;
                out.push(f);
                last = Some(f);
            }
            _ => {}
        }
    }
    Ok(out)
}

/// 11 `segmentDistance`: a zero-length segment measures to its start.
fn segment_distance(p: Point, a: Point, b: Point) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l = dx * dx + dy * dy;
    let t = if l != 0.0 && !l.is_nan() {
        max2(
            0.0,
            min2(1.0, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l),
        )
    } else {
        0.0
    };
    hypot2(p[0] - a[0] - dx * t, p[1] - a[1] - dy * t)
}

/// `{left, right}` of the even (x) coordinates.
fn x_bounds(p: &[f64]) -> (f64, f64) {
    let xs = p.iter().step_by(2).copied();
    (min_of(xs.clone()), max_of(xs))
}

/// 11: the attachments hold at Bold in both profiles.
fn attachments(set: &SetContext<'_>) -> Result<(Evidence, String), VerifyError> {
    const BOLD: f64 = 2.5;
    let (mut queue_gap, mut offline_gap, mut cloud_gap, mut reconnect_frames) =
        (f64::INFINITY, f64::INFINITY, f64::INFINITY, 0u32);
    let mut cloud_worst: Option<Value> = None;
    let frames = || (0..=144u32).map(|f| (f, f64::from(f) / 60.0));
    let sample = |node: &SceneNode, t: f64| Sampler::Ge.coords(&node.points, t).into_owned();
    // Icons adopted from a spec have no scenes; their own build verified them.
    let mut adopted: Vec<String> = Vec::new();
    for mode in ["", "-strong"] {
        let skip = |name: String, adopted: &mut Vec<String>| {
            let skipped = set.is_adopted(&name);
            if skipped && !adopted.contains(&name) {
                adopted.push(name);
            }
            skipped
        };
        if !skip(format!("queued{mode}"), &mut adopted) {
        let queued = set.scene(&format!("queued{mode}"))?;
        let (stop, ticket) = (
            pick_prefix(&queued, "queued-stop")?,
            pick_prefix(&queued, "queued-ticket-front-frame")?,
        );
        for (_, t) in frames() {
            let gap = coord(&sample(stop, t), 0) - x_bounds(&sample(ticket, t)).1 - BOLD;
            queue_gap = min2(queue_gap, gap);
            ensure(gap >= 1.99, || "Ticket and stop merge at Bold".into())?;
        }
        }
        for direction in ["", "-disconnect"] {
            if skip(format!("reconnect{direction}{mode}"), &mut adopted) {
                continue;
            }
            let sc = set.scene(&format!("reconnect{direction}{mode}"))?;
            for (_, t) in frames() {
                let at = |n: &str| -> Result<Vec<f64>, VerifyError> {
                    Ok(sample(pick_prefix(&sc, &format!("reconnect-{n}"))?, t))
                };
                let (left, right) = (
                    x_bounds(&at("plug-left-frame")?),
                    x_bounds(&at("plug-right-frame")?),
                );
                let before_last = |p: &[f64]| at_index(p.len(), -2).map_or(f64::NAN, |i| p[i]);
                ensure(
                    (before_last(&at("cable-left")?) - left.0).abs() < 0.0001,
                    || "Left cable detaches".into(),
                )?;
                ensure(
                    (coord(&at("cable-right")?, 0) - right.1).abs() < 0.0001,
                    || "Right cable detaches".into(),
                )?;
                for n in ["pin-upper", "pin-lower"] {
                    let p = at(n)?;
                    ensure((coord(&p, 0) - left.1).abs() < 0.0001, || {
                        "Pin detaches".into()
                    })?;
                    ensure(before_last(&p) <= right.0 + 0.0001, || {
                        "Pin enters the receiving counter".into()
                    })?;
                }
                reconnect_frames += 1;
            }
        }
        if !skip(format!("offline{mode}"), &mut adopted) {
        let offline = set.scene(&format!("offline{mode}"))?;
        for n in [
            "wave-outer-left",
            "wave-outer-right",
            "wave-inner-left",
            "wave-inner-right",
        ] {
            for p in contour(pick_prefix(&offline, &format!("offline-{n}"))?, 0.0)? {
                let gap = segment_distance(p, [6.0, 5.0], [26.0, 27.0]) - BOLD;
                offline_gap = min2(offline_gap, gap);
                ensure(gap > 0.65, || {
                    "Offline arc and slash lack a readable Bold gap".into()
                })?;
            }
        }
        }
        if skip(format!("cloud-sync{mode}"), &mut adopted) {
            continue;
        }
        let cloud = set.scene(&format!("cloud-sync{mode}"))?;
        let edge = contour(pick_prefix(&cloud, "cloud-sync-cloud")?, 0.0)?;
        for (f, t) in frames() {
            for part in ["upload", "download"] {
                for p in contour(pick_prefix(&cloud, &format!("cloud-sync-{part}"))?, t)? {
                    let d = edge.windows(2).fold(f64::INFINITY, |d, s| {
                        min2(d, segment_distance(p, s[0], s[1]))
                    });
                    let gap = d - BOLD;
                    if gap < cloud_gap {
                        cloud_gap = gap;
                        cloud_worst =
                            Some(json!({"mode": mode, "part": part, "frame": f, "point": p}));
                    }
                }
            }
        }
    }
    ensure(cloud_gap > 0.5, || "Cloud clearance below 0.5 units".into())?;
    let mut report = Map::new();
    report.insert("result".into(), json!("PASS"));
    report.insert("outlineWeight".into(), json!(BOLD));
    // A gap whose icon was adopted from a spec is not measured here.
    for (key, gap) in [("minimumQueueGap", queue_gap), ("minimumOfflineGap", offline_gap), ("minimumCloudGap", cloud_gap)] {
        if gap.is_finite() || adopted.is_empty() {
            report.insert(key.into(), Value::from(gap));
        }
    }
    report.insert("reconnectFrames".into(), json!(reconnect_frames));
    if !adopted.is_empty() {
        report.insert("adoptedSpecs".into(), json!(adopted));
    }
    if let Some(worst) = cloud_worst {
        report.insert("cloudWorst".into(), worst);
    }
    let evidence = Evidence {
        path: "evidence/attachment-verification.json".into(),
        contents: evidence_json(&Value::Object(report)),
    };
    Ok((
        evidence,
        "PASS attachments: queue, reconnect, offline and cloud-sync clearances".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamped_sampler_and_segment_distance() {
        let keys: Track<Vec<f64>> = vec![
            (0.0, vec![0.0, 10.0]),
            (1.0, vec![10.0, 10.0]),
            (2.0, vec![10.0, 0.0]),
        ];
        assert_eq!(sample_clamped(&keys, -1.0).unwrap(), [0.0, 10.0]);
        assert_eq!(sample_clamped(&keys, 0.5).unwrap(), [5.0, 10.0]);
        assert_eq!(sample_clamped(&keys, 1.5).unwrap(), [10.0, 5.0]);
        assert_eq!(sample_clamped(&keys, 9.0).unwrap(), [10.0, 0.0]);
        assert!(sample_clamped(&keys[..1].to_vec(), 0.0).is_err());
        assert_eq!(segment_distance([3.0, 4.0], [0.0, 0.0], [0.0, 0.0]), 5.0);
        assert_eq!(segment_distance([1.0, 1.0], [0.0, 0.0], [2.0, 0.0]), 1.0);
    }

    #[test]
    fn contour_subdivides_lines() {
        let node = SceneNode {
            name: "n".into(),
            commands: vec![0, 1, 3],
            points: vec![(0.0, vec![0.0, 0.0, 16.0, 0.0])],
            base_points: None,
            transform: false,
            kind: None,
            width: vec![(0.0, 2.0)],
            visibility: None,
        };
        let p = contour(&node, 0.0).unwrap();
        assert_eq!(p.len(), 18);
        assert_eq!(p[1], [1.0, 0.0]);
        assert_eq!(p[16], [16.0, 0.0]);
        assert_eq!(p[17], [0.0, 0.0]);
    }
}
