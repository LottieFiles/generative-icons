//! The project authoring flow end to end: `init`, `create --from` a library icon, then
//! `build` through an exporter that replays that icon's committed natives, so the
//! whole local pipeline (sampling, scenes, post-export, packaging, verification,
//! previews, install) runs offline and must reproduce the library's `.lottie`.
//! Needs this checkout's gitignored natives of set 46 and its built packages.

mod common;

use common::*;
use gai_core::export::{Exporter, NativeExport};
use gai_core::sha256_hex;
use gai_engine::export::EXPORT_SCRIPT;
use gai_package::zip::{ZipDate, ZipEntry, read_zip, write_zip};
use generative_icons::Context;
use generative_icons::cli::BuildArgs;
use generative_icons::commands::build::build_with;
use generative_icons::output::{Format, Output};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The library icon the local icon starts from, and its set.
const REFERENCE: &str = "folder-open";
const SET: &str = "46-files-folders";
const LOCAL: &str = "my-folder";

/// Answers each scene with the committed native of the reference icon's action.
struct Replay {
    natives: PathBuf,
    calls: AtomicUsize,
}

impl Exporter for Replay {
    fn export(&self, name: &str, scene_json: &str) -> anyhow::Result<NativeExport> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let source = name.replacen(LOCAL, REFERENCE, 1);
        let text = std::fs::read_to_string(self.natives.join(format!("{source}.json")))?;
        let provenance = json!({
            "engine": "replay",
            "server": {"name": "replay", "version": "1.0.0"},
            "scriptSha256": sha256_hex(EXPORT_SCRIPT),
            "sceneSha256": sha256_hex(scene_json),
            "sha256": sha256_hex(&text),
            "bytes": text.len(),
        });
        Ok(NativeExport { text, provenance })
    }
}

/// The library's Strong archive of the reference icon with its actions renamed, as the
/// local build must write it.
fn renamed_reference(repo: &Path) -> Vec<u8> {
    let bytes = std::fs::read(repo.join(format!("prototype/icons/46/assets/{REFERENCE}-strong.lottie"))).unwrap();
    let entries: Vec<ZipEntry> = read_zip(&bytes)
        .unwrap()
        .into_iter()
        .map(|file| {
            let name = file.name.replace(REFERENCE, LOCAL);
            let data = match file.name.as_str() {
                "manifest.json" => String::from_utf8(file.data).unwrap().replace(REFERENCE, LOCAL).into_bytes(),
                _ => file.data,
            };
            ZipEntry::deflate(name, &data, ZipDate::SETS)
        })
        .collect();
    write_zip(&entries).unwrap()
}

fn ctx(app: &Path, repo: &Path) -> Context {
    let mut ctx = Context::new(app.canonicalize().unwrap(), Output::new(Format::Json, true, false, false));
    ctx.library_root = Some(repo.to_path_buf());
    ctx
}

#[test]
fn a_local_icon_builds_to_the_library_bytes() {
    let repo = repo_root();
    let natives = repo.join(format!("motion/icon-sets/{SET}/native"));
    require!(natives.join(format!("{REFERENCE}-strong.json")).is_file().then_some(()), "set 46 natives");
    require!(packages_dist(), "packages/icons/dist");
    let repo_arg = repo.display().to_string();
    let dir = tempfile::tempdir().unwrap();
    let app = &dir.path().join("app");
    std::fs::create_dir(app).unwrap();
    web_app(app);
    run(app, &["init"]).ok();

    let created =
        run(app, &["create", LOCAL, "--batch", "--from", REFERENCE, "--label", "Folder open", "--library-root", &repo_arg]).ok();
    assert_eq!(created["from"], json!({"id": REFERENCE, "set": SET}));
    assert_eq!(created["context"], format!(".generative-icons/context/{LOCAL}.json"));
    assert_eq!(created["references"], json!([REFERENCE]));
    assert_eq!(
        read(app, "generative-icons/icons/definitions.mjs"),
        format!("import {{ icons as b0 }} from './batches/{LOCAL}.mjs';\nexport const icons = [...b0];\n")
    );
    let packet: Value = serde_json::from_str(&read(app, &format!(".generative-icons/context/{LOCAL}.json"))).unwrap();
    assert_eq!(packet["references"][0]["id"], REFERENCE);
    // The copied parts (and any constraints) take the new id, as the engine will name them.
    let batch_path = format!("generative-icons/icons/batches/{LOCAL}.mjs");
    let batch = read(app, &batch_path);
    assert!(batch.contains("const rename = name => typeof name === 'string' && name.startsWith(\"folder-open-\") ? \"my-folder-\" + name.slice(12) : name;"), "{batch}");
    // A helper in a subfolder is part of the source like any batch.
    let helper = "generative-icons/icons/batches/shared/nudge.mjs";
    write(app, helper, "export const nudge = 0;\n");
    write(app, &batch_path, format!("import {{nudge}} from './shared/nudge.mjs';\n{batch}"));
    // A second create of the same id is refused; a library id collides.
    run(app, &["create", LOCAL, "--batch", "--library-root", &repo_arg]).fails(3, "EXISTS");
    run(app, &["create", REFERENCE, "--batch", "--library-root", &repo_arg]).fails(3, "ID_COLLISION");

    // The library set's identity, so composition names and meta.set match its assets.
    write(
        app,
        "generative-icons/icons/set-config.json",
        format!(
            "{{\"id\": \"{SET}\", \"name\": \"Files & folders\", \"sharedSources\": [\"drawing.mjs\", \"direction.mjs\"]}}\n"
        ),
    );
    let check = run(app, &["check", "--source", "--library-root", &repo_arg]).ok();
    assert_eq!(check["checks"][0]["status"], "PASS", "{check}");

    let replay = Replay { natives, calls: AtomicUsize::new(0) };
    let args = BuildArgs { ids: vec![LOCAL.into()], all: false, no_install: false, jobs: 2, emit_call: None, from_bundle: None };
    let report = build_with(&ctx(app, &repo), &args, &replay, None).unwrap();
    assert!(!report.failed, "{}", report.text);
    assert_eq!(replay.calls.load(Ordering::SeqCst), 4);
    let icon = &report.value["icons"][0];
    assert_eq!(icon["checks"], "PASS");
    let actions: Vec<&str> = icon["actions"].as_array().unwrap().iter().map(|a| a["name"].as_str().unwrap()).collect();
    assert_eq!(actions, ["my-folder", "my-folder-reverse", "my-folder-strong", "my-folder-reverse-strong"]);

    // The installed asset is the library's Strong archive, action names aside.
    let installed = std::fs::read(app.join(format!("public/generative-icons/{LOCAL}.lottie"))).unwrap();
    assert_eq!(sha256_hex(&installed), sha256_hex(renamed_reference(&repo)));
    assert_eq!(icon["asset"]["sha256"], sha256_hex(&installed));
    let snapshot: Value =
        serde_json::from_str(&read(app, &format!(".generative-icons/build/{LOCAL}/sources.snapshot.json"))).unwrap();
    let listed: Vec<&String> = snapshot["files"].as_object().unwrap().keys().collect();
    assert!(
        listed.iter().any(|f| *f == "batches/shared/nudge.mjs") && listed.iter().any(|f| *f == "definitions.mjs"),
        "{snapshot}"
    );
    assert_eq!(snapshot["files"]["batches/shared/nudge.mjs"], sha(app, helper));
    assert_eq!(
        read(app, &format!(".generative-icons/build/{LOCAL}/batches/shared/nudge.mjs")),
        "export const nudge = 0;\n"
    );
    let lock: Value = serde_json::from_str(&read(app, ".generative-icons/lock.json")).unwrap();
    let record = &lock["icons"][LOCAL];
    assert_eq!(
        record["build"]["sourcesSha256"],
        sha(app, &format!(".generative-icons/build/{LOCAL}/sources.snapshot.json"))
    );
    let keys: Vec<&String> = record.as_object().unwrap().keys().collect();
    assert_eq!(keys, ["version", "origin", "sha256", "source", "asset", "build"]);
    assert_eq!((record["version"].as_str(), record["origin"].as_str()), (Some("local"), Some("local")));
    assert_eq!(record["build"]["engine"], "replay@1.0.0");
    assert_eq!(record["build"]["actions"].as_object().unwrap().len(), 4);
    let module = read(app, &format!("src/generative-icons/{LOCAL}.mjs"));
    assert!(
        module.contains("\"license\": \"UNLICENSED\"") && module.contains("\"category\": \"Files & folders\""),
        "{module}"
    );
    // The reference's own states (closed, open) are not shipped under the new id.
    assert!(module.contains("\"startState\": \"prepared\"") && module.contains("\"endState\": \"result\""), "{module}");

    // The preview sheets exist and decode.
    let sheets = icon["preview"].as_array().unwrap();
    assert_eq!(sheets.len(), 2);
    for sheet in sheets {
        let file = std::fs::File::open(app.join(sheet["path"].as_str().unwrap())).unwrap();
        let mut reader = png::Decoder::new(std::io::BufReader::new(file)).read_info().unwrap();
        let mut pixels = vec![0; reader.output_buffer_size()];
        let frame = reader.next_frame(&mut pixels).unwrap();
        assert_eq!(
            (frame.width, frame.height),
            (sheet["width"].as_u64().unwrap() as u32, sheet["height"].as_u64().unwrap() as u32)
        );
        assert!(pixels.iter().any(|&b| b != 0));
    }

    // The built icon passes every check, previews from its build, and can be
    // re-added, searched and bundled deterministically.
    let check = run(app, &["check", "--library-root", &repo_arg]).ok();
    assert_eq!(check["status"], "PASS", "{check}");
    let statuses: Vec<&str> =
        check["checks"].as_array().unwrap().iter().map(|c| c["status"].as_str().unwrap()).collect();
    assert_eq!(statuses, ["PASS", "PASS", "PASS", "PASS"]);
    let preview = run(app, &["preview", LOCAL, "--motion", "strong", "--frames", "3"]).ok();
    assert_eq!((preview["source"].as_str(), preview["files"].as_array().unwrap().len()), (Some("build"), 1));
    // That custom sheet replaced build's Strong review sheet; reverse sheets and a stray
    // PNG sit beside it. None of them is what submit bundles.
    let previews = format!(".generative-icons/previews/{LOCAL}");
    let review: Vec<(String, String)> = sheets
        .iter()
        .map(|sheet| (sheet["path"].as_str().unwrap().to_string(), sheet["sha256"].as_str().unwrap().to_string()))
        .collect();
    assert_eq!(
        review.iter().map(|(path, _)| path.as_str()).collect::<Vec<_>>(),
        [format!("{previews}/{LOCAL}.png"), format!("{previews}/{LOCAL}-strong.png")]
    );
    assert_ne!(sha(app, &review[1].0), review[1].1);
    run(app, &["preview", LOCAL, "--motion", "reverse"]).ok();
    write(app, &format!("{previews}/old-sheet.png"), b"not a sheet of this build");
    std::fs::remove_file(app.join(&review[0].0)).unwrap();
    // The columns run from the start to the rest frame of Light or Strong, whichever ends later.
    let built: Value =
        serde_json::from_str(&read(app, &format!(".generative-icons/build/{LOCAL}/set.json"))).unwrap();
    let end = built["icons"][0]["motions"].as_array().unwrap().iter().fold(0.0, |end: f64, m| {
        end.max(m["active"].as_f64().unwrap()).max(m["rest"].as_f64().unwrap() / m["fps"].as_f64().unwrap())
    });
    let round = |t: f64| (t * 1e5).round() / 1e5;
    assert_eq!(preview["files"][0]["times"], json!([0, round(end / 2.0), round(end)]));
    let search = run(app, &["search", "folder", "--local-only"]).ok();
    assert_eq!(search[0]["id"], LOCAL);
    let added = run(app, &["add", LOCAL, "--library-root", &repo_arg]).ok();
    assert!(
        !added["changed"].as_array().unwrap().iter().any(|path| path.as_str().unwrap().ends_with(".lottie")),
        "{added}"
    );
    assert_eq!(std::fs::read(app.join(format!("public/generative-icons/{LOCAL}.lottie"))).unwrap(), installed);
    // Editing the helper makes the build stale, and a stale build is not submitted.
    write(app, helper, "export const nudge = 1;\n");
    let check = run(app, &["check", "--native"]).ok();
    // Stale leads the summary, never PASS, and names what to rebuild; the exit stays 0.
    assert_eq!(
        (check["status"].as_str(), check["checks"][0]["status"].as_str(), &check["stale"]),
        (Some("STALE"), Some("STALE"), &json!([LOCAL])),
        "{check}"
    );
    let text = run(app, &["check", "--library-root", &repo_arg, "--format", "text"]);
    assert_eq!(text.code, 0, "{text:?}");
    let lines: Vec<&str> = text.stdout.lines().collect();
    assert_eq!(
        lines[0],
        "STALE: my-folder changed since it was built; rebuild it (generative-icons build my-folder) before previewing or submitting. Source and contract checks; not a visual review"
    );
    assert!(lines.contains(&"  STALE  native"), "{}", text.stdout);
    let stale = &check["checks"][0]["diagnostics"][0];
    assert_eq!(
        (stale["code"].as_str(), stale["message"].as_str()),
        (
            Some("STALE_BUILD"),
            Some(
                "The source changed since my-folder was built (batches/shared/nudge.mjs); run generative-icons build my-folder"
            )
        )
    );
    let refused = run(app, &["submit", LOCAL, "--dry-run", "--output", "stale.zip", "--library-root", &repo_arg]);
    let error = refused.fails(1, "CHECK_FAILED");
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("my-folder: the source changed since it was built (batches/shared/nudge.mjs)"),
        "{error}"
    );
    write(app, helper, "export const nudge = 0;\n");
    let one = run(app, &["submit", LOCAL, "--dry-run", "--output", "one.zip", "--library-root", &repo_arg]).ok();
    let two = run(app, &["submit", LOCAL, "--dry-run", "--output", "two.zip", "--library-root", &repo_arg]).ok();
    assert_eq!(one["sha256"], two["sha256"]);
    run(app, &["submit", LOCAL, "--dry-run", "--output", "one.zip", "--library-root", &repo_arg]).fails(3, "EXISTS");
    let bundle = generative_icons::weight::read_zip(&std::fs::read(app.join("one.zip")).unwrap()).unwrap();
    let names: Vec<&str> = bundle.iter().map(|e| e.name.as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    for name in [
        "checks.json",
        "submission.json",
        "built/my-folder/my-folder-strong.lottie",
        "previews/my-folder/my-folder.png",
        "source/generative-icons/icons/batches/my-folder.mjs",
        "source/generative-icons/icons/batches/shared/nudge.mjs",
        "built/my-folder/sources.snapshot.json",
    ] {
        assert!(names.contains(&name), "{name} in {names:?}");
    }
    let submission: Value =
        serde_json::from_slice(&bundle.iter().find(|e| e.name == "submission.json").unwrap().data).unwrap();
    assert_eq!(
        (submission["license"].as_str(), submission["kitSha256"].as_str()),
        (Some("MIT"), Some(generative_icons::kit::sha256().as_str()))
    );
    // Only the build's review sheets, as build rendered them: the missing one and the
    // custom one are rendered again from the verified build, here and in the bundle.
    let bundled: Vec<(String, String)> = bundle
        .iter()
        .filter(|e| e.name.starts_with("previews/"))
        .map(|e| (format!(".generative-icons/{}", e.name), sha256_hex(&e.data)))
        .collect();
    let mut expected_sheets = review.clone();
    expected_sheets.sort();
    assert_eq!(bundled, expected_sheets);
    for (path, digest) in &review {
        assert_eq!(&sha(app, path), digest, "{path}");
        assert_eq!(submission["files"][path.strip_prefix(".generative-icons/").unwrap()], *digest);
    }
    assert_eq!(one["regenerated"], json!([review[0].0, review[1].0]), "{one}");
    assert_eq!(two["regenerated"], json!([]), "{two}");
    // The source is the build's snapshot of the files the icon depends on, with a
    // definitions.mjs that imports only the submitted icon.
    let snapshot: Value =
        serde_json::from_str(&read(app, &format!(".generative-icons/build/{LOCAL}/sources.snapshot.json"))).unwrap();
    let depends: Vec<&str> = snapshot["depends"].as_array().unwrap().iter().map(|d| d.as_str().unwrap()).collect();
    let sources = |bundle: &[generative_icons::weight::Entry]| -> Vec<String> {
        let names = bundle.iter().map(|e| e.name.as_str());
        names.filter_map(|name| name.strip_prefix("source/generative-icons/icons/")).map(str::to_string).collect()
    };
    let mut expected: Vec<String> =
        depends.iter().map(|d| d.to_string()).chain(["evidence/design-review.md".to_string()]).collect();
    expected.sort();
    assert!(!depends.contains(&"batches/zz-other.mjs") && depends.contains(&"batches/shared/nudge.mjs"), "{depends:?}");
    assert_eq!(sources(&bundle), expected);

    // Creating another icon rewrites definitions.mjs and adds a batch; this build stays current.
    run(app, &["create", "zz-other", "--batch", "--library-root", &repo_arg]).ok();
    let check = run(app, &["check", "--native"]).ok();
    assert_eq!(check["checks"][0]["diagnostics"], json!([]), "{check}");
    assert_eq!((check["status"].as_str(), check.get("stale")), (Some("PASS"), None), "{check}");

    // Neither that unbuilt icon, an unrelated batch, a later edit to another file nor a
    // symlinked batch reaches the bundle: only what the build verified.
    write(app, "generative-icons/icons/batches/_scratch.mjs", "export const scratch = 'private work';\n");
    let secret = dir.path().join("secret.txt");
    std::fs::write(&secret, "secret-value-ABC123").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&secret, app.join("generative-icons/icons/batches/_leak.mjs")).unwrap();
    run(app, &["submit", LOCAL, "--dry-run", "--output", "three.zip", "--library-root", &repo_arg]).ok();
    let bundle = generative_icons::weight::read_zip(&std::fs::read(app.join("three.zip")).unwrap()).unwrap();
    assert_eq!(sources(&bundle), expected);
    let definitions = bundle.iter().find(|e| e.name == "source/generative-icons/icons/definitions.mjs").unwrap();
    assert_eq!(
        String::from_utf8_lossy(&definitions.data),
        format!("import {{ icons as b0 }} from './batches/{LOCAL}.mjs';\nexport const icons = [...b0];\n")
    );
    assert!(!bundle.iter().any(|e| String::from_utf8_lossy(&e.data).contains("secret-value")));
    assert!(!bundle.iter().any(|e| String::from_utf8_lossy(&e.data).contains("private work")));

    // A file the bundle would hold is never read through a symlink, in the build or the source.
    #[cfg(unix)]
    {
        let copy = dir.path().join("nudge-copy.mjs");
        let built_helper = app.join(format!(".generative-icons/build/{LOCAL}/batches/shared/nudge.mjs"));
        std::fs::copy(&built_helper, &copy).unwrap();
        std::fs::remove_file(&built_helper).unwrap();
        std::os::unix::fs::symlink(&copy, &built_helper).unwrap();
        let refused = run(app, &["submit", LOCAL, "--dry-run", "--output", "four.zip", "--library-root", &repo_arg]);
        refused.fails(3, "SYMLINK_PATH");
        std::fs::remove_file(&built_helper).unwrap();
        std::fs::copy(&copy, &built_helper).unwrap();
        // A source the build depends on is never compared through a symlink, even one that
        // stays in the icons folder, so the build counts as stale until it is a file again.
        let live_helper = app.join(helper);
        std::fs::rename(&live_helper, live_helper.with_file_name("nudge-real.mjs")).unwrap();
        std::os::unix::fs::symlink("nudge-real.mjs", &live_helper).unwrap();
        let check = run(app, &["check", "--native"]).ok();
        let stale = &check["checks"][0]["diagnostics"][0];
        assert_eq!(
            (stale["code"].as_str(), stale["message"].as_str()),
            (
                Some("STALE_BUILD"),
                Some(
                    "The source changed since my-folder was built (batches/shared/nudge.mjs is or runs through a symlink, which is never followed; replace it with the file it points to); run generative-icons build my-folder"
                )
            )
        );
        let refused = run(app, &["submit", LOCAL, "--dry-run", "--output", "four.zip", "--library-root", &repo_arg]);
        let error = refused.fails(3, "SYMLINK_PATH");
        assert_eq!(error["path"], helper, "{error}");
        assert!(!app.join("four.zip").exists());
        std::fs::remove_file(&live_helper).unwrap();
        std::fs::rename(live_helper.with_file_name("nudge-real.mjs"), &live_helper).unwrap();
    }

    // An import the scan cannot trace makes the icon depend on every module its build
    // loaded, another icon's batch included: a real submit refuses before sending anything,
    // and --include-untraced bundles them all.
    write(app, &batch_path, format!("{}const lazy = name => import(name);\n", read(app, &batch_path)));
    let report = build_with(&ctx(app, &repo), &args, &replay, None).unwrap();
    assert!(!report.failed, "{}", report.text);
    // check and build say so before the export.
    let untraced = report.warnings.iter().find(|w| w.contains("The import `import(name)` cannot be traced"));
    assert!(untraced.is_some_and(|w| w.starts_with(&format!("{batch_path}:"))), "{:?}", report.warnings);
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let endpoint = format!("http://{}/api/submissions", listener.local_addr().unwrap());
    let submit = ["submit", LOCAL, "--endpoint", &endpoint, "--library-root", &repo_arg];
    let refused = run_with(app, &submit, &[("GAI_ENGINE_TOKEN", "token")]);
    let error = refused.fails(1, "CHECK_FAILED");
    let message = error["message"].as_str().unwrap();
    assert!(message.contains("could not be traced") && message.ends_with("batches/zz-other.mjs"), "{error}");
    assert!(listener.accept().is_err(), "the submission was sent");
    let untraced =
        ["submit", LOCAL, "--dry-run", "--include-untraced", "--output", "five.zip", "--library-root", &repo_arg];
    let included = run(app, &untraced);
    included.ok();
    assert!(included.stderr.contains("its imports could not be traced"), "{included:?}");
    let bundle = generative_icons::weight::read_zip(&std::fs::read(app.join("five.zip")).unwrap()).unwrap();
    assert!(sources(&bundle).contains(&"batches/zz-other.mjs".to_string()));
}

#[test]
fn builds_need_a_session_and_a_passing_source() {
    let dir = tempfile::tempdir().unwrap();
    let app = &dir.path().join("app");
    std::fs::create_dir(app).unwrap();
    web_app(app);
    let library = fixture_library(&dir.path().join("library")).display().to_string();
    run(app, &["init"]).ok();
    let created = run(app, &["create", "rocket-launch", "--batch", "--intent", "A rocket lifts off.", "--library", &library]).ok();
    assert_eq!(created["from"], Value::Null);
    let batch = read(app, "generative-icons/icons/batches/rocket-launch.mjs");
    assert!(
        batch.starts_with("// rocket-launch: Rocket launch\n") && batch.contains("export const icons = [\n"),
        "{batch}"
    );
    // The template draws nothing, so the source check fails on purpose.
    let check = run(app, &["check", "--source", "--library", &library]);
    assert_eq!(check.code, 1, "{check:?}");
    assert_eq!(check.json()["checks"][0]["diagnostics"][0]["code"], "EMPTY_SET");
    run(app, &["build", "--library", &library]).fails(4, "NOT_LOGGED_IN");

    // An edited kit copy stops every evaluation. A build refuses it before any export;
    // the token override stands in for a session and nothing reaches the network.
    write(app, "generative-icons/library/drawing.mjs", "export const edited = true;\n");
    let check = run(app, &["check", "--source", "--library", &library]);
    assert_eq!(check.json()["checks"][0]["diagnostics"][0]["code"], "MODIFIED_FILE");
    run(app, &["create", "second", "--batch", "--library", &library]).fails(3, "MODIFIED_FILE");
    let build = run_with(app, &["build", "--library", &library], &[("GAI_ENGINE_TOKEN", "token")]);
    let error = build.fails(3, "MODIFIED_FILE");
    assert_eq!(
        (error["path"].as_str(), error["hint"].as_str()),
        (
            Some("generative-icons/library/drawing.mjs"),
            Some("Move your changes into your own batch files, then run generative-icons init --update-kit")
        )
    );
}

/// Fails every export of `failing`, replays the rest.
struct FailOne {
    replay: Replay,
    failing: &'static str,
}

impl Exporter for FailOne {
    fn export(&self, name: &str, scene_json: &str) -> anyhow::Result<NativeExport> {
        if name.starts_with(self.failing) {
            anyhow::bail!("the engine is down");
        }
        self.replay.export(name, scene_json)
    }
}

#[test]
fn icons_built_before_a_failed_one_are_still_installed() {
    let repo = repo_root();
    let natives = repo.join(format!("motion/icon-sets/{SET}/native"));
    require!(natives.join(format!("{REFERENCE}-strong.json")).is_file().then_some(()), "set 46 natives");
    require!(packages_dist(), "packages/icons/dist");
    let repo_arg = repo.display().to_string();
    let dir = tempfile::tempdir().unwrap();
    let app = &dir.path().join("app");
    std::fs::create_dir(app).unwrap();
    web_app(app);
    run(app, &["init"]).ok();
    for id in [LOCAL, "zz-folder"] {
        run(app, &["create", id, "--batch", "--from", REFERENCE, "--library-root", &repo_arg]).ok();
    }
    write(
        app,
        "generative-icons/icons/set-config.json",
        format!(
            "{{\"id\": \"{SET}\", \"name\": \"Files & folders\", \"sharedSources\": [\"drawing.mjs\", \"direction.mjs\"]}}\n"
        ),
    );
    let exporter = FailOne { replay: Replay { natives, calls: AtomicUsize::new(0) }, failing: "zz-folder" };
    let args = BuildArgs { ids: vec![LOCAL.into(), "zz-folder".into()], all: false, no_install: false, jobs: 1, emit_call: None, from_bundle: None };
    let error = build_with(&ctx(app, &repo), &args, &exporter, None).unwrap_err();
    assert_eq!(
        (error.code, error.id.as_deref()),
        (generative_icons::error::ErrorCode::EngineError, Some("zz-folder"))
    );
    assert!(error.message.ends_with(" (my-folder built and installed before it)"), "{}", error.message);
    assert!(app.join(format!("public/generative-icons/{LOCAL}.lottie")).is_file());
    assert!(!app.join("public/generative-icons/zz-folder.lottie").exists());
    let lock: Value = serde_json::from_str(&read(app, ".generative-icons/lock.json")).unwrap();
    assert!(lock["icons"].get(LOCAL).is_some() && lock["icons"].get("zz-folder").is_none(), "{lock}");
}

#[test]
fn create_saves_the_references_chosen_with_context() {
    let dir = tempfile::tempdir().unwrap();
    let app = &dir.path().join("app");
    std::fs::create_dir(app).unwrap();
    web_app(app);
    let library = fixture_library(&dir.path().join("library")).display().to_string();
    run(app, &["init"]).ok();
    let created =
        run(app, &["create", "alarm-nudge", "--batch", "--neighbor", "menu", "--neighbor", "heart", "--library", &library]).ok();
    assert_eq!(created["context"], ".generative-icons/context/alarm-nudge.json");
    let packet: Value = serde_json::from_str(&read(app, ".generative-icons/context/alarm-nudge.json")).unwrap();
    let references: Vec<&str> =
        packet["references"].as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap()).collect();
    assert_eq!(references, ["menu", "heart"]);
    // The output names the references chosen, not a lexical list that ignores them.
    assert_eq!(created["references"], json!(["menu", "heart"]));
    assert!(created.get("nearest").is_none(), "{created}");
    let lexical = run(app, &["create", "bell-swing", "--batch", "--intent", "The bell swings.", "--library", &library]).ok();
    let packet: Value = serde_json::from_str(&read(app, ".generative-icons/context/bell-swing.json")).unwrap();
    let embedded: Vec<&Value> = packet["references"].as_array().unwrap().iter().map(|r| &r["id"]).collect();
    assert_eq!(lexical["references"], json!(embedded));
    assert_eq!(lexical["nearest"][0]["id"], packet["candidates"][0]["id"], "{lexical}");
    // An unknown reference stops create before anything is written.
    let error = run(app, &["create", "other", "--batch", "--neighbor", "no-such", "--library", &library]).fails(3, "UNKNOWN_ICON");
    assert!(error["hint"].as_str().unwrap().contains("--neighbor"), "{error}");
    assert!(!app.join("generative-icons/icons/batches/other.mjs").exists());
}

#[test]
fn create_refuses_ids_submit_could_never_bundle() {
    let dir = tempfile::tempdir().unwrap();
    let app = &dir.path().join("app");
    std::fs::create_dir(app).unwrap();
    web_app(app);
    let library = fixture_library(&dir.path().join("library")).display().to_string();
    run(app, &["init"]).ok();
    // Kebab-case, but Windows device names: submit refuses their files on every platform.
    for id in ["con", "aux", "nul", "prn", "com1", "lpt9"] {
        let error = run(app, &["create", id, "--batch", "--library", &library]).fails(2, "USAGE");
        assert_eq!(
            error["message"],
            format!(
                "{id} cannot be an icon id: Windows reserves it as a device name (con, prn, aux, nul, com1 to com9, lpt1 to lpt9), so submit could never bundle its files"
            )
        );
        assert_eq!(error["hint"], format!("Choose another id, such as {id}-icon"));
        assert!(!app.join(format!("generative-icons/icons/batches/{id}.mjs")).exists());
    }
    assert!(!app.join("generative-icons/icons/definitions.mjs").exists());
    run(app, &["create", "con-icon", "--batch", "--library", &library]).ok();
}
