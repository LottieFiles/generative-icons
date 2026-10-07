//! `check` around the authoring loop, offline with a fixture library: before and after a
//! build, scaffolds that export nothing, source errors located in the file that raised
//! them, constraint part names, modules outside the icon source, library ids taken from
//! the cached catalog, and verification failures found before anything is exported.

mod common;

use common::*;
use gai_core::export::{Exporter, NativeExport};
use generative_icons::Context;
use generative_icons::cli::BuildArgs;
use generative_icons::commands::build::build_with;
use generative_icons::output::{Format, Output};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// A small icon that passes the source preflight; `ID` is replaced by the icon id.
const ICON: &str = "import {make, F, framed, line, rigid, rect} from '../../library/ai-authoring.mjs';

const bob = F([[0, 0], [.3, -2, 'out'], [.6, 0]],
              [[0, 0], [.08, .6], [.28, -3.5, 'hit'], [.5, 0]]);
export const icons = [
  make('ID', 'Bell nudge', 'The bell lifts once and settles.',
    [...framed('bell', rect(9, 8, 14, 13, 5)),
     ...line('clapper', [16, 23], [16, 25], {pose: rigid((t, s) => ({dy: bob(t, s)}))})],
    .6, .5)
];
";

const BATCH: &str = "generative-icons/icons/batches/bell-nudge.mjs";

/// An initialized web app with `bell-nudge` created from the template.
fn project(dir: &Path) -> (PathBuf, String) {
    let app = dir.join("app");
    std::fs::create_dir(&app).unwrap();
    web_app(&app);
    let library = fixture_library(&dir.join("library")).display().to_string();
    run(&app, &["init"]).ok();
    run(&app, &["create", "bell-nudge", "--batch", "--library", &library]).ok();
    (app, library)
}

fn check(app: &Path, args: &[&str]) -> (i32, Value) {
    let result = run(app, &[&["check"], args].concat());
    (result.code, result.json())
}

fn check_of<'a>(report: &'a Value, id: &str) -> &'a Value {
    report["checks"].as_array().unwrap().iter().find(|check| check["id"] == id).unwrap()
}

fn codes(check: &Value) -> Vec<&str> {
    check["diagnostics"].as_array().unwrap().iter().map(|d| d["code"].as_str().unwrap()).collect()
}

#[test]
fn check_works_before_a_build_and_names_what_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let (app, library) = project(dir.path());

    // The template exports nothing: check points at the batch, never back at create.
    let (code, report) = check(&app, &["bell-nudge", "--library", &library]);
    assert_eq!(code, 1, "{report}");
    let source = check_of(&report, "source");
    let finding = source["diagnostics"].as_array().unwrap().iter().find(|d| d["code"] == "ICON_NOT_EXPORTED").unwrap();
    assert_eq!(finding["path"], BATCH);
    assert_eq!(
        finding["message"],
        format!(
            "{BATCH} exports no icon named bell-nudge; draw its parts with make('bell-nudge', …) in its icons array"
        )
    );
    // An id without a batch is still unknown.
    run(&app, &["check", "no-such-icon", "--library", &library]).fails(3, "UNKNOWN_ICON");

    // Drawn but not built: the source passes and the native check waits for a build.
    write(&app, BATCH, ICON.replace("ID", "bell-nudge"));
    let (code, report) = check(&app, &["bell-nudge", "--library", &library]);
    assert_eq!((code, report["status"].as_str()), (0, Some("PASS")), "{report}");
    let native = check_of(&report, "native");
    assert_eq!(native["status"], "SKIP");
    assert_eq!(
        native["diagnostics"][0]["message"],
        "bell-nudge is not built yet; generative-icons build bell-nudge builds and verifies it"
    );
    // Asking for the native check by name requires the build.
    let (code, report) = check(&app, &["bell-nudge", "--native"]);
    assert_eq!(code, 1);
    assert_eq!(codes(check_of(&report, "native")), ["NOT_BUILT"]);

    // A batch definitions.mjs no longer imports is named with the file to fix.
    write(&app, "generative-icons/icons/definitions.mjs", "export const icons = [];\n");
    let (code, report) = check(&app, &["bell-nudge", "--source", "--library", &library]);
    assert_eq!(code, 1);
    let finding = &check_of(&report, "source")["diagnostics"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(
        (finding["code"].as_str(), finding["path"].as_str()),
        (Some("ICON_NOT_EXPORTED"), Some("generative-icons/icons/definitions.mjs"))
    );
    assert!(
        finding["message"]
            .as_str()
            .unwrap()
            .contains("does not import generative-icons/icons/batches/bell-nudge.mjs"),
        "{finding}"
    );
}

#[test]
fn source_errors_name_the_file_and_line() {
    let dir = tempfile::tempdir().unwrap();
    let (app, library) = project(dir.path());
    let check = |app: &Path| run(app, &["check", "--source", "--library", &library]);

    write(
        &app,
        BATCH,
        "import {make} from '../../library/ai-authoring.mjs';\n\nexport const icons = [\n  make('x', 'y', [;\n];\n",
    );
    let error = check(&app).fails(1, "SOURCE_ERROR");
    // A syntax error has the right line; QuickJS columns are not reliable, so none is given.
    assert_eq!(error["message"], format!("{BATCH}:4: unexpected token in expression: ';'"));
    assert_eq!(error["path"], BATCH);

    // The frame in the project's own helper, not the importer.
    let helper = "generative-icons/icons/batches/shared/helper.mjs";
    write(&app, helper, "export function part() {\n  return null.x;\n}\n");
    write(&app, BATCH, "import {part} from './shared/helper.mjs';\nexport const icons = [part()];\n");
    let error = check(&app).fails(1, "SOURCE_ERROR");
    assert_eq!(error["path"], helper);
    // QuickJS places the throw at the function's start (line 1); it is on line 2.
    assert_eq!(error["message"], format!("{helper}: cannot read property 'x' of null (at or after line 1)"));

    // A thrown error is located at or before the real line (here the array that holds the call).
    write(
        &app,
        BATCH,
        "import {make, rect} from '../../library/ai-authoring.mjs';\nexport const icons = [\n  make('bell-nudge', 'B', 'i',\n    [rect(1, 2, 3, 4, 5),\n     tubez(1)],\n  .5, .4)];\n",
    );
    let error = check(&app).fails(1, "SOURCE_ERROR");
    assert_eq!(error["message"], format!("{BATCH}: tubez is not defined (at or after line 4)"));

    // A refused import names the importer, with project-relative paths only.
    write(&app, BATCH, "import {x} from './nope.mjs';\nexport const icons = [];\n");
    let error = check(&app).fails(1, "SOURCE_ERROR");
    assert_eq!(
        error["message"],
        format!(
            "generative-icons/icons: cannot import './nope.mjs' from '{BATCH}': generative-icons/icons/batches/nope.mjs does not exist"
        )
    );
    assert_eq!(error["path"], BATCH);
}

#[test]
fn constraint_parts_and_modules_outside_the_source_are_findings() {
    let dir = tempfile::tempdir().unwrap();
    let (app, library) = project(dir.path());
    // Constraints left on the parts' old names, as a copied icon would have them.
    let icon = ICON.replace("ID", "bell-nudge").replace(
        "    .6, .5)\n];",
        "    .6, .5)\n].map(icon => ({...icon, constraints: [{type: 'rigid', part: 'bell-nudge-bell-frame'}, {type: 'rigid', part: 'photo-delete-lid'}]}));",
    );
    write(&app, BATCH, icon);
    let (code, report) = check(&app, &["--source", "--library", &library]);
    assert_eq!(code, 1, "{report}");
    let source = check_of(&report, "source");
    let finding =
        source["diagnostics"].as_array().unwrap().iter().find(|d| d["code"] == "UNKNOWN_CONSTRAINT_PART").unwrap();
    assert_eq!(finding["icon"], "bell-nudge");
    assert_eq!(
        finding["message"],
        "constraints part photo-delete-lid is not a part of bell-nudge; its parts are bell-nudge-bell-interior, bell-nudge-bell-frame, bell-nudge-bell-solid, bell-nudge-clapper, bell-nudge-clapper-solid (rename constraints along with the parts)"
    );
    assert_eq!(finding["path"], BATCH);
    assert_eq!(codes(source).iter().filter(|c| **c == "UNKNOWN_CONSTRAINT_PART").count(), 1);

    // A module added to the kit folder is not snapshotted with the source, so it is refused.
    write(&app, "generative-icons/library/mine.mjs", "export const nudge = 1;\n");
    write(
        &app,
        BATCH,
        format!("import {{nudge}} from '../../library/mine.mjs';\n{}", ICON.replace("ID", "bell-nudge")),
    );
    let (code, report) = check(&app, &["--source", "--library", &library]);
    assert_eq!(code, 1);
    let finding = check_of(&report, "source")["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["code"] == "UNMANAGED_MODULE")
        .unwrap()
        .clone();
    assert_eq!(finding["path"], "generative-icons/library/mine.mjs");
}

#[test]
fn check_compares_library_ids_with_the_cached_catalog_only() {
    let dir = tempfile::tempdir().unwrap();
    let served = dir.path().join("served");
    fixture_library(&served);
    let url = serve(served.clone());
    let app = dir.path().join("app");
    std::fs::create_dir(&app).unwrap();
    web_app(&app);
    run(&app, &["init"]).ok();
    run(&app, &["create", "heart-pulse", "--batch", "--library", &url]).ok();
    // "heart" is a library id; create refuses it, so the batch is written by hand.
    write(&app, "generative-icons/icons/batches/heart-pulse.mjs", ICON.replace("ID", "heart"));

    // Nothing cached yet: check stays offline and says so.
    let fresh = run_with(
        &app,
        &["check", "--source", "--library", &url],
        &[("GEN_ANIMATED_ICONS_CACHE_DIR", dir.path().join("empty").to_str().unwrap())],
    );
    assert_eq!(fresh.code, 0, "{fresh:?}");
    assert!(fresh.stderr.contains("no library catalog is cached yet"), "{}", fresh.stderr);

    // create cached the catalog; with the served one gone, check still finds the clash.
    std::fs::remove_file(served.join("catalog.json")).unwrap();
    let (code, report) = check(&app, &["--source", "--library", &url]);
    assert_eq!(code, 1, "{report}");
    assert!(codes(check_of(&report, "source")).contains(&"CATALOG_ID_COLLISION"), "{report}");
}

/// An exporter that must not be reached.
struct Unreachable;

impl Exporter for Unreachable {
    fn export(&self, name: &str, _scene_json: &str) -> anyhow::Result<NativeExport> {
        panic!("{name} was exported although its source fails verification");
    }
}

#[test]
fn verification_failures_are_found_before_an_export() {
    let dir = tempfile::tempdir().unwrap();
    let (app, library) = project(dir.path());
    let hint = "hint: in reverse and reset actions, poses get r = true";

    // The clapper jumps again after rest, so set verification would fail after the export.
    let late = ICON
        .replace("ID", "bell-nudge")
        .replace("({dy: bob(t, s)})", "({dy: bob(t, s) + (t > 1 && t < 1.5 ? 1 : 0)})")
        .replace("    .6, .5)\n];", "    .6, .5)\n].map(icon => ({...icon, duotoneFill: true}));");
    write(&app, BATCH, &late);
    let (code, report) = check(&app, &["bell-nudge", "--library", &library]);
    assert_eq!(code, 1, "{report}");
    let found = check_of(&report, "source")["diagnostics"].clone();
    assert_eq!(
        found,
        serde_json::json!([
            {"severity": "error", "code": "MOVES_AFTER_REST", "icon": "bell-nudge", "message": format!("bell-nudge: moving after rest: bell-nudge-clapper-light must hold its final pose from rest (0.650 s) to the end, but is away from it at 1.017 s; {hint}"), "path": BATCH},
            {"severity": "error", "code": "MOVES_AFTER_REST", "icon": "bell-nudge", "message": format!("bell-nudge-strong: moving after rest: bell-nudge-clapper-light must hold its final pose from rest (0.550 s) to the end, but is away from it at 1.017 s; {hint}"), "path": BATCH}
        ]),
        "{report}"
    );
    // build stops at the same findings, before the engine is reached.
    let mut ctx = Context::new(app.canonicalize().unwrap(), Output::new(Format::Json, true, false, false));
    ctx.library = Some(library.clone());
    let args = BuildArgs { ids: vec!["bell-nudge".into()], all: false, no_install: false, jobs: 1, emit_call: None, from_bundle: None };
    let built = build_with(&ctx, &args, &Unreachable, None).unwrap();
    assert!(built.failed);
    assert_eq!(built.value["stage"], "source");
    assert_eq!(built.value["checks"][0]["diagnostics"], found);

    // A stateful icon whose Light and Strong keep different results.
    let apart = "import {make, P, framed, line, rigid, rect} from '../../library/ai-authoring.mjs';
const lift = P([[0, 0], [.6, -2]], [[0, 0], [.5, -3.5]]);
export const icons = [
  make('bell-nudge', 'Bell nudge', 'The clapper lifts and stays up.',
    [...framed('bell', rect(9, 8, 14, 13, 5)),
     ...line('clapper', [16, 23], [16, 25], {pose: rigid((t, s) => ({dy: lift(t, s)}))})],
    .6, .5, ['clapper'])
].map(icon => ({...icon, duotoneFill: true}));
";
    write(&app, BATCH, apart);
    let (code, report) = check(&app, &["--source", "--library", &library]);
    assert_eq!(code, 1, "{report}");
    let finding = &check_of(&report, "source")["diagnostics"][0];
    assert_eq!(finding["code"], "LIGHT_STRONG_DIFFER", "{report}");
    assert_eq!(
        finding["message"],
        "bell-nudge final drawing: bell-nudge-clapper-light differs between Light and Strong; hint: both motions must start and end on the same pose"
    );

    // The same lift for both passes.
    write(&app, BATCH, apart.replace("[.5, -3.5]", "[.5, -2]"));
    let (code, report) = check(&app, &["--source", "--library", &library]);
    assert_eq!((code, report["status"].as_str()), (0, Some("PASS")), "{report}");

    // Another icon's broken source does not hide this icon's findings.
    write(&app, BATCH, &late);
    run(&app, &["create", "bell-two", "--batch", "--library", &library]).ok();
    let two = ICON
        .replace("ID", "bell-two")
        .replace("    .6, .5)\n];", "    .6, .5)\n].map(icon => ({...icon, duotoneFill: false}));");
    write(&app, "generative-icons/icons/batches/bell-two.mjs", two);
    let (code, report) = check(&app, &["--source", "--library", &library]);
    assert_eq!(code, 1, "{report}");
    let source = check_of(&report, "source");
    let of = |icon: &str| -> Vec<&str> {
        let found = source["diagnostics"].as_array().unwrap().iter().filter(|d| d["icon"] == icon);
        found.map(|d| d["code"].as_str().unwrap()).collect()
    };
    assert_eq!(of("bell-nudge"), ["MOVES_AFTER_REST", "MOVES_AFTER_REST"], "{report}");
    assert_eq!(of("bell-two"), ["DUOTONE_POLICY"], "{report}");
}

#[test]
fn sampling_failures_and_paint_policy_say_where_and_how_to_fix() {
    let dir = tempfile::tempdir().unwrap();
    let (app, library) = project(dir.path());
    // A helper that throws once the drop starts falling, and a Duotone policy that
    // contradicts the tinted bell interior.
    let helper = "generative-icons/icons/batches/shared/drops.mjs";
    write(
        &app,
        helper,
        "export function drop(t) {\n  if (t > 0.3) {\n    return undefinedFn(t);\n  }\n  return 'M15 26 L17 26';\n}\n",
    );
    let icon = ICON
        .replace("ID", "bell-nudge")
        .replace("import {make,", "import {drop} from './shared/drops.mjs';\nimport {stroke, make,")
        .replace("     ...line('clapper'", "     stroke('drop', t => drop(t)),\n     ...line('clapper'")
        .replace("    .6, .5)\n];", "    .6, .5)\n].map(icon => ({...icon, duotoneFill: false}));");
    write(&app, BATCH, icon);
    let (code, report) = check(&app, &["--source", "--library", &library]);
    assert_eq!(code, 1, "{report}");
    let diagnostics = check_of(&report, "source")["diagnostics"].as_array().unwrap().clone();
    let policy = diagnostics.iter().find(|d| d["code"] == "DUOTONE_POLICY").unwrap();
    assert_eq!(
        policy["message"],
        "Declared Duotone policy disagrees with tint parts: bell-nudge-bell-interior has style tint, but duotoneFill is not true; hint: set duotoneFill: true when a tint part fills a real enclosed surface, else draw no tint part"
    );
    assert_eq!(policy["path"], BATCH);
    let thrown: Vec<&Value> = diagnostics.iter().filter(|d| d["code"] == "INVALID_GEOMETRY").collect();
    assert_eq!(thrown.len(), 6, "{report}");
    for finding in thrown {
        assert_eq!(finding["message"], "undefinedFn is not defined");
        // QuickJS places the throw at the start of its statement (the `if`), at or before it.
        assert_eq!((finding["path"].as_str(), finding["line"].as_u64()), (Some(helper), Some(2)), "{finding}");
    }
    let text = run(&app, &["check", "--source", "--library", &library, "--format", "text"]);
    assert!(text.stdout.contains(&format!("in {helper} (at or after line 2)")), "{}", text.stdout);

    // Without the throw, only the policy is left; declaring it fixes the icon.
    write(&app, helper, "export function drop(t) {\n  return 'M15 26 L17 26';\n}\n");
    let (_, report) = check(&app, &["--source", "--library", &library]);
    assert_eq!(codes(check_of(&report, "source")), ["DUOTONE_POLICY"], "{report}");
    let fixed = read(&app, BATCH).replace("duotoneFill: false", "duotoneFill: true");
    write(&app, BATCH, fixed);
    let (code, report) = check(&app, &["--source", "--library", &library]);
    assert_eq!((code, report["status"].as_str()), (0, Some("PASS")), "{report}");
}

/// What the CLI reports for the project's source stopped at a 200 ms limit.
fn stopped(app: &Path) -> generative_icons::error::CliError {
    use gai_js::{Limits, SetRuntime, SetSource, run_on_worker};
    use generative_icons::commands::check::{Origin, source_error_in};
    let root = app.canonicalize().unwrap();
    let icons = root.join("generative-icons/icons");
    let limits = Limits { call_timeout: std::time::Duration::from_millis(200), ..Limits::default() };
    let source = SetSource::user_project(&icons);
    let error = run_on_worker(&limits.clone(), move || {
        let mut runtime = SetRuntime::load(&source, &limits)?;
        let description = runtime.install()?;
        (0..description.icons[0].variants.len()).try_for_each(|v| runtime.sample_variant(0, v).map(drop))
    })
    .expect_err("the source runs too long");
    let origin = Origin { root: &root, set_dir: &icons, label: Some("generative-icons/icons") };
    source_error_in(error, Some(origin))
}

#[test]
fn code_stopped_at_its_time_limit_names_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let (app, _) = project(dir.path());
    // A loop in the module itself, as it is evaluated.
    write(&app, BATCH, "export const icons = [];\nlet n = 0;\nfor (;;) {\n  n++;\n}\n");
    let error = stopped(&app);
    assert_eq!((error.code.as_str(), error.path.as_deref()), ("SOURCE_ERROR", Some(BATCH)), "{error:?}");
    // QuickJS places module code at the module's first line.
    assert_eq!(error.message, format!("{BATCH}: stopped after 200ms (running at or after line 1)"));
    assert!(error.hint.unwrap().starts_with("The icon code ran too long"));

    // A loop in a part's callback, while an action is sampled.
    let spin = ICON.replace("ID", "bell-nudge").replace("({dy: bob(t, s)})", "({dy: (() => { for (;;) {} })()})");
    write(&app, BATCH, &spin);
    let error = stopped(&app);
    assert_eq!(error.path.as_deref(), Some(BATCH), "{error:?}");
    assert_eq!(
        error.message,
        format!("{BATCH}: stopped after 200ms (icon bell-nudge, variant bell-nudge) (running at or after line 8)")
    );
}

#[test]
fn untraced_imports_are_reported_before_a_build() {
    let dir = tempfile::tempdir().unwrap();
    let (app, library) = project(dir.path());
    let untraced = |report: &Value| -> Vec<Value> {
        let diagnostics = check_of(report, "source")["diagnostics"].as_array().unwrap().clone();
        diagnostics.into_iter().filter(|d| d["code"] == "UNTRACED_IMPORTS").collect()
    };
    let helper = "generative-icons/icons/batches/shared/nudge.mjs";
    write(&app, helper, "export const nudge = 0;\n");
    // An escape in a specifier names the module the loader reads, but the scan cannot follow it.
    let escaped = format!("import {{nudge}} from '.\\x2fshared/nudge.mjs';\n{}", ICON.replace("ID", "bell-nudge"));
    write(&app, BATCH, &escaped);
    let (code, report) = check(&app, &["bell-nudge", "--source", "--library", &library]);
    assert_eq!((code, report["status"].as_str()), (0, Some("PASS")), "{report}");
    assert_eq!(
        untraced(&report),
        [serde_json::json!({
            "severity": "warning",
            "code": "UNTRACED_IMPORTS",
            "icon": "bell-nudge",
            "message": "The import `import {nudge} from '.\\x2fshared/nudge.mjs'` cannot be traced (an import of a computed name, or an escape in a specifier), so a build of bell-nudge depends on every module the set loads, and submit refuses it; import each module by a plain string such as './shared/h.mjs'",
            "path": BATCH,
            "line": 1
        })],
        "{report}"
    );
    // A computed import in a helper the batch imports, named in that helper.
    write(&app, helper, "export const nudge = 0;\nexport const load = name => import(name);\n");
    write(&app, BATCH, format!("import {{nudge}} from './shared/nudge.mjs';\n{}", ICON.replace("ID", "bell-nudge")));
    let (code, report) = check(&app, &["--source", "--library", &library]);
    assert_eq!(code, 0, "{report}");
    let found = untraced(&report);
    assert_eq!(found.len(), 1, "{report}");
    assert_eq!((found[0]["path"].as_str(), found[0]["line"].as_u64()), (Some(helper), Some(2)), "{report}");
    assert!(found[0]["message"].as_str().unwrap().starts_with("The import `import(name)` cannot be traced"));
    let text = run(&app, &["check", "--source", "--library", &library, "--format", "text"]);
    let line = "warning UNTRACED_IMPORTS bell-nudge: The import `import(name)` cannot be traced";
    assert!(text.stdout.contains(line), "{}", text.stdout);
    // Plain specifiers are traced; nothing is reported.
    write(&app, helper, "export const nudge = 0;\n");
    let (_, report) = check(&app, &["--source", "--library", &library]);
    assert!(untraced(&report).is_empty(), "{report}");
}
