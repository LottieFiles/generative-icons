//! Build orchestration on a small user project written into a temporary folder: single-icon
//! builds, offline scenes, full builds through a synthetic exporter, staging, the script
//! check and the AI scope guard. Needs no oracle files.

use gai_build::build::{AnimationReport, BuildError, BuildOptions, Mode, Target, build};
use gai_build::records::builder_identity_sha256;
use gai_build::template::{ENGINE_SCRIPT, ENGINE_SCRIPT_SHA256};
use gai_core::export::{Exporter, NativeExport};
use gai_core::sha256_hex;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

const TIMING: &str = "export const timePair = (light, strong) => ({
  light: {active: light, rest: light + .04, keyPoses: [0, light]},
  strong: {active: strong, rest: strong + .04, keyPoses: [0, strong]},
});
";

const SHAPES: &str = "import {timePair} from '../../library/timing.mjs';
export const icons = [
  {name: 'dot', label: 'Dot', heading: 'Dot', description: 'A bar grows.', intent: 'A bar grows.',
   brief: {light: 'grows', strong: 'grows fast'}, number: 1, cycleDegrees: 90, timings: timePair(.5, .6),
   parts: [
     {name: 'dot-bar', style: 'outline', type: 'stroke', d: (t, s) => `M8 16 L${8 + 16 * Math.min(1, t / (s ? .6 : .5))} 16`},
     {name: 'dot-fill', style: 'tint', d: 'M10 10 L22 10 L22 22 Z'},
   ]},
  {name: 'switch', label: 'Switch', heading: 'Switch', description: 'A knob slides.', intent: 'A knob slides.',
   stateful: true, initialState: 'off', finalState: 'on', reverseSuffix: '-reverse', forwardLabel: 'on', reverseLabel: 'off',
   timings: timePair(.4, .5), reverseTimings: timePair(.3, .3), constraints: [{type: 'fixed', part: 'switch-track'}],
   parts: [
     {name: 'switch-track', style: 'solid', d: 'M6 12 L26 12 L26 20 L6 20 Z'},
     {name: 'switch-knob', style: 'outline', type: 'stroke', d: 'M10 16 L12 16',
      pose: (x, y, t, s, r) => [x + 10 * (r ? 1 - Math.min(1, t / .3) : Math.min(1, t / .4)), y]},
   ]},
];
";

/// `gen-animated-icons/{icons,library}` and a build folder in a temporary directory.
struct Project {
    dir: tempfile::TempDir,
}

impl Project {
    fn new(config: Value) -> Project {
        let dir = tempfile::tempdir().expect("temporary folder");
        let files = [
            ("gen-animated-icons/library/timing.mjs", TIMING.to_owned()),
            ("gen-animated-icons/icons/batches/shapes.mjs", SHAPES.to_owned()),
            ("gen-animated-icons/icons/batches/.gitkeep", String::new()),
            (
                "gen-animated-icons/icons/definitions.mjs",
                "import {icons as shapes} from './batches/shapes.mjs';\nexport const icons = shapes;\n".to_owned(),
            ),
            ("gen-animated-icons/icons/set-config.json", config.to_string()),
        ];
        for (path, text) in files {
            let path = dir.path().join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        Project { dir }
    }

    fn default() -> Project {
        Project::new(json!({"id": "70-fixture", "name": "Fixture icons", "sharedSources": ["timing.mjs"],
            "builder": {"defaultMotion": "light"}}))
    }

    fn icons(&self) -> PathBuf {
        self.dir.path().join("gen-animated-icons/icons")
    }

    fn build_dir(&self) -> PathBuf {
        self.dir.path().join(".gen-animated-icons/build")
    }

    fn target(&self) -> Target {
        Target::project(self.icons(), self.build_dir()).expect("project target")
    }

    fn read(&self, relative: &str) -> Vec<u8> {
        fs::read(self.build_dir().join(relative)).unwrap_or_else(|error| panic!("{relative}: {error}"))
    }

    fn files(&self) -> Vec<String> {
        let mut files = Vec::new();
        collect(&self.build_dir(), &self.build_dir(), &mut files);
        files.sort();
        files
    }
}

fn collect(root: &Path, dir: &Path, files: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        if entry.path().is_dir() {
            collect(root, &entry.path(), files);
        } else {
            files.push(entry.path().strip_prefix(root).unwrap().to_string_lossy().into_owned());
        }
    }
}

/// Answers each scene with a native whose layers mirror its nodes, in reverse order.
struct Synthetic {
    script_sha256: String,
    fail_at: Option<usize>,
    calls: AtomicUsize,
}

impl Synthetic {
    fn new() -> Synthetic {
        Synthetic { script_sha256: ENGINE_SCRIPT_SHA256.into(), fail_at: None, calls: AtomicUsize::new(0) }
    }
}

impl Exporter for Synthetic {
    fn export(&self, name: &str, scene_json: &str) -> anyhow::Result<NativeExport> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        anyhow::ensure!(self.fail_at != Some(call), "engine unavailable for {name}");
        let scene: Value = serde_json::from_str(scene_json)?;
        let nodes = scene["nodes"].as_array().expect("scene nodes");
        let layers: Vec<Value> = nodes
            .iter()
            .rev()
            .enumerate()
            .map(|(i, node)| {
                let paint = if node["type"] == "stroke" {
                    json!({"c": {"a": 0, "k": [0.0, 0.8666667, 0.7019608, 1.0]}, "o": {"a": 0, "k": 0.0}, "ty": "st", "w": {"a": 0, "k": 2.0}})
                } else {
                    json!({"c": {"a": 0, "k": [0.0, 0.8666667, 0.7019608, 1.0]}, "o": {"a": 0, "k": 100.0}, "ty": "fl"})
                };
                json!({"ind": i + 1, "nm": node["name"], "shapes": [{"ks": {"a": 0, "k": 0}, "nm": "path", "ty": "sh"}, paint], "ty": 4})
            })
            .collect();
        let native = json!({"assets": [], "fr": 60.0, "h": 32.0, "ip": 0.0, "layers": layers, "nm": name, "op": 144.0, "w": 32.0});
        Ok(NativeExport {
            text: serde_json::to_string(&native)?,
            provenance: json!({"scriptSha256": self.script_sha256, "sceneSha256": sha256_hex(scene_json)}),
        })
    }
}

fn options(icon: Option<&str>) -> BuildOptions<'static> {
    BuildOptions { icon: icon.map(str::to_owned), ..BuildOptions::default() }
}

#[test]
fn offline_scenes_for_one_icon() {
    let project = Project::default();
    let lines = Mutex::new(Vec::new());
    let progress = |report: &AnimationReport| lines.lock().unwrap().push(report.to_string());
    let options = BuildOptions { progress: Some(&progress), ..options(Some("switch")) };
    let report = build(&project.target(), Mode::OfflineScenes, &options).unwrap();
    assert_eq!(report.set_id, "70-fixture");
    assert!(report.set_json.is_none());
    assert_eq!(
        lines.into_inner().unwrap(),
        [
            "Switch / Light motion: scene written",
            "Switch / Light motion: scene written",
            "Switch / Strong motion: scene written",
            "Switch / Strong motion: scene written"
        ]
    );
    let names = ["switch", "switch-reverse", "switch-strong", "switch-reverse-strong"];
    let mut expected: Vec<String> =
        names.iter().flat_map(|n| [format!("scenes/{n}.engine.js"), format!("scenes/{n}.scene.json")]).collect();
    expected.sort();
    assert_eq!(project.files(), expected);
    assert_eq!(project.read("scenes/switch.engine.js"), ENGINE_SCRIPT);
    let scene: Value = serde_json::from_slice(&project.read("scenes/switch-reverse.scene.json")).unwrap();
    assert_eq!(scene["rest"], json!(0.35));
    assert_eq!(scene["nodes"].as_array().unwrap().len(), 4);
}

#[test]
fn one_icon_of_a_project_through_the_engine() {
    let project = Project::default();
    let exporter = Synthetic::new();
    let report = build(&project.target(), Mode::Full(&exporter), &options(Some("dot"))).unwrap();
    assert_eq!(
        report.animations.iter().map(ToString::to_string).collect::<Vec<_>>(),
        ["Dot / Light motion: 4 native layers exported", "Dot / Strong motion: 4 native layers exported"]
    );
    // A single icon never writes set.json.
    assert!(!project.build_dir().join("set.json").exists());
    assert_eq!(report.files.len(), 2 * 8);

    let asset: Value = serde_json::from_slice(&project.read("dot.json")).unwrap();
    assert_eq!(asset["nm"], "Motion / Fixture icons / Dot");
    assert_eq!(asset["layers"][0]["nm"], "dot-fill");
    assert_eq!(asset["layers"][2]["shapes"][1]["w"]["sid"], "style.outline.width");
    assert_eq!(asset["meta"], json!({"generator": "Engine MCP native export", "set": "70-fixture", "restFrame": 33}));
    assert_eq!(asset["markers"][0]["dr"], json!(30));
    let provenance = String::from_utf8(project.read("native/dot-strong.json.provenance.json")).unwrap();
    assert!(provenance.starts_with("{\n  \"scriptSha256\": ") && provenance.ends_with("}\n"));

    let set = report.set_json.expect("records were made");
    assert_eq!((&set["category"], &set["defaultMotion"]), (&json!("Fixture icons"), &json!("light")));
    let icons = set["icons"].as_array().unwrap();
    assert_eq!(icons.len(), 1);
    let record = &icons[0]["animations"][1];
    let identity = builder_identity_sha256();
    assert_eq!(
        (record["sourceSha256"].as_str(), record["setBuilderSha256"].as_str()),
        (Some(&*identity), Some(&*identity))
    );
    let authoring: Vec<&String> = record["authoringSources"].as_object().unwrap().keys().collect();
    assert_eq!(authoring, ["set-config.json", "batches/shapes.mjs"]);
    let direction: Vec<&String> = record["directionSources"].as_object().unwrap().keys().collect();
    assert_eq!(direction, ["timing.mjs"]);
    assert_eq!(record["nativeSha256"].as_str().unwrap(), sha256_hex(project.read("native/dot-strong.json")));
    assert_eq!(record["brief"], json!({"light": "grows", "strong": "grows fast"}));
    assert_eq!(
        (&record["cycleDegrees"], &record["continuous"], &record["duotoneFill"]),
        (&json!(90), &json!(false), &json!(true))
    );
    assert_eq!((&record["rest"], &record["frames"], &record["constraints"]), (&json!(39), &json!(144), &json!([])));
    assert_eq!(icons[0]["number"], json!(1));
}

#[test]
fn a_whole_project_writes_set_json() {
    let project = Project::default();
    let report =
        build(&project.target(), Mode::Full(&Synthetic::new()), &BuildOptions { jobs: 4, ..options(None) }).unwrap();
    assert_eq!(report.animations.len(), 6);
    let written: Value = serde_json::from_slice(&project.read("set.json")).unwrap();
    assert_eq!(Some(&written), report.set_json.as_ref());
    let switch = &written["icons"][1];
    assert_eq!(switch["motions"][1]["reverse"]["name"], "switch-reverse-strong");
    assert_eq!(switch["reverse"]["name"], "switch-reverse");
    assert_eq!((&switch["forwardLabel"], &switch["reverseLabel"]), (&json!("on"), &json!("off")));
    assert_eq!(switch["animations"][1]["startState"], "on");
    assert_eq!(switch["constraints"], json!([{"type": "fixed", "part": "switch-track"}]));
    assert_eq!(switch["duotoneFill"], json!(false));
}

#[test]
fn staging_keeps_earlier_outputs_when_a_build_fails() {
    let project = Project::default();
    let failing = Synthetic { fail_at: Some(2), ..Synthetic::new() };
    let staged = BuildOptions { staging: true, ..options(None) };
    let error = build(&project.target(), Mode::Full(&failing), &staged).unwrap_err();
    assert!(matches!(&error, BuildError::Export { animation, .. } if animation == "switch"), "{error}");
    assert!(project.files().is_empty());

    // Without staging, the animations before the failure stay written, as they did in JS.
    let failing = Synthetic { fail_at: Some(2), ..Synthetic::new() };
    build(&project.target(), Mode::Full(&failing), &options(None)).unwrap_err();
    let files = project.files();
    assert!(files.contains(&"dot-strong.theme-light.json".to_owned()));
    assert!(files.contains(&"scenes/switch.scene.json".to_owned()));
    assert!(!files.iter().any(|file| file == "switch.json" || file == "set.json"));

    // A staged build that succeeds replaces them.
    let report = build(&project.target(), Mode::Full(&Synthetic::new()), &staged).unwrap();
    assert!(report.files.contains(&project.build_dir().join("set.json")));
    assert_eq!(project.files().len(), report.files.len());
}

#[test]
fn the_written_script_must_be_the_exported_one() {
    let project = Project::default();
    let hosted = Synthetic { script_sha256: sha256_hex("hosted script"), ..Synthetic::new() };
    let error = build(&project.target(), Mode::Full(&hosted), &options(Some("dot"))).unwrap_err();
    assert!(matches!(error, BuildError::Script { .. }), "{error}");

    let options = BuildOptions { engine_script: b"hosted script", ..options(Some("dot")) };
    build(&project.target(), Mode::Full(&hosted), &options).unwrap();
    assert_eq!(project.read("scenes/dot.engine.js"), b"hosted script");
}

#[test]
fn guards() {
    let project = Project::default();
    let error = build(&project.target(), Mode::OfflineScenes, &options(Some("missing"))).unwrap_err();
    assert!(matches!(&error, BuildError::UnknownIcon(name) if name == "missing"), "{error}");

    // run-ai.mjs: a set in the AI scope must define exactly the scope's icons.
    let scope = project.dir.path().join("gen-animated-icons/library/design-reviews/ai-category/scope.json");
    fs::create_dir_all(scope.parent().unwrap()).unwrap();
    fs::write(&scope, r#"{"sets":[{"id":"70-fixture","icons":["switch","dot"]}]}"#).unwrap();
    let error = build(&project.target(), Mode::ReuseNative, &options(None)).unwrap_err();
    assert_eq!(error.to_string(), "Incomplete or misordered AI scope: 70-fixture");
    fs::write(&scope, r#"{"sets":[{"id":"70-fixture","icons":["dot","switch"]}]}"#).unwrap();
    build(&project.target(), Mode::OfflineScenes, &options(None)).unwrap();

    let missing = Project::new(json!({"id": "71-x", "name": "X", "builder": {"catgory": "typo"}}));
    assert!(Target::project(missing.icons(), missing.build_dir()).is_err());
}

#[test]
fn frozen_library_sets_are_refused() {
    let heart = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../motion/icon-sets/01-heart-bell");
    if !heart.is_dir() {
        eprintln!("skipped: no library checkout");
        return;
    }
    let error = Target::library_set(&heart).unwrap_err();
    assert!(error.to_string().contains("frozen"), "{error}");
}
