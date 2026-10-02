//! Oracle tests against repo files; each skips when its files are missing. Whole-catalog
//! variants are `#[ignore]`d: `cargo test --release -p gai-js --test oracles -- --ignored`.
//!
//! - T-svgpath-bundle: the embedded bundle regenerated from `motion/node_modules/svgpath`.
//! - T-legacy-drift: the 01/02 shims' verbatim blocks against their `build.mjs`.
//! - describe(): variant metadata against the committed `set.json` records.
//! - T-measure: `probe()` recomputes `evidence/motion-measurements.json` of sets 22–39
//!   byte-for-byte (the three script variants of js-runtime §9.1).

use gai_core::{sha256_hex, to_js_json, to_js_json_pretty};
use gai_js::{
    FRAMES, Limits, ProbeOutcome, ProbeRequest, ProbeResult, ProbeWidth, SetRuntime, SetSource, map_parallel,
    run_on_worker,
};
use serde_json::{Map, Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn icon_sets() -> PathBuf {
    repo().join("motion/icon-sets")
}

fn set_dir(prefix: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(icon_sets()).ok()?;
    let mut matches: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.file_name().is_some_and(|name| name.to_string_lossy().starts_with(&format!("{prefix}-"))))
        .collect();
    matches.sort();
    matches.pop()
}

fn workers() -> usize {
    thread::available_parallelism().map_or(4, |n| n.get())
}

#[test]
fn svgpath_bundle_matches_node_modules() {
    const EMBEDDED: &str = include_str!("../js/svgpath.mjs");
    assert_eq!(sha256_hex(EMBEDDED), "c023c1db3a28aa5f2cf9b374e7833ad7ca92c39c506d60d36b7b54b564deab22");
    let package = repo().join("motion/node_modules/svgpath");
    let Ok(manifest) = fs::read_to_string(package.join("package.json")) else {
        eprintln!("skipped: {} is not installed", package.display());
        return;
    };
    let manifest: Value = serde_json::from_str(&manifest).unwrap();
    assert_eq!(manifest["version"], "2.6.0");
    // js-runtime-prototype/bundle-svgpath.mjs
    let files = [
        ("./lib/svgpath", "lib/svgpath.js"),
        ("./path_parse", "lib/path_parse.js"),
        ("./transform_parse", "lib/transform_parse.js"),
        ("./matrix", "lib/matrix.js"),
        ("./a2c", "lib/a2c.js"),
        ("./ellipse", "lib/ellipse.js"),
    ];
    let mut bundle = String::from(
        "// Generated from svgpath@2.6.0 (MIT). CommonJS modules wrapped for an ES module host.\nconst __defs = {\n",
    );
    for (id, file) in files {
        let source = fs::read_to_string(package.join(file)).unwrap();
        bundle +=
            &format!("{}: function (module, exports, require) {{\n{source}\n}},\n", serde_json::to_string(id).unwrap());
    }
    bundle += concat!(
        "};\nconst __cache = {};\nfunction __require(id) {\n",
        "  const key = id === \"./lib/svgpath\" ? id : \"./\" + id.replace(/^\\.\\/(lib\\/)?/, \"\");\n",
        "  if (__cache[key]) return __cache[key].exports;\n  const def = __defs[key];\n",
        "  if (!def) throw new Error(\"svgpath bundle: unknown module \" + id);\n",
        "  const module = { exports: {} };\n  __cache[key] = module;\n",
        "  def.call(module.exports, module, module.exports, __require);\n  return module.exports;\n}\n",
        "const svgpath = __require(\"./lib/svgpath\");\nexport default svgpath;\n",
    );
    assert!(bundle == EMBEDDED, "the embedded svgpath bundle is stale");
}

#[test]
fn legacy_shims_match_their_builders() {
    for (shim, set, lines) in [
        (include_str!("../js/legacy01.mjs"), "01-heart-bell", 23..=119),
        (include_str!("../js/legacy02.mjs"), "02-search", 23..=79),
    ] {
        let Ok(build) = fs::read_to_string(icon_sets().join(set).join("build.mjs")) else {
            eprintln!("skipped: no {set}/build.mjs");
            continue;
        };
        let block: Vec<&str> = shim
            .lines()
            .skip_while(|line| *line != "import svgpath from 'svgpath';")
            .skip(1)
            .take_while(|line| *line != "export { definitions };")
            .collect();
        let original: Vec<&str> = build.lines().skip(lines.start() - 1).take(lines.end() - lines.start() + 1).collect();
        assert_eq!(block, original, "{set}");
    }
}

/// Fields each `set.json` animation record copies from the builder's variant object.
const RECORD_FIELDS: [&str; 11] = [
    "icon",
    "motionId",
    "motionLabel",
    "direction",
    "heading",
    "description",
    "label",
    "intent",
    "active",
    "keyPoses",
    "brief",
];

/// The describe() variants of one set against its committed set.json; returns the records
/// checked and the records of adopted spec icons, which describe() no longer covers.
fn check_description(set: &Path) -> (usize, usize) {
    let number = &set.file_name().unwrap().to_string_lossy()[..2];
    let Ok(text) = fs::read_to_string(repo().join(format!("prototype/icons/{number}/assets/set.json"))) else {
        return (0, 0);
    };
    let committed: Value = serde_json::from_str(&text).unwrap();
    let limits = Limits::default();
    let source = SetSource::library_set(set).unwrap();
    let description = run_on_worker(&limits, || SetRuntime::load(&source, &limits)?.install()).unwrap();
    let (mut checked, mut adopted) = (0, 0);
    for entry in committed["icons"].as_array().unwrap() {
        // An icon adopted from a spec (`library adopt`) is described by its spec, not by
        // the definitions, which keep the old icon only for the legacy checks.
        if entry.get("source").and_then(Value::as_str) == Some("gai-icon/2") {
            adopted += entry.get("animations").and_then(Value::as_array).map_or(0, Vec::len);
            continue;
        }
        let icon = description.icons.iter().find(|icon| icon.name == entry["icon"]).expect("icon in describe()");
        for key in ["number", "forwardLabel", "reverseLabel"] {
            assert_eq!(entry.get(key).map(to_js_json), icon.meta.get(key).map(to_js_json), "{}.{key}", icon.name);
        }
        let records = entry.get("animations").or(entry.get("motions")).and_then(Value::as_array).unwrap();
        for record in records {
            let variant = icon.variants.iter().find(|v| v.name == record["name"]).expect("variant in describe()");
            for key in RECORD_FIELDS {
                assert_eq!(
                    record.get(key).map(to_js_json),
                    variant.meta.get(key).map(to_js_json),
                    "{}.{key}",
                    variant.name
                );
            }
            if let Some(constraints) = record.get("constraints") {
                let described = variant.meta.get("constraints").cloned().unwrap_or(json!([]));
                assert_eq!(to_js_json(constraints), to_js_json(&described), "{}.constraints", variant.name);
            }
            assert_eq!(record["rest"].as_f64(), variant.rest().map(|rest| rest * 60.0), "{}.rest", variant.name);
            checked += 1;
        }
    }
    (checked, adopted)
}

#[test]
fn description_matches_committed_set_json() {
    let sets: Vec<PathBuf> = ["01", "03", "07", "12", "40"].iter().filter_map(|n| set_dir(n)).collect();
    let checked: usize = sets.iter().map(|set| check_description(set).0).sum();
    eprintln!("{checked} set.json records match describe()");
}

#[test]
#[ignore = "whole catalog"]
fn every_description_matches_committed_set_json() {
    let sets: Vec<PathBuf> = (1..=63).filter_map(|n| set_dir(&format!("{n:02}"))).collect();
    let counts = map_parallel(sets, workers(), &Limits::default(), |set| check_description(&set));
    let (checked, adopted) = counts.into_iter().fold((0, 0), |(c, a), (x, y)| (c + x, a + y));
    eprintln!("{checked} set.json records match describe(); {adopted} come from adopted specs");
    assert!(checked + adopted > 3000 || checked == 0);
}

/// JS `Math.max(...values)`: NaN if any value is NaN, and +0 above -0.
fn js_max(values: impl IntoIterator<Item = f64>) -> f64 {
    values.into_iter().fold(f64::NEG_INFINITY, |max, v| {
        if max.is_nan() || v.is_nan() {
            f64::NAN
        } else if v > max || (v == 0.0 && max == 0.0 && v.is_sign_positive()) {
            v
        } else {
            max
        }
    })
}

fn js_min(values: impl IntoIterator<Item = f64>) -> f64 {
    -js_max(values.into_iter().map(|v| -v))
}

fn number(value: f64) -> Value {
    serde_json::Number::from_f64(value).map_or(Value::Null, Value::Number)
}

/// `rms(a, b)` of the measure-motion scripts: `(v - b)**2` is `x*x` under V8's pow.
fn rms(a: &[f64], b: &[f64]) -> f64 {
    let sum = a.iter().zip(b).fold(0.0, |n, (v, w)| n + (v - w) * (v - w));
    (sum / (a.len() as f64 / 2.0)).sqrt()
}

/// One profile's metrics, following `27-forms-input/measure-motion.mjs:26-35`.
fn metrics(probe: &ProbeResult, active: &Value, visible: bool) -> Map<String, Value> {
    for sample in &probe.samples {
        assert_eq!(sample.outcome, ProbeOutcome::Complete, "t={}", sample.time);
    }
    let points: Vec<Vec<f64>> =
        probe.samples.iter().map(|s| s.posed.iter().flat_map(|p| [p.x, p.y]).collect()).collect();
    let alpha: Vec<f64> =
        probe.samples.iter().map(|s| s.visibility.map_or(1.0, |v| if v.nullish { 1.0 } else { v.value })).collect();
    let time = |f: usize| probe.samples[f].time;
    let distances: Vec<f64> = points.iter().map(|p| rms(&points[0], p)).collect();
    let speeds: Vec<f64> = (1..points.len()).map(|f| rms(&points[f - 1], &points[f]) * 60.0).collect();
    let visible_speeds: Vec<f64> = speeds
        .iter()
        .enumerate()
        .map(|(j, v)| if js_max([alpha[j], alpha[j + 1]]) > 0.01 { *v } else { 0.0 })
        .collect();
    let peak = js_max(distances.iter().copied());
    let first_change = (0..points.len()).find(|&f| distances[f] > 0.01 || (alpha[f] - alpha[0]).abs() > 0.01).map(time);
    let first10 = (0..points.len()).find(|&f| distances[f] >= peak * 0.1).map(time);
    let first90 = (0..points.len()).find(|&f| distances[f] >= peak * 0.9).map(time);
    let sum = |values: &[f64]| values.iter().fold(0.0, |a, b| a + b) / 60.0;
    let mut out = Map::new();
    out.insert("activeSeconds".into(), active.clone());
    out.insert("firstChangeSeconds".into(), first_change.map_or(Value::Null, number));
    out.insert("peakRmsDisplacement".into(), number(peak));
    out.insert("travelRms".into(), number(sum(&speeds)));
    out.insert("peakRmsSpeed".into(), number(js_max(speeds.iter().copied())));
    if visible {
        out.insert("visibleTravelRms".into(), number(sum(&visible_speeds)));
        out.insert("visiblePeakRmsSpeed".into(), number(js_max(visible_speeds.iter().copied())));
    }
    out.insert("first10To90Seconds".into(), number(first90.unwrap_or(f64::NAN) - first10.unwrap_or(f64::NAN)));
    if probe.callbacks.transform {
        let transforms: Vec<[f64; 3]> =
            probe.samples.iter().map(|s| s.transform.as_ref().expect("transform evaluated").or).collect();
        for (key, i) in [("rotationRange", 0), ("horizontalRange", 1), ("verticalRange", 2)] {
            let range = js_max(transforms.iter().map(|t| t[i])) - js_min(transforms.iter().map(|t| t[i]));
            out.insert(key.into(), number(range));
        }
    }
    out
}

/// `motion-measurements.json` recomputed from probe() data, or `None` without a script.
fn measure(set: &Path) -> Option<(String, String)> {
    let script = fs::read_to_string(set.join("measure-motion.mjs")).ok()?;
    let oracle = fs::read_to_string(set.join("evidence/motion-measurements.json")).ok()?;
    let primary_line = script.lines().find(|line| line.starts_with("const primary="))?;
    let primary: Vec<String> =
        serde_json::from_str(&primary_line["const primary=".len()..].trim_end_matches(';').replace('\'', "\""))
            .unwrap();
    let visible = script.contains("visibleSpeeds");
    let limits = Limits::default();
    let source = SetSource::library_set(set).unwrap();
    let times: Vec<f64> = (0..FRAMES).map(gai_js::frame_time).collect();
    let records = run_on_worker(&limits, || -> gai_js::Result<(Vec<Value>, usize)> {
        let runtime = SetRuntime::load(&source, &limits)?;
        let icons = runtime.with_icons(|scope| scope.project(scope.icons().clone()))?.unwrap();
        let icons = icons.as_array().unwrap().clone();
        let mut records = Vec::new();
        for (i, icon) in icons.iter().enumerate() {
            let name = format!("{}-{}", icon["name"].as_str().unwrap(), primary[i]);
            let part = icon["parts"].as_array().unwrap().iter().position(|p| p["name"] == name.as_str()).unwrap();
            let mut all = Map::new();
            for strong in [false, true] {
                let request = ProbeRequest {
                    icon: i,
                    part,
                    strong,
                    reverse: false,
                    width: ProbeWidth::Value(2.0),
                    times: &times,
                };
                let profile = if strong { "strong" } else { "light" };
                all.insert(
                    profile.into(),
                    Value::Object(metrics(&runtime.probe(&request)?, &icon["timings"][profile]["active"], visible)),
                );
            }
            let (speed, travel) =
                if visible { ("visiblePeakRmsSpeed", "visibleTravelRms") } else { ("peakRmsSpeed", "travelRms") };
            let ratio = |key: &str| {
                number(all["strong"][key].as_f64().unwrap_or(f64::NAN) / all["light"][key].as_f64().unwrap_or(f64::NAN))
            };
            let mut record = Map::new();
            record.insert("icon".into(), icon["name"].clone());
            record.insert("mainPart".into(), json!(name));
            if let Some(intent) = icon.get("intent") {
                record.insert("intent".into(), intent.clone());
            }
            let (speed_ratio, travel_ratio) = (ratio(speed), ratio(travel));
            record.insert("metrics".into(), Value::Object(all));
            record.insert("strongToLightPeakSpeed".into(), speed_ratio);
            record.insert("strongToLightTravel".into(), travel_ratio);
            records.push(Value::Object(record));
        }
        Ok((records, icons.len()))
    })
    .unwrap_or_else(|error| panic!("{}: {error}", set.display()));
    let icon_count = if script.contains("icons:40,") { 40 } else { records.1 };
    let report = json!({
        "scope": "Source geometry and rigid transforms at 60fps; grid units and seconds. RMS includes curve control points. Native normal-speed review required.",
        "definitionsSha256": sha256_hex(fs::read(set.join("definitions.mjs")).unwrap()),
        "icons": icon_count,
        "records": records.0,
    });
    Some((to_js_json_pretty(&report, 2) + "\n", oracle))
}

fn check_measurements(numbers: &[&str]) -> usize {
    let sets: Vec<PathBuf> = numbers.iter().filter_map(|n| set_dir(n)).collect();
    let results = map_parallel(sets.clone(), workers(), &Limits::default(), |set| measure(&set));
    let mut checked = 0;
    for (set, result) in sets.iter().zip(results) {
        if let Some((built, oracle)) = result {
            assert!(built == oracle, "{}: motion-measurements.json differs", set.display());
            checked += 1;
        }
    }
    checked
}

/// One set per measure-motion script variant: 22, 23–26 and 27–39.
#[test]
fn probe_reproduces_motion_measurements() {
    let checked = check_measurements(&["22", "26", "27"]);
    eprintln!("{checked} motion-measurements.json files reproduced");
}

#[test]
#[ignore = "whole catalog"]
fn probe_reproduces_every_motion_measurement() {
    let numbers: Vec<String> = (22..=39).map(|n| n.to_string()).collect();
    let checked = check_measurements(&numbers.iter().map(String::as_str).collect::<Vec<_>>());
    eprintln!("{checked} motion-measurements.json files reproduced");
    assert!(checked == 18 || checked == 0);
}
