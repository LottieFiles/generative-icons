//! `generation-harness.test.mjs` cases for preflight and the checked build, and the preflight
//! spec's edge cases. Expected reports are `JSON.stringify(checkDefinitions(icons))` from Node
//! 26.7.0 running the real `preflight.mjs` on the same module text.
//!
//! The harness's retrieval and reference-packet cases belong to the CLI's `search` and
//! `context` commands, and its catalog-contract case is in `preflight_catalog.rs`.

use gai_check::preflight::{
    Catalog, CheckedReport, Error, FamilyContract, LaunchOutcome, Preflight, Report, Status, Target, check_definitions,
};
use gai_core::{sha256_hex, to_js_json};
use gai_js::{Limits, ProbeOutcome, ProbeRequest, ProbeWidth, SetRuntime, SetSource, rquickjs};
use serde_json::Value as Json;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

/// `generation-harness.test.mjs:10-12`.
const PRELUDE: &str = "// generation-harness.test.mjs:10-12
const fixture = () => ({ name: 'test-arrow', label: 'Test arrow', duotoneFill: false,
  parts: [{ name: 'shaft', d: 'M8 16 L24 16', type: 'stroke', width: 2, style: 'outline' }],
  timings: { light: { active: .4, rest: .5, keyPoses: [0, .4] }, strong: { active: .6, rest: .7, keyPoses: [0, .6] } } });
";

/// The repo root (`GAI_LIBRARY_ROOT`, else two folders above this crate).
fn root() -> PathBuf {
    std::env::var_os("GAI_LIBRARY_ROOT")
        .map_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."), PathBuf::from)
}

fn temp_set(prefix: &str, definitions: &str) -> TempDir {
    let dir = tempfile::Builder::new().prefix(prefix).tempdir().unwrap();
    fs::write(dir.path().join("definitions.mjs"), definitions).unwrap();
    dir
}

/// Runs `f` on a runtime loaded from `PRELUDE + body`.
fn with_fixture<T: Send>(body: &str, f: impl FnOnce(&SetRuntime) -> T + Send) -> T {
    let set = temp_set("preflight-fixture-", &format!("{PRELUDE}{body}\n"));
    let source = SetSource::user_project(set.path());
    gai_js::run_on_worker(&Limits::default(), || f(&SetRuntime::load(&source, &Limits::default()).unwrap()))
}

fn check(body: &str) -> gai_check::preflight::Result<Report> {
    let contract = FamilyContract::load(&root()).unwrap();
    with_fixture(body, |runtime| check_definitions(runtime, &contract))
}

/// Each case: the module body after `PRELUDE`, then Node's compact report.
macro_rules! fixtures {
    ($($name:ident: $body:literal => $expected:literal;)*) => {
        $(
            #[test]
            fn $name() {
                assert_eq!(to_js_json(&check($body).unwrap().to_json()), $expected);
            }
        )*
    };
}

fixtures! {
    accepts_fractional_control_points: r#"const icon = fixture(); icon.parts[0].d = 'M8.17 16 C11.31 12.12 20.39 16 24 16'; export const icons = [icon];"# => r#"{"status":"PASS","icons":1,"actions":2,"samples":870,"diagnostics":[]}"#;
    topology_failure_names_profile_time_and_weight: r#"const icon = fixture(); icon.parts[0].d = (t, strong, reverse, width) => strong && width === 2.5 && t > .2 ? 'M8 16 C12 12 20 12 24 16' : 'M8 16 L24 16'; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":870,"diagnostics":[{"severity":"error","code":"MORPH_TOPOLOGY","icon":"test-arrow","part":"shaft","message":"Normalized path commands change during this action","profile":"strong","reverse":false,"weight":"bold","time":0.21666666666666667}]}"#;
    reverse_and_invisible_geometry_are_checked: r#"const icon = fixture(); Object.assign(icon, { stateful: true, initialState: 'open', finalState: 'closed', reverseSuffix: '-reset', reverseTimings: icon.timings }); icon.parts[0].visibility = () => 0; icon.parts[0].pose = (x, y, t, s, reverse) => reverse && t > .1 ? [NaN, y] : [x, y]; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":4,"samples":912,"diagnostics":[{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Pose must return two finite coordinates","profile":"light","reverse":true,"weight":"light","time":0.11666666666666667},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Pose must return two finite coordinates","profile":"light","reverse":true,"weight":"regular","time":0.11666666666666667},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Pose must return two finite coordinates","profile":"light","reverse":true,"weight":"bold","time":0.11666666666666667},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Pose must return two finite coordinates","profile":"strong","reverse":true,"weight":"light","time":0.11666666666666667},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Pose must return two finite coordinates","profile":"strong","reverse":true,"weight":"regular","time":0.11666666666666667},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Pose must return two finite coordinates","profile":"strong","reverse":true,"weight":"bold","time":0.11666666666666667}]}"#;
    opacity_overshoot_is_an_error: r#"const icon = fixture(); icon.parts[0].visibility = t => t > .2 && t < .5 ? 1.06 : 1; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":870,"diagnostics":[{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"light","reverse":false,"weight":"light","time":0.21666666666666667},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"light","reverse":false,"weight":"regular","time":0.21666666666666667},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"light","reverse":false,"weight":"bold","time":0.21666666666666667},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"strong","reverse":false,"weight":"light","time":0.21666666666666667},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"strong","reverse":false,"weight":"regular","time":0.21666666666666667},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"strong","reverse":false,"weight":"bold","time":0.21666666666666667}]}"#;
    paint_duotone_and_duplicates_fail_independently: r#"const icon = fixture(); icon.parts[0].width = 1.7; icon.duotoneFill = true; export const icons = [icon, icon];"# => r#"{"status":"FAIL","icons":2,"actions":4,"samples":1740,"diagnostics":[{"severity":"error","code":"OUTLINE_WIDTH","icon":"test-arrow","part":"shaft","message":"Author the regular centerline at width 2; the builder creates weight presets"},{"severity":"error","code":"DUOTONE_POLICY","icon":"test-arrow","message":"Declared Duotone policy disagrees with tint parts"},{"severity":"error","code":"DUPLICATE_ICON","icon":"test-arrow","message":"Icon names must be unique within the set"}]}"#;
    rigid_pivots_must_be_constant: r#"const icon = fixture(), pose = (x, y) => [x, y]; pose.transform = t => ({ pivot: [16 + t, 16] }); icon.parts[0].pose = pose; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":870,"diagnostics":[{"severity":"error","code":"MOVING_PIVOT","icon":"test-arrow","part":"shaft","message":"Native rigid transforms require a constant pivot","profile":"light","reverse":false,"weight":"light","time":0.016666666666666666},{"severity":"error","code":"MOVING_PIVOT","icon":"test-arrow","part":"shaft","message":"Native rigid transforms require a constant pivot","profile":"light","reverse":false,"weight":"regular","time":0.016666666666666666},{"severity":"error","code":"MOVING_PIVOT","icon":"test-arrow","part":"shaft","message":"Native rigid transforms require a constant pivot","profile":"light","reverse":false,"weight":"bold","time":0.016666666666666666},{"severity":"error","code":"MOVING_PIVOT","icon":"test-arrow","part":"shaft","message":"Native rigid transforms require a constant pivot","profile":"strong","reverse":false,"weight":"light","time":0.016666666666666666},{"severity":"error","code":"MOVING_PIVOT","icon":"test-arrow","part":"shaft","message":"Native rigid transforms require a constant pivot","profile":"strong","reverse":false,"weight":"regular","time":0.016666666666666666},{"severity":"error","code":"MOVING_PIVOT","icon":"test-arrow","part":"shaft","message":"Native rigid transforms require a constant pivot","profile":"strong","reverse":false,"weight":"bold","time":0.016666666666666666}]}"#;
    hull_and_ambiguous_names_are_advisories: r#"const icon = fixture(); icon.parts[0].d = 'M8 16 C-5 10 40 22 24 16'; icon.parts.push({ ...icon.parts[0], style: 'solid' }); export const icons = [icon];"# => r#"{"status":"PASS","icons":1,"actions":2,"samples":1160,"diagnostics":[{"severity":"warning","code":"AMBIGUOUS_LAYER_NAME","icon":"test-arrow","part":"shaft","message":"Separate paints share a layer name; use unique names in the next drawing revision"},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"light","reverse":false,"weight":"light","time":0},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"light","reverse":false,"weight":"regular","time":0},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"light","reverse":false,"weight":"bold","time":0},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"light","reverse":false,"weight":"authored","time":0},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"strong","reverse":false,"weight":"light","time":0},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"strong","reverse":false,"weight":"regular","time":0},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"strong","reverse":false,"weight":"bold","time":0},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"strong","reverse":false,"weight":"authored","time":0}]}"#;
    continuous_sources_allow_rest_zero: r#"const icon = fixture(); icon.continuous = true; for (const timing of Object.values(icon.timings)) Object.assign(timing, { active: 2.4, rest: 0 }); export const icons = [icon];"# => r#"{"status":"PASS","icons":1,"actions":2,"samples":870,"diagnostics":[]}"#;
    empty_set: r#"export const icons = [];"# => r#"{"status":"FAIL","icons":0,"actions":0,"samples":0,"diagnostics":[{"severity":"error","code":"EMPTY_SET","message":"Export a nonempty icons array"}]}"#;
    not_an_array: r#"export const icons = undefined;"# => r#"{"status":"FAIL","icons":0,"actions":0,"samples":0,"diagnostics":[{"severity":"error","code":"EMPTY_SET","message":"Export a nonempty icons array"}]}"#;
    no_icons_export: r#"export const other = [fixture()];"# => r#"{"status":"FAIL","icons":0,"actions":0,"samples":0,"diagnostics":[{"severity":"error","code":"EMPTY_SET","message":"Export a nonempty icons array"}]}"#;
    bad_names: r#"export const icons = [null, { name: 'Bad Name' }, { label: 'x' }, { name: 5 }];"# => r#"{"status":"FAIL","icons":4,"actions":0,"samples":0,"diagnostics":[{"severity":"error","code":"ICON_NAME","message":"Use a stable kebab-case name"},{"severity":"error","code":"ICON_NAME","icon":"Bad Name","message":"Use a stable kebab-case name"},{"severity":"error","code":"EMPTY_PARTS","message":"Named vector parts are required"},{"severity":"error","code":"ICON_NAME","icon":5,"message":"Use a stable kebab-case name"}]}"#;
    no_parts: r#"export const icons = [{ name: 'a', label: '' }];"# => r#"{"status":"FAIL","icons":1,"actions":0,"samples":0,"diagnostics":[{"severity":"error","code":"LABEL","icon":"a","message":"A readable label is required"},{"severity":"error","code":"EMPTY_PARTS","icon":"a","message":"Named vector parts are required"}]}"#;
    part_names: r#"export const icons = [{ name: 'a', label: 'A', duotoneFill: false, parts: [null, { name: '  ', d: 'M0 0 L1 1', style: 'x', type: 'line' }, { name: 'p', d: 'M0 0 L1 1', style: 'solid', type: 'stroke', width: 0, pose: 1, visibility: 'v' }], timings: fixture().timings }];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":290,"diagnostics":[{"severity":"error","code":"PART_NAME","icon":"a","message":"Every part needs a name"},{"severity":"error","code":"STROKE_WIDTH","icon":"a","part":"p","message":"Stroke width must be positive and finite"},{"severity":"error","code":"CALLBACK","icon":"a","part":"p","message":"pose must be a function"},{"severity":"error","code":"INVALID_GEOMETRY","icon":"a","part":"p","message":"part.visibility is not a function","profile":"light","reverse":false,"weight":"authored","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"a","part":"p","message":"part.visibility is not a function","profile":"strong","reverse":false,"weight":"authored","time":0}]}"#;
    timing_and_key_poses: r#"const icon = fixture(); icon.timings.light = { active: 0, rest: 1, keyPoses: [0] }; icon.timings.strong = { active: 1, rest: 1, keyPoses: [0.5, 0.2] }; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":0,"diagnostics":[{"severity":"error","code":"TIMING","icon":"test-arrow","message":"Action/rest must fit the current timeline; gesture rest follows active motion","profile":"light","reverse":false},{"severity":"error","code":"KEY_POSES","icon":"test-arrow","message":"Key poses must be ordered finite seconds inside the timeline","profile":"strong","reverse":false}]}"#;
    stateful_without_states: r#"const icon = fixture(); icon.stateful = true; delete icon.duotoneFill; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":4,"samples":870,"diagnostics":[{"severity":"info","code":"DUOTONE_UNDECLARED","icon":"test-arrow","message":"Legacy source: native paint policy remains authoritative; explicitly declare surface eligibility for new icons"},{"severity":"error","code":"STATE_POLICY","icon":"test-arrow","message":"Stateful icons need names for both states"},{"severity":"info","code":"LEGACY_REVERSE_NAME","icon":"test-arrow","message":"Verify the set-specific builder names its reverse/reset; the shared builder requires reverseSuffix"},{"severity":"error","code":"TIMING","icon":"test-arrow","message":"Action/rest must fit the current timeline; gesture rest follows active motion","profile":"light","reverse":true},{"severity":"error","code":"TIMING","icon":"test-arrow","message":"Action/rest must fit the current timeline; gesture rest follows active motion","profile":"strong","reverse":true}]}"#;
    callback_throws_error: r#"const icon = fixture(); icon.parts[0].d = t => { if (t > 1) throw Error('boom at ' + t); return 'M8 16 L24 16'; }; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":366,"diagnostics":[{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"boom at 1.0166666666666666","profile":"light","reverse":false,"weight":"light","time":1.0166666666666666},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"boom at 1.0166666666666666","profile":"light","reverse":false,"weight":"regular","time":1.0166666666666666},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"boom at 1.0166666666666666","profile":"light","reverse":false,"weight":"bold","time":1.0166666666666666},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"boom at 1.0166666666666666","profile":"strong","reverse":false,"weight":"light","time":1.0166666666666666},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"boom at 1.0166666666666666","profile":"strong","reverse":false,"weight":"regular","time":1.0166666666666666},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"boom at 1.0166666666666666","profile":"strong","reverse":false,"weight":"bold","time":1.0166666666666666}]}"#;
    callback_throws_non_error: r#"const icon = fixture(); icon.parts[0].d = t => { if (t > 1) throw 'str'; return 'M8 16 L24 16'; }; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":366,"diagnostics":[{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","profile":"light","reverse":false,"weight":"light","time":1.0166666666666666},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","profile":"light","reverse":false,"weight":"regular","time":1.0166666666666666},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","profile":"light","reverse":false,"weight":"bold","time":1.0166666666666666},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","profile":"strong","reverse":false,"weight":"light","time":1.0166666666666666},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","profile":"strong","reverse":false,"weight":"regular","time":1.0166666666666666},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","profile":"strong","reverse":false,"weight":"bold","time":1.0166666666666666}]}"#;
    pose_not_a_function: r#"const icon = fixture(); icon.parts[0].pose = 3; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":0,"diagnostics":[{"severity":"error","code":"CALLBACK","icon":"test-arrow","part":"shaft","message":"pose must be a function"},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"part.pose is not a function","profile":"light","reverse":false,"weight":"light","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"part.pose is not a function","profile":"light","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"part.pose is not a function","profile":"light","reverse":false,"weight":"bold","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"part.pose is not a function","profile":"strong","reverse":false,"weight":"light","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"part.pose is not a function","profile":"strong","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"part.pose is not a function","profile":"strong","reverse":false,"weight":"bold","time":0}]}"#;
    transform_not_finite: r#"const icon = fixture(), pose = (x, y) => [x, y]; pose.transform = t => ({ pivot: [16, 16], angle: NaN }); icon.parts[0].pose = pose; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":0,"diagnostics":[{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Rigid transform must have a finite pivot, translation and angle","profile":"light","reverse":false,"weight":"light","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Rigid transform must have a finite pivot, translation and angle","profile":"light","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Rigid transform must have a finite pivot, translation and angle","profile":"light","reverse":false,"weight":"bold","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Rigid transform must have a finite pivot, translation and angle","profile":"strong","reverse":false,"weight":"light","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Rigid transform must have a finite pivot, translation and angle","profile":"strong","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Rigid transform must have a finite pivot, translation and angle","profile":"strong","reverse":false,"weight":"bold","time":0}]}"#;
    bigint_stroke_width: r#"const icon = fixture(); Object.assign(icon.parts[0], { style: 'solid', width: 4n }); export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":0,"diagnostics":[{"severity":"error","code":"STROKE_WIDTH","icon":"test-arrow","part":"shaft","message":"Stroke width must be positive and finite"},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot mix BigInt and other types, use explicit conversions","profile":"light","reverse":false,"weight":"authored","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot mix BigInt and other types, use explicit conversions","profile":"strong","reverse":false,"weight":"authored","time":0}]}"#;
    symbol_stroke_width: r#"const icon = fixture(); Object.assign(icon.parts[0], { style: 'solid', width: Symbol('w') }); export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":0,"diagnostics":[{"severity":"error","code":"STROKE_WIDTH","icon":"test-arrow","part":"shaft","message":"Stroke width must be positive and finite"},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot convert a Symbol value to a number","profile":"light","reverse":false,"weight":"authored","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot convert a Symbol value to a number","profile":"strong","reverse":false,"weight":"authored","time":0}]}"#;
    transform_returns_null: r#"const icon = fixture(), pose = (x, y) => [x, y]; pose.transform = () => null; icon.parts[0].pose = pose; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":0,"diagnostics":[{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot read properties of null (reading 'pivot')","profile":"light","reverse":false,"weight":"light","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot read properties of null (reading 'pivot')","profile":"light","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot read properties of null (reading 'pivot')","profile":"light","reverse":false,"weight":"bold","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot read properties of null (reading 'pivot')","profile":"strong","reverse":false,"weight":"light","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot read properties of null (reading 'pivot')","profile":"strong","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot read properties of null (reading 'pivot')","profile":"strong","reverse":false,"weight":"bold","time":0}]}"#;
    transform_returns_undefined: r#"const icon = fixture(), pose = (x, y) => [x, y]; pose.transform = () => {}; icon.parts[0].pose = pose; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":0,"diagnostics":[{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot read properties of undefined (reading 'pivot')","profile":"light","reverse":false,"weight":"light","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot read properties of undefined (reading 'pivot')","profile":"light","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot read properties of undefined (reading 'pivot')","profile":"light","reverse":false,"weight":"bold","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot read properties of undefined (reading 'pivot')","profile":"strong","reverse":false,"weight":"light","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot read properties of undefined (reading 'pivot')","profile":"strong","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Cannot read properties of undefined (reading 'pivot')","profile":"strong","reverse":false,"weight":"bold","time":0}]}"#;
    transform_returns_primitive: r#"const icon = fixture(), pose = (x, y) => [x, y]; pose.transform = () => 5; icon.parts[0].pose = pose; export const icons = [icon];"# => r#"{"status":"PASS","icons":1,"actions":2,"samples":870,"diagnostics":[]}"#;
    transform_not_a_function: r#"const icon = fixture(), pose = (x, y) => [x, y]; pose.transform = 5; icon.parts[0].pose = pose; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":0,"diagnostics":[{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"part.pose.transform is not a function","profile":"light","reverse":false,"weight":"light","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"part.pose.transform is not a function","profile":"light","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"part.pose.transform is not a function","profile":"light","reverse":false,"weight":"bold","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"part.pose.transform is not a function","profile":"strong","reverse":false,"weight":"light","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"part.pose.transform is not a function","profile":"strong","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"part.pose.transform is not a function","profile":"strong","reverse":false,"weight":"bold","time":0}]}"#;
    transform_ignored_for_dynamic_d: r#"const icon = fixture(), pose = (x, y) => [x, y]; pose.transform = () => { throw Error('never'); }; icon.parts[0].pose = pose; icon.parts[0].d = () => 'M8 16 L24 16'; export const icons = [icon];"# => r#"{"status":"PASS","icons":1,"actions":2,"samples":870,"diagnostics":[]}"#;
    first_bad_pose_wins_over_later_throw: r#"const icon = fixture(); icon.parts[0].pose = (x, y) => { if (x === 24) throw Error('second'); return [NaN, y]; }; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":0,"diagnostics":[{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Pose must return two finite coordinates","profile":"light","reverse":false,"weight":"light","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Pose must return two finite coordinates","profile":"light","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Pose must return two finite coordinates","profile":"light","reverse":false,"weight":"bold","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Pose must return two finite coordinates","profile":"strong","reverse":false,"weight":"light","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Pose must return two finite coordinates","profile":"strong","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Pose must return two finite coordinates","profile":"strong","reverse":false,"weight":"bold","time":0}]}"#;
    string_visibility_fails_and_counts_for_hull: r#"const icon = fixture(); icon.parts[0].d = 'M-1 16 L24 16'; icon.parts[0].visibility = () => '1'; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":870,"diagnostics":[{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"light","reverse":false,"weight":"light","time":0},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"light","reverse":false,"weight":"light","time":0},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"light","reverse":false,"weight":"regular","time":0},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"light","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"light","reverse":false,"weight":"bold","time":0},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"light","reverse":false,"weight":"bold","time":0},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"strong","reverse":false,"weight":"light","time":0},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"strong","reverse":false,"weight":"light","time":0},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"strong","reverse":false,"weight":"regular","time":0},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"strong","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"strong","reverse":false,"weight":"bold","time":0},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"strong","reverse":false,"weight":"bold","time":0}]}"#;
    undefined_visibility_fails: r#"const icon = fixture(); icon.parts[0].visibility = () => undefined; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":870,"diagnostics":[{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"light","reverse":false,"weight":"light","time":0},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"light","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"light","reverse":false,"weight":"bold","time":0},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"strong","reverse":false,"weight":"light","time":0},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"strong","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"strong","reverse":false,"weight":"bold","time":0}]}"#;
    pivot_with_a_hole_passes: r#"const icon = fixture(), pose = (x, y) => [x, y]; pose.transform = () => ({ pivot: [, 5] }); icon.parts[0].pose = pose; export const icons = [icon];"# => r#"{"status":"PASS","icons":1,"actions":2,"samples":870,"diagnostics":[]}"#;
    array_name_is_reported_as_json: r#"const icon = fixture(); icon.name = ['test-arrow']; icon.label = ' '; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":870,"diagnostics":[{"severity":"error","code":"LABEL","icon":["test-arrow"],"message":"A readable label is required"}]}"#;
    nameless_icons_collapse: r#"export const icons = [{ label: 'x' }, { label: 'y' }];"# => r#"{"status":"FAIL","icons":2,"actions":0,"samples":0,"diagnostics":[{"severity":"error","code":"EMPTY_PARTS","message":"Named vector parts are required"},{"severity":"error","code":"DUPLICATE_ICON","message":"Icon names must be unique within the set"}]}"#;
    number_export: r#"export const icons = 5;"# => r#"{"status":"FAIL","icons":0,"actions":0,"samples":0,"diagnostics":[{"severity":"error","code":"EMPTY_SET","message":"Export a nonempty icons array"}]}"#;
    nan_name: r#"export const icons = [{ name: NaN }];"# => r#"{"status":"FAIL","icons":1,"actions":0,"samples":0,"diagnostics":[{"severity":"error","code":"ICON_NAME","icon":null,"message":"Use a stable kebab-case name"}]}"#;
    topology_change_and_hull: r#"const icon = fixture(); icon.parts[0].d = (t) => t > 0.5 ? 'M8 16 C-5 10 40 22 24 16' : 'M8 16 L24 16'; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":870,"diagnostics":[{"severity":"error","code":"MORPH_TOPOLOGY","icon":"test-arrow","part":"shaft","message":"Normalized path commands change during this action","profile":"light","reverse":false,"weight":"light","time":0.5166666666666667},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"light","reverse":false,"weight":"light","time":0.5166666666666667},{"severity":"error","code":"MORPH_TOPOLOGY","icon":"test-arrow","part":"shaft","message":"Normalized path commands change during this action","profile":"light","reverse":false,"weight":"regular","time":0.5166666666666667},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"light","reverse":false,"weight":"regular","time":0.5166666666666667},{"severity":"error","code":"MORPH_TOPOLOGY","icon":"test-arrow","part":"shaft","message":"Normalized path commands change during this action","profile":"light","reverse":false,"weight":"bold","time":0.5166666666666667},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"light","reverse":false,"weight":"bold","time":0.5166666666666667},{"severity":"error","code":"MORPH_TOPOLOGY","icon":"test-arrow","part":"shaft","message":"Normalized path commands change during this action","profile":"strong","reverse":false,"weight":"light","time":0.5166666666666667},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"strong","reverse":false,"weight":"light","time":0.5166666666666667},{"severity":"error","code":"MORPH_TOPOLOGY","icon":"test-arrow","part":"shaft","message":"Normalized path commands change during this action","profile":"strong","reverse":false,"weight":"regular","time":0.5166666666666667},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"strong","reverse":false,"weight":"regular","time":0.5166666666666667},{"severity":"error","code":"MORPH_TOPOLOGY","icon":"test-arrow","part":"shaft","message":"Normalized path commands change during this action","profile":"strong","reverse":false,"weight":"bold","time":0.5166666666666667},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"strong","reverse":false,"weight":"bold","time":0.5166666666666667}]}"#;
    arc_semicircle: r#"const icon = fixture(); icon.parts[0].d = 'M6 16 A10 10 0 0 1 26 16'; export const icons = [icon];"# => r#"{"status":"PASS","icons":1,"actions":2,"samples":870,"diagnostics":[]}"#;
    unsupported_command: r#"const icon = fixture(); icon.parts[0].d = 'M0 0 Q1 1 2 2'; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":0,"diagnostics":[{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Unsupported native command Q","profile":"light","reverse":false,"weight":"light","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Unsupported native command Q","profile":"light","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Unsupported native command Q","profile":"light","reverse":false,"weight":"bold","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Unsupported native command Q","profile":"strong","reverse":false,"weight":"light","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Unsupported native command Q","profile":"strong","reverse":false,"weight":"regular","time":0},{"severity":"error","code":"INVALID_GEOMETRY","icon":"test-arrow","part":"shaft","message":"Unsupported native command Q","profile":"strong","reverse":false,"weight":"bold","time":0}]}"#;
    static_part_with_string_width: r#"const icon = fixture(); Object.assign(icon.parts[0], { style: 'solid', width: '3', d: 'M1 16 L24 16' }); export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":290,"diagnostics":[{"severity":"error","code":"STROKE_WIDTH","icon":"test-arrow","part":"shaft","message":"Stroke width must be positive and finite"},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"light","reverse":false,"weight":"authored","time":0},{"severity":"warning","code":"CONTROL_HULL_MARGIN","icon":"test-arrow","part":"shaft","message":"Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry","profile":"strong","reverse":false,"weight":"authored","time":0}]}"#;
    outline_fill_shares_a_name: r#"const icon = fixture(); icon.parts.push({ name: 'shaft', d: 'M8 8 L9 9 Z', style: 'outline' }); export const icons = [icon];"# => r#"{"status":"PASS","icons":1,"actions":2,"samples":1740,"diagnostics":[{"severity":"warning","code":"AMBIGUOUS_LAYER_NAME","icon":"test-arrow","part":"shaft","message":"Separate paints share a layer name; use unique names in the next drawing revision"}]}"#;
    key_times_between_frames: r#"const icon = fixture(); icon.timings.light = { active: 0.4, rest: 0.51, keyPoses: [0, 0.125, 0.4] }; icon.parts[0].d = 'M2 16 L24 16'; icon.parts[0].visibility = t => t === 0.125 ? 2 : 1; export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":876,"diagnostics":[{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"light","reverse":false,"weight":"light","time":0.125},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"light","reverse":false,"weight":"regular","time":0.125},{"severity":"error","code":"VISIBILITY","icon":"test-arrow","part":"shaft","message":"Visibility must remain between 0 and 1","profile":"light","reverse":false,"weight":"bold","time":0.125}]}"#;
    getter_labels_and_symbol_type: r#"const icon = fixture(); Object.defineProperty(icon, 'label', { get: () => 'Arrow' }); icon.parts[0].type = Symbol('t'); export const icons = [icon];"# => r#"{"status":"FAIL","icons":1,"actions":2,"samples":870,"diagnostics":[{"severity":"error","code":"PAINT_TYPE","icon":"test-arrow","part":"shaft","message":"Expected fill or stroke"}]}"#;
}

/// `E_times`: key poses between frames add samples, and `d` sees every time in order.
#[test]
fn key_times_between_frames_are_sampled_in_order() {
    let body = "const icon = fixture(); const seen = []; icon.parts[0].d = (t) => { seen.push(t); return 'M8 16 L24 16'; }; \
                icon.timings.light.keyPoses = [0, 0.125, 0.4]; icon.timings.light.active = 0.4; icon.timings.light.rest = 0.51; \
                export const icons = [icon]; icons.seen = seen;";
    let contract = FamilyContract::load(&root()).unwrap();
    let (report, seen) = with_fixture(body, |runtime| {
        let report = check_definitions(runtime, &contract).unwrap();
        let seen = runtime
            .with_icons(|scope| {
                let seen: rquickjs::Value = scope.icons().as_object().unwrap().get("seen")?;
                scope.project(seen)
            })
            .unwrap();
        (report, seen.unwrap())
    });
    assert_eq!((report.status, report.samples), (Status::Pass, 876));
    let seen: Vec<f64> = seen.as_array().unwrap().iter().map(|t| t.as_f64().unwrap()).collect();
    // Light: 145 frames plus 0.125 and 0.51 for each of 3 weights; strong: 145 each.
    assert_eq!(seen.len(), 147 * 3 + 145 * 3);
    let first = [
        0.0,
        1.0 / 60.0,
        2.0 / 60.0,
        0.05,
        4.0 / 60.0,
        5.0 / 60.0,
        0.1,
        7.0 / 60.0,
        0.125,
        8.0 / 60.0,
        0.15,
        10.0 / 60.0,
    ];
    assert_eq!(seen[..12], first);
    assert_eq!(seen[31..34], [0.5, 0.51, 31.0 / 60.0]);
    // The light weight's last times, then the regular weight starts over.
    assert_eq!(seen[145..148], [143.0 / 60.0, 2.4, 0.0]);
}

/// A hole in `keyPoses` fails KEY_POSES. JS skipped it in `some` and then sampled at
/// `undefined`; the probe takes numeric times only, so this is a deliberate divergence.
#[test]
fn key_pose_holes_fail_key_poses() {
    let report =
        check("const icon = fixture(); icon.timings.light.keyPoses = [0, , 0.4]; export const icons = [icon];")
            .unwrap();
    let codes: Vec<_> = report.diagnostics.iter().map(|d| (d.code.as_str(), to_js_json(&d.to_json()))).collect();
    assert_eq!(report.samples, 435);
    assert_eq!(codes.len(), 1);
    assert!(codes[0].1.contains(r#""code":"KEY_POSES","icon":"test-arrow","message":"Key poses must be ordered finite seconds inside the timeline","profile":"light","reverse":false"#));
}

/// `throw null` in a callback crashed `preflight.mjs` reading `.message`: fatal, no report.
#[test]
fn callbacks_throwing_null_abort_the_run() {
    let result = check(
        "const icon = fixture(); icon.parts[0].d = t => { if (t > 1) throw null; return 'M8 16 L24 16'; }; export const icons = [icon];",
    );
    assert!(matches!(result, Err(Error::ThrewNullish { stage: "d", .. })), "{result:?}");
}

/// `malformed SVG cannot silently become an empty or partial native shape`, and every
/// `parseGeometry` vector of the preflight spec (12.4), through `probe()`.
#[test]
fn parse_geometry_matches_node() {
    let parts: Vec<Json> = PATHS.iter().map(|(d, _)| serde_json::json!({ "name": "p", "d": d })).collect();
    let module = format!("export const icons = [{{ name: 'g', parts: {} }}];\n", Json::from(parts));
    let set = temp_set("preflight-paths-", &module);
    let source = SetSource::user_project(set.path());
    gai_js::run_on_worker(&Limits::default(), || {
        let runtime = SetRuntime::load(&source, &Limits::default()).unwrap();
        for (part, (d, expected)) in PATHS.iter().enumerate() {
            let request = ProbeRequest {
                icon: 0,
                part,
                strong: false,
                reverse: false,
                width: ProbeWidth::Authored,
                times: &[0.0],
            };
            let sample = runtime.probe(&request).unwrap().samples.remove(0);
            let got = match (&sample.outcome, &sample.geometry) {
                (ProbeOutcome::InvalidGeometry { message, .. }, _) => Err(message.clone()),
                (ProbeOutcome::Complete, Some(geometry)) => {
                    let points = geometry.points.iter().map(|&v| Json::from(v)).collect();
                    Ok((geometry.topology.clone(), to_js_json(&Json::Array(points))))
                }
                other => panic!("{d:?}: {other:?}"),
            };
            let expected =
                expected.map(|(topology, points)| (topology.to_owned(), points.to_owned())).map_err(str::to_owned);
            assert_eq!(got, expected, "{d:?}");
        }
    });
}

/// The harness's `definitions.mjs`: `JSON.stringify([fixture()])` with this width and name.
fn harness_definitions(name: &str, width: u32) -> String {
    format!(
        r#"export const icons=[{{"name":"{name}","label":"Test arrow","duotoneFill":false,"parts":[{{"name":"shaft","d":"M8 16 L24 16","type":"stroke","width":{width},"style":"outline"}}],"timings":{{"light":{{"active":0.4,"rest":0.5,"keyPoses":[0,0.4]}},"strong":{{"active":0.6,"rest":0.7,"keyPoses":[0,0.6]}}}}}}];"#
    )
}

/// The inputs `checkSet` lists for a temp set holding only `definitions.mjs`: the library
/// sources and the set's file in JS order over whole paths, then the catalog files.
fn expected_inputs(set: &Path) -> Json {
    let library = root().join("motion/icon-sets/library");
    let library = fs::canonicalize(&library).unwrap();
    let mut files: Vec<String> = fs::read_dir(&library)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.ends_with(".mjs") || name.ends_with(".json"))
        .map(|name| format!("{}/{name}", library.display()))
        .chain([format!("{}/definitions.mjs", set.display())])
        .collect();
    files.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
    let catalog = Catalog::load(&root()).unwrap();
    let hashed = files.iter().map(|path| (path.clone(), sha256_hex(fs::read(path).unwrap())));
    let inputs = hashed.chain(catalog.inputs.iter().map(|input| (input.path.clone(), input.sha256.clone())));
    inputs.map(|(path, sha256)| serde_json::json!({ "path": path, "sha256": sha256 })).collect()
}

fn preflight() -> Preflight {
    Preflight::open(fs::canonicalize(root()).unwrap()).unwrap()
}

/// The report's JSON with `inputs` checked against the expected inventory and replaced by
/// `placeholder`, as the spec's goldens elide it.
fn elide_inputs(report: &CheckedReport, set: &Path, placeholder: &str) -> String {
    let mut json = report.to_json();
    assert_eq!(json["inputs"], expected_inputs(set));
    assert_eq!(json["inputs"].as_array().unwrap().len(), 24 + 1 + 59);
    json["inputs"] = placeholder.into();
    to_js_json(&json)
}

/// `checked build prevents the writer from running when preflight fails`.
#[test]
fn checked_build_blocks_the_builder_when_preflight_fails() {
    let set = temp_set("icon-harness-fail-", &harness_definitions("test-arrow", 9));
    let mut calls = 0;
    let report = preflight()
        .build_checked(set.path(), |_| {
            calls += 1;
            LaunchOutcome::built()
        })
        .unwrap();
    assert!(!report.built());
    assert_eq!(calls, 0);
    let listing: Vec<_> = fs::read_dir(set.path()).unwrap().map(|entry| entry.unwrap().file_name()).collect();
    assert_eq!(listing, ["definitions.mjs"]);
    let name = set.path().file_name().unwrap().to_str().unwrap();
    assert_eq!(
        elide_inputs(&report, set.path(), "84 entries"),
        format!(
            r#"{{"set":"{name}","status":"FAIL","icons":1,"actions":2,"samples":870,"diagnostics":[{{"severity":"error","code":"OUTLINE_WIDTH","icon":"test-arrow","part":"shaft","message":"Author the regular centerline at width 2; the builder creates weight presets"}}],"inputs":"84 entries","built":false}}"#
        )
    );
}

/// `checked build calls the existing builder only after clean source screening`, and the
/// launch outcomes the JS function mapped (spec 11.11).
#[test]
fn checked_build_launches_only_after_clean_screening() {
    let set = temp_set("icon-harness-pass-", &harness_definitions("test-arrow", 2));
    let name = set.path().file_name().unwrap().to_str().unwrap().to_owned();
    let preflight = preflight();
    let mut launched = None;
    let report = preflight
        .build_checked(set.path(), |dir| {
            launched = Some(dir.to_owned());
            LaunchOutcome::built()
        })
        .unwrap();
    assert_eq!(launched.as_deref(), Some(set.path()));
    assert!(report.built());
    let prefix = format!(r#"{{"set":"{name}","status":"#);
    let screened = r#""icons":1,"actions":2,"samples":870,"diagnostics":[],"inputs":"#;
    assert_eq!(
        elide_inputs(&report, set.path(), "84 entries"),
        format!(r#"{prefix}"PASS",{screened}"84 entries","built":true,"buildExitCode":0}}"#)
    );
    for (outcome, tail) in [
        (LaunchOutcome { status: Some(3), error: None }, r#""built":false,"buildExitCode":3}"#),
        (LaunchOutcome::failed(), r#""built":false,"buildExitCode":1}"#),
        (LaunchOutcome::not_started("spawn ENOENT"), r#""built":false,"buildExitCode":null,"error":"spawn ENOENT"}"#),
    ] {
        let report = preflight.build_checked(set.path(), |_| outcome).unwrap();
        assert_eq!(report.status(), Status::Fail);
        assert_eq!(
            elide_inputs(&report, set.path(), "[elided]"),
            format!(r#"{prefix}"FAIL",{screened}"[elided]",{tail}"#)
        );
    }
}

/// `legacy inline source is disclosed, never imported as if it were a read-only definition`.
#[test]
fn legacy_sources_are_disclosed_not_imported() {
    let dir = tempfile::Builder::new().prefix("icon-harness-legacy-").tempdir().unwrap();
    let error = preflight().check_set(dir.path()).unwrap_err();
    assert_eq!(
        error.to_string(),
        format!(
            "No separate definitions.mjs in {}; legacy inline builders require their existing checks",
            dir.path().display()
        )
    );
    let error = preflight().run(&Target::Set(dir.path().to_owned())).unwrap_err();
    assert!(matches!(error, Error::Legacy(_)));
}

/// `new source sets cannot duplicate a public icon ID`, and the negative control.
#[test]
fn new_sets_cannot_duplicate_a_public_id() {
    let set = temp_set("icon-catalog-collision-", &harness_definitions("heart", 2));
    let report = preflight().check_set(set.path()).unwrap();
    let mut json = report.to_json();
    json["inputs"] = "[elided]".into();
    let name = set.path().file_name().unwrap().to_str().unwrap();
    assert_eq!(
        to_js_json(&json),
        format!(
            r#"{{"set":"{name}","status":"FAIL","icons":1,"actions":2,"samples":870,"diagnostics":[{{"severity":"error","code":"CATALOG_ID_COLLISION","icon":"heart","message":"ID already belongs to 01-heart-bell; revise there or choose a distinct concept/ID"}}],"inputs":"[elided]"}}"#
        )
    );
    let voice = preflight().check_set(&root().join("motion/icon-sets/16-voice")).unwrap();
    assert!(voice.report.diagnostics.iter().all(|d| d.code.as_str() != "CATALOG_ID_COLLISION"));
}

/// A failing set reports on stdout with FAIL, which the CLI turns into exit 1; a module
/// that throws at top level aborts with its message.
#[test]
fn runs_report_failures_and_abort_on_module_errors() {
    let set = temp_set("icon-harness-fail-", &harness_definitions("test-arrow", 9));
    let run = preflight().run(&Target::Set(set.path().to_owned())).unwrap();
    assert_eq!(run.status(), Status::Fail);
    assert!(run.stdout().starts_with("{\n  \"status\": \"FAIL\",\n  \"scope\": \"Source screening only; no native export or visual review\",\n  \"reports\": [\n    {\n      \"set\": \"icon-harness-fail-"));
    assert!(run.stdout().ends_with("  \"skipped\": []\n}\n"));
    let set = temp_set("icon-module-throws-", "throw Error('Voice scope must contain ten distinct icons');\n");
    match preflight().run(&Target::Set(set.path().to_owned())) {
        Err(Error::Js(gai_js::Error::Script { message, .. })) => {
            assert_eq!(message, "Voice scope must contain ten distinct icons")
        }
        other => panic!("{other:?}"),
    }
}

/// Node's `parseGeometry` result: `(topology, JSON.stringify(points))`, or the message thrown.
type Parsed = Result<(&'static str, &'static str), &'static str>;

/// `parseGeometry` inputs (the preflight spec's table 12.4) and Node's results.
#[rustfmt::skip]
const PATHS: &[(&str, Parsed)] = &[
    ("", Err("Path must be a nonempty SVG path string")),
    (" \t\n", Err("Path must be a nonempty SVG path string")),
    ("\u{180e}", Err("Path has no drawable segment")),
    ("M4 4", Err("Path has no drawable segment")),
    ("M4 4 LNaN 8", Err("SvgPath: param should start with 0..9 or `.` (at pos 6)")),
    ("M4 4 L9", Err("SvgPath: missed param (at pos 7)")),
    ("X", Err("SvgPath: bad command X (at pos 0)")),
    ("L0 0", Err("SvgPath: string should start with `M` or `m`")),
    ("L0", Err("SvgPath: missed param (at pos 2)")),
    ("L0 0 X", Err("SvgPath: string should start with `M` or `m`")),
    ("M0 0 H", Err("SvgPath: missed param (at pos 6)")),
    ("M0,0L1,1,", Err("SvgPath: missed param (at pos 9)")),
    ("M0 0 L1 1 0", Err("SvgPath: missed param (at pos 11)")),
    ("M0 0 L1.5.5 2", Err("SvgPath: missed param (at pos 13)")),
    ("M0 0 L09 1", Err("SvgPath: numbers started with `0` such as `09` are illegal (at pos 6)")),
    ("M0 0 L.e1 1", Err("SvgPath: invalid float exponent (at pos 7)")),
    ("M0 0 L5.e3 1", Err("SvgPath: invalid float exponent (at pos 8)")),
    ("M0 0 L55.e3 1", Ok(("ML", "[0,0,55000,1]"))),
    ("M0 0L1 1e", Err("SvgPath: invalid float exponent (at pos 9)")),
    ("M0 0 L. 1", Err("Nonfinite path coordinates")),
    ("M 1e400 0 L 1 1", Err("Nonfinite path coordinates")),
    ("M0 0 A 5 5 0 2 1 10 0", Err("SvgPath: arc flag can be 0 or 1 only (at pos 13)")),
    ("M0 0 Q1 1 2 2", Err("Unsupported native command Q")),
    ("M0 0 L1 1 T4 4", Err("Unsupported native command Q")),
    ("M0 0 R1 1 2 2", Err("Unsupported native command R")),
    ("M0 0 Q1 1 2 2 L1e999 0", Err("Unsupported native command Q")),
    ("M8 16 L24 16", Ok(("ML", "[8,16,24,16]"))),
    ("M8.17 16 C11.31 12.12 20.39 16 24 16", Ok(("MC", "[8.17,16,11.31,12.12,20.39,16,24,16]"))),
    ("M-0 -0 L1 1", Ok(("ML", "[0,0,1,1]"))),
    ("M0 0 L1-1", Ok(("ML", "[0,0,1,-1]"))),
    ("M0 0 L1 1 2 2", Ok(("MLL", "[0,0,1,1,2,2]"))),
    ("M0 0 L1 1 0 2", Ok(("MLL", "[0,0,1,1,0,2]"))),
    ("m1 1 l2 2 h3 v4 z", Ok(("MLLLZ", "[1,1,3,3,6,3,6,7]"))),
    ("M0 0 L1 1 Z Z", Ok(("MLZZ", "[0,0,1,1]"))),
    ("M0 0 L1 1 m1 1 L3 3", Ok(("MLML", "[0,0,1,1,2,2,3,3]"))),
    ("M0 0 L10 0 Z m5 5 l1 1", Ok(("MLZML", "[0,0,10,0,5,5,6,6]"))),
    ("M0 0 L10 0 L10 10 Z H5", Ok(("MLLZL", "[0,0,10,0,10,10,5,10]"))),
    ("M0 0 L1 1 Z h5", Ok(("MLZL", "[0,0,1,1,5,1]"))),
    ("M0 0 L1 1 Z v5", Ok(("MLZL", "[0,0,1,1,1,5]"))),
    ("M1 2 L3 4 Z L5 5", Ok(("MLZL", "[1,2,3,4,5,5]"))),
    ("M0 0 S 5 5 10 0", Ok(("MC", "[0,0,0,0,5,5,10,0]"))),
    ("M0 0 C 1 2 3 4 5 6 S 7 8 9 10", Ok(("MCC", "[0,0,1,2,3,4,5,6,7,8,7,8,9,10]"))),
    ("M0 0 A 5 5 0 0 1 10 0", Ok(("MCC", "[0,0,0,-2.761423749153966,2.238576250846032,-4.999999999999999,4.999999999999999,-5,7.761423749153966,-5,10,-2.761423749153968,10,-1.2246467991473533e-15]"))),
    ("M0 0 A 0 5 0 0 1 10 0", Ok(("ML", "[0,0,10,0]"))),
    ("M0 0 A 5 5 0 0 1 0 0", Ok(("ML", "[0,0,0,0]"))),
    ("M0 0 A1e999 5 0 0 1 10 0", Ok(("ML", "[0,0,10,0]"))),
    ("M12.05 6.82 A10 10 0 1 0 25.18 19.95", Ok(("MCCC", "[12.05,6.82,7.917156690404546,8.590619529732283,5.474153117645869,12.901772776401739,6.078950607524632,17.35707381415145,6.683748097403395,21.81237485190116,10.187625148098828,25.316251902596598,14.642926185848538,25.921049392475364,19.098227223598247,26.52584688235413,23.40938047026771,24.082843309595464,25.179999999999996,19.95000000000001]"))),
];
