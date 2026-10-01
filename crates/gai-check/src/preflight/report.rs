//! Preflight's reports and their `JSON.stringify` layout, with the key orders of
//! `preflight.mjs` and `build-checked.mjs`.

use super::catalog::Input;
use gai_core::{to_js_json, to_js_json_pretty};
use serde_json::{Map, Value as Json};

/// The `scope` sentence of every preflight run.
pub const SCOPE: &str = "Source screening only; no native export or visual review";

/// Why `--all` skips a set folder without `definitions.mjs`.
pub const SKIP_REASON: &str = "Legacy inline builder; existing native/source checks required";

/// A report's verdict. Only errors fail; warnings and infos never do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
}

impl Status {
    /// `PASS` or `FAIL`.
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Pass => "PASS",
            Status::Fail => "FAIL",
        }
    }
}

/// How serious a diagnostic is. Only errors fail a report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    /// `error`, `warning` or `info`.
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }
}

/// A diagnostic code. Each has a fixed severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Code {
    EmptySet,
    IconName,
    DuplicateIcon,
    Label,
    EmptyParts,
    PartName,
    AmbiguousLayerName,
    PaintStyle,
    PaintType,
    StrokeWidth,
    OutlineWidth,
    Callback,
    DuotonePolicy,
    DuotoneUndeclared,
    StatePolicy,
    LegacyReverseName,
    Timing,
    KeyPoses,
    MorphTopology,
    Visibility,
    MovingPivot,
    ControlHullMargin,
    InvalidGeometry,
    CatalogIdCollision,
}

impl Code {
    /// The code as reported, for example `MORPH_TOPOLOGY`.
    pub fn as_str(self) -> &'static str {
        match self {
            Code::EmptySet => "EMPTY_SET",
            Code::IconName => "ICON_NAME",
            Code::DuplicateIcon => "DUPLICATE_ICON",
            Code::Label => "LABEL",
            Code::EmptyParts => "EMPTY_PARTS",
            Code::PartName => "PART_NAME",
            Code::AmbiguousLayerName => "AMBIGUOUS_LAYER_NAME",
            Code::PaintStyle => "PAINT_STYLE",
            Code::PaintType => "PAINT_TYPE",
            Code::StrokeWidth => "STROKE_WIDTH",
            Code::OutlineWidth => "OUTLINE_WIDTH",
            Code::Callback => "CALLBACK",
            Code::DuotonePolicy => "DUOTONE_POLICY",
            Code::DuotoneUndeclared => "DUOTONE_UNDECLARED",
            Code::StatePolicy => "STATE_POLICY",
            Code::LegacyReverseName => "LEGACY_REVERSE_NAME",
            Code::Timing => "TIMING",
            Code::KeyPoses => "KEY_POSES",
            Code::MorphTopology => "MORPH_TOPOLOGY",
            Code::Visibility => "VISIBILITY",
            Code::MovingPivot => "MOVING_PIVOT",
            Code::ControlHullMargin => "CONTROL_HULL_MARGIN",
            Code::InvalidGeometry => "INVALID_GEOMETRY",
            Code::CatalogIdCollision => "CATALOG_ID_COLLISION",
        }
    }

    /// Warnings: `AMBIGUOUS_LAYER_NAME`, `CONTROL_HULL_MARGIN`. Infos: `DUOTONE_UNDECLARED`,
    /// `LEGACY_REVERSE_NAME`. Everything else is an error.
    pub fn severity(self) -> Severity {
        match self {
            Code::AmbiguousLayerName | Code::ControlHullMargin => Severity::Warning,
            Code::DuotoneUndeclared | Code::LegacyReverseName => Severity::Info,
            _ => Severity::Error,
        }
    }

    /// The message of codes whose message never varies.
    pub fn fixed_message(self) -> Option<&'static str> {
        Some(match self {
            Code::EmptySet => "Export a nonempty icons array",
            Code::IconName => "Use a stable kebab-case name",
            Code::DuplicateIcon => "Icon names must be unique within the set",
            Code::Label => "A readable label is required",
            Code::EmptyParts => "Named vector parts are required",
            Code::PartName => "Every part needs a name",
            Code::AmbiguousLayerName => {
                "Separate paints share a layer name; use unique names in the next drawing revision"
            }
            Code::PaintType => "Expected fill or stroke",
            Code::StrokeWidth => "Stroke width must be positive and finite",
            Code::OutlineWidth => "Author the regular centerline at width 2; the builder creates weight presets",
            Code::DuotonePolicy => "Declared Duotone policy disagrees with tint parts",
            Code::DuotoneUndeclared => {
                "Legacy source: native paint policy remains authoritative; explicitly declare surface eligibility for new icons"
            }
            Code::StatePolicy => "Stateful icons need names for both states",
            Code::LegacyReverseName => {
                "Verify the set-specific builder names its reverse/reset; the shared builder requires reverseSuffix"
            }
            Code::Timing => "Action/rest must fit the current timeline; gesture rest follows active motion",
            Code::KeyPoses => "Key poses must be ordered finite seconds inside the timeline",
            Code::MorphTopology => "Normalized path commands change during this action",
            Code::Visibility => "Visibility must remain between 0 and 1",
            Code::MovingPivot => "Native rigid transforms require a constant pivot",
            Code::ControlHullMargin => {
                "Control-point hull plus stroke radius crosses canvas; inspect actual curve paint before changing geometry"
            }
            Code::PaintStyle | Code::Callback | Code::InvalidGeometry | Code::CatalogIdCollision => return None,
        })
    }
}

/// Where in the sampling a diagnostic arose.
#[derive(Debug, Clone, PartialEq)]
pub enum Detail {
    /// Icon- and part-level checks.
    None,
    /// `TIMING` and `KEY_POSES`: one action.
    Action { profile: String, reverse: bool },
    /// One sample: the action, the outline weight (or `authored`) and the time in seconds.
    Sample { profile: String, reverse: bool, weight: String, time: f64 },
}

/// One finding.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub code: Code,
    /// `icon.name` as `JSON.stringify` sees it; `None` for `undefined`, which omits the key.
    pub icon: Option<Json>,
    /// The part name. The key is written only when it is a nonempty string.
    pub part: Option<String>,
    /// `None` for `undefined` (a thrown non-Error), which omits the key.
    pub message: Option<Json>,
    pub detail: Detail,
}

impl Diagnostic {
    /// The code's severity.
    pub fn severity(&self) -> Severity {
        self.code.severity()
    }

    /// `{severity, code, icon, part?, message, ...detail}`, `undefined` keys omitted.
    pub fn to_json(&self) -> Json {
        let mut map = Map::new();
        map.insert("severity".into(), self.severity().as_str().into());
        map.insert("code".into(), self.code.as_str().into());
        if let Some(icon) = &self.icon {
            map.insert("icon".into(), icon.clone());
        }
        if let Some(part) = self.part.as_ref().filter(|part| !part.is_empty()) {
            map.insert("part".into(), part.as_str().into());
        }
        if let Some(message) = &self.message {
            map.insert("message".into(), message.clone());
        }
        match &self.detail {
            Detail::None => {}
            Detail::Action { profile, reverse } => {
                map.insert("profile".into(), profile.as_str().into());
                map.insert("reverse".into(), (*reverse).into());
            }
            Detail::Sample { profile, reverse, weight, time } => {
                map.insert("profile".into(), profile.as_str().into());
                map.insert("reverse".into(), (*reverse).into());
                map.insert("weight".into(), weight.as_str().into());
                map.insert("time".into(), (*time).into());
            }
        }
        Json::Object(map)
    }

    /// `JSON.stringify([severity, code, icon, part, profile, reverse, weight])`: the time
    /// and message are not part of it, so only the first of each is kept
    /// (`preflight.mjs:43`).
    pub(crate) fn dedup_key(&self) -> String {
        let (profile, reverse, weight) = match &self.detail {
            Detail::None => (Json::Null, Json::Null, Json::Null),
            Detail::Action { profile, reverse } => (profile.as_str().into(), (*reverse).into(), Json::Null),
            Detail::Sample { profile, reverse, weight, .. } => {
                (profile.as_str().into(), (*reverse).into(), weight.as_str().into())
            }
        };
        let key = Json::Array(vec![
            self.severity().as_str().into(),
            self.code.as_str().into(),
            self.icon.clone().unwrap_or(Json::Null),
            self.part.as_deref().map_or(Json::Null, Json::from),
            profile,
            reverse,
            weight,
        ]);
        to_js_json(&key)
    }
}

/// `checkDefinitions`' result.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    /// FAIL when any diagnostic is an error (or, after `checkSet`, a catalog collision).
    pub status: Status,
    /// `icons.length` (0 when the export is not an array).
    pub icons: usize,
    /// Actions (direction × profile) of icons that passed the name and parts checks.
    pub actions: usize,
    /// Samples that completed every step.
    pub samples: usize,
    pub diagnostics: Vec<Diagnostic>,
}

impl Report {
    /// A report whose status follows its diagnostics.
    pub(crate) fn new(icons: usize, actions: usize, samples: usize, diagnostics: Vec<Diagnostic>) -> Self {
        let failed = diagnostics.iter().any(|d| d.severity() == Severity::Error);
        let status = if failed { Status::Fail } else { Status::Pass };
        Report { status, icons, actions, samples, diagnostics }
    }

    /// The report for a missing, non-array or empty `icons` export (`preflight.mjs:47`).
    pub(crate) fn empty_set() -> Self {
        let empty = Diagnostic {
            code: Code::EmptySet,
            icon: None,
            part: None,
            message: Code::EmptySet.fixed_message().map(Json::from),
            detail: Detail::None,
        };
        Report { status: Status::Fail, icons: 0, actions: 0, samples: 0, diagnostics: vec![empty] }
    }

    /// `{status, icons, actions, samples, diagnostics}`.
    pub fn to_json(&self) -> Json {
        let mut map = Map::new();
        self.write(&mut map, self.status);
        Json::Object(map)
    }

    fn write(&self, map: &mut Map<String, Json>, status: Status) {
        map.insert("status".into(), status.as_str().into());
        map.insert("icons".into(), self.icons.into());
        map.insert("actions".into(), self.actions.into());
        map.insert("samples".into(), self.samples.into());
        map.insert("diagnostics".into(), self.diagnostics.iter().map(Diagnostic::to_json).collect());
    }
}

/// `checkSet`'s result: one set's report and the inputs it depends on.
#[derive(Debug, Clone, PartialEq)]
pub struct SetReport {
    /// The folder name.
    pub set: String,
    pub report: Report,
    /// Set, library and batch sources in JS sort order, then the catalog files in catalog order.
    pub inputs: Vec<Input>,
}

impl SetReport {
    /// `{set, status, icons, actions, samples, diagnostics, inputs}`.
    pub fn to_json(&self) -> Json {
        Json::Object(self.fields(self.report.status))
    }

    fn fields(&self, status: Status) -> Map<String, Json> {
        let mut map = Map::new();
        map.insert("set".into(), self.set.as_str().into());
        self.report.write(&mut map, status);
        map.insert("inputs".into(), self.inputs.iter().map(Input::to_json).collect());
        map
    }
}

/// How the build launched after a clean preflight ended, as `spawnSync` reported it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchOutcome {
    /// The builder's exit status; `None` when it never ran to an exit.
    pub status: Option<i32>,
    /// Why the builder could not start.
    pub error: Option<String>,
}

impl LaunchOutcome {
    /// The build succeeded.
    pub fn built() -> Self {
        LaunchOutcome { status: Some(0), error: None }
    }

    /// The build ran and failed; its error goes to stderr, not into the report.
    pub fn failed() -> Self {
        LaunchOutcome { status: Some(1), error: None }
    }

    /// The builder could not be started.
    pub fn not_started(error: impl Into<String>) -> Self {
        LaunchOutcome { status: None, error: Some(error.into()) }
    }
}

/// `buildChecked`'s result: the preflight report, then the build's outcome when preflight passed.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckedReport {
    pub report: SetReport,
    /// `None` when preflight failed and the build was never launched.
    pub launch: Option<LaunchOutcome>,
}

impl CheckedReport {
    /// PASS only when preflight passed and the build exited 0.
    pub fn status(&self) -> Status {
        match &self.launch {
            None => self.report.report.status,
            Some(launch) if launch.status == Some(0) => Status::Pass,
            Some(_) => Status::Fail,
        }
    }

    /// The build ran and exited 0.
    pub fn built(&self) -> bool {
        self.launch.as_ref().is_some_and(|launch| launch.status == Some(0))
    }

    /// The set report with `status` replaced in place, then `built`, and after a launch
    /// `buildExitCode` and (when the builder could not start) `error`.
    pub fn to_json(&self) -> Json {
        let mut map = self.report.fields(self.status());
        map.insert("built".into(), self.built().into());
        if let Some(launch) = &self.launch {
            map.insert("buildExitCode".into(), launch.status.map_or(Json::Null, Json::from));
            if let Some(error) = &launch.error {
                map.insert("error".into(), error.as_str().into());
            }
        }
        Json::Object(map)
    }

    /// The CLI's stdout: the report at indent 2 and a newline.
    pub fn stdout(&self) -> String {
        pretty(&self.to_json())
    }
}

/// A preflight run over one set or all of them.
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    pub reports: Vec<SetReport>,
    /// Folder names skipped by `--all` for lacking `definitions.mjs`.
    pub skipped: Vec<String>,
}

impl Run {
    /// FAIL when any report is not PASS; PASS with no reports.
    pub fn status(&self) -> Status {
        if self.reports.iter().all(|r| r.report.status == Status::Pass) { Status::Pass } else { Status::Fail }
    }

    /// `{status, scope, reports, skipped: [{set, reason}]}`.
    pub fn to_json(&self) -> Json {
        let mut map = Map::new();
        map.insert("status".into(), self.status().as_str().into());
        map.insert("scope".into(), SCOPE.into());
        map.insert("reports".into(), self.reports.iter().map(SetReport::to_json).collect());
        let skipped = self.skipped.iter().map(|set| {
            let mut entry = Map::new();
            entry.insert("set".into(), set.as_str().into());
            entry.insert("reason".into(), SKIP_REASON.into());
            Json::Object(entry)
        });
        map.insert("skipped".into(), skipped.collect());
        Json::Object(map)
    }

    /// The CLI's stdout: the run at indent 2 and a newline.
    pub fn stdout(&self) -> String {
        pretty(&self.to_json())
    }
}

/// `console.log(JSON.stringify(value, null, 2))`.
fn pretty(value: &Json) -> String {
    let mut out = to_js_json_pretty(value, 2);
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(code: Code, time: f64) -> Diagnostic {
        Diagnostic {
            code,
            icon: Some("a".into()),
            part: Some("p".into()),
            message: Some("m".into()),
            detail: Detail::Sample { profile: "light".into(), reverse: false, weight: "bold".into(), time },
        }
    }

    #[test]
    fn diagnostics_serialize_in_key_order() {
        let d = sample(Code::MorphTopology, 13.0 / 60.0);
        assert_eq!(
            to_js_json(&d.to_json()),
            r#"{"severity":"error","code":"MORPH_TOPOLOGY","icon":"a","part":"p","message":"m","profile":"light","reverse":false,"weight":"bold","time":0.21666666666666667}"#
        );
        let bare = Diagnostic { icon: None, part: Some(String::new()), message: None, detail: Detail::None, ..d };
        assert_eq!(to_js_json(&bare.to_json()), r#"{"severity":"error","code":"MORPH_TOPOLOGY"}"#);
    }

    #[test]
    fn dedup_keys_ignore_time_and_message() {
        let mut other = sample(Code::MorphTopology, 1.0);
        other.message = None;
        assert_eq!(sample(Code::MorphTopology, 0.5).dedup_key(), other.dedup_key());
        assert_eq!(other.dedup_key(), r#"["error","MORPH_TOPOLOGY","a","p","light",false,"bold"]"#);
        assert_ne!(sample(Code::Visibility, 0.5).dedup_key(), other.dedup_key());
    }

    #[test]
    fn checked_reports_overwrite_status_in_place() {
        let report = SetReport { set: "s".into(), report: Report::new(1, 2, 3, vec![]), inputs: vec![] };
        let blocked =
            CheckedReport { report: SetReport { report: Report::empty_set(), ..report.clone() }, launch: None };
        assert_eq!(
            to_js_json(&blocked.to_json()),
            r#"{"set":"s","status":"FAIL","icons":0,"actions":0,"samples":0,"diagnostics":[{"severity":"error","code":"EMPTY_SET","message":"Export a nonempty icons array"}],"inputs":[],"built":false}"#
        );
        let spawn = CheckedReport { report: report.clone(), launch: Some(LaunchOutcome::not_started("spawn ENOENT")) };
        assert_eq!(
            to_js_json(&spawn.to_json()),
            r#"{"set":"s","status":"FAIL","icons":1,"actions":2,"samples":3,"diagnostics":[],"inputs":[],"built":false,"buildExitCode":null,"error":"spawn ENOENT"}"#
        );
        let built = CheckedReport { report, launch: Some(LaunchOutcome::built()) };
        assert_eq!(built.status(), Status::Pass);
        assert!(built.to_json().to_string().ends_with(r#""built":true,"buildExitCode":0}"#));
    }

    #[test]
    fn runs_print_at_indent_two() {
        let run = Run { reports: vec![], skipped: vec!["01-heart-bell".into()] };
        assert_eq!(
            run.stdout(),
            "{\n  \"status\": \"PASS\",\n  \"scope\": \"Source screening only; no native export or visual review\",\n  \"reports\": [],\n  \"skipped\": [\n    {\n      \"set\": \"01-heart-bell\",\n      \"reason\": \"Legacy inline builder; existing native/source checks required\"\n    }\n  ]\n}\n"
        );
    }
}
