//! Compile-level checks: every diagnostic an agent can act on, and the shapes of the plans.

use gai_spec::{Level, load};
use serde_json::{Value, json};

fn spec(extra: Value) -> String {
    let mut base = json!({
        "format": "gai-icon/2", "id": "x", "label": "X",
        "intent": "Light: a small nudge. Strong: a bigger, sharper nudge.",
    });
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base.to_string()
}

/// `(code, path)` of every error.
fn errors(extra: Value) -> Vec<(String, String)> {
    match load(&spec(extra)) {
        Ok(_) => Vec::new(),
        Err(d) => d.0.iter().filter(|x| x.level == Level::Error).map(|x| (x.code.to_string(), x.path.clone())).collect(),
    }
}

fn codes(extra: Value) -> Vec<String> {
    errors(extra).into_iter().map(|(c, _)| c).collect()
}

fn line() -> Value {
    json!({"shape": {"line": [[10, 16], [22, 16]]}})
}

#[test]
fn names_resolve_with_suggestions() {
    let d = load(&spec(json!({
        "parts": {"bar": line()},
        "rigs": {"bar": {"pivot": [16, 16], "carries": ["baz"]}},
        "motion": {"bar.rotate": {"light": [[0, 0], [0.2, 5], [0.4, 0]], "strong": [[0, 0], [0.2, 9], [0.4, 0]]}}
    })))
    .unwrap_err();
    let e = d.errors().next().unwrap();
    assert_eq!((e.code, e.path.as_str()), ("UNKNOWN_NAME", "/rigs/bar/carries/0"));
    assert_eq!(e.hint.as_deref(), Some("did you mean `bar`?"));
}

#[test]
fn rotation_needs_a_constant_pivot() {
    let e = errors(json!({
        "parts": {"bar": line()},
        "rigs": {"arm": {"carries": ["bar"]}},
        "motion": {"arm.rotate": {"light": [[0, 0], [0.2, 5], [0.4, 0]], "strong": [[0, 0], [0.2, 9], [0.4, 0]]}}
    }));
    assert_eq!(e, [("PIVOT".to_string(), "/motion/arm.rotate".to_string())]);
    // A part keyed by rotation without a rig of its own also needs one.
    assert_eq!(codes(json!({"parts": {"bar": line()}, "motion": {"bar.rotate": {"light": [[0, 0], [0.2, 5], [0.4, 0]], "strong": [[0, 0], [0.2, 9], [0.4, 0]]}}})), ["PIVOT"]);
}

#[test]
fn rest_values_catch_factors_keyed_as_offsets() {
    // The critique's case: scale keyed as an offset collapses the part at rest.
    let e = errors(json!({
        "parts": {"dot": {"shape": {"circle": [[16, 16], 4]}, "role": "fill"}},
        "rigs": {"dot": {"pivot": [16, 16]}},
        "motion": {"dot.scale": {"light": [[0, 0], [0.3, 0.2], [0.6, 0]], "strong": [[0, 1], [0.3, 1.4], [0.6, 1]]}}
    }));
    assert_eq!(e[0], ("REST_VALUE".to_string(), "/motion/dot.scale/light".to_string()));
    // A gesture ends where it starts; a whole turn is the same pose.
    assert_eq!(codes(json!({"parts": {"bar": line()}, "rigs": {"bar": {"pivot": [16, 16]}}, "motion": {"bar.rotate": {"light": [[0, 0], [0.4, 30]], "strong": [[0, 0], [0.4, 60]]}}})), ["REST_VALUE", "REST_VALUE"]);
    assert!(codes(json!({"parts": {"bar": line()}, "rigs": {"bar": {"pivot": [16, 16]}}, "motion": {"bar.rotate": {"light": [[0, 0], [0.4, 360]], "strong": [[0, 0], [0.4, 720]]}}})).is_empty());
    // A stateful icon keeps its result.
    assert!(codes(json!({
        "state": {"from": "off", "to": "on", "labels": ["On", "Off"]},
        "parts": {"bar": line()}, "rigs": {"bar": {"pivot": [16, 16]}},
        "motion": {"bar.rotate": {"light": [[0, 0], [0.4, 30]], "strong": [[0, 0], [0.2, 40], [0.4, 30]]}}
    })).is_empty());
}

#[test]
fn timing_errors() {
    let moves_after = codes(json!({
        "parts": {"bar": line()}, "timing": {"light": 0.2},
        "motion": {"bar.x": {"light": [[0, 0], [0.2, 2], [0.4, 0]], "strong": [[0, 0], [0.2, 3], [0.4, 0]]}}
    }));
    assert_eq!(moves_after, ["MOVES_AFTER_REST"]);
    let order = codes(json!({"parts": {"bar": line()}, "motion": {"bar.x": {"light": [[0, 0], [0.3, 2], [0.2, 0]], "strong": [[0, 0], [0.2, 3], [0.4, 0]]}}}));
    assert_eq!(order, ["KEY_ORDER"]);
    let missing = codes(json!({"parts": {"bar": line()}, "motion": {"bar.x": {"light": [[0, 0], [0.2, 2], [0.4, 0]]}}}));
    assert_eq!(missing, ["NO_MOTION"]);
}

#[test]
fn rigs_cannot_loop_or_share_a_part() {
    assert!(codes(json!({
        "parts": {"bar": line()},
        "rigs": {"a": {"pivot": [16, 16], "carries": ["b"]}, "b": {"pivot": [16, 16], "carries": ["a"]}},
        "motion": {"a.rotate": {"light": [[0, 0], [0.2, 5], [0.4, 0]], "strong": [[0, 0], [0.2, 9], [0.4, 0]]}}
    })).contains(&"RIG_CYCLE".to_string()));
    assert!(codes(json!({
        "parts": {"bar": line()},
        "rigs": {"a": {"pivot": [16, 16], "carries": ["bar"]}, "b": {"pivot": [16, 16], "carries": ["bar"]}},
        "motion": {"a.rotate": {"light": [[0, 0], [0.2, 5], [0.4, 0]], "strong": [[0, 0], [0.2, 9], [0.4, 0]]}}
    })).contains(&"CARRIED_TWICE".to_string()));
}

#[test]
fn moving_knockouts_must_share_a_rig() {
    // world-clock: two independently turning hands knocked out of one Solid dial.
    let e = codes(json!({
        "parts": {
            "dial": {"shape": {"circle": [[16, 16], 9]}, "role": "surface", "solid": "fill", "knockout": ["hour", "minute"]},
            "hour": {"shape": {"line": [[16, 16], [19, 16]]}},
            "minute": {"shape": {"line": [[16, 16], [16, 10]]}}
        },
        "rigs": {"hour": {"pivot": [16, 16]}, "minute": {"pivot": [16, 16]}},
        "motion": {
            "hour.rotate": {"light": [[0, 0], [0.4, 30], [0.6, 0]], "strong": [[0, 0], [0.4, 60], [0.6, 0]]},
            "minute.rotate": {"light": [[0, 0], [0.6, 360]], "strong": [[0, 0], [0.6, 720]]}
        }
    }));
    assert_eq!(e, ["KNOCKOUT_RIGS"]);
}

#[test]
fn morphs_need_one_topology() {
    let e = codes(json!({
        "parts": {"m": {"shape": {"morph": [{"polyline": [[4, 4], [8, 8], [12, 4]]}, {"polyline": [[4, 4], [12, 4]]}], "by": "open"}}},
        "motion": {"open": {"light": [[0, 0], [0.3, 1], [0.6, 0]], "strong": [[0, 0], [0.3, 1], [0.6, 0]]}}
    }));
    assert_eq!(e, ["MORPH_TOPOLOGY"]);
    // A bare name keys a morph's driver; anything else says so.
    let e = codes(json!({"parts": {"bar": line()}, "motion": {"open": {"light": [[0, 0], [0.3, 1], [0.6, 0]], "strong": [[0, 0], [0.3, 1], [0.6, 0]]}}}));
    assert_eq!(e, ["UNKNOWN_NAME"]);
}

#[test]
fn unimplemented_terms_say_so() {
    let e = codes(json!({
        "parts": {"door": {"shape": {"rect": [8, 6, 16, 20]}}},
        "rigs": {"door": {"swing": {"hinge": [8, 16]}}},
        "motion": {"door.swing": {"light": [[0, 0], [0.4, 80]], "strong": [[0, 0], [0.4, 100]]}}
    }));
    assert!(e.iter().all(|c| c == "NOT_IMPLEMENTED") && !e.is_empty(), "{e:?}");
    let e = codes(json!({
        "parts": {"c": {"shape": {"minus": [{"circle": [[16, 16], 8]}, {"circle": [[20, 16], 7]}]}, "role": "fill"}},
        "motion": {"c.opacity": {"light": [[0, 1], [0.2, 0.5], [0.4, 1]], "strong": [[0, 1], [0.2, 0.2], [0.4, 1]]}}
    }));
    assert_eq!(e, ["NOT_IMPLEMENTED"]);
}

#[test]
fn one_path_changing_track_per_shape() {
    let e = codes(json!({
        "parts": {"dot": {"shape": {"circle": [[16, 16], 4]}}},
        "rigs": {"dot": {"pivot": [16, 16]}},
        "motion": {
            "dot.r": {"light": [[0, 0], [0.3, 2], [0.6, 0]], "strong": [[0, 0], [0.3, 3], [0.6, 0]]},
            "dot.scale": {"light": [[0, 1], [0.3, 1.2], [0.6, 1]], "strong": [[0, 1], [0.3, 1.4], [0.6, 1]]}
        }
    }));
    assert!(e.contains(&"COMBINED_PATH_TRACKS".to_string()), "{e:?}");
}

#[test]
fn paint_past_the_canvas_is_an_error_and_near_it_a_warning() {
    let e = errors(json!({"parts": {"bar": line()}, "motion": {"bar.x": {"light": [[0, 0], [0.2, 12], [0.4, 0]], "strong": [[0, 0], [0.2, 9], [0.4, 0]]}}}));
    assert!(e.iter().any(|(c, p)| c == "CANVAS_MARGIN" && p == "/parts/bar"), "{e:?}");
    let c = load(&spec(json!({"parts": {"bar": line()}, "motion": {"bar.x": {"light": [[0, 0], [0.2, 8.4], [0.4, 0]], "strong": [[0, 0], [0.2, 8], [0.4, 0]]}}}))).unwrap();
    assert!(c.diagnostics.0.iter().any(|d| d.code == "CANVAS_MARGIN" && d.level == Level::Warning), "{:?}", c.diagnostics);
}

#[test]
fn contact_pivots_become_two_rigs_one_per_sign() {
    let text = include_str!("fixtures/alarm-clock.icon.json");
    let c = load(text).unwrap();
    let names: Vec<(&str, Option<&str>)> = c.plan.rigs.iter().map(|r| (r.name.as_str(), r.parent.as_deref())).collect();
    assert_eq!(names, [("rig-clock-b", None), ("rig-clock-a", Some("rig-clock-b")), ("rig-bell-l", Some("rig-clock-a")), ("rig-bell-r", Some("rig-clock-a"))]);
    let strong = &c.plan.motions[1];
    let b = strong.tracks.iter().find(|t| t.target == "rig-clock-b").unwrap();
    let a = strong.tracks.iter().find(|t| t.target == "rig-clock-a").unwrap();
    // Positive (clockwise) angles tip about the right foot (engine radians are counter-clockwise).
    let scalar = |k: &gai_spec::plan::PKey| if let gai_spec::plan::KeyValue::Scalar(v) = k.v { v } else { 0.0 };
    assert!(b.keys.iter().all(|k| scalar(k) <= 1e-12) && a.keys.iter().all(|k| scalar(k) >= -1e-12));
}

#[test]
fn reverse_modes_name_their_animations() {
    let base = |mode: &str, from: &str, to: &str, labels: [&str; 2]| {
        load(&spec(json!({
            "state": {"from": from, "to": to, "labels": labels, "reverse": mode},
            "parts": {"bar": line()},
            "motion": {"bar.y": {"light": [[0, 0], [0.3, 3]], "strong": [[0, 0], [0.15, -1], [0.3, 4], {"settle": 0.5}]}}
        })))
    };
    let auto = base("auto", "off", "on", ["On", "Off"]).unwrap();
    let names: Vec<&str> = auto.motions.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, ["x", "x-strong", "x-reverse", "x-reverse-strong"]);
    let fade = base("fade", "prepared", "result", ["Play action", "Reset preview"]).unwrap();
    let names: Vec<(&str, f64)> = fade.motions.iter().map(|m| (m.name.as_str(), m.active)).collect();
    assert_eq!(names, [("x", 0.3), ("x-strong", 0.5), ("x-reset", 0.28), ("x-reset-strong", 0.3)]);
    let wrong = base("fade", "off", "on", ["On", "Off"]).unwrap_err();
    assert_eq!(wrong.errors().next().unwrap().code, "FADE_STATE");
}

#[test]
fn roles_expand_to_the_style_layers() {
    let c = load(&spec(json!({
        "parts": {
            "frame": {"shape": {"rect": [6, 6, 20, 20], "r": 3}, "role": "surface"},
            "mark": {"shape": {"circle": [[16, 16], 2]}, "role": "fill"},
            "rim": {"shape": {"line": [[8, 28], [24, 28]]}, "solid": "none"}
        },
        "motion": {"mark.opacity": {"light": [[0, 1], [0.2, 0.4], [0.4, 1]], "strong": [[0, 1], [0.2, 0], [0.4, 1]]}}
    })))
    .unwrap();
    let layers: Vec<(String, Vec<gai_spec::plan::Paint>)> = c.plan.layers.iter().map(|l| (l.full_name(), l.paints.clone())).collect();
    use gai_spec::plan::Paint::*;
    let fill = Fill { even_odd: false };
    assert_eq!(
        layers,
        [
            ("x-frame-tint__color.primary__tint".into(), vec![fill]),
            ("x-frame-light__color.primary__outline.light".into(), vec![Stroke(1.5)]),
            ("x-frame-regular__color.primary__outline.regular".into(), vec![Stroke(2.0)]),
            ("x-frame-bold__color.primary__outline.bold".into(), vec![Stroke(2.5)]),
            ("x-frame-solid__color.primary__solid".into(), vec![fill, Stroke(2.6)]),
            ("x-mark-light__color.primary__outline.light".into(), vec![fill]),
            ("x-mark-regular__color.primary__outline.regular".into(), vec![fill]),
            ("x-mark-bold__color.primary__outline.bold".into(), vec![fill]),
            ("x-mark-solid__color.primary__solid".into(), vec![fill]),
            ("x-rim-light__color.primary__outline.light".into(), vec![Stroke(1.5)]),
            ("x-rim-regular__color.primary__outline.regular".into(), vec![Stroke(2.0)]),
            ("x-rim-bold__color.primary__outline.bold".into(), vec![Stroke(2.5)]),
        ]
    );
    assert!(c.duotone_fill);
}

#[test]
fn compiles_are_reproducible_and_small() {
    let text = include_str!("fixtures/alarm-clock.icon.json");
    let a = load(text).unwrap().plan.to_json().to_string();
    let b = load(text).unwrap().plan.to_json().to_string();
    assert_eq!(a, b);
    // v1 sent 269,791 B of scene params for alarm-clock Strong alone.
    assert!(a.len() < 9000, "{}", a.len());
}

#[test]
fn knockouts_that_draw_on_or_fade_ride_a_matte_with_their_trim() {
    // A static mask would leave the check's hole in the Solid card before the check draws on.
    let surface = json!({"shape": {"rect": [4, 8, 24, 18], "r": 2}, "role": "surface", "knockout": ["check"]});
    let check = json!({"shape": {"polyline": [[9, 17], [13, 21], [22, 12]]}});
    let c = load(&spec(json!({
        "parts": {"card": surface, "check": check},
        "motion": {"check.draw": {"light": [[0, 0], [0.4, 1, "out"]], "strong": [[0.1, 0], [0.3, 1, "hit"]]}}
    })))
    .unwrap();
    let solid = c.plan.layers.iter().find(|l| l.part == "card" && l.variant == "solid").unwrap();
    assert!(solid.masks.is_empty(), "{:?}", solid.masks);
    let matte = solid.matte.as_ref().expect("a matte");
    assert_eq!((matte.trim.as_deref(), matte.vis.as_deref(), matte.geos.as_slice()), (Some("check.draw"), Some("check.opacity"), ["check/0".to_string()].as_slice()));
    // Without a track of its own, on the card's rig, the same knockout is a static mask.
    let c = load(&spec(json!({
        "parts": {"card": surface, "check": check, "spark": {"shape": {"circle": [[26, 5], 1.5]}, "role": "fill"}},
        "motion": {"spark.opacity": {"light": [[0, 1], [0.2, 0.4], [0.4, 1]], "strong": [[0, 1], [0.2, 0], [0.4, 1]]}}
    })))
    .unwrap();
    let solid = c.plan.layers.iter().find(|l| l.part == "card" && l.variant == "solid").unwrap();
    assert!(solid.matte.is_none() && !solid.masks.is_empty());
}

#[test]
fn one_matte_cannot_follow_two_parts_tracks() {
    let e = errors(json!({
        "parts": {
            "card": {"shape": {"rect": [4, 8, 24, 18]}, "role": "surface", "knockout": ["check", "dot"]},
            "check": {"shape": {"polyline": [[9, 17], [13, 21], [22, 12]]}},
            "dot": {"shape": {"circle": [[24, 12], 1.2]}, "role": "fill"}
        },
        "motion": {
            "check.draw": {"light": [[0, 0], [0.4, 1]], "strong": [[0.1, 0], [0.3, 1, "hit"]]},
            "dot.opacity": {"light": [[0, 1], [0.2, 0.4], [0.4, 1]], "strong": [[0, 1], [0.2, 0], [0.4, 1]]}
        }
    }));
    assert_eq!(e, [("KNOCKOUT_TRACKS".to_string(), "/parts/card/knockout".to_string())]);
}

#[test]
fn holes_of_a_reshaped_surface_follow_its_keys() {
    let c = load(&spec(json!({
        "parts": {"badge": {"shape": {"circle": [[16, 16], 8]}, "role": "surface", "holes": [{"circle": [[16, 16], 2.5]}]}},
        "rigs": {"badge": {"pivot": [16, 16]}},
        "motion": {"badge.scale": {"light": [[0, 1], [0.2, 1.1], [0.4, 1]], "strong": [[0, 1], [0.15, 0.85], [0.3, 1.2, "hit"], [0.5, 1]]}}
    })))
    .unwrap();
    for variant in ["tint", "solid"] {
        let layer = c.plan.layers.iter().find(|l| l.variant == variant).unwrap();
        assert_eq!(layer.masks.iter().map(|m| m.geo.as_deref()).collect::<Vec<_>>(), [Some("badge/h0")], "{variant}");
    }
    let keyed: Vec<&str> = c.plan.motions[1].tracks.iter().filter(|t| t.prop == gai_spec::plan::Prop::PathPoints).map(|t| t.target.as_str()).collect();
    assert!(keyed.contains(&"badge/h0") && keyed.contains(&"badge/0"), "{keyed:?}");
    // The plan names the geometry, so the engine script keys the mask with it.
    let plan = c.plan.to_json();
    let tint = plan["layers"].as_array().unwrap().iter().find(|l| l["style"] == "tint").unwrap();
    assert_eq!(tint["masks"][0]["geo"], "badge/h0");
    assert!(gai_spec::SCRIPT.contains("if (m.geo) add(m.geo, mask)"));
}

#[test]
fn lints_find_parts_lost_in_solid_and_cuts_that_miss() {
    let warnings = |extra: Value| -> Vec<(String, String)> {
        let c = load(&spec(extra)).unwrap();
        c.diagnostics.0.iter().filter(|d| d.level == Level::Warning).map(|d| (d.code.to_string(), d.path.clone())).collect()
    };
    let motion = json!({"mark.opacity": {"light": [[0, 1], [0.2, 0.5], [0.4, 1]], "strong": [[0, 1], [0.2, 0.1], [0.4, 1]]}});
    // A mark inside a filled Solid card vanishes in Solid.
    let lost = warnings(json!({
        "parts": {"card": {"shape": {"rect": [4, 6, 24, 20], "r": 2}, "role": "surface"}, "mark": {"shape": {"line": [[10, 16], [22, 16]]}}},
        "motion": motion
    }));
    assert_eq!(lost, [("SOLID_HIDDEN".to_string(), "/parts/mark".to_string())]);
    // Knocked out, or on a hollow Solid, it reads.
    for card in [json!({"shape": {"rect": [4, 6, 24, 20], "r": 2}, "role": "surface", "knockout": ["mark"]}), json!({"shape": {"rect": [4, 6, 24, 20], "r": 2}, "role": "surface", "solid": "ring"})] {
        assert!(warnings(json!({"parts": {"card": card, "mark": {"shape": {"line": [[10, 16], [22, 16]]}}}, "motion": motion})).is_empty());
    }
    // A hole off the surface cuts nothing; a knockout off it vanishes in Solid.
    let miss = warnings(json!({
        "parts": {
            "card": {"shape": {"rect": [4, 6, 14, 20]}, "role": "surface", "holes": [{"circle": [[26, 10], 1.5]}], "knockout": ["mark"]},
            "mark": {"shape": {"line": [[22, 20], [28, 20]]}}
        },
        "motion": motion
    }));
    assert_eq!(miss, [("HOLE_OUTSIDE".to_string(), "/parts/card/holes/0".to_string()), ("HOLE_OUTSIDE".to_string(), "/parts/card/knockout/0".to_string())]);
}

#[test]
fn a_follow_takes_a_comment_and_a_bounded_scale() {
    // Two bars, the second turning against the first; comments where the guide allows them.
    let bars = |follow: Value| {
        json!({
            "//": "spec",
            "parts": {"a": {"//": "part", "shape": {"line": [[8, 12], [24, 12]]}}, "b": {"shape": {"line": [[8, 20], [24, 20]]}}},
            "rigs": {"a": {"//": "rig", "pivot": [16, 12]}, "b": {"pivot": [16, 20]}},
            "motion": {
                "a.rotate": {"//": "track", "light": [[0, 0], [0.2, 5], [0.4, 0]], "strong": [[0, 0], [0.2, 9], [0.4, 0]]},
                "b.rotate": follow,
            }
        })
    };
    let whole = bars(json!({"//": "mirrors a", "follow": "a.rotate", "scale": -1}));
    let compiled = load(&spec(whole)).unwrap();
    assert!(compiled.diagnostics.0.is_empty(), "{:?}", compiled.diagnostics);
    let per_motion = json!({"light": {"//": "light only", "follow": "a.rotate", "lag": 0.04}, "strong": [[0, 0], [0.2, -9], [0.4, 0]]});
    assert_eq!(errors(bars(per_motion)), []);
    // scale is from -4 to 4, as the guide says.
    assert!(errors(bars(json!({"follow": "a.rotate", "scale": 4}))).is_empty());
    assert_eq!(errors(bars(json!({"follow": "a.rotate", "scale": -4.5}))), [("V2_SCHEMA".to_string(), "/motion/b.rotate/scale".to_string())]);
}
