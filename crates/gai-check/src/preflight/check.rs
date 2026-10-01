//! `checkDefinitions` (`preflight.mjs:39-126`): the icon and part checks from [`inspect`],
//! then every sampled part probed at every frame and authored time, for each action and
//! outline weight, with the diagnostics deduplicated as `add` did.

use super::{
    contract::FamilyContract,
    error::{Error, Result},
    inspect::{self, Action, IconPlan, Plan, Radius, SampledPart, Weight},
    report::{Code, Detail, Diagnostic, Report},
};
use gai_js::{ProbeOutcome, ProbeRequest, ProbeSample, ProbeStage, SetRuntime};
use serde_json::Value as Json;
use std::collections::HashSet;

/// `preflight.mjs:13`.
const EPSILON: f64 = 1e-8;

/// Runs `checkDefinitions` over a loaded (not installed) runtime.
///
/// Author values are read first, all icons at once; sampling then calls the author's
/// callbacks. JS interleaved the two per icon, which differs only for callbacks that
/// rewrite other icons' definitions.
pub fn check_definitions(runtime: &SetRuntime, contract: &FamilyContract) -> Result<Report> {
    let (count, icons) = match runtime.with_icons(|scope| inspect::inspect(scope, contract))? {
        Plan::EmptySet => return Ok(Report::empty_set()),
        Plan::Icons { count, icons } => (count, icons),
    };
    let mut checker = Checker { runtime, contract, sink: Sink::default(), actions: 0, samples: 0 };
    for (index, icon) in icons.into_iter().enumerate() {
        checker.icon(index, icon)?;
    }
    Ok(Report::new(count, checker.actions, checker.samples, checker.sink.diagnostics))
}

/// `add`: keeps the first diagnostic of each dedup key.
#[derive(Default)]
struct Sink {
    diagnostics: Vec<Diagnostic>,
    seen: HashSet<String>,
}

impl Sink {
    fn add(&mut self, diagnostic: Diagnostic) {
        if self.seen.insert(diagnostic.dedup_key()) {
            self.diagnostics.push(diagnostic);
        }
    }
}

struct Checker<'a> {
    runtime: &'a SetRuntime,
    contract: &'a FamilyContract,
    sink: Sink,
    actions: usize,
    samples: usize,
}

/// One sample and what it belongs to.
struct At<'a> {
    icon: &'a Option<Json>,
    action: &'a Action,
    part: &'a SampledPart,
    weight: &'a Weight,
    sample: &'a ProbeSample,
    /// `typeof part.d === 'string'`: only then is the rigid transform checked.
    static_d: bool,
    /// `part.pose?.transform` is truthy.
    transform: bool,
}

impl At<'_> {
    fn diagnostic(&self, code: Code, message: Option<Json>) -> Diagnostic {
        Diagnostic {
            code,
            icon: self.icon.clone(),
            part: Some(self.part.name.clone()),
            message,
            detail: Detail::Sample {
                profile: self.action.profile.clone(),
                reverse: self.action.reverse,
                weight: self.weight.name.clone(),
                time: self.sample.time,
            },
        }
    }
}

/// The reference topology and pivot of one weight's samples.
#[derive(Default)]
struct Series {
    topology: Option<String>,
    pivot: Option<String>,
}

/// How one sample ended.
enum Step {
    /// Every step completed; the sample counts.
    Counted,
    /// A step threw: `INVALID_GEOMETRY` with this message, and the weight's remaining
    /// times are skipped.
    Failed(Option<Json>),
}

impl Checker<'_> {
    fn icon(&mut self, index: usize, icon: IconPlan) -> Result<()> {
        for check in icon.checks {
            self.sink.add(check);
        }
        let Some(sampling) = icon.sampling else { return Ok(()) };
        for action in &sampling.actions {
            self.actions += 1;
            let times = match &action.times {
                Ok(times) => times,
                Err(code) => {
                    self.sink.add(Diagnostic {
                        code: *code,
                        icon: icon.name.clone(),
                        part: None,
                        message: code.fixed_message().map(Json::from),
                        detail: Detail::Action { profile: action.profile.clone(), reverse: action.reverse },
                    });
                    continue;
                }
            };
            for part in &sampling.parts {
                for weight in &part.weights {
                    // A constant part's first time stands for all of them: the parse cache
                    // the spec allows, with every time still counted.
                    let (probed, repeat) = if part.constant { (&times[..1], times.len()) } else { (&times[..], 1) };
                    let request = ProbeRequest {
                        icon: index,
                        part: part.index,
                        strong: action.strong,
                        reverse: action.reverse,
                        width: weight.width,
                        times: probed,
                    };
                    let probe = self.runtime.probe(&request)?;
                    let mut series = Series::default();
                    for sample in &probe.samples {
                        let at = At {
                            icon: &icon.name,
                            action,
                            part,
                            weight,
                            sample,
                            static_d: probe.static_d,
                            transform: probe.callbacks.transform,
                        };
                        match self.sample(&at, &mut series)? {
                            Step::Counted => self.samples += repeat,
                            Step::Failed(message) => {
                                self.sink.add(at.diagnostic(Code::InvalidGeometry, message));
                                break;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// One sample's steps in `preflight.mjs:96-116` order: geometry, topology, visibility,
    /// pose, rigid transform, control hull.
    fn sample(&mut self, at: &At<'_>, series: &mut Series) -> Result<Step> {
        let sample = at.sample;
        let failed = |message: &str| Ok(Step::Failed(Some(message.into())));
        if let Some(step) = self.threw(at, ProbeStage::D)? {
            return Ok(step);
        }
        if let ProbeOutcome::InvalidGeometry { message, .. } = &sample.outcome {
            return failed(message);
        }
        let geometry = sample.geometry.as_ref().ok_or_else(|| self.malformed(at))?;
        if series.topology.as_ref().is_some_and(|topology| *topology != geometry.topology) {
            self.add(at, Code::MorphTopology);
        }
        series.topology.get_or_insert_with(|| geometry.topology.clone());

        if let Some(step) = self.threw(at, ProbeStage::Visibility)? {
            return Ok(step);
        }
        let visibility = sample.visibility.ok_or_else(|| self.malformed(at))?;
        let v = visibility.value;
        if !visibility.is_number || !v.is_finite() || !(-EPSILON..=1.0 + EPSILON).contains(&v) {
            self.add(at, Code::Visibility);
        }

        // The JS stops at the first pose that is not a finite pair, before any later pose
        // call could throw; the probe poses every point until one throws.
        if sample.posed.iter().any(|point| !point.ok) {
            return failed("Pose must return two finite coordinates");
        }
        if let Some(step) = self.threw(at, ProbeStage::Pose)? {
            return Ok(step);
        }
        if sample.posed.len() * 2 != geometry.points.len() {
            return Err(self.malformed(at));
        }

        if at.static_d && at.transform {
            if let Some(step) = self.threw(at, ProbeStage::Transform)? {
                return Ok(step);
            }
            if let ProbeOutcome::TransformReturnedNullish { null } = sample.outcome {
                let receiver = if null { "null" } else { "undefined" };
                return failed(&format!("Cannot read properties of {receiver} (reading 'pivot')"));
            }
            let transform = sample.transform.as_ref().ok_or_else(|| self.malformed(at))?;
            if !transform.ok {
                return failed("Rigid transform must have a finite pivot, translation and angle");
            }
            // The JS compares JSON.stringify(pivot) between times.
            if series.pivot.as_ref().is_some_and(|pivot| *pivot != transform.pivot_json) {
                self.add(at, Code::MovingPivot);
            }
            series.pivot.get_or_insert_with(|| transform.pivot_json.clone());
        }

        let radius = match at.weight.radius {
            Radius::Number(radius) => radius,
            Radius::Throws(message) => return failed(message),
        };
        // A hole in a posed pair is NaN here, and NaN never crosses, as `some` skipped it.
        let grid = self.contract.grid;
        let crosses = |v: f64| v - radius < -EPSILON || v + radius > grid + EPSILON;
        if v > 0.0 && sample.posed.iter().any(|point| crosses(point.x) || crosses(point.y)) {
            self.add(at, Code::ControlHullMargin);
        }
        Ok(Step::Counted)
    }

    /// Adds a sample diagnostic with its code's fixed message.
    fn add(&mut self, at: &At<'_>, code: Code) {
        self.sink.add(at.diagnostic(code, code.fixed_message().map(Json::from)));
    }

    /// The failure when `stage`'s callback threw: its `message`, or V8's TypeError text when
    /// the JS called something that is not a function (QuickJS words that error differently).
    fn threw(&self, at: &At<'_>, stage: ProbeStage) -> Result<Option<Step>> {
        let part = at.part;
        match &at.sample.outcome {
            ProbeOutcome::Threw { stage: threw, message } if *threw == stage => {
                let not_a_function = match stage {
                    ProbeStage::D => None,
                    ProbeStage::Visibility => (!part.visibility_callable).then_some("part.visibility"),
                    ProbeStage::Pose => (!part.pose_callable).then_some("part.pose"),
                    ProbeStage::Transform => (!part.transform_callable).then_some("part.pose.transform"),
                };
                let message = match not_a_function {
                    Some(callee) => Some(format!("{callee} is not a function").into()),
                    None => message.clone(),
                };
                Ok(Some(Step::Failed(message)))
            }
            ProbeOutcome::ThrewNullish { stage: threw } if *threw == stage => Err(Error::ThrewNullish {
                set: self.runtime.label().to_owned(),
                part: part.name.clone(),
                stage: match stage {
                    ProbeStage::D => "d",
                    ProbeStage::Visibility => "visibility",
                    ProbeStage::Pose => "pose",
                    ProbeStage::Transform => "pose.transform",
                },
            }),
            _ => Ok(None),
        }
    }

    fn malformed(&self, at: &At<'_>) -> Error {
        Error::Probe { set: self.runtime.label().to_owned(), part: at.part.name.clone() }
    }
}
