//! Rule-engine behaviour on synthetic scenes: values, gates, key order and failure modes.

use gai_check::contracts::{Clearance, ContractError, RuleSet, Scene, SceneSource, SetManifest};
use serde_json::{Value, json};
use std::collections::HashMap;

/// Scenes held in memory, keyed by action name.
struct Scenes(HashMap<String, Scene>);

impl SceneSource for Scenes {
    fn load(&self, name: &str) -> Result<Scene, ContractError> {
        self.0
            .get(name)
            .cloned()
            .ok_or_else(|| ContractError::Geometry(format!("no scene {name}")))
    }
}

/// A stroked straight line from `from` to `to`, hidden from `hide_after + 0.01` seconds.
fn line(name: &str, from: [f64; 2], to: [f64; 2], width: f64, hide_after: Option<f64>) -> Value {
    let visibility = match hide_after {
        Some(t) => json!([[0, 1], [t, 1], [t + 0.01, 0], [2.4, 0]]),
        None => json!([[0, 1], [2.4, 1]]),
    };
    json!({
        "name": format!("{name}__color.primary__outline.bold"),
        "commands": [0, 1],
        "points": [[0, [from[0], from[1], to[0], to[1]]], [2.4, [from[0], from[1], to[0], to[1]]]],
        "type": "stroke", "color": "#00ddb3",
        "opacity": [[0, 0], [2.4, 0]], "width": [[0, width], [2.4, width]],
        "visibility": visibility
    })
}

/// One icon `cup` with a Light forward action and a Strong reverse action, both drawing
/// two horizontal Bold lines 4 units apart (painted gap 2 with width 2).
fn fixture() -> (SetManifest, Scenes) {
    let manifest: SetManifest = serde_json::from_value(json!({
        "id": "90-test",
        "icons": [{"name": "cup", "animations": [
            {"name": "cup", "direction": "forward", "motionId": "light"},
            {"name": "cup-reset-strong", "direction": "reverse", "motionId": "strong"}
        ]}]
    }))
    .unwrap();
    let nodes = json!([
        line("cup-top-bold", [8.0, 10.0], [24.0, 10.0], 2.0, None),
        line("cup-bottom-bold", [8.0, 14.0], [24.0, 14.0], 2.0, Some(1.0)),
    ]);
    let scene = |name: &str| {
        serde_json::from_value::<Scene>(json!({"name": name, "nodes": nodes})).unwrap()
    };
    let scenes = ["cup", "cup-reset-strong"]
        .into_iter()
        .map(|n| (n.to_string(), scene(n)))
        .collect();
    (manifest, Scenes(scenes))
}

fn run(contract: Value) -> Result<gai_check::contracts::ClearanceReport, ContractError> {
    let (manifest, scenes) = fixture();
    let rules = RuleSet::from_slice(&serde_json::to_vec(&contract).unwrap())?;
    Clearance::Rules(Box::new(rules)).run(&manifest, &scenes)
}

fn gap(label: &str, extra: Value) -> Value {
    let mut rule = json!({"kind": "gap", "a": "top", "b": "bottom", "label": label});
    rule.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    rule
}

#[test]
fn gap_values_follow_the_painted_model() {
    let report = run(json!({"icons": {"cup": [gap("rails", json!({"min": 1.5}))]}})).unwrap();
    assert_eq!(report.evidence["gaps"]["cup rails"], 2.0);
    // Margin: the lines sit 7 units inside the left and right edges, less the 1-unit radius.
    assert_eq!(report.evidence["minimumPaintedMargin"], 7.0);
    // Per action: both margins and the gap while the bottom line shows (f = 0..=60),
    // then the top margin alone.
    let both = 61;
    assert_eq!(
        report.evidence["checks"],
        json!((both * 3 + (145 - both)) * 2)
    );
    assert!(report.passed());
}

#[test]
fn gap_keys_follow_first_evaluation_and_merge_duplicates() {
    let contract = json!({"icons": {"cup": [
        gap("late", json!({"from": 0.5})),
        gap("first", json!({})),
        gap("first", json!({"min": 3})),
    ]}});
    let report = run(contract).unwrap();
    let keys: Vec<&String> = report.evidence["gaps"]
        .as_object()
        .unwrap()
        .keys()
        .collect();
    assert_eq!(keys, ["cup first", "cup late"]);
    // The duplicate with min 3 fails with the v1 message, labelled but not keyed.
    assert_eq!(report.errors[0], "cup first frame 0: 2 < 3");
    assert_eq!(report.evidence["result"], "FAIL");
}

#[test]
fn profile_direction_and_time_gates() {
    let contract = json!({"icons": {"cup": [
        gap("strong only", json!({"profile": "strong", "min": 9})),
        gap("forward only", json!({"direction": "forward", "min": 9, "to": 0.05})),
    ]}});
    let report = run(contract).unwrap();
    assert!(report.errors.iter().all(
        |e| e.starts_with("cup-reset-strong strong only") || e.starts_with("cup forward only")
    ));
    // `to` is inclusive: 3/60 == 0.05, so frames 0..=3 run.
    assert_eq!(
        report
            .errors
            .iter()
            .filter(|e| e.starts_with("cup forward only"))
            .count(),
        4
    );
    assert_eq!(
        report
            .errors
            .iter()
            .filter(|e| e.starts_with("cup-reset-strong"))
            .count(),
        61
    );
}

#[test]
fn compare_rules_record_gate_and_report() {
    let contract = json!({
        "format": "gai-verify-rules", "version": 2, "set": "90-test",
        "dialect": {"sampler": "gt", "flatten": {"cubicSteps": 20, "skipDegenerate": false, "close": "first", "unknown": "throw"},
                    "pick": "bold", "visible": {"op": ">=", "value": 0.05}, "slack": 0.015, "gapRadius": "sequential", "failure": "collect"},
        "icons": {"cup": [
            {"kind": "compare", "label": "top length", "value": {"sub": [{"at": [{"box": "top"}, 2]}, {"at": [{"box": "top"}, 0]}]},
             "op": ">=", "limit": 18},
            {"kind": "compare", "label": "hidden", "value": {"vis": "bottom"}, "op": "<=", "limit": 0.5, "slack": 0, "record": false,
             "when": {"gt": [{"vis": "top"}, 0.5]}, "message": "{action} bottom still shows at {f}"}
        ]},
        "evidence": {"fields": [["result", {"status": true}], ["gaps", {"map": "rules"}], ["checks", {"counter": "checks"}], ["errors", {"errors": true}]],
                     "key": "{icon}: {label}", "message": "{action} {key} frame {f}: {value} < {min}"}
    });
    let report = run(contract).unwrap();
    // Box width 16 plus the 1-unit radius on both sides.
    assert_eq!(report.evidence["gaps"], json!({"cup: top length": 18.0}));
    assert_eq!(report.evidence["checks"], 145 * 2 * 2);
    assert_eq!(report.errors.len(), 2 * 61);
    assert_eq!(report.errors[0], "cup bottom still shows at 0");
    assert_eq!(
        report
            .evidence
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        ["result", "gaps", "checks", "errors"]
    );
}

#[test]
fn assert_mode_aborts_with_the_rule_message() {
    let contract = json!({"icons": {"cup": [gap("tight", json!({"min": 5, "failure": "assert", "message": "{icon} {label} at {f}: {value}"}))]}});
    match run(contract) {
        Err(ContractError::Assertion(m)) => assert_eq!(m, "cup tight at 0: 2"),
        other => panic!("expected an assertion, got {other:?}"),
    }
}

#[test]
fn missing_contract_and_lazy_errors() {
    assert!(
        matches!(run(json!({"icons": {}})), Err(ContractError::MissingContract(i)) if i == "cup")
    );
    assert!(matches!(
        run(json!({"icons": {"cup": null}})),
        Err(ContractError::MissingContract(_))
    ));
    // A live unknown kind fails once its part is visible.
    assert!(
        matches!(run(json!({"icons": {"cup": [{"kind": "wobble", "a": "top", "label": "w"}]}})), Err(ContractError::UnknownRule(k)) if k == "wobble")
    );
    // Gated off everywhere: never evaluated, never an error.
    let quiet = json!({"icons": {"cup": [
        {"kind": "wobble", "a": "top", "label": "w", "profile": "none"},
        {"kind": "gap", "a": "missing", "b": "top", "label": "m", "from": 9}
    ]}});
    assert!(run(quiet).unwrap().passed());
    // A missing second part fails only once the first part is visible.
    assert!(
        matches!(run(json!({"icons": {"cup": [gap("m", json!({"b": "missing"}))]}})), Err(ContractError::MissingPart(p)) if p == "cup-missing")
    );
}
