//! Parity with the committed browser evidence (`motion/icon-sets/*/evidence`).
//!
//! Oracle tests skip when the repository's oracle files are missing. The
//! whole-catalog tests are ignored; run them in release, since ThorVG's debug
//! build is about seven times slower. ThorVG must be built without FMA
//! contraction (the crate docs' build requirement):
//!
//! ```text
//! CXXFLAGS=-ffp-contract=off cargo test --release -p gai-render --test parity -- --ignored --nocapture
//! ```

mod common;

use common::{SetDirs, adopt_clock, adopt_clock_record, oracle_record, repo, sheet_difference};
use gai_core::{to_js_json, to_py_json};
use gai_render::legacy::{ActionValidation, compare_shared_keys, validate_evidence};
use gai_render::player::{Paint, Style, Theme};
use gai_render::preview::{WeightMethod, contact_sheet, render_frame};
use gai_render::raster::fnv1a;
use gai_render::{
    ActionOptions, ContactSheet, Look, Outcome, PaintedPlayer, Report, SetManifest, SetOptions, SetRules, SheetImage, verify_action,
    verify_set,
};
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::fs;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Instant;

macro_rules! require_set {
    ($id:expr) => {
        match SetDirs::find($id) {
            Some(dirs) => dirs,
            None => {
                eprintln!("skipped: oracle files for set {} are missing", $id);
                return;
            }
        }
    };
}

fn manifest(dirs: &SetDirs) -> SetManifest {
    SetManifest::from_slice(&fs::read(dirs.assets.join("set.json")).expect("set.json")).expect("set.json parses")
}

/// Browser tolerance per sheet kind: (expected size, rows of drawing, max channel difference
/// there). The label band below the drawing uses another font and is only measured; alpha
/// must match everywhere.
fn sheet_tolerance(name: &str, key_poses: usize) -> ((u32, u32), u32, u8) {
    if name.ends_with("-rest.png") {
        ((192, 192), 192, 1)
    } else if name.ends_with("-styles.png") {
        ((960, 224), 192, 1)
    } else {
        ((128 * key_poses as u32, 158), 128, 1)
    }
}

/// Every evidence JSON is a fixed point of the report model and writer:
/// parsing into [`Report`] and writing it back gives the same bytes.
#[test]
fn evidence_json_round_trips_through_the_report_writer() {
    let root = repo().join("motion/icon-sets");
    let Ok(sets) = fs::read_dir(&root) else {
        eprintln!("skipped: no motion/icon-sets");
        return;
    };
    let mut files = Vec::new();
    for set in sets.flatten() {
        for dir in [set.path().join("evidence"), set.path().join("evidence/round-1")] {
            for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let is_report = name.starts_with("browser-verification")
                    || name.starts_with("directed-browser-progress")
                    || name.starts_with("failed-render-");
                if is_report && name.ends_with(".json") {
                    files.push(entry.path());
                }
            }
        }
    }
    // The page's key orders; other files were written by other tools or by hand.
    const PAGE: [&str; 7] = ["date", "batch", "runtime", "assets", "errors", "checking", "result"];
    let mut checked = 0;
    for path in &files {
        let bytes = fs::read(path).unwrap();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        let keys: Vec<&str> = value.as_object().unwrap().keys().map(String::as_str).collect();
        let page_shape = keys.len() >= 5
            && keys[..5] == PAGE[..5]
            && keys[5..].iter().all(|k| PAGE[5..].contains(k))
            && keys.windows(2).all(|w| PAGE.iter().position(|k| k == &w[0]) < PAGE.iter().position(|k| k == &w[1]));
        if !page_shape {
            eprintln!("not a page report: {}", path.display());
            continue;
        }
        let text = Report::from_value(&value).unwrap().to_json();
        assert!(text.as_bytes() == bytes.as_slice(), "{} does not round-trip", path.display());
        checked += 1;
    }
    eprintln!("{checked} of {} evidence reports round-trip byte for byte", files.len());
    assert!(checked > 0);
}

/// Canonical sets: full action records equal the browser's byte for byte, the
/// sheets' drawings are within ±1 of the browser's PNGs with alpha exact, and no
/// sheet file is larger than the browser's.
#[test]
fn canonical_actions_match_the_browser() {
    for (id, pick) in [("35", Some("partly-cloudy")), ("41", None)] {
        let dirs = require_set!(id);
        let set = manifest(&dirs);
        let (_, oracle) = dirs.oracle();
        let meta = set
            .all_actions()
            .into_iter()
            .find(|a| pick.map_or(a.continuous, |name| a.name == name))
            .expect("sample action");
        let run = verify_action(&set, meta, &dirs.lottie(&meta.name), &SetRules::for_set(id), ActionOptions { sheets: true });
        let Outcome::Passed(record) = run.outcome else { panic!("{} failed: {:?}", meta.name, run.outcome) };
        let expected = oracle_record(&oracle, &meta.name);
        let mut record = Value::Object(record);
        adopt_clock_record(&mut record, expected);
        assert_eq!(compare_shared_keys(expected, &record), Vec::<String>::new(), "{}", meta.name);
        assert_eq!(to_py_json(&record, Some(2), true), to_py_json(expected, Some(2), true), "{}", meta.name);
        assert_eq!(run.checking, format!("Checking {} / duotone / 2.5 / 192 px / light", meta.name));
        for SheetImage { name, png } in &run.images {
            let Ok(browser) = fs::read(dirs.evidence.join(name)) else { continue };
            let (size, rows, tolerance) = sheet_tolerance(name, meta.key_poses.len());
            let difference = sheet_difference(png, &browser, size, rows).unwrap();
            let (max, alpha) = (difference.drawing, difference.alpha);
            assert!(max <= tolerance && !alpha, "{name}: max channel difference {max}, alpha differs: {alpha}");
            assert!(png.len() <= browser.len(), "{name}: {} bytes, browser {}", png.len(), browser.len());
        }
    }
}

/// Legacy pages: set 34's counter rule reproduces its records byte for byte;
/// set 12's loader record matches on every key the old page wrote.
#[test]
fn set_rules_reproduce_legacy_evidence() {
    for (id, name, identical) in [("34", "world-clock-strong", true), ("12", "orbit-comet", false)] {
        let dirs = require_set!(id);
        let validation = validate_evidence(&dirs.assets, id, &dirs.oracle_report(), Some(&[name.to_string()])).unwrap();
        assert_eq!(validation.actions, [(name.to_string(), ActionValidation::Matched { identical })]);
    }
}

/// Runtime-painted previews equal the verified pixels for authored weights;
/// in-between weights are rewritten because the stroke-width slot is ignored.
#[test]
fn render_frame_matches_verified_pixels() {
    let dirs = require_set!("35");
    let (_, oracle) = dirs.oracle();
    let lottie = dirs.lottie("partly-cloudy");
    let sample = &oracle_record(&oracle, "partly-cloudy")["sizes"][3];
    assert_eq!(sample["size"], 192);
    let mut hashes = Vec::new();
    for paint in sample["themes"].as_array().unwrap() {
        let look = Look {
            theme: Theme::parse(paint["theme"].as_str().unwrap()).unwrap(),
            style: Style::parse(paint["style"].as_str().unwrap()).unwrap(),
            weight: paint["width"].as_f64().unwrap(),
            color: None,
        };
        let frame = render_frame(&lottie, None, 30.0, 192, &look).unwrap();
        assert_eq!(frame.weight_method, WeightMethod::Authored);
        assert_eq!(u64::from(fnv1a(&frame.rgba)), paint["restHash"].as_u64().unwrap(), "{look:?}");
        hashes.push(fnv1a(&frame.rgba));
    }
    let between = Look { theme: Theme::Dark, style: Style::Outline, weight: 1.75, color: None };
    let rewritten = render_frame(&lottie, None, 30.0, 192, &between).unwrap();
    assert_eq!(rewritten.weight_method, WeightMethod::Rewrite);
    assert!(!hashes.contains(&fnv1a(&rewritten.rgba)));
    // Without the rewrite the slot alone leaves Regular unchanged (ThorVG 1.1.1).
    let mut player = PaintedPlayer::open(&lottie, 192).unwrap();
    player.paint(&Paint::runtime(&between), None).unwrap();
    player.set_frame(30.0);
    let regular = render_frame(&lottie, None, 30.0, 192, &Look::regular(Theme::Dark)).unwrap();
    assert_eq!(fnv1a(player.rgba()) != fnv1a(&regular.rgba), gai_render::STROKE_WIDTH_SLOT_SUPPORTED);
    let tinted = Look { color: Some([255, 64, 0]), ..Look::regular(Theme::Dark) };
    assert_ne!(fnv1a(&render_frame(&lottie, None, 30.0, 192, &tinted).unwrap().rgba), fnv1a(&regular.rgba));
    let strong = render_frame(&lottie, Some("partly-cloudy-strong"), 10.0, 48, &Look::regular(Theme::Light)).unwrap();
    assert_eq!(strong.rgba.len(), 48 * 48 * 4);
    assert!(render_frame(&lottie, None, 144.0, 48, &Look::regular(Theme::Dark)).is_err());
    assert!(render_frame(&lottie, Some("missing"), 0.0, 48, &Look::regular(Theme::Dark)).is_err());
}

#[test]
fn contact_sheet_follows_the_documented_layout() {
    let dirs = require_set!("35");
    let lottie = dirs.lottie("partly-cloudy");
    let mut sheet = ContactSheet::at_seconds(&[0.0, 0.092, 0.46]);
    sheet.cell = 64;
    sheet.looks.push(Look { weight: 1.25, ..Look::regular(Theme::Dark) });
    let (width, height, rgba) = common::decode_png(&contact_sheet(&lottie, &sheet).unwrap());
    // PAD + column labels + 7 rows of 64 px with 4 px gaps + PAD.
    assert_eq!(height, 8 + 16 + 7 * 64 + 6 * 4 + 8);
    let grid_width = 3 * 64 + 2 * 4;
    assert!(width > 16 + grid_width);
    // The last cell of the first row: dark ground in the corner, icon inside.
    let (x0, y0) = (width - 8 - 64, 8 + 16);
    let pixel = |x: u32, y: u32| {
        let i = ((y * width + x) * 4) as usize;
        [rgba[i], rgba[i + 1], rgba[i + 2]]
    };
    assert_eq!(pixel(x0, y0), [16, 23, 29]);
    assert_eq!(pixel(x0 - 1, y0), [11, 15, 20]);
    assert!((0..64).flat_map(|y| (0..64).map(move |x| (x0 + x, y0 + y))).any(|(x, y)| pixel(x, y) != [16, 23, 29]));
    sheet.title = Some("partly-cloudy".into());
    let (_, titled, _) = common::decode_png(&contact_sheet(&lottie, &sheet).unwrap());
    assert_eq!(titled, height + 16);
    sheet.frames.clear();
    assert!(contact_sheet(&lottie, &sheet).is_err());
}

/// The call sequences that crash dotlottie-rs natively (a second slot flush
/// after a render) run safely, and a frame's pixels do not depend on history.
#[test]
fn painted_player_survives_the_native_crash_sequences() {
    let dirs = require_set!("35");
    let lottie = dirs.lottie("partly-cloudy");
    let dark = Paint::bind_style(Theme::Dark, Style::Outline, 2.0);
    let light = Paint::bind_style(Theme::Light, Style::Solid, 2.0);
    let mut player = PaintedPlayer::open(&lottie, 32).unwrap();
    player.paint(&dark, None).unwrap();
    player.set_frame(10.0);
    player.paint(&light, None).unwrap();
    player.set_frame(20.0);
    player.paint(&dark, None).unwrap();
    player.paint(&dark.clone().with_color([0.35, 0.89, 0.91, 1.0]), None).unwrap();
    player.select("partly-cloudy-strong");
    assert_eq!(player.animation(), "partly-cloudy-strong");
    player.paint(&light, None).unwrap();
    player.paint(&dark, None).unwrap();
    player.select("partly-cloudy");
    player.paint(&light, None).unwrap();
    player.set_segment(0.0, 143.0);
    player.set_frame(12.0);
    player.paint(&dark, None).unwrap();
    player.paint(&light, None).unwrap();
    let history = fnv1a(player.rgba());
    let mut fresh = PaintedPlayer::open(&lottie, 32).unwrap();
    fresh.paint(&light, None).unwrap();
    fresh.set_frame(12.0);
    assert_eq!(history, fnv1a(fresh.rgba()));
    player.select("missing");
    assert_eq!(player.animation(), "partly-cloudy");
    assert_eq!(player.paint(&dark, Some("Theme rejected")), Ok(()));
}

// ---- Whole catalog (ignored): one child process per set, run in parallel. ----

const JOB: &str = "GAI_RENDER_PARITY_JOB";
const OUT: &str = "GAI_RENDER_PARITY_OUT";

/// Runs one job (`"canonical NN"` or `"legacy NN"`) when spawned by a
/// whole-catalog test; does nothing otherwise.
#[test]
#[ignore = "worker process for the whole-catalog tests"]
fn catalog_worker() {
    let (Ok(job), Ok(out)) = (std::env::var(JOB), std::env::var(OUT)) else { return };
    let (kind, id) = job.split_once(' ').expect("job");
    let summary = match kind {
        "canonical" => canonical_set(id),
        "legacy" => legacy_set(id),
        other => panic!("unknown job {other}"),
    };
    fs::write(out, to_js_json(&summary)).expect("summary");
}

/// Verifies a canonical set in process and compares the whole report and its sheets.
fn canonical_set(id: &str) -> Value {
    let dirs = SetDirs::find(id).expect("set");
    let (bytes, oracle) = dirs.oracle();
    let set = manifest(&dirs);
    let started = Instant::now();
    let options = SetOptions::for_set(id);
    let date = oracle["date"].as_str().unwrap_or_default().to_string();
    let run = verify_set(&dirs.assets, &options, date, |_| Ok(())).expect("set run");
    let mut report = run.report().to_value();
    adopt_clock(&mut report, &oracle);
    let text = to_py_json(&report, Some(2), true) + "\n";
    let mut differing = Vec::new();
    for old in oracle["assets"].as_array().into_iter().flatten() {
        let name = old["name"].as_str().unwrap_or_default();
        let new = report["assets"].as_array().and_then(|a| a.iter().find(|r| r["name"] == name));
        match new {
            None => differing.push(json!({"name": name, "differences": ["missing"]})),
            Some(new) if to_py_json(new, Some(2), true) != to_py_json(old, Some(2), true) => {
                let mut differences = compare_shared_keys(old, new);
                differences.truncate(3);
                if differences.is_empty() {
                    differences.push("key order or keys differ".into());
                }
                differing.push(json!({"name": name, "differences": differences}));
            }
            Some(_) => {}
        }
    }
    let mut png = json!({"checked": 0, "rest": 0, "styles": 0, "poses": 0, "labels": 0, "alpha": 0, "bytes": 0, "browserBytes": 0,
                         "outside": []});
    let add = |png: &mut Value, key: &str, n: u64| png[key] = json!(png[key].as_u64().unwrap_or(0) + n);
    let raise = |png: &mut Value, key: &str, n: u8| png[key] = json!(png[key].as_u64().unwrap_or(0).max(u64::from(n)));
    for SheetImage { name, png: ours } in run.images() {
        let Ok(browser) = fs::read(dirs.evidence.join(name)) else { continue };
        let action = name.rsplit_once('-').map_or("", |(a, _)| a);
        let poses = set.all_actions().iter().find(|a| a.name == action).map_or(0, |a| a.key_poses.len());
        let (size, rows, tolerance) = sheet_tolerance(name, poses);
        let kind = ["rest", "styles", "poses"].into_iter().find(|k| name.ends_with(&format!("-{k}.png"))).unwrap_or("poses");
        add(&mut png, "checked", 1);
        add(&mut png, "bytes", ours.len() as u64);
        add(&mut png, "browserBytes", browser.len() as u64);
        match sheet_difference(ours, &browser, size, rows) {
            Ok(difference) => {
                let (max, alpha) = (difference.drawing, difference.alpha);
                raise(&mut png, kind, max);
                raise(&mut png, "labels", difference.labels);
                if alpha {
                    add(&mut png, "alpha", 1);
                }
                if max > tolerance || alpha {
                    png["outside"].as_array_mut().unwrap().push(json!(name));
                }
            }
            Err(message) => png["outside"].as_array_mut().unwrap().push(json!(format!("{name}: {message}"))),
        }
    }
    json!({"set": id, "actions": oracle["assets"].as_array().map_or(0, Vec::len), "identical": text.as_bytes() == bytes.as_slice(),
           "differing": differing, "errors": run.report().errors, "png": png, "seconds": started.elapsed().as_secs_f64()})
}

/// Re-verifies a legacy set's evidence with the canonical checks plus its rules.
fn legacy_set(id: &str) -> Value {
    let dirs = SetDirs::find(id).expect("set");
    let started = Instant::now();
    let validation = validate_evidence(&dirs.assets, id, &dirs.oracle_report(), None).expect("validation");
    let mut differs = Vec::new();
    let mut failed = Vec::new();
    let (mut stale, mut missing) = (0, 0);
    for (name, outcome) in &validation.actions {
        match outcome {
            ActionValidation::Matched { .. } => {}
            ActionValidation::Differs(list) => differs.push(json!({"name": name, "differences": list.iter().take(3).collect::<Vec<_>>()})),
            ActionValidation::Failed(message) => failed.push(json!({"name": name, "message": message})),
            ActionValidation::Stale => stale += 1,
            ActionValidation::Missing => missing += 1,
        }
    }
    json!({"set": id, "actions": validation.actions.len(), "matched": validation.matched(), "identical": validation.identical(),
           "differs": differs, "failed": failed, "stale": stale, "missing": missing, "seconds": started.elapsed().as_secs_f64()})
}

/// Runs jobs in child processes of this test binary, in parallel.
fn run_jobs(jobs: Vec<String>) -> Vec<Value> {
    let exe = std::env::current_exe().expect("test binary");
    let scratch = tempfile::tempdir().expect("scratch");
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get().saturating_sub(2).max(1));
    let queue = Arc::new(Mutex::new(jobs.into_iter().enumerate().collect::<VecDeque<_>>()));
    let results = Arc::new(Mutex::new(Vec::new()));
    let threads: Vec<_> = (0..workers)
        .map(|_| {
            let (queue, results, exe, dir) = (queue.clone(), results.clone(), exe.clone(), scratch.path().to_path_buf());
            std::thread::spawn(move || {
                loop {
                    // Take the job first: a guard in the loop condition would hold the lock.
                    let next = queue.lock().unwrap().pop_front();
                    let Some((index, job)) = next else { break };
                    let out = dir.join(format!("{index}.json"));
                    let status = Command::new(&exe)
                        .args(["catalog_worker", "--exact", "--ignored", "--nocapture", "--test-threads=1"])
                        .env(JOB, &job)
                        .env(OUT, &out)
                        .output()
                        .expect("worker spawns");
                    let summary = match fs::read(&out) {
                        Ok(bytes) if status.status.success() => serde_json::from_slice(&bytes).expect("summary"),
                        _ => json!({"job": job, "crashed": status.status.to_string(),
                                    "stderr": String::from_utf8_lossy(&status.stderr).lines().rev().take(5).collect::<Vec<_>>()}),
                    };
                    results.lock().unwrap().push((index, summary));
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().expect("worker thread");
    }
    let mut results = std::mem::take(&mut *results.lock().unwrap());
    results.sort_by_key(|(index, _)| *index);
    results.into_iter().map(|(_, summary)| summary).collect()
}

fn catalog_ids(range: std::ops::RangeInclusive<u32>) -> Vec<String> {
    SetDirs::all_ids().into_iter().filter(|id| id.parse().is_ok_and(|n: u32| range.contains(&n))).collect()
}

/// Sets 35 onwards: every `browser-verification.json` is reproduced byte for
/// byte (run date and observed frame-event counts taken from the browser), every
/// sheet's drawing is within ±1 of the browser's PNG with alpha exact, and the
/// sheets take no more bytes than the browser's. Label bands are only measured.
#[test]
#[ignore = "whole catalog; run in release"]
fn canonical_sets_reproduce_browser_verification() {
    let ids = catalog_ids(35..=99);
    if ids.is_empty() {
        eprintln!("skipped: no canonical oracle sets");
        return;
    }
    let started = Instant::now();
    let summaries = run_jobs(ids.iter().map(|id| format!("canonical {id}")).collect());
    let (mut actions, mut identical_sets, mut pngs, mut max, mut bytes) = (0, 0, 0, [0u64; 4], [0u64; 2]);
    for s in &summaries {
        eprintln!("{}", to_js_json(s));
        actions += s["actions"].as_u64().unwrap_or(0);
        identical_sets += u64::from(s["identical"] == true);
        pngs += s["png"]["checked"].as_u64().unwrap_or(0);
        for (i, kind) in ["rest", "styles", "poses", "labels"].iter().enumerate() {
            max[i] = max[i].max(s["png"][kind].as_u64().unwrap_or(0));
        }
        for (i, key) in ["bytes", "browserBytes"].iter().enumerate() {
            bytes[i] += s["png"][key].as_u64().unwrap_or(0);
        }
    }
    eprintln!(
        "canonical: {identical_sets}/{} sets byte-identical, {actions} actions, {pngs} PNGs (max channel difference rest {}, styles {}, poses {}; labels {}), {} bytes against the browser's {}, {:.0} s",
        summaries.len(),
        max[0],
        max[1],
        max[2],
        max[3],
        bytes[0],
        bytes[1],
        started.elapsed().as_secs_f64()
    );
    for s in &summaries {
        assert_eq!(s["identical"], true, "set {} differs: {}", s["set"], to_js_json(s));
        assert_eq!(s["png"]["outside"], json!([]), "set {} sheets outside tolerance", s["set"]);
    }
    assert!(bytes[0] <= bytes[1], "sheets take {} bytes, the browser's {}", bytes[0], bytes[1]);
}

/// Sets 01–34: re-verifying with the canonical checks plus their rules
/// reproduces every value their pages recorded.
#[test]
#[ignore = "whole catalog; run in release"]
fn legacy_sets_validate_existing_evidence() {
    let ids = catalog_ids(1..=34);
    if ids.is_empty() {
        eprintln!("skipped: no legacy oracle sets");
        return;
    }
    let started = Instant::now();
    let summaries = run_jobs(ids.iter().map(|id| format!("legacy {id}")).collect());
    let mut totals = [0u64; 4];
    for s in &summaries {
        eprintln!("{}", to_js_json(s));
        for (i, key) in ["actions", "matched", "identical"].iter().enumerate() {
            totals[i] += s[key].as_u64().unwrap_or(0);
        }
        totals[3] += s["failed"].as_array().map_or(0, |f| f.len() as u64) + s["differs"].as_array().map_or(0, |d| d.len() as u64);
    }
    eprintln!(
        "legacy: {} actions, {} match on shared keys ({} byte-identical), {} differ or fail, {:.0} s",
        totals[0],
        totals[1],
        totals[2],
        totals[3],
        started.elapsed().as_secs_f64()
    );
    for s in &summaries {
        assert!(s.get("crashed").is_none(), "worker crashed: {}", to_js_json(s));
        assert_eq!(s["matched"], s["actions"], "set {}: {}", s["set"], to_js_json(s));
    }
}
