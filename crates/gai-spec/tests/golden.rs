//! Golden compiles: the plan (the engine call's params) of each example, byte for byte.
//! Regenerate with `GAI_SPEC_BLESS=1 cargo test -p gai-spec --test golden` and review the diff.

use std::path::PathBuf;

const EXAMPLES: [&str; 4] = ["alarm-clock", "late-checkout", "toggle-switch", "task-done"];

fn dir(sub: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join(sub)
}

fn check(name: &str, actual: &str) {
    let path = dir("golden").join(name);
    if std::env::var_os("GAI_SPEC_BLESS").is_some() {
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing golden {name}; run with GAI_SPEC_BLESS=1"));
    assert!(expected == actual, "{name} changed; run with GAI_SPEC_BLESS=1 and review the diff");
}

#[test]
fn plans_match_their_goldens() {
    for id in EXAMPLES {
        let text = std::fs::read_to_string(dir("fixtures").join(format!("{id}.icon.json"))).unwrap();
        let compiled = gai_spec::load(&text).unwrap_or_else(|d| panic!("{id}: {d:?}"));
        let plan = serde_json::to_string_pretty(&compiled.plan.to_json()).unwrap() + "\n";
        check(&format!("{id}.plan.json"), &plan);
    }
}

#[test]
fn engine_calls_match_their_goldens() {
    for id in ["alarm-clock", "toggle-switch"] {
        let text = std::fs::read_to_string(dir("fixtures").join(format!("{id}.icon.json"))).unwrap();
        let compiled = gai_spec::load(&text).unwrap();
        let build = serde_json::to_string_pretty(&gai_spec::engine_call(&compiled, &format!("{id}.json"))).unwrap() + "\n";
        check(&format!("{id}.call.json"), &build);
        let sheet = serde_json::to_string_pretty(&compiled.sheet()).unwrap() + "\n";
        check(&format!("{id}.sheet.json"), &sheet);
    }
}

#[test]
fn the_script_reads_every_plan_field_it_is_given() {
    // The plan keys the compiler writes are the ones the script reads.
    for key in ["P.rigs", "P.geos", "P.layers", "P.motions", "r.pivot", "r.parent", "r.split", "L.geos", "L.paints", "L.masks", "L.trim", "L.vis", "L.opacity", "L.matte", "L.rig", "L.style", "t.target", "t.prop", "t.keys", "motion.active", "motion.rest", "params.file", "params.sheet"] {
        assert!(gai_spec::SCRIPT.contains(key), "{key}");
    }
    assert!(gai_spec::SCRIPT.contains("creator.removeComp(c.id)"));
    assert!(!gai_spec::SCRIPT.contains("deleteNodes("));
}

#[test]
fn the_schema_is_json_and_names_the_format() {
    let schema: serde_json::Value = serde_json::from_str(gai_spec::SCHEMA).unwrap();
    assert_eq!(schema["properties"]["format"]["const"], gai_spec::FORMAT);
}
