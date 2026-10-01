//! `library measure <set>… | --all`: `measure-motion.mjs` for sets 22–39. Samples each
//! icon's main part through the gai-js probe (unexpanded parts, unrounded, `width` 2),
//! hands the samples to `gai_check::measure`, and writes
//! `evidence/motion-measurements.json`. Sets are measured in parallel; the report keeps
//! set order. Evidence only: icons whose Strong is not clearly stronger than Light are
//! listed for review at native speed, and nothing fails on them.

use super::{resolve_set, set_folders, sets_dir};
use crate::cli::LibraryMeasureArgs as MeasureArgs;
use crate::error::{CliError, Result};
use crate::output::Report;
use gai_check::measure::{
    EVIDENCE_FILE, MeasureError, PartProbe, ProbeError, ProbeIcon, ProbeSource, builtin_spec, measure_motion,
};
use gai_js::rquickjs::{Object, Value as Js};
use gai_js::{Limits, ProbeOutcome, ProbeRequest, ProbeWidth, SetRuntime, SetSource};
use serde_json::json;
use std::fmt::Write;
use std::path::Path;

/// A set's runtime as a [`ProbeSource`]: the definitions are loaded but not installed, so
/// the probe sees the authored parts without the builder's weight expansion.
struct RuntimeProbe<'a> {
    runtime: &'a SetRuntime,
}

impl ProbeSource for RuntimeProbe<'_> {
    fn icons(&mut self) -> std::result::Result<Vec<ProbeIcon>, ProbeError> {
        self.runtime.with_icons(|scope| probe_icons(scope.icons())).map_err(ProbeError::new)
    }

    fn probe(
        &mut self,
        icon: usize,
        part: usize,
        strong: bool,
        times: &[f64],
    ) -> std::result::Result<PartProbe, ProbeError> {
        let request = ProbeRequest { icon, part, strong, reverse: false, width: ProbeWidth::Value(2.0), times };
        let result = self.runtime.probe(&request).map_err(ProbeError::new)?;
        let mut probe = PartProbe {
            transform: result.callbacks.transform.then(|| Vec::with_capacity(times.len())),
            ..PartProbe::default()
        };
        for sample in &result.samples {
            if sample.outcome != ProbeOutcome::Complete {
                return Err(ProbeError::new(format!(
                    "icon {icon} part {part} at {} s: {:?}",
                    gai_core::js_number_to_string(sample.time),
                    sample.outcome
                )));
            }
            probe.points.push(sample.posed.iter().flat_map(|point| [point.x, point.y]).collect());
            // `part.visibility?.(t, strong, false) ?? 1`.
            probe.alpha.push(sample.visibility.filter(|v| !v.nullish).map_or(1.0, |v| v.value));
            if let Some(transform) = &mut probe.transform {
                let or = sample.transform.as_ref().map(|t| t.or).ok_or_else(|| {
                    ProbeError::new(format!("icon {icon} part {part}: pose.transform returned no object"))
                })?;
                transform.push(or);
            }
        }
        Ok(probe)
    }
}

/// `{name, intent, timings.{light,strong}.active, parts[].name}` of each icon.
fn probe_icons(icons: &Js<'_>) -> gai_js::rquickjs::Result<Vec<ProbeIcon>> {
    let text = |object: &Object<'_>, key: &str| -> gai_js::rquickjs::Result<Option<String>> {
        object.get::<_, Js>(key)?.as_string().map(|s| s.to_string()).transpose()
    };
    let active = |icon: &Object<'_>, profile: &str| -> gai_js::rquickjs::Result<Option<f64>> {
        let Some(timings) = icon.get::<_, Js>("timings")?.into_object() else { return Ok(None) };
        let Some(timing) = timings.get::<_, Js>(profile)?.into_object() else { return Ok(None) };
        Ok(timing.get::<_, Js>("active")?.as_number())
    };
    let mut out = Vec::new();
    let Some(icons) = icons.as_array() else { return Ok(out) };
    for icon in icons.iter::<Js>() {
        let Some(icon) = icon?.into_object() else { continue };
        let mut parts = Vec::new();
        if let Some(list) = icon.get::<_, Js>("parts")?.into_array() {
            for part in list.iter::<Js>() {
                let name = part?.into_object().map(|part| text(&part, "name")).transpose()?.flatten();
                parts.push(name.unwrap_or_default());
            }
        }
        out.push(ProbeIcon {
            name: text(&icon, "name")?.unwrap_or_default(),
            intent: text(&icon, "intent")?,
            active_seconds: [active(&icon, "light")?, active(&icon, "strong")?],
            parts,
        });
    }
    Ok(out)
}

/// One set's measurement: the evidence bytes and the icons flagged for native review.
struct Measured {
    json: String,
    review: Vec<String>,
}

/// Measures the set in `dir` (on the calling thread, which must be a gai-js worker).
fn measure_set(dir: &Path, set: &str) -> std::result::Result<Measured, String> {
    let spec = builtin_spec(set).ok_or_else(|| format!("{set} has no motion measurement"))?;
    let definitions = std::fs::read(dir.join("definitions.mjs")).map_err(|error| error.to_string())?;
    let source = SetSource::library_set(dir).map_err(|error| error.to_string())?;
    let limits = Limits::default();
    let runtime = SetRuntime::load(&source, &limits).map_err(|error| error.to_string())?;
    let measured = measure_motion(&spec, &definitions, &mut RuntimeProbe { runtime: &runtime })
        .map_err(|error: MeasureError| error.to_string())?;
    let review = measured.review_at_native_speed().map(|record| record.icon.clone()).collect();
    Ok(Measured { json: measured.to_json(), review })
}

/// Runs `library measure`.
pub fn run(repo: &Path, args: &MeasureArgs) -> Result<Report> {
    let sets = match (args.all, args.sets.is_empty()) {
        (true, true) => set_folders(repo)?.into_iter().filter(|set| builtin_spec(set).is_some()).collect(),
        (true, false) => return Err(CliError::usage("Name sets or pass --all, not both")),
        (false, true) => return Err(CliError::usage("Name at least one set, or pass --all")),
        (false, false) => {
            let sets = args.sets.iter().map(|name| resolve_set(repo, name)).collect::<Result<Vec<_>>>()?;
            if let Some(set) = sets.iter().find(|set| builtin_spec(set).is_none()) {
                return Err(CliError::usage(format!(
                    "{set} has no motion measurement; sets 22 to 39 have one (18 to 21 kept their own scripts)"
                )));
            }
            sets
        }
    };
    let limits = Limits::default();
    let jobs = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    let outcomes = gai_js::map_parallel(sets.clone(), jobs, &limits, |set| {
        let dir = sets_dir(repo).join(&set);
        measure_set(&dir, &set).and_then(|measured| {
            crate::managed::write_inside(&dir, EVIDENCE_FILE, measured.json.as_bytes())
                .map_err(|error| error.message)?;
            Ok(measured)
        })
    });
    let mut results = Vec::new();
    let mut text = String::new();
    let mut failed = false;
    for (set, outcome) in sets.iter().zip(outcomes) {
        let evidence = format!("motion/icon-sets/{set}/{EVIDENCE_FILE}");
        match outcome {
            Ok(measured) => {
                let _ = writeln!(text, "{set}: wrote {evidence}");
                if !measured.review.is_empty() {
                    let _ = writeln!(text, "  review at native speed: {}", measured.review.join(", "));
                }
                results.push(json!({ "set": set, "status": "PASS", "evidence": evidence, "reviewAtNativeSpeed": measured.review }));
            }
            Err(error) => {
                failed = true;
                let _ = writeln!(text, "FAIL {set}: {error}");
                results.push(json!({ "set": set, "status": "FAIL", "error": error }));
            }
        }
    }
    let value = json!({ "status": if failed { "FAIL" } else { "PASS" }, "sets": results });
    Ok(Report { failed, ..Report::new(value, text) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCode;

    #[test]
    fn only_sets_with_a_spec_are_measured() {
        let dir = tempfile::tempdir().unwrap();
        for set in ["16-voice", "23-get-help"] {
            std::fs::create_dir_all(dir.path().join("motion/icon-sets").join(set)).unwrap();
        }
        let args = |sets: &[&str], all| MeasureArgs { sets: sets.iter().map(|s| s.to_string()).collect(), all };
        let error = run(dir.path(), &args(&["16"], false)).unwrap_err();
        assert_eq!(
            error.message,
            "16-voice has no motion measurement; sets 22 to 39 have one (18 to 21 kept their own scripts)"
        );
        assert_eq!(run(dir.path(), &args(&[], false)).unwrap_err().code, ErrorCode::Usage);
        // --all takes 23 only; it has no definitions here, so its measurement fails.
        let report = run(dir.path(), &args(&[], true)).unwrap();
        assert!(report.failed);
        assert_eq!(report.value["sets"].as_array().unwrap().len(), 1);
        assert_eq!(
            (report.value["sets"][0]["set"].as_str(), report.value["sets"][0]["status"].as_str()),
            (Some("23-get-help"), Some("FAIL"))
        );
    }
}
