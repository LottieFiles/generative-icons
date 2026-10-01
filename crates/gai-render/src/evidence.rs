//! The set report verify.html assembled and the evidence files
//! `serve-verification.py` wrote from it.

use crate::catalog::SetManifest;
use crate::probe::require_browser_parity;
use crate::rules::SetRules;
use crate::sheets::SheetImage;
use crate::verify::{ActionOptions, ActionRun, Outcome, is_png_name, verify_action};
use crate::RenderError;
use gai_core::to_py_json;
use serde_json::{Map, Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The `runtime` recorded in reports. The renderer is dotlottie-rs at
/// `e112d3f`, the core this dotlottie-web release was built from, so the
/// evidence stays byte-identical to the browser's.
pub const RUNTIME: &str = "@lottiefiles/dotlottie-web@0.80.0";

/// The native renderer, for tool output that names it (`info`, `preview`).
pub const RENDERER: &str = "dotlottie-rs@e112d3f";

/// A finished report's verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Every selected action passed.
    Pass,
    /// At least one action failed.
    Fail,
}

impl Verdict {
    /// `"PASS"` or `"FAIL"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Pass => "PASS",
            Verdict::Fail => "FAIL",
        }
    }
}

/// verify.html's `report` object.
#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    /// Run start, ISO 8601 UTC with milliseconds.
    pub date: String,
    /// Batch label (`""` when none).
    pub batch: String,
    /// Recorded runtime, [`RUNTIME`].
    pub runtime: String,
    /// Result records of passing actions.
    pub assets: Vec<Map<String, Value>>,
    /// `{name, message}` per failed action, then the final `{message}`.
    pub errors: Vec<Value>,
    /// Progress text; set once the first action starts.
    pub checking: Option<String>,
    /// Set when the run finishes.
    pub result: Option<Verdict>,
}

impl Report {
    /// An empty report, as at the start of a run.
    pub fn new(date: String, batch: String, runtime: String) -> Report {
        Report { date, batch, runtime, assets: Vec::new(), errors: Vec::new(), checking: None, result: None }
    }

    /// The report as JSON, keys in the page's insertion order.
    pub fn to_value(&self) -> Value {
        let mut map = Map::new();
        map.insert("date".into(), json!(self.date));
        map.insert("batch".into(), json!(self.batch));
        map.insert("runtime".into(), json!(self.runtime));
        map.insert("assets".into(), Value::Array(self.assets.iter().cloned().map(Value::Object).collect()));
        map.insert("errors".into(), Value::Array(self.errors.clone()));
        if let Some(checking) = &self.checking {
            map.insert("checking".into(), json!(checking));
        }
        if let Some(result) = self.result {
            map.insert("result".into(), json!(result.as_str()));
        }
        Value::Object(map)
    }

    /// The evidence file text: Python `json.dumps(report, indent=2) + "\n"`.
    pub fn to_json(&self) -> String {
        to_py_json(&self.to_value(), Some(2), true) + "\n"
    }

    /// Reads a report written by the browser or by [`Report::to_json`].
    pub fn from_value(value: &Value) -> Result<Report, RenderError> {
        let text = |key: &str| value.get(key).and_then(Value::as_str).map(str::to_string);
        let list = |key: &str| value.get(key).and_then(Value::as_array).cloned().unwrap_or_default();
        let assets = list("assets")
            .into_iter()
            .map(|a| match a {
                Value::Object(map) => Ok(map),
                _ => Err(RenderError::Evidence("an asset record is not an object".into())),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let result = match text("result").as_deref() {
            Some("PASS") => Some(Verdict::Pass),
            Some("FAIL") => Some(Verdict::Fail),
            _ => None,
        };
        Ok(Report {
            date: text("date").ok_or_else(|| RenderError::Evidence("report has no date".into()))?,
            batch: text("batch").unwrap_or_default(),
            runtime: text("runtime").unwrap_or_default(),
            assets,
            errors: list("errors"),
            checking: text("checking"),
            result,
        })
    }
}

/// A set verification in progress: the report, the sheets produced so far
/// and the page's `context`.
///
/// Drive it with [`SetRun::start_action`] and [`SetRun::record`] per action
/// (in process with [`verify_set`], or from child-process results), then
/// [`SetRun::finish`].
#[derive(Clone, Debug, PartialEq)]
pub struct SetRun {
    report: Report,
    images: Vec<SheetImage>,
    context: String,
}

impl SetRun {
    /// Starts a run.
    pub fn new(date: String, batch: String, runtime: String) -> SetRun {
        SetRun { report: Report::new(date, batch, runtime), images: Vec::new(), context: String::new() }
    }

    /// The page's state at an action's start (`context = checking = name`).
    pub fn start_action(&mut self, name: &str) {
        self.context = name.to_string();
        self.report.checking = Some(name.to_string());
    }

    /// Adds an action's outcome and sheets.
    pub fn record(&mut self, run: ActionRun) {
        match run.outcome {
            Outcome::Passed(record) => self.report.assets.push(record),
            Outcome::Failed(message) => self.report.errors.push(json!({"name": run.name, "message": message})),
        }
        self.context = run.context;
        self.report.checking = Some(run.checking);
        self.images.extend(run.images);
    }

    /// Ends the run: PASS when no action failed, otherwise FAIL with the page's
    /// final "Renderer errors" message.
    pub fn finish(&mut self) {
        if self.report.errors.is_empty() {
            self.report.result = Some(Verdict::Pass);
        } else {
            self.report.result = Some(Verdict::Fail);
            let message = format!("Error: {} \u{2014} Renderer errors", self.context);
            self.report.errors.push(json!({"message": message}));
        }
    }

    /// Keeps previous records for actions this (filtered) run did not check,
    /// when their `sha256` still matches set.json, in set.json order. Records
    /// from this run win; stale ones are dropped.
    pub fn merge_previous(&mut self, previous: &Report, set: &SetManifest) {
        let find = |list: &[Map<String, Value>], name: &str| {
            list.iter().find(|r| r.get("name").and_then(Value::as_str) == Some(name)).cloned()
        };
        let merged = set
            .all_actions()
            .into_iter()
            .filter_map(|meta| {
                find(&self.report.assets, &meta.name).or_else(|| {
                    find(&previous.assets, &meta.name)
                        .filter(|r| r.get("sha256").and_then(Value::as_str) == Some(meta.sha256.as_str()))
                })
            })
            .collect();
        self.report.assets = merged;
    }

    /// The report so far.
    pub fn report(&self) -> &Report {
        &self.report
    }

    /// Every sheet produced so far, in order.
    pub fn images(&self) -> &[SheetImage] {
        &self.images
    }
}

/// What a set verification covers.
#[derive(Clone, Debug)]
pub struct SetOptions {
    /// Only these icons (`--icons`).
    pub icons: Option<Vec<String>>,
    /// Only these actions (`--actions`).
    pub actions: Option<Vec<String>>,
    /// Batch label; also the evidence file suffix when it is `[a-z0-9-]{1,30}`.
    pub batch: String,
    /// Per-set additions ([`SetRules::for_set`]).
    pub rules: SetRules,
    /// Per-action options.
    pub action: ActionOptions,
    /// Recorded runtime.
    pub runtime: String,
}

impl SetOptions {
    /// Every action of a set, with the set's rules ([`SetRules::for_set`]).
    pub fn for_set(set_id: &str) -> SetOptions {
        SetOptions { rules: SetRules::for_set(set_id), ..SetOptions::default() }
    }
}

impl Default for SetOptions {
    fn default() -> SetOptions {
        SetOptions {
            icons: None,
            actions: None,
            batch: String::new(),
            rules: SetRules::CANONICAL,
            action: ActionOptions::default(),
            runtime: RUNTIME.into(),
        }
    }
}

/// Verifies a set in process: every selected action of `assets_dir/set.json`
/// against `assets_dir/{action}.lottie`. `save` runs at the page's save points
/// (each action's start and end, and the end of the run). Fails with
/// [`RenderError::RendererBuild`] on a build that cannot match the browser.
pub fn verify_set(
    assets_dir: &Path,
    options: &SetOptions,
    date: String,
    mut save: impl FnMut(&SetRun) -> Result<(), RenderError>,
) -> Result<SetRun, RenderError> {
    require_browser_parity()?;
    let path = assets_dir.join("set.json");
    let bytes = fs::read(&path).map_err(|source| RenderError::Io { path, source })?;
    let set = SetManifest::from_slice(&bytes)?;
    let mut run = SetRun::new(date, options.batch.clone(), options.runtime.clone());
    for meta in set.actions(options.icons.as_deref(), options.actions.as_deref()) {
        run.start_action(&meta.name);
        save(&run)?;
        let action = match fs::read(assets_dir.join(format!("{}.lottie", meta.name))) {
            Ok(lottie) => verify_action(&set, meta, &lottie, &options.rules, options.action),
            Err(_) => ActionRun {
                name: meta.name.clone(),
                outcome: Outcome::Failed("Error: Load error".into()),
                context: meta.name.clone(),
                checking: meta.name.clone(),
                images: Vec::new(),
            },
        };
        run.record(action);
        save(&run)?;
    }
    run.finish();
    save(&run)?;
    Ok(run)
}

/// The evidence file suffix for a batch label (`serve-verification.py`).
pub fn batch_suffix(batch: &str) -> String {
    let valid = (1..=30).contains(&batch.len())
        && batch.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if valid { format!("-{batch}") } else { String::new() }
}

/// Writes evidence exactly where and in the order `serve-verification.py` did.
///
/// Per save: `failed-render-{UTC stamp}{suffix}.json` when the run failed,
/// `directed-browser-progress{suffix}.json` always, each new PNG to the
/// evidence folder and the set's assets folder, and
/// `browser-verification{suffix}.json` when the run passed.
#[derive(Clone, Debug)]
pub struct EvidenceWriter {
    evidence_dir: PathBuf,
    assets_dir: Option<PathBuf>,
    images_written: usize,
}

impl EvidenceWriter {
    /// Writes into `evidence_dir` (created if missing) and copies PNGs into
    /// `assets_dir` when given.
    pub fn new(evidence_dir: PathBuf, assets_dir: Option<PathBuf>) -> EvidenceWriter {
        EvidenceWriter { evidence_dir, assets_dir, images_written: 0 }
    }

    fn write(path: PathBuf, bytes: &[u8]) -> Result<PathBuf, RenderError> {
        fs::write(&path, bytes).map_err(|source| RenderError::Io { path: path.clone(), source })?;
        Ok(path)
    }

    /// Saves the run's current state; returns the files written.
    pub fn save(&mut self, run: &SetRun, now: SystemTime) -> Result<Vec<PathBuf>, RenderError> {
        let dir = &self.evidence_dir;
        fs::create_dir_all(dir).map_err(|source| RenderError::Io { path: dir.clone(), source })?;
        let report = run.report();
        let suffix = batch_suffix(&report.batch);
        let text = report.to_json();
        let mut written = Vec::new();
        if report.result == Some(Verdict::Fail) {
            let name = format!("failed-render-{}{suffix}.json", failure_stamp(now));
            written.push(Self::write(dir.join(name), text.as_bytes())?);
        }
        written.push(Self::write(dir.join(format!("directed-browser-progress{suffix}.json")), text.as_bytes())?);
        for image in &run.images()[self.images_written.min(run.images().len())..] {
            if !is_png_name(&image.name) {
                return Err(RenderError::Evidence(format!("bad image name {:?}", image.name)));
            }
            if let Some(assets) = &self.assets_dir {
                written.push(Self::write(assets.join(&image.name), &image.png)?);
            }
            written.push(Self::write(dir.join(&image.name), &image.png)?);
            self.images_written += 1;
        }
        if report.result == Some(Verdict::Pass) {
            written.push(Self::write(dir.join(format!("browser-verification{suffix}.json")), text.as_bytes())?);
        }
        Ok(written)
    }
}

/// UTC civil date and time of a system time: (y, m, d, H, M, S, µs).
fn utc(now: SystemTime) -> (i64, u32, u32, u32, u32, u32, u32) {
    let since = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = since.as_secs() as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day, (rem / 3600) as u32, (rem % 3600 / 60) as u32, (rem % 60) as u32, since.subsec_micros())
}

/// JS `new Date().toISOString()`: `2026-09-29T07:36:01.662Z`.
pub fn iso_date(now: SystemTime) -> String {
    let (y, mo, d, h, mi, s, us) = utc(now);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{:03}Z", us / 1000)
}

/// Python `strftime('%Y%m%dT%H%M%S%fZ')` in UTC: `20260929T092707897379Z`.
pub fn failure_stamp(now: SystemTime) -> String {
    let (y, mo, d, h, mi, s, us) = utc(now);
    format!("{y:04}{mo:02}{d:02}T{h:02}{mi:02}{s:02}{us:06}Z")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn timestamps_match_js_and_python() {
        let t = UNIX_EPOCH + Duration::from_micros(1_790_674_027_897_379);
        assert_eq!(failure_stamp(t), "20260929T092707897379Z");
        assert_eq!(iso_date(t), "2026-09-29T09:27:07.897Z");
        assert_eq!(iso_date(UNIX_EPOCH + Duration::from_millis(951_782_400_000)), "2000-02-29T00:00:00.000Z");
    }

    #[test]
    fn batch_suffix_follows_the_sink() {
        assert_eq!(batch_suffix(""), "");
        assert_eq!(batch_suffix("retry-2"), "-retry-2");
        assert_eq!(batch_suffix("Retry"), "");
        assert_eq!(batch_suffix(&"a".repeat(31)), "");
    }

    #[test]
    fn report_keys_follow_the_page() {
        let mut run = SetRun::new("2026-09-29T07:36:01.662Z".into(), String::new(), RUNTIME.into());
        assert_eq!(run.report().to_json(), "{\n  \"date\": \"2026-09-29T07:36:01.662Z\",\n  \"batch\": \"\",\n  \"runtime\": \"@lottiefiles/dotlottie-web@0.80.0\",\n  \"assets\": [],\n  \"errors\": []\n}\n");
        run.start_action("a");
        run.record(ActionRun {
            name: "a".into(),
            outcome: Outcome::Failed("Error: a / duotone / 2.5 / 192 / light \u{2014} Color override has no visible effect".into()),
            context: "a / duotone / 2.5 / 192 / light".into(),
            checking: "Checking a / duotone / 2.5 / 192 px / light".into(),
            images: Vec::new(),
        });
        run.finish();
        let text = run.report().to_json();
        assert!(text.contains("\"message\": \"Error: a / duotone / 2.5 / 192 / light \\u2014 Renderer errors\""));
        assert!(text.ends_with("\"checking\": \"Checking a / duotone / 2.5 / 192 px / light\",\n  \"result\": \"FAIL\"\n}\n"));
        let back = Report::from_value(&serde_json::from_str(&text).unwrap()).unwrap();
        assert_eq!(&back, run.report());
    }

    #[test]
    fn writer_follows_the_sink_order() {
        let dir = tempfile::tempdir().unwrap();
        let (evidence, assets) = (dir.path().join("evidence"), dir.path().join("assets"));
        fs::create_dir_all(&assets).unwrap();
        let mut writer = EvidenceWriter::new(evidence.clone(), Some(assets.clone()));
        let mut run = SetRun::new("d".into(), "b-1".into(), RUNTIME.into());
        run.start_action("x");
        run.record(ActionRun {
            name: "x".into(),
            outcome: Outcome::Passed(Map::new()),
            context: "x".into(),
            checking: "x".into(),
            images: vec![SheetImage { name: "x-rest.png".into(), png: vec![7] }],
        });
        let now = UNIX_EPOCH + Duration::from_secs(1);
        let names = |paths: Vec<PathBuf>| -> Vec<String> {
            paths.iter().map(|p| p.strip_prefix(dir.path()).unwrap().display().to_string()).collect()
        };
        assert_eq!(
            names(writer.save(&run, now).unwrap()),
            ["evidence/directed-browser-progress-b-1.json", "assets/x-rest.png", "evidence/x-rest.png"]
        );
        run.finish();
        assert_eq!(
            names(writer.save(&run, now).unwrap()),
            ["evidence/directed-browser-progress-b-1.json", "evidence/browser-verification-b-1.json"]
        );
        let mut failed = SetRun::new("d".into(), String::new(), RUNTIME.into());
        failed.record(ActionRun::crashed("y", "signal 11"));
        failed.finish();
        let written = names(EvidenceWriter::new(evidence, None).save(&failed, now).unwrap());
        assert_eq!(written, ["evidence/failed-render-19700101T000001000000Z.json", "evidence/directed-browser-progress.json"]);
    }
}
