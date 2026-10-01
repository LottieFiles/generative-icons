//! What set verification asserts about an icon's scenes, checked on the source before a
//! build: each visible part holds its pose from rest to the end of every action (unless
//! the icon is continuous), a gesture ends where it starts, Light and Strong start and end
//! on the same pose, and a stateful icon's reverse actions undo its forward ones.
//!
//! The scenes are the ones `build` writes and verification reads, and the checks are those
//! of verification's template variant (`gai-check` `verify/generic.rs`, `check_node` and
//! `pair_icon`), so a finding here is the failure the build would report after the engine
//! export. The hold and gesture messages state the rule rather than verification's terse
//! text ("reset changes visible geometry"). Only the first failure of each action is
//! reported, and the pairing is checked only once every action passes, as there.

use gai_build::profile::Continuous;
use gai_check::contracts::scene::{Scene, SceneNode, Track};
use gai_check::verify::js::{same_numbers, same_value, truthy};
use gai_js::{Description, FPS, Variant};

/// Why a reverse or reset action's part may move after rest.
const REVERSE_HINT: &str = "hint: in reverse and reset actions, poses get r = true";

/// A part still moves after the action's rest time.
pub const MOVES_AFTER_REST: &str = "MOVES_AFTER_REST";
/// A gesture's visible part ends away from its first pose.
pub const RESET_MOVED: &str = "RESET_MOVED";
/// Light and Strong start or end on different poses.
pub const LIGHT_STRONG_DIFFER: &str = "LIGHT_STRONG_DIFFER";
/// A reverse action does not start where its forward action ends, or end where it starts.
pub const REVERSE_MISMATCH: &str = "REVERSE_MISMATCH";

/// One failed check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// One of the codes above.
    pub code: &'static str,
    /// Verification's message.
    pub message: String,
}

/// Checks icon `icon` of `description` against its scenes, as `gai_build::scene` assembles
/// them (one per variant, in variant order). `continuous` is the set profile's rule for
/// the records' `continuous` field. Fails on scenes that do not parse.
pub fn check_icon(
    description: &Description,
    icon: usize,
    scenes: &[gai_build::scene::Scene],
    continuous: Continuous,
) -> Result<Vec<Finding>, String> {
    let entry = &description.icons[icon];
    let parsed: Vec<Scene> = scenes
        .iter()
        .map(|scene| Scene::from_slice(scene.json.as_bytes()).map_err(|error| format!("{}: {error}", scene.name)))
        .collect::<Result<_, _>>()?;
    let continuous = match continuous {
        Continuous::Omit => false,
        Continuous::Icon => truthy(entry.meta.get("continuous")),
        Continuous::Always => true,
    };
    let mut findings = Vec::new();
    for (variant, scene) in entry.variants.iter().zip(&parsed) {
        if let Some(finding) = check_action(variant, scene, continuous) {
            findings.push(finding);
        }
    }
    if !findings.is_empty() {
        return Ok(findings);
    }
    let scene_of = |strong: bool, reverse: bool| {
        entry.variants.iter().position(|v| v.strong == strong && v.reverse == reverse).map(|i| &parsed[i])
    };
    let (Some(light), Some(strong)) = (scene_of(false, false), scene_of(true, false)) else { return Ok(findings) };
    let flag = |key: &str| entry.variants.first().is_some_and(|v| truthy(v.meta.get(key)));
    if !continuous && !flag("cycleDegrees") {
        findings.extend(pair_light_strong(&entry.name, light, strong));
    }
    if flag("stateful") {
        for (forward, strong) in [(light, false), (strong, true)] {
            if let Some(reverse) = scene_of(strong, true) {
                findings.extend(pair_reverse(&entry.name, forward, reverse));
            }
        }
    }
    Ok(findings)
}

fn first_last<T>(track: &Track<T>) -> Option<(&T, &T)> {
    Some((&track.first()?.1, &track.last()?.1))
}

/// `check_node` for every node of one action: the reset of a gesture and the rest hold.
fn check_action(variant: &Variant, scene: &Scene, continuous: bool) -> Option<Finding> {
    let name = &variant.name;
    let flag = |key: &str| truthy(variant.meta.get(key));
    // The record's rest is in frames; verification divides it back.
    let rest = variant.rest()? * FPS / 60.0;
    for node in &scene.nodes {
        let part = node.base();
        let ((first, last), (v0, v1)) = (first_last(&node.points)?, first_last(visibility(node)?)?);
        if !flag("stateful") && !flag("cycleDegrees") && *v0 > 0.0 && *v1 > 0.0 && !same_numbers(first, last) {
            let message = format!(
                "{name}: {part} ends away from its first pose, but a gesture must end where it starts; hint: return it to rest by the end, or make the icon stateful (rearm) to keep a result"
            );
            return Some(Finding { code: RESET_MOVED, message });
        }
        if continuous {
            continue;
        }
        if *v1 > 0.0
            && let Some((t, _)) = node.points.iter().find(|(t, p)| *t >= rest && !same_numbers(p, last))
        {
            let message = format!(
                "{name}: moving after rest: {part} must hold its final pose from rest ({rest:.3} s) to the end, but is away from it at {t:.3} s; {REVERSE_HINT}"
            );
            return Some(Finding { code: MOVES_AFTER_REST, message });
        }
        if let Some((t, v)) = visibility(node)?.iter().find(|(t, v)| *t >= rest && !same_value(*v, *v1)) {
            let (v, end) = (gai_core::js_number_to_string(*v), gai_core::js_number_to_string(*v1));
            let message = format!(
                "{name}: visibility moving after rest: {part} must hold its final opacity {end} from rest ({rest:.3} s) to the end, but is {v} at {t:.3} s; {REVERSE_HINT}"
            );
            return Some(Finding { code: MOVES_AFTER_REST, message });
        }
    }
    None
}

fn visibility(node: &SceneNode) -> Option<&Track<f64>> {
    node.visibility.as_ref()
}

/// Light and Strong share their visible start and end, node by node.
fn pair_light_strong(icon: &str, light: &Scene, strong: &Scene) -> Option<Finding> {
    let differs = |what: &str, part: &str| Finding {
        code: LIGHT_STRONG_DIFFER,
        message: format!(
            "{icon} {what}: {part} differs between Light and Strong; hint: both motions must start and end on the same pose"
        ),
    };
    for (a, b) in light.nodes.iter().zip(&strong.nodes) {
        let part = a.base();
        let ((a0, a1), (b0, b1)) = (first_last(visibility(a)?)?, first_last(visibility(b)?)?);
        let ((p0, p1), (q0, q1)) = (first_last(&a.points)?, first_last(&b.points)?);
        let failed = [
            (!same_value(*a0, *b0), "initial visibility"),
            (!same_value(*a1, *b1), "final visibility"),
            (*a0 > 0.0 && !same_numbers(p0, q0), "initial drawing"),
            (*a1 > 0.0 && !same_numbers(p1, q1), "final drawing"),
        ];
        if let Some((_, what)) = failed.into_iter().find(|(failed, _)| *failed) {
            return Some(differs(what, part));
        }
    }
    None
}

/// A reverse action starts where its forward action ends and ends where it starts.
fn pair_reverse(icon: &str, forward: &Scene, reverse: &Scene) -> Option<Finding> {
    let (f_name, r_name) = (&forward.name, &reverse.name);
    for (f, r) in forward.nodes.iter().zip(&reverse.nodes) {
        let part = f.base();
        let ((fv0, fv1), (_, rv1)) = (first_last(visibility(f)?)?, first_last(visibility(r)?)?);
        let ((fp0, fp1), (rp0, rp1)) = (first_last(&f.points)?, first_last(&r.points)?);
        let message = if *fv0 > 0.0 && !same_numbers(fp0, rp1) {
            format!("{icon} reverse final geometry: {part} in {r_name} does not end where {f_name} starts")
        } else if *fv1 > 0.0 && !same_numbers(fp1, rp0) {
            format!("{icon} reverse initial geometry: {part} in {r_name} does not start where {f_name} ends")
        } else if !same_value(*fv0, *rv1) {
            format!("Reverse visibility mismatch: {part} in {r_name} does not end as visible as {f_name} starts")
        } else {
            continue;
        };
        return Some(Finding { code: REVERSE_MISMATCH, message });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use gai_js::{Icon, Kind};
    use serde_json::{Value, json};

    /// One scene node of the part `recycle-bin-lid`.
    fn node(points: Value, visibility: Value) -> Value {
        json!({"name": "recycle-bin-lid__color.primary__outline", "commands": [0], "points": points,
            "width": [[0, 2]], "visibility": visibility})
    }

    const STILL: (&str, &str) = ("[[0, [0, 0]], [2.4, [0, 0]]]", "[[0, 1], [2.4, 1]]");

    fn still() -> Value {
        node(serde_json::from_str(STILL.0).unwrap(), serde_json::from_str(STILL.1).unwrap())
    }

    /// An icon whose variants (light, strong, then their reverses when `stateful`) rest at
    /// 0.8 s and draw `nodes[i]`.
    fn icon(stateful: bool, nodes: [Value; 4]) -> (Description, Vec<gai_build::scene::Scene>) {
        let mut variants = Vec::new();
        let mut scenes = Vec::new();
        let names = ["recycle-bin", "recycle-bin-reset", "recycle-bin-strong", "recycle-bin-reset-strong"];
        for (i, (name, node)) in names.iter().zip(nodes).enumerate() {
            let (strong, reverse) = (i >= 2, i % 2 == 1);
            if reverse && !stateful {
                continue;
            }
            let meta = json!({"name": name, "rest": 0.8, "stateful": stateful});
            variants.push(Variant { name: name.to_string(), strong, reverse, meta });
            let scene = json!({"name": name, "nodes": [node]});
            scenes.push(gai_build::scene::Scene {
                icon: 0,
                variant: scenes.len(),
                name: name.to_string(),
                json: scene.to_string(),
            });
        }
        let meta = json!({"name": "recycle-bin", "stateful": stateful});
        let icon = Icon { name: "recycle-bin".into(), meta, source_parts: Vec::new(), parts: Vec::new(), variants };
        (Description { kind: Kind::Modern, icons: vec![icon] }, scenes)
    }

    fn findings(stateful: bool, nodes: [Value; 4]) -> Vec<(&'static str, String)> {
        let (description, scenes) = icon(stateful, nodes);
        let found = check_icon(&description, 0, &scenes, Continuous::Icon).unwrap();
        found.into_iter().map(|finding| (finding.code, finding.message)).collect()
    }

    #[test]
    fn a_still_icon_passes() {
        assert!(findings(false, [still(), still(), still(), still()]).is_empty());
        assert!(findings(true, [still(), still(), still(), still()]).is_empty());
    }

    // The checks are verification's (gai-check `verify/generic.rs` tests); the hold and
    // gesture messages say what the rule is.
    #[test]
    fn holds_and_resets_fail_as_verification_does() {
        let moving = node(json!([[0, [0, 0]], [0.85, [1, 0]], [1, [0, 0]]]), json!([[0, 1], [2.4, 1]]));
        let found = findings(false, [moving, Value::Null, still(), Value::Null]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, MOVES_AFTER_REST);
        assert_eq!(
            found[0].1,
            format!(
                "recycle-bin: moving after rest: recycle-bin-lid must hold its final pose from rest (0.800 s) to the end, but is away from it at 0.850 s; {REVERSE_HINT}"
            )
        );

        let fading = node(serde_json::from_str(STILL.0).unwrap(), json!([[0, 1], [0.9, 0.5], [1, 1]]));
        let found = findings(false, [still(), Value::Null, fading, Value::Null]);
        assert_eq!(
            found,
            [(
                MOVES_AFTER_REST,
                format!(
                    "recycle-bin-strong: visibility moving after rest: recycle-bin-lid must hold its final opacity 1 from rest (0.800 s) to the end, but is 0.5 at 0.900 s; {REVERSE_HINT}"
                )
            )]
        );

        let open = node(json!([[0, [0, 0]], [1, [1, 0]]]), json!([[0, 1], [1, 1]]));
        let found = findings(false, [still(), Value::Null, open.clone(), Value::Null]);
        assert_eq!(
            found,
            [(
                RESET_MOVED,
                "recycle-bin-strong: recycle-bin-lid ends away from its first pose, but a gesture must end where it starts; hint: return it to rest by the end, or make the icon stateful (rearm) to keep a result"
                    .to_string()
            )]
        );
        // A stateful icon keeps its result, so only the pairing applies.
        let found = findings(true, [still(), still(), open, still()]);
        assert_eq!(found[0].0, LIGHT_STRONG_DIFFER);
    }

    #[test]
    fn a_part_away_from_its_pose_at_the_rest_frame_says_so() {
        let at_rest = node(json!([[0, [0, 0]], [0.8, [1, 0]], [1, [0, 0]]]), json!([[0, 1], [2.4, 1]]));
        let found = findings(false, [at_rest, Value::Null, still(), Value::Null]);
        assert_eq!(
            found[0].1,
            format!(
                "recycle-bin: moving after rest: recycle-bin-lid must hold its final pose from rest (0.800 s) to the end, but is away from it at 0.800 s; {REVERSE_HINT}"
            )
        );
    }

    #[test]
    fn pairs_fail_as_verification_does() {
        let ends_apart = node(json!([[0, [0, 0]], [1, [1, 0]]]), json!([[0, 1], [1, 1]]));
        let found = findings(true, [still(), still(), ends_apart, still()]);
        assert_eq!(
            found[0],
            (LIGHT_STRONG_DIFFER, "recycle-bin final drawing: recycle-bin-lid differs between Light and Strong; hint: both motions must start and end on the same pose".to_string())
        );

        let lifted = node(json!([[0, [0, 0]], [1, [2, 0]]]), json!([[0, 1], [1, 1]]));
        let found = findings(true, [lifted.clone(), still(), lifted, still()]);
        assert_eq!(
            found,
            [
                (REVERSE_MISMATCH, "recycle-bin reverse initial geometry: recycle-bin-lid in recycle-bin-reset does not start where recycle-bin ends".to_string()),
                (REVERSE_MISMATCH, "recycle-bin reverse initial geometry: recycle-bin-lid in recycle-bin-reset-strong does not start where recycle-bin-strong ends".to_string()),
            ]
        );
    }

    #[test]
    fn continuous_icons_skip_the_hold_and_the_pairing() {
        let moving = node(json!([[0, [0, 0]], [0.85, [1, 0]], [1, [0, 0]]]), json!([[0, 1], [2.4, 1]]));
        let (mut description, scenes) = icon(false, [moving.clone(), Value::Null, still(), Value::Null]);
        description.icons[0].meta["continuous"] = json!(true);
        assert!(check_icon(&description, 0, &scenes, Continuous::Icon).unwrap().is_empty());
        let (description, scenes) = icon(false, [moving, Value::Null, still(), Value::Null]);
        assert!(check_icon(&description, 0, &scenes, Continuous::Always).unwrap().is_empty());
        assert_eq!(check_icon(&description, 0, &scenes, Continuous::Omit).unwrap().len(), 1);
    }
}
