//! End-to-end project commands against temp web, SwiftUI and Android apps, using a
//! fixture library so they run anywhere.

mod common;

use common::*;
use generative_icons::embedded::SKILL_FILES;
use generative_icons::kit;
use generative_icons::weight::{read_zip, rewrite_stroke_width};
use serde_json::{Value, json};
use std::path::Path;

/// `generative-icons.json` as `init` writes it for the React fixture (cli.md §6.5).
const WEB_CONFIG: &str = r##"{
  "schemaVersion": 1,
  "platform": "web",
  "framework": "react",
  "packageManager": "npm",
  "sourceDirectory": "src",
  "assets": "public/generative-icons",
  "descriptors": "src/generative-icons",
  "publicPath": "/generative-icons",
  "defaults": {
    "color": "#008c76",
    "style": "outline",
    "weight": 2,
    "motion": "light"
  },
  "local": {
    "source": "generative-icons/icons",
    "license": "UNLICENSED"
  },
  "skills": [
    ".claude/skills",
    ".agents/skills"
  ]
}
"##;

fn library_arg(dir: &Path) -> String {
    let library = dir.join("library");
    fixture_library(&library);
    library.display().to_string()
}

fn icon_descriptor(library: &str, id: &str) -> Value {
    let catalog: Value = serde_json::from_str(&read(Path::new(library), "catalog.json")).unwrap();
    catalog["icons"].as_array().unwrap().iter().find(|icon| icon["id"] == id).unwrap().clone()
}

#[test]
fn web_init_writes_config_skills_kit_and_lock() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path();
    web_app(app);
    let result = run(app, &["init"]).ok();
    assert_eq!(read(app, "generative-icons.json"), WEB_CONFIG);
    assert_eq!(
        (
            result["platform"].as_str(),
            result["framework"].as_str(),
            result["packageManager"].as_str(),
            result["sourceDirectory"].as_str()
        ),
        (Some("web"), Some("react"), Some("npm"), Some("src"))
    );
    assert_eq!(
        result["skills"],
        json!([".claude/skills/generative-icons/SKILL.md", ".agents/skills/generative-icons/SKILL.md"])
    );
    let changed: Vec<&str> = result["changed"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(changed.first(), Some(&"generative-icons.json"));
    assert_eq!(changed.last(), Some(&".generative-icons/lock.json"));
    assert_eq!(changed.len(), 1 + 2 * SKILL_FILES.len() + kit::files().len() + 2);
    for root in [".claude/skills", ".agents/skills"] {
        for (relative, content) in SKILL_FILES {
            assert_eq!(read(app, &format!("{root}/generative-icons/{relative}")), content);
        }
    }
    for file in kit::files() {
        assert_eq!(read(app, &format!("generative-icons/library/{}", file.name)), file.text);
    }
    assert_eq!(read(app, ".generative-icons/.gitignore"), "*\n!.gitignore\n!lock.json\n");
    let lock: Value = serde_json::from_str(&read(app, ".generative-icons/lock.json")).unwrap();
    let keys: Vec<&String> = lock.as_object().unwrap().keys().collect();
    assert_eq!(keys, ["schemaVersion", "generator", "icons", "files"]);
    assert_eq!(
        lock["files"][".claude/skills/generative-icons/SKILL.md"],
        sha(app, ".claude/skills/generative-icons/SKILL.md")
    );
    assert!(lock["files"].get("generative-icons.json").is_none());
    // AGENTS.md, CLAUDE.md and MCP settings are never written; the MCP entry is printed.
    assert!(!app.join("AGENTS.md").exists() && !app.join("CLAUDE.md").exists() && !app.join(".mcp.json").exists());
    assert_eq!(result["mcp"]["mcpServers"]["generative-icons"]["args"], json!(["generative-icons", "mcp"]));
    // The entry for this very binary works without the npm package.
    let exe = std::fs::canonicalize(env!("CARGO_BIN_EXE_generative-icons")).unwrap();
    let local = json!({"mcpServers": {"generative-icons": {"command": exe.to_str().unwrap(), "args": ["mcp"]}}});
    assert_eq!(result["mcpLocal"], local);

    // A second run changes nothing, and says both entries in text.
    assert_eq!(run(app, &["init"]).ok()["changed"], json!([]));
    let text = run(app, &["init", "--format", "text"]);
    assert!(
        text.stdout.contains(&format!(
            "To run this binary rather than the npm package, point the entry at it instead:\n  {}\n",
            gai_core::to_js_json(&local)
        )),
        "{}",
        text.stdout
    );
}

#[test]
fn init_upgrades_unedited_skills_and_preserves_edited_ones() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path();
    web_app(app);
    run(app, &["init", "--agents", "claude"]).ok();
    let skill = ".claude/skills/generative-icons/SKILL.md";
    assert!(!app.join(".agents/skills").exists());

    // An older release's skill, recorded in the lock, is replaced.
    write(app, skill, "old skill\n");
    let lock_path = app.join(".generative-icons/lock.json");
    let mut lock: Value = serde_json::from_str(&read(app, ".generative-icons/lock.json")).unwrap();
    lock["files"][skill] = json!(gai_core::sha256_hex("old skill\n"));
    std::fs::write(&lock_path, gai_core::to_js_json_pretty(&lock, 2) + "\n").unwrap();
    let result = run(app, &["init"]).ok();
    assert_eq!(result["changed"], json!([skill, ".generative-icons/lock.json"]));
    assert_eq!(read(app, skill), SKILL_FILES[0].1);

    // A local edit is preserved.
    write(app, skill, "my notes\n");
    let error = run(app, &["init"]).fails(3, "MODIFIED_FILE");
    assert_eq!(error["message"], format!("Preserving modified file: {skill}"));
    assert_eq!(read(app, skill), "my notes\n");
    run(app, &["init", "--agents", "none"]).fails(2, "USAGE");
}

#[test]
fn web_add_installs_descriptors_assets_and_lock() {
    let dir = tempfile::tempdir().unwrap();
    let app = &dir.path().join("app");
    std::fs::create_dir(app).unwrap();
    web_app(app);
    let library = library_arg(dir.path());
    run(app, &["init"]).ok();
    let result = run(app, &["add", "heart", "menu", "heart", "--library", &library]).ok();
    assert_eq!(
        result,
        json!({
            "changed": [
                "src/generative-icons/settings.mjs", "src/generative-icons/settings.d.mts",
                "public/generative-icons/heart.lottie", "src/generative-icons/heart.mjs", "src/generative-icons/heart.d.mts",
                "public/generative-icons/menu.lottie", "src/generative-icons/menu.mjs", "src/generative-icons/menu.d.mts",
                ".generative-icons/lock.json"
            ],
            "icons": ["heart", "menu", "heart"],
            "version": FIXTURE_VERSION,
            "platform": "web"
        })
    );
    let descriptor = icon_descriptor(&library, "heart");
    assert_eq!(
        read(app, "src/generative-icons/heart.mjs"),
        format!(
            "// Generated by generative-icons. Review local edits before updating.\nexport const icon = {};\nexport const src = \"/generative-icons/heart.lottie\";\nexport default icon;\n",
            gai_core::to_js_json_pretty(&descriptor, 2)
        )
    );
    assert_eq!(
        read(app, "src/generative-icons/settings.mjs"),
        "// Shared project defaults. Edit generative-icons.json, then run add to update.\nexport const defaults = {\n  \"color\": \"#008c76\",\n  \"style\": \"outline\",\n  \"weight\": 2,\n  \"motion\": \"light\"\n};\n"
    );
    assert_eq!(sha(app, "public/generative-icons/heart.lottie"), descriptor["asset"]["sha256"].as_str().unwrap());
    let lock: Value = serde_json::from_str(&read(app, ".generative-icons/lock.json")).unwrap();
    let keys: Vec<&String> = lock.as_object().unwrap().keys().collect();
    assert_eq!(keys, ["schemaVersion", "libraryVersion", "generator", "icons", "files"]);
    assert_eq!(
        lock["icons"]["heart"],
        json!({"version": FIXTURE_VERSION, "sha256": descriptor["asset"]["sha256"],
               "source": "src/generative-icons/heart.mjs", "asset": "public/generative-icons/heart.lottie"})
    );
    assert_eq!(lock["files"]["src/generative-icons/menu.mjs"], sha(app, "src/generative-icons/menu.mjs"));

    // Re-adding is a no-op; a dry run of a new icon writes nothing.
    assert_eq!(run(app, &["add", "heart", "--library", &library]).ok()["changed"], json!([]));
    let dry = run(app, &["add", "bell-ring", "--dry-run", "--library", &library]).ok();
    assert_eq!(dry["dryRun"], json!(true));
    assert!(!app.join("public/generative-icons/bell-ring.lottie").exists());
    run(app, &["add", "nope", "--library", &library]).fails(3, "UNKNOWN_ICON");
}

#[test]
fn edited_files_block_add_without_partial_writes() {
    // tests/packages.test.mjs:76-80: a modified heart.mjs makes `add bell heart` fail.
    let dir = tempfile::tempdir().unwrap();
    let app = &dir.path().join("app");
    std::fs::create_dir(app).unwrap();
    web_app(app);
    let library = library_arg(dir.path());
    run(app, &["init"]).ok();
    run(app, &["add", "heart", "--library", &library]).ok();
    let before = read(app, ".generative-icons/lock.json");
    write(app, "src/generative-icons/heart.mjs", "// mine\n");
    let error = run(app, &["add", "bell-ring", "heart", "--library", &library]).fails(3, "MODIFIED_FILE");
    assert_eq!(error["path"], "src/generative-icons/heart.mjs");
    assert!(!app.join("public/generative-icons/bell-ring.lottie").exists());
    assert_eq!(read(app, ".generative-icons/lock.json"), before);
    let diff = run(app, &["diff"]).ok();
    let modified: Vec<&Value> = diff.as_array().unwrap().iter().filter(|s| s["status"] == "modified").collect();
    assert_eq!(modified, [&json!({"file": "src/generative-icons/heart.mjs", "status": "modified"})]);
}

#[test]
fn paths_and_project_roots_are_validated() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path();
    web_app(app);
    run(app, &["init"]).ok();
    let library = {
        let library = tempfile::tempdir().unwrap();
        fixture_library(library.path());
        library
    };
    let library = library.path().display().to_string();

    let config = read(app, "generative-icons.json");
    write(app, "generative-icons.json", config.replace("\"public/generative-icons\"", "\"../escape\""));
    let error = run(app, &["add", "heart", "--library", &library]).fails(3, "INVALID_PATH");
    assert_eq!(error["message"], "Expected a project-relative path: ../escape");
    write(app, "generative-icons.json", &config);

    #[cfg(unix)]
    {
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), app.join("public")).unwrap();
        // Config validation already refuses the symlinked assets folder, as readConfig did.
        let error = run(app, &["add", "heart", "--library", &library]).fails(3, "SYMLINK_PATH");
        assert_eq!(error["message"], "Symlinks are not allowed in managed paths: public/generative-icons");
        assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
    }

    let workspace = tempfile::tempdir().unwrap();
    write(workspace.path(), "package.json", r#"{"private":true,"workspaces":["apps/*"]}"#);
    let error = run(workspace.path(), &["init"]).fails(3, "WORKSPACE_ROOT");
    assert_eq!(error["message"], "This is a workspace root. Select an app with --cwd <app-folder>");
    assert!(!workspace.path().join(".generative-icons").exists());

    let empty = tempfile::tempdir().unwrap();
    run(empty.path(), &["init"]).fails(3, "PROJECT_NOT_FOUND");
    run(empty.path(), &["add", "heart"]).fails(3, "PROJECT_NOT_FOUND");
    run(empty.path(), &["--cwd", "missing", "info"]).fails(3, "PROJECT_NOT_FOUND");
}

#[test]
fn swiftui_project_gets_bundle_assets_and_json_descriptors() {
    let dir = tempfile::tempdir().unwrap();
    let app = &dir.path().join("app");
    std::fs::create_dir(app).unwrap();
    swiftui_app(app);
    let library = library_arg(dir.path());
    let result = run(app, &["init"]).ok();
    assert_eq!(
        (
            result["platform"].as_str(),
            result["framework"].as_str(),
            result["packageManager"].as_str(),
            result["sourceDirectory"].as_str()
        ),
        (Some("ios"), Some("swiftui"), Some("spm"), Some("Demo"))
    );
    let config: Value = serde_json::from_str(&read(app, "generative-icons.json")).unwrap();
    assert_eq!(config["assets"], "Demo/GenerativeIcons");
    assert_eq!(config["descriptors"], "generative-icons/descriptors");
    assert!(config.get("publicPath").is_none());

    let result = run(app, &["add", "date-range", "--library", &library]).ok();
    assert_eq!(
        result["changed"],
        json!([
            "Demo/GenerativeIcons/date-range.lottie",
            "generative-icons/descriptors/date-range.json",
            ".generative-icons/lock.json"
        ])
    );
    assert_eq!(result["platform"], "ios");
    let descriptor = icon_descriptor(&library, "date-range");
    assert_eq!(
        read(app, "generative-icons/descriptors/date-range.json"),
        gai_core::to_js_json_pretty(&descriptor, 2) + "\n"
    );
}

#[test]
fn android_project_bakes_in_between_weights() {
    let dir = tempfile::tempdir().unwrap();
    let app = &dir.path().join("app");
    std::fs::create_dir(app).unwrap();
    android_app(app);
    let library = library_arg(dir.path());
    let result = run(app, &["init"]).ok();
    assert_eq!(
        (result["platform"].as_str(), result["framework"].as_str(), result["sourceDirectory"].as_str()),
        (Some("android"), Some("compose"), Some("app/src/main"))
    );
    let config = read(app, "generative-icons.json");
    assert!(config.contains("\"assets\": \"app/src/main/assets\""));

    run(app, &["add", "heart", "--library", &library]).ok();
    let original = std::fs::read(Path::new(&library).join("assets/heart.lottie")).unwrap();
    assert_eq!(std::fs::read(app.join("app/src/main/assets/heart.lottie")).unwrap(), original);

    let result = run(app, &["add", "heart", "--bake-weight", "1.75", "--library", &library]).ok();
    assert_eq!(result["changed"], json!(["app/src/main/assets/heart.lottie", ".generative-icons/lock.json"]));
    let baked = std::fs::read(app.join("app/src/main/assets/heart.lottie")).unwrap();
    assert_eq!(baked, rewrite_stroke_width(&original, 1.75).unwrap());
    let animation = &read_zip(&baked).unwrap()[1];
    assert!(String::from_utf8_lossy(&animation.data).contains(r#""w":{"a":0,"k":1.75,"sid":"style.outline.width"}"#));
    let lock: Value = serde_json::from_str(&read(app, ".generative-icons/lock.json")).unwrap();
    assert_eq!(lock["icons"]["heart"]["variant"], json!({"weight": 1.75}));
    assert_eq!(lock["icons"]["heart"]["sha256"], gai_core::sha256_hex(&baked));

    // --installed keeps the baked weight.
    assert_eq!(run(app, &["add", "--installed", "--library", &library]).ok()["changed"], json!([]));
    run(app, &["add", "heart", "--bake-weight", "2", "--library", &library]).fails(2, "USAGE");

    // An in-between default weight without baking warns about snapping.
    let config = read(app, "generative-icons.json").replace("\"weight\": 2", "\"weight\": 1.8");
    write(app, "generative-icons.json", config);
    let run = run(app, &["add", "menu", "--library", &library]);
    assert_eq!(run.code, 0, "{run:?}");
    assert!(run.stderr.contains("players show 2"), "{}", run.stderr);
}

#[test]
fn android_module_is_chosen_or_asked_for() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path();
    write(app, "settings.gradle", "include ':phone', ':wear'\n");
    for module in ["phone", "wear"] {
        write(app, &format!("{module}/build.gradle"), "plugins { id 'com.android.application' }\n");
        write(app, &format!("{module}/src/main/AndroidManifest.xml"), "<manifest/>\n");
    }
    let error = run(app, &["init"]).fails(3, "AMBIGUOUS_PLATFORM");
    assert_eq!(error["hint"], "Choose one with --module <name>");
    run(app, &["init", "--module", "wear"]).ok();
    assert!(read(app, "generative-icons.json").contains("\"assets\": \"wear/src/main/assets\""));
}

#[test]
fn several_platforms_need_platform() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path();
    web_app(app);
    swiftui_app(app);
    let error = run(app, &["init"]).fails(3, "AMBIGUOUS_PLATFORM");
    assert_eq!(error["hint"], "Choose one with --platform web|ios");
    assert_eq!(run(app, &["init", "--platform", "ios"]).ok()["platform"], "ios");
    // The config now decides.
    assert_eq!(run(app, &["info"]).ok()["platform"], "ios");
}

#[test]
fn remove_deletes_only_unedited_files() {
    let dir = tempfile::tempdir().unwrap();
    let app = &dir.path().join("app");
    std::fs::create_dir(app).unwrap();
    web_app(app);
    let library = library_arg(dir.path());
    run(app, &["init"]).ok();
    run(app, &["add", "heart", "menu", "--library", &library]).ok();
    write(app, "src/generative-icons/menu.d.mts", "// edited\n");
    run(app, &["remove", "menu"]).fails(3, "MODIFIED_FILE");
    assert!(app.join("public/generative-icons/menu.lottie").exists());

    let result = run(app, &["remove", "heart"]).ok();
    assert_eq!(
        result,
        json!({"removed": ["heart"], "changed": [
            "public/generative-icons/heart.lottie", "src/generative-icons/heart.mjs",
            "src/generative-icons/heart.d.mts", ".generative-icons/lock.json"]})
    );
    assert!(!app.join("src/generative-icons/heart.mjs").exists());
    let lock: Value = serde_json::from_str(&read(app, ".generative-icons/lock.json")).unwrap();
    assert!(lock["icons"].get("heart").is_none());
    assert!(lock["files"].get("src/generative-icons/heart.mjs").is_none());
    run(app, &["remove", "heart"]).fails(3, "UNKNOWN_ICON");
}

#[test]
fn info_and_doctor_describe_the_setup() {
    let dir = tempfile::tempdir().unwrap();
    let app = &dir.path().join("app");
    std::fs::create_dir(app).unwrap();
    let library = library_arg(dir.path());

    // Outside a project, info reports nulls and still succeeds.
    let info = run(app, &["info", "--library", &library]).ok();
    assert_eq!(
        (info["platform"].clone(), info["config"].clone(), info["files"].clone()),
        (Value::Null, Value::Null, json!([]))
    );
    assert_eq!(info["libraryVersion"], FIXTURE_VERSION);
    let keys: Vec<&String> = info.as_object().unwrap().keys().collect();
    assert_eq!(
        keys,
        [
            "cli",
            "libraryVersion",
            "librarySource",
            "platform",
            "framework",
            "packageManager",
            "sourceDirectory",
            "config",
            "files",
            "localIcons",
            "local",
            "auth",
            "renderer"
        ]
    );
    assert_eq!(info["auth"]["loggedIn"], false);

    web_app(app);
    let doctor = run(app, &["doctor", "--offline"]);
    assert_eq!(doctor.code, 1, "{doctor:?}");
    let report = doctor.json();
    assert_eq!(report["status"], "FAIL");
    assert_eq!(
        report["checks"][2],
        json!({"id": "project", "status": "fail", "message": "A web app without generative-icons.json", "hint": "Run generative-icons init"})
    );

    run(app, &["init"]).ok();
    let report = run(app, &["doctor", "--offline"]).ok();
    assert_eq!(report["status"], "PASS");
    let status: Vec<(&str, &str)> = report["checks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| (c["id"].as_str().unwrap(), c["status"].as_str().unwrap()))
        .collect();
    assert_eq!(
        status,
        [
            ("binary", "pass"),
            ("cpu", "pass"),
            ("project", "pass"),
            ("skills", "pass"),
            ("kit", "pass"),
            ("lock", "pass"),
            ("library", "skip"),
            ("quickjs", "pass"),
            ("renderer", "pass"),
            ("auth", "skip"),
            ("engine", "skip")
        ]
    );
    assert_eq!(report["checks"][9]["hint"], generative_icons::engine::LOGIN_HINT);
    // The library the commands would use: the hosted one is not cached here.
    let library_check = |report: &Value| report["checks"][6].clone();
    let hosted = library_check(&report);
    assert_eq!(
        hosted["message"],
        "Offline, and no catalog of https://generativeicons.com/library/v1/ is cached"
    );
    assert!(hosted["hint"].as_str().unwrap().contains("--library <dir>"), "{hosted}");
    let local = library_check(&run(app, &["doctor", "--offline", "--library", &library]).ok());
    assert_eq!(local["status"], "pass", "{local}");
    let expected =
        format!("{FIXTURE_VERSION} (4 icons) from {}", Path::new(&library).canonicalize().unwrap().display());
    assert_eq!(local["message"], expected);
    let missing = run(app, &["doctor", "--offline", "--library", "no-such-folder"]);
    assert_eq!(missing.code, 1, "{missing:?}");
    let missing = library_check(&missing.json());
    assert_eq!(missing["status"], "fail", "{missing}");
    assert!(missing["hint"].as_str().unwrap().contains("--library <dir>"), "{missing}");
    // The cpu check evaluates the built-in V8 vectors, not just the CPU features.
    let cpu = report["checks"][1]["message"].as_str().unwrap();
    assert!(cpu.ends_with(": icon math matches V8 (23 vectors)"), "{cpu}");

    // An edited kit copy fails doctor and blocks init until --update-kit.
    write(app, "generative-icons/library/drawing.mjs", "// edited\n");
    let doctor = run(app, &["doctor", "--offline"]);
    assert_eq!(doctor.code, 1);
    let kit = doctor.json()["checks"][4].clone();
    assert_eq!(kit["status"], "fail");
    assert_eq!(kit["hint"], "Move your changes into your own files, then run generative-icons init --update-kit");
    run(app, &["init"]).fails(3, "MODIFIED_FILE");
    assert_eq!(run(app, &["init", "--update-kit"]).ok()["changed"], json!(["generative-icons/library/drawing.mjs"]));
    assert_eq!(run(app, &["doctor", "--offline"]).code, 0);

    let info = run(app, &["info", "--library", &library]).ok();
    assert_eq!(info["platform"], "web");
    assert_eq!(info["config"]["publicPath"], "/generative-icons");
    assert!(info["files"].as_array().unwrap().iter().all(|s| s["status"] == "unchanged"));
    assert_eq!((info["localIcons"].clone(), info["local"]["icons"].clone()), (json!([]), json!([])));

    // Icons created but not built yet are work in progress a new session must find.
    run(app, &["create", "watering-can", "--batch", "--library", &library]).ok();
    let info = run(app, &["info", "--library", &library]).ok();
    assert_eq!(info["localIcons"], json!(["watering-can"]));
    assert_eq!(
        info["local"],
        json!({"source": "generative-icons/icons", "icons": [{"id": "watering-can", "spec": null,
            "batch": "generative-icons/icons/batches/watering-can.mjs", "build": "not-built", "installed": false}]})
    );
    let text = run(app, &["info", "--library", &library, "--format", "text"]).stdout;
    assert!(text.contains("Local icons: watering-can (not built)\n"), "{text}");
}

#[test]
fn text_mode_and_usage_errors() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path();
    let error = run(app, &["frobnicate"]).fails(2, "UNKNOWN_COMMAND");
    assert_eq!(error["hint"], "Run generative-icons --help");
    run(app, &["search", "--cwd"]).fails(2, "MISSING_VALUE");
    run(app, &["search", "bell", "--limit", "0"]).fails(2, "USAGE");
    run(app, &["search"]).fails(2, "USAGE");

    let text = run(app, &["add", "heart", "--format", "text"]);
    assert_eq!(text.code, 3);
    assert!(text.stdout.is_empty());
    assert!(text.stderr.starts_with("error: No generative-icons.json in "), "{}", text.stderr);
    assert!(text.stderr.contains("\nhint: Run generative-icons init"), "{}", text.stderr);

    web_app(app);
    let text = run(app, &["init", "--format", "text"]);
    assert_eq!(text.code, 0);
    assert!(
        text.stdout.starts_with("generative-icons is set up for this web app (react, npm).\n"),
        "{}",
        text.stdout
    );

    let help = run(app, &["--help"]);
    assert_eq!(help.code, 0);
    assert!(help.stdout.contains("search"));
}
