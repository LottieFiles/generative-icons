//! Source motion measurement (`measure-motion.mjs`, sets 22–39): RMS displacement and
//! speed of each icon's main part at 60 fps, per profile, and the Strong-to-Light ratios.
//! Evidence only, never a gate; the one output besides the file is the
//! "review at native speed" flag list.
//!
//! The authoring functions (`part.d`, `part.pose`, `part.visibility`, `pose.transform`)
//! stay in JavaScript behind [`ProbeSource`]; everything after sampling (RMS, sums,
//! extrema, ratios, JSON) is here. Values are unrounded, so byte parity needs a probe
//! with V8-exact math.

use crate::contracts::evidence::{evidence_json, js_number};
use crate::contracts::jsnum::{max_of, max2, min_of, sum};
use serde::Deserialize;
use serde_json::{Map, Value};

/// Where measurement evidence lives, relative to the set folder.
pub const EVIDENCE_FILE: &str = "evidence/motion-measurements.json";

/// The fixed `scope` sentence of every report.
pub const SCOPE: &str = "Source geometry and rigid transforms at 60fps; grid units and seconds. RMS includes curve control points. Native normal-speed review required.";

/// Strong must beat Light by this factor on peak speed or on travel, or the icon is
/// flagged for side-by-side review at native speed.
pub const STRONG_TO_LIGHT_REVIEW_RATIO: f64 = 1.5;

/// Frames measured per profile: `f = 0..=144` at `t = f/60`.
const FRAMES: u32 = 145;

/// A failure to sample authoring code.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ProbeError(Box<dyn std::error::Error + Send + Sync>);

impl ProbeError {
    /// Wraps any error or message.
    pub fn new(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> ProbeError {
        ProbeError(error.into())
    }
}

/// Measurement failures.
#[derive(Debug, thiserror::Error)]
pub enum MeasureError {
    /// The probe failed.
    #[error(transparent)]
    Probe(#[from] ProbeError),
    /// An icon has no part named `<icon>-<primary>` (the JS throws the name).
    #[error("{0}")]
    MissingPart(String),
    /// The main part's point count changed between frames.
    #[error("Topology changed {0}")]
    TopologyChanged(String),
    /// The probe returned the wrong number of samples.
    #[error("probe returned {got} samples for {part}, expected {expected}")]
    SampleCount {
        /// Part name.
        part: String,
        /// Samples requested.
        expected: usize,
        /// Samples returned.
        got: usize,
    },
}

/// One icon as the definitions describe it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeIcon {
    /// `icon.name`.
    pub name: String,
    /// `icon.intent`; `None` when undefined (the key is then omitted).
    pub intent: Option<String>,
    /// `icon.timings.light.active` and `icon.timings.strong.active`; `None` when undefined.
    pub active_seconds: [Option<f64>; 2],
    /// Names of the icon's **unexpanded** source parts, in definition order.
    pub parts: Vec<String>,
}

/// One source part sampled at a list of times.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PartProbe {
    /// Posed flat coordinates per time: every pair of `svgpath(d).abs().unarc().unshort()`,
    /// `H`/`V` expanded against the previous un-posed point, `Z` skipped, each pair through
    /// `pose(x, y, t, strong, false)`. Control points included.
    pub points: Vec<Vec<f64>>,
    /// `part.visibility?.(t, strong, false) ?? 1` per time.
    pub alpha: Vec<f64>,
    /// `[angle || 0, dx || 0, dy || 0]` of `pose.transform(t, strong, false)` per time,
    /// when the part has a transform.
    pub transform: Option<Vec<[f64; 3]>>,
}

/// Access to a set's authoring code, evaluated in a fresh context (the builder's
/// weight expansion must not be visible).
pub trait ProbeSource {
    /// Icons in definitions order.
    fn icons(&mut self) -> Result<Vec<ProbeIcon>, ProbeError>;
    /// Samples source part `part` of icon `icon` at `times`, with `reverse = false`
    /// and `width = 2`.
    fn probe(
        &mut self,
        icon: usize,
        part: usize,
        strong: bool,
        times: &[f64],
    ) -> Result<PartProbe, ProbeError>;
}

/// Which speeds the Strong-to-Light ratios compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ratios {
    /// Raw speeds (22–26); the visible figures are not reported.
    Raw,
    /// Speeds only where the part is visible (27–39), also reported as
    /// `visibleTravelRms`/`visiblePeakRmsSpeed`.
    Visible,
}

/// How a set is measured.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasureSpec {
    /// Ratio variant.
    pub ratios: Ratios,
    /// A literal for the report's `icons` field (22 writes 40); otherwise the icon count.
    #[serde(default)]
    pub icons: Option<u64>,
    /// Main part suffix per icon, index-aligned with the definitions: `<icon>-<primary>`.
    pub primary: Vec<String>,
}

/// The built-in specs of sets 22–39, extracted from their `measure-motion.mjs`.
const BUILTIN_SPECS: &str = include_str!("../../rules/measure-motion.json");

/// The built-in measurement spec for a set id, if it has one.
pub fn builtin_spec(set_id: &str) -> Option<MeasureSpec> {
    let mut all: serde_json::Map<String, Value> =
        serde_json::from_str(BUILTIN_SPECS).expect("built-in measure specs parse");
    all.remove(set_id)
        .map(|v| serde_json::from_value(v).expect("built-in measure spec parses"))
}

/// Metrics of one profile.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileMetrics {
    /// `icon.timings[profile].active`.
    pub active_seconds: Option<f64>,
    /// First frame time whose RMS distance from the start exceeds 0.01 or whose alpha
    /// moved by more than 0.01.
    pub first_change_seconds: Option<f64>,
    /// Peak RMS distance from the first frame.
    pub peak_rms_displacement: f64,
    /// Sum of per-frame RMS speeds / 60.
    pub travel_rms: f64,
    /// Peak per-frame RMS speed (units per second).
    pub peak_rms_speed: f64,
    /// Travel counting only frames where the part is visible.
    pub visible_travel_rms: f64,
    /// Peak speed counting only frames where the part is visible.
    pub visible_peak_rms_speed: f64,
    /// Time from 10% to 90% of the peak displacement (NaN when never reached).
    pub first10_to90_seconds: f64,
    /// Rotation, horizontal and vertical ranges of the rigid transform, if any.
    pub ranges: Option<[f64; 3]>,
}

/// One icon's record.
#[derive(Debug, Clone, PartialEq)]
pub struct MotionRecord {
    /// Icon name.
    pub icon: String,
    /// The measured part, `<icon>-<primary>`.
    pub main_part: String,
    /// `icon.intent`.
    pub intent: Option<String>,
    /// Light profile.
    pub light: ProfileMetrics,
    /// Strong profile.
    pub strong: ProfileMetrics,
    /// Strong over Light peak speed (visible or raw per [`Ratios`]).
    pub strong_to_light_peak_speed: f64,
    /// Strong over Light travel (visible or raw per [`Ratios`]).
    pub strong_to_light_travel: f64,
}

impl MotionRecord {
    /// Strong is not clearly stronger than Light on either axis. Computed from the live
    /// values: NaN and +∞ ratios never qualify (they would read as `null` from the file).
    pub fn needs_native_review(&self) -> bool {
        self.strong_to_light_peak_speed < STRONG_TO_LIGHT_REVIEW_RATIO
            && self.strong_to_light_travel < STRONG_TO_LIGHT_REVIEW_RATIO
    }
}

/// A set's `motion-measurements.json`.
#[derive(Debug, Clone, PartialEq)]
pub struct MotionMeasurements {
    /// Ratio variant (decides whether visible figures are written).
    pub ratios: Ratios,
    /// sha256 of the raw `definitions.mjs` bytes.
    pub definitions_sha256: String,
    /// The `icons` field.
    pub icons: u64,
    /// One record per icon, in definitions order.
    pub records: Vec<MotionRecord>,
}

/// Measures every icon's main part.
pub fn measure_motion(
    spec: &MeasureSpec,
    definitions: &[u8],
    probe: &mut dyn ProbeSource,
) -> Result<MotionMeasurements, MeasureError> {
    let icons = probe.icons()?;
    // `points(part, 0)` is its own call, then one sample per frame.
    let times: Vec<f64> = std::iter::once(0.0)
        .chain((0..FRAMES).map(|f| f64::from(f) / 60.0))
        .collect();
    let mut records = Vec::with_capacity(icons.len());
    for (i, icon) in icons.iter().enumerate() {
        let primary = spec.primary.get(i).map_or("undefined", String::as_str);
        let name = format!("{}-{primary}", icon.name);
        let part = icon
            .parts
            .iter()
            .position(|p| *p == name)
            .ok_or_else(|| MeasureError::MissingPart(name.clone()))?;
        let mut profiles = Vec::with_capacity(2);
        for (strong, active) in [
            (false, icon.active_seconds[0]),
            (true, icon.active_seconds[1]),
        ] {
            let samples = probe.probe(i, part, strong, &times)?;
            profiles.push(profile_metrics(&name, &samples, times.len(), active)?);
        }
        let strong = profiles.pop().expect("two profiles");
        let light = profiles.pop().expect("two profiles");
        let (peak, travel) = match spec.ratios {
            Ratios::Raw => (
                strong.peak_rms_speed / light.peak_rms_speed,
                strong.travel_rms / light.travel_rms,
            ),
            Ratios::Visible => (
                strong.visible_peak_rms_speed / light.visible_peak_rms_speed,
                strong.visible_travel_rms / light.visible_travel_rms,
            ),
        };
        records.push(MotionRecord {
            icon: icon.name.clone(),
            main_part: name,
            intent: icon.intent.clone(),
            light,
            strong,
            strong_to_light_peak_speed: peak,
            strong_to_light_travel: travel,
        });
    }
    Ok(MotionMeasurements {
        ratios: spec.ratios,
        definitions_sha256: gai_core::sha256_hex(definitions),
        icons: spec.icons.unwrap_or(icons.len() as u64),
        records,
    })
}

/// `sqrt(Σ (a_i - b_i)² / (a.length / 2))`, summed in index order from +0.
fn rms(a: &[f64], b: &[f64]) -> f64 {
    let total = sum(a.iter().enumerate().map(|(i, v)| {
        let d = v - b.get(i).copied().unwrap_or(f64::NAN);
        d * d
    }));
    (total / (a.len() as f64 / 2.0)).sqrt()
}

fn profile_metrics(
    part: &str,
    probe: &PartProbe,
    expected: usize,
    active: Option<f64>,
) -> Result<ProfileMetrics, MeasureError> {
    let count_ok = probe.points.len() == expected
        && probe.alpha.len() == expected
        && probe.transform.as_ref().is_none_or(|t| t.len() == expected);
    if !count_ok {
        return Err(MeasureError::SampleCount {
            part: part.to_string(),
            expected,
            got: probe.points.len(),
        });
    }
    let first = &probe.points[0];
    let frames = &probe.points[1..];
    let alpha = &probe.alpha[1..];
    if frames.iter().any(|p| p.len() != first.len()) {
        return Err(MeasureError::TopologyChanged(part.to_string()));
    }
    let time = |f: usize| f as f64 / 60.0;
    let distances: Vec<f64> = frames.iter().map(|p| rms(first, p)).collect();
    let speeds: Vec<f64> = frames
        .windows(2)
        .map(|w| rms(&w[0], &w[1]) * 60.0)
        .collect();
    // Invisible position wraps are implementation resets, not perceived speed.
    let visible_speeds: Vec<f64> = speeds
        .iter()
        .enumerate()
        .map(|(j, &v)| {
            if max2(alpha[j], alpha[j + 1]) > 0.01 {
                v
            } else {
                0.0
            }
        })
        .collect();
    let peak = max_of(distances.iter().copied());
    let first_change = (0..frames.len())
        .find(|&f| distances[f] > 0.01 || (alpha[f] - alpha[0]).abs() > 0.01)
        .map(time);
    let reach = |q: f64| {
        (0..frames.len())
            .find(|&f| distances[f] >= peak * q)
            .map_or(f64::NAN, time)
    };
    let (first10, first90) = (reach(0.1), reach(0.9));
    let ranges = probe.transform.as_ref().map(|tr| {
        let tr = &tr[1..];
        let range = |k: usize| max_of(tr.iter().map(|v| v[k])) - min_of(tr.iter().map(|v| v[k]));
        [range(0), range(1), range(2)]
    });
    Ok(ProfileMetrics {
        active_seconds: active,
        first_change_seconds: first_change,
        peak_rms_displacement: peak,
        travel_rms: sum(speeds.iter().copied()) / 60.0,
        peak_rms_speed: max_of(speeds.iter().copied()),
        visible_travel_rms: sum(visible_speeds.iter().copied()) / 60.0,
        visible_peak_rms_speed: max_of(visible_speeds.iter().copied()),
        first10_to90_seconds: first90 - first10,
        ranges,
    })
}

impl MotionMeasurements {
    /// The records flagged for side-by-side review at native speed.
    pub fn review_at_native_speed(&self) -> impl Iterator<Item = &MotionRecord> {
        self.records.iter().filter(|r| r.needs_native_review())
    }

    /// The evidence object, keys in output order.
    pub fn to_value(&self) -> Value {
        let metrics = |m: &ProfileMetrics| {
            let mut o = Map::new();
            if let Some(a) = m.active_seconds {
                o.insert("activeSeconds".into(), js_number(a));
            }
            o.insert(
                "firstChangeSeconds".into(),
                m.first_change_seconds.map_or(Value::Null, js_number),
            );
            o.insert(
                "peakRmsDisplacement".into(),
                js_number(m.peak_rms_displacement),
            );
            o.insert("travelRms".into(), js_number(m.travel_rms));
            o.insert("peakRmsSpeed".into(), js_number(m.peak_rms_speed));
            if self.ratios == Ratios::Visible {
                o.insert("visibleTravelRms".into(), js_number(m.visible_travel_rms));
                o.insert(
                    "visiblePeakRmsSpeed".into(),
                    js_number(m.visible_peak_rms_speed),
                );
            }
            o.insert(
                "first10To90Seconds".into(),
                js_number(m.first10_to90_seconds),
            );
            if let Some([rotation, horizontal, vertical]) = m.ranges {
                o.insert("rotationRange".into(), js_number(rotation));
                o.insert("horizontalRange".into(), js_number(horizontal));
                o.insert("verticalRange".into(), js_number(vertical));
            }
            Value::Object(o)
        };
        let records = self
            .records
            .iter()
            .map(|r| {
                let mut o = Map::new();
                o.insert("icon".into(), r.icon.clone().into());
                o.insert("mainPart".into(), r.main_part.clone().into());
                if let Some(intent) = &r.intent {
                    o.insert("intent".into(), intent.clone().into());
                }
                let mut profiles = Map::new();
                profiles.insert("light".into(), metrics(&r.light));
                profiles.insert("strong".into(), metrics(&r.strong));
                o.insert("metrics".into(), Value::Object(profiles));
                o.insert(
                    "strongToLightPeakSpeed".into(),
                    js_number(r.strong_to_light_peak_speed),
                );
                o.insert(
                    "strongToLightTravel".into(),
                    js_number(r.strong_to_light_travel),
                );
                Value::Object(o)
            })
            .collect();
        let mut report = Map::new();
        report.insert("scope".into(), SCOPE.into());
        report.insert(
            "definitionsSha256".into(),
            self.definitions_sha256.clone().into(),
        );
        report.insert("icons".into(), self.icons.into());
        report.insert("records".into(), Value::Array(records));
        Value::Object(report)
    }

    /// The evidence file bytes: `JSON.stringify(report, null, 2) + "\n"`.
    pub fn to_json(&self) -> String {
        evidence_json(&self.to_value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A two-point part sliding right at a per-profile speed, hidden after 2 s.
    struct Slide {
        speeds: [f64; 2],
        transform: bool,
    }

    impl ProbeSource for Slide {
        fn icons(&mut self) -> Result<Vec<ProbeIcon>, ProbeError> {
            Ok(vec![ProbeIcon {
                name: "cup".into(),
                intent: None,
                active_seconds: [Some(0.8), None],
                parts: vec!["cup-rim".into(), "cup-steam".into()],
            }])
        }

        fn probe(
            &mut self,
            icon: usize,
            part: usize,
            strong: bool,
            times: &[f64],
        ) -> Result<PartProbe, ProbeError> {
            assert_eq!((icon, part), (0, 1));
            let v = self.speeds[usize::from(strong)];
            Ok(PartProbe {
                points: times
                    .iter()
                    .map(|&t| vec![10.0 + v * t, 4.0, 12.0 + v * t, 4.0])
                    .collect(),
                alpha: times
                    .iter()
                    .map(|&t| if t > 2.0 { 0.0 } else { 1.0 })
                    .collect(),
                transform: self
                    .transform
                    .then(|| times.iter().map(|&t| [t * 10.0, 0.0, -t]).collect()),
            })
        }
    }

    fn spec(ratios: Ratios) -> MeasureSpec {
        MeasureSpec {
            ratios,
            icons: None,
            primary: vec!["steam".into()],
        }
    }

    #[test]
    fn raw_report_layout() {
        let m = measure_motion(
            &spec(Ratios::Raw),
            b"",
            &mut Slide {
                speeds: [1.0, 2.0],
                transform: true,
            },
        )
        .unwrap();
        let v = m.to_value();
        let rec = &v["records"][0];
        assert_eq!(
            rec.as_object().unwrap().keys().collect::<Vec<_>>(),
            [
                "icon",
                "mainPart",
                "metrics",
                "strongToLightPeakSpeed",
                "strongToLightTravel"
            ]
        );
        let light = rec["metrics"]["light"].as_object().unwrap();
        assert_eq!(
            light.keys().collect::<Vec<_>>(),
            [
                "activeSeconds",
                "firstChangeSeconds",
                "peakRmsDisplacement",
                "travelRms",
                "peakRmsSpeed",
                "first10To90Seconds",
                "rotationRange",
                "horizontalRange",
                "verticalRange"
            ]
        );
        assert!(
            !rec["metrics"]["strong"]
                .as_object()
                .unwrap()
                .contains_key("activeSeconds")
        );
        assert_eq!(v["definitionsSha256"], gai_core::sha256_hex(b""));
        assert_eq!(v["icons"], 1);
        assert!(!m.records[0].needs_native_review());
    }

    #[test]
    fn visible_ratios_and_flags() {
        let m = measure_motion(
            &spec(Ratios::Visible),
            b"",
            &mut Slide {
                speeds: [1.0, 1.2],
                transform: false,
            },
        )
        .unwrap();
        let r = &m.records[0];
        assert!(r.light.visible_travel_rms < r.light.travel_rms);
        assert!(r.needs_native_review());
        assert!(
            m.to_value()["records"][0]["metrics"]["light"]
                .as_object()
                .unwrap()
                .contains_key("visiblePeakRmsSpeed")
        );
        // A still Light part gives 0/0 and x/0 ratios: written as null, never flagged.
        let still = measure_motion(
            &spec(Ratios::Raw),
            b"",
            &mut Slide {
                speeds: [0.0, 1.0],
                transform: false,
            },
        )
        .unwrap();
        assert!(!still.records[0].needs_native_review());
        assert!(still.to_json().contains("\"strongToLightPeakSpeed\": null"));
        // Nothing moves, so the first change is the fade at frame 121.
        assert_eq!(
            still.records[0].light.first_change_seconds,
            Some(121.0 / 60.0)
        );
    }

    #[test]
    fn missing_part_and_builtin_specs() {
        let bad = MeasureSpec {
            ratios: Ratios::Raw,
            icons: None,
            primary: vec!["lid".into()],
        };
        assert!(
            matches!(measure_motion(&bad, b"", &mut Slide { speeds: [1.0, 2.0], transform: false }), Err(MeasureError::MissingPart(p)) if p == "cup-lid")
        );
        let teamwork = builtin_spec("22-teamwork").unwrap();
        assert_eq!(
            (teamwork.ratios, teamwork.icons, teamwork.primary.len()),
            (Ratios::Raw, Some(40), 40)
        );
        assert_eq!(
            builtin_spec("39-food-drink").unwrap().ratios,
            Ratios::Visible
        );
        assert!(builtin_spec("16-voice").is_none());
    }
}
