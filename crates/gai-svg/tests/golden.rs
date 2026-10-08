//! gai-svg against the golden files `packages/runtime/src/svg.mjs` writes
//! (`tests/svg.test.mjs`): both implementations must produce the same bytes.

use gai_svg::{Style, SvgError, SvgOptions, Trigger, icon_to_svg};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn golden() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn options(o: &Value) -> SvgOptions {
    let text = |key: &str| o[key].as_str().map(str::to_string);
    let mut options = SvgOptions::default();
    if let Some(motion) = text("motion") {
        options.motion = motion;
    }
    if let Some(style) = text("style") {
        options.style = Style::parse(&style).unwrap();
    }
    if let Some(weight) = o["weight"].as_f64() {
        options.weight = weight;
    }
    if let Some(color) = text("color") {
        options.color = color;
    }
    if let Some(trigger) = text("trigger") {
        options.trigger = Trigger::parse(&trigger).unwrap();
    }
    options.id = text("id");
    options.size = o["size"].as_f64();
    options.state = text("state");
    options
}

fn animations(set: &str) -> impl Fn(&str) -> Option<Value> {
    let dir = root().join("prototype/icons").join(set).join("assets");
    move |name| serde_json::from_str(&std::fs::read_to_string(dir.join(format!("{name}.json"))).ok()?).ok()
}

fn cases() -> Vec<Value> {
    let cases: Value = serde_json::from_str(&std::fs::read_to_string(golden().join("cases.json")).unwrap()).unwrap();
    cases["cases"].as_array().unwrap().clone()
}

#[test]
fn exports_match_the_javascript_goldens() {
    let cases = cases();
    assert!(cases.len() >= 10);
    for case in cases {
        let file = case["file"].as_str().unwrap();
        let svg = icon_to_svg(&case["icon"], &animations(case["set"].as_str().unwrap()), &options(&case["options"]))
            .unwrap_or_else(|error| panic!("{file}: {error}"));
        let expected = std::fs::read_to_string(golden().join(file)).unwrap();
        assert!(svg == expected, "{file} ({}) differs from the golden file", case["why"]);
    }
}

#[test]
fn options_that_dont_fit_the_icon_are_refused() {
    let cases = cases();
    let find = |id: &str| cases.iter().find(|c| c["icon"]["id"] == id).unwrap();
    let (bell, sort) = (find("bell"), find("sort-order"));
    let run = |case: &Value, options: SvgOptions| {
        icon_to_svg(&case["icon"], &animations(case["set"].as_str().unwrap()), &options).unwrap_err()
    };
    let error = run(bell, SvgOptions { trigger: Trigger::Toggle, ..Default::default() });
    assert_eq!(error, SvgError::Options("bell is not stateful: use hover, click, loop, load or js".into()));
    let error = run(sort, SvgOptions { trigger: Trigger::Hover, ..Default::default() });
    assert_eq!(error, SvgError::Options("sort-order is stateful: use the toggle or js trigger".into()));
    let error = run(sort, SvgOptions { state: Some("sideways".into()), ..Default::default() });
    assert_eq!(error, SvgError::Options("Unknown state: sideways".into()));
    let error = run(bell, SvgOptions { motion: "medium".into(), ..Default::default() });
    assert_eq!(error, SvgError::Options("bell has no medium motion".into()));
    let missing = icon_to_svg(&bell["icon"], &|_| None, &SvgOptions::default()).unwrap_err();
    assert_eq!(missing, SvgError::Invalid("Animation bell is missing".into()));
}

#[test]
fn unsupported_lottie_is_an_error() {
    let precomp = serde_json::json!({
        "w": 32, "h": 32, "fr": 60, "ip": 0, "op": 10,
        "layers": [{"ty": 0, "nm": "precomp", "ind": 1, "ks": {}}]
    });
    let options = SvgOptions { trigger: Trigger::Loop, ..Default::default() };
    let error = gai_svg::lottie_to_svg(&precomp, None, None, &options).unwrap_err();
    assert!(matches!(error, SvgError::Unsupported(_)), "{error}");
}

/// The whole library against a reference the JavaScript writes, one JSON line per export:
/// `{set, icon, options, svg, error}`. Run with `GAI_SVG_REFERENCE=<file> cargo test -p
/// gai-svg -- --ignored`.
#[test]
#[ignore = "needs GAI_SVG_REFERENCE, a JSON-lines reference written by svg.mjs"]
fn the_library_matches_the_javascript_reference() {
    use std::io::BufRead;
    let path = std::env::var("GAI_SVG_REFERENCE").expect("GAI_SVG_REFERENCE");
    let (mut count, mut differ) = (0, Vec::new());
    for line in std::io::BufReader::new(std::fs::File::open(path).unwrap()).lines() {
        let case: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let result = icon_to_svg(&case["icon"], &animations(case["set"].as_str().unwrap()), &options(&case["options"]));
        let same = match (&result, case["svg"].as_str()) {
            (Ok(svg), Some(expected)) => svg == expected,
            (Err(_), None) => true,
            _ => false,
        };
        if !same {
            differ.push(format!("{} {}", case["icon"]["id"], case["options"]));
        }
        count += 1;
    }
    assert!(differ.is_empty(), "{} of {count} differ, e.g. {:?}", differ.len(), &differ[..differ.len().min(5)]);
    eprintln!("{count} exports match");
}
