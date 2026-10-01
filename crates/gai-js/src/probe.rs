//! `probe()`: one authored (unexpanded) part evaluated at chosen times, unrounded, for
//! checks such as preflight and motion measurement (js-runtime §7.2).

use crate::{describe::Callbacks, runtime::Fail};
use serde_json::Value as Json;

/// What to evaluate.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeRequest<'a> {
    /// Index into [`Description::icons`](crate::Description::icons), or into the
    /// definitions' `icons` array when the set is not installed.
    pub icon: usize,
    /// Index into the icon's authored `parts`.
    pub part: usize,
    pub strong: bool,
    pub reverse: bool,
    /// The 4th argument of `d`.
    pub width: ProbeWidth,
    pub times: &'a [f64],
}

/// The width passed to `d`, which always receives 4 arguments.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProbeWidth {
    /// `part.width` as authored (possibly `undefined`).
    Authored,
    Value(f64),
}

/// Every requested time of one probed part.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeResult {
    pub callbacks: Callbacks,
    /// `typeof part.d === 'string'`: preflight applies the transform only then, while
    /// measure-motion applies it to every part that has one.
    pub static_d: bool,
    /// One per requested time.
    pub samples: Vec<ProbeSample>,
}

/// One time, evaluated in preflight's order: `d`, geometry, visibility, `pose` for each
/// point pair, then `pose.transform`. Evaluation stops at the first failure, which
/// [`outcome`](Self::outcome) reports; the fields before it are filled.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSample {
    pub time: f64,
    pub outcome: ProbeOutcome,
    /// preflight's `parseGeometry` result.
    pub geometry: Option<ProbeGeometry>,
    pub visibility: Option<ProbeVisibility>,
    /// One per point pair evaluated (all of them unless `pose` threw).
    pub posed: Vec<PosedPoint>,
    /// Present when `part.pose.transform` exists and returned an object.
    pub transform: Option<ProbeTransform>,
}

/// How far one time's evaluation got.
#[derive(Debug, Clone, PartialEq)]
pub enum ProbeOutcome {
    Complete,
    /// `parseGeometry` refused the path with `message` (preflight's wording). `svg_error`
    /// is svgpath's own parse error, empty when the path parsed.
    InvalidGeometry {
        message: String,
        svg_error: String,
    },
    /// A callback threw. `message` is `error.message` as `JSON.stringify` would carry it
    /// (`None` when it is `undefined`, for example after `throw 'x'`).
    Threw {
        stage: ProbeStage,
        message: Option<Json>,
    },
    /// A callback threw `null` or `undefined`, whose `.message` cannot be read.
    ThrewNullish {
        stage: ProbeStage,
    },
    /// `pose.transform()` returned `null` (`true`) or `undefined` (`false`).
    TransformReturnedNullish {
        null: bool,
    },
}

/// The callback that threw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeStage {
    D,
    Visibility,
    Pose,
    Transform,
}

/// The path at one time, normalized by preflight's `parseGeometry`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeGeometry {
    /// Command letters after normalization, for example `MCCZ` (`H`/`V` become `L`).
    pub topology: String,
    /// The builder's point sequence: `H`/`V` expand to a pair, `Z` adds none.
    pub points: Vec<f64>,
}

/// `part.visibility(t, strong, reverse)`, or 1 without one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProbeVisibility {
    /// The value after ToNumber (`NaN` for a BigInt or Symbol).
    pub value: f64,
    /// `typeof value === 'number'` (preflight's `Number.isFinite` does not coerce).
    pub is_number: bool,
    /// `value == null` (measure-motion's `?? 1` treats it as 1).
    pub nullish: bool,
}

/// The pose of one point pair: `pose(x, y, t, strong, reverse)`, or the pair itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PosedPoint {
    /// `NaN` unless `ok`.
    pub x: f64,
    pub y: f64,
    /// The pose returned an array of exactly two finite numbers.
    pub ok: bool,
}

/// `pose.transform(t, strong, reverse)`, read with preflight's and measure-motion's defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeTransform {
    /// `[dx ?? 0, dy ?? 0, angle ?? 0]` after ToNumber (preflight).
    pub nullish: [f64; 3],
    /// `pivot ?? [16, 16]` when that is a pair of finite numbers.
    pub pivot: Option<[f64; 2]>,
    /// The pivot is such a pair and all of `nullish` are finite numbers without coercion.
    pub ok: bool,
    /// `JSON.stringify(pivot ?? [16, 16])` in QuickJS, for equality between times only.
    pub pivot_json: String,
    /// `[angle || 0, dx || 0, dy || 0]` after ToNumber (measure-motion).
    pub or: [f64; 3],
}

/// The host's packed arrays for one probe call.
pub(crate) struct Packed {
    pub callbacks: Callbacks,
    pub static_d: bool,
    pub status: Vec<i32>,
    pub stage: Vec<u8>,
    pub messages: Vec<Option<Json>>,
    pub geo_error: Vec<String>,
    pub svg_err: Vec<String>,
    pub topology: Vec<i32>,
    pub topologies: Vec<String>,
    pub point_offsets: Vec<i32>,
    pub points: Vec<f64>,
    pub posed: Vec<f64>,
    pub posed_ok: Vec<u8>,
    pub posed_pairs: Vec<i32>,
    pub visibility: Vec<f64>,
    pub vis_flags: Vec<u8>,
    pub tr_flags: Vec<u8>,
    pub tr_nullish: Vec<f64>,
    pub pivot: Vec<f64>,
    pub pivot_json: Vec<String>,
    pub tr_or: Vec<f64>,
}

impl Packed {
    pub fn unpack(self, times: &[f64]) -> Result<ProbeResult, Fail> {
        let bad = || Fail::Protocol("probe(): malformed result".into());
        let count = times.len();
        let lengths_ok =
            [self.status.len(), self.stage.len(), self.messages.len(), self.geo_error.len(), self.svg_err.len()]
                .into_iter()
                .chain([self.topology.len(), self.posed_pairs.len(), self.visibility.len(), self.vis_flags.len()])
                .chain([self.tr_flags.len(), self.pivot_json.len()])
                .all(|len| len == count)
                && self.point_offsets.len() == count + 1
                && self.posed.len() == self.points.len()
                && self.posed_ok.len() * 2 == self.points.len()
                && [self.tr_nullish.len(), self.tr_or.len()].iter().all(|len| *len == 3 * count)
                && self.pivot.len() == 2 * count;
        if !lengths_ok {
            return Err(bad());
        }
        let stage = |code: u8| match code {
            1 => Ok(ProbeStage::D),
            3 => Ok(ProbeStage::Visibility),
            4 => Ok(ProbeStage::Pose),
            5 => Ok(ProbeStage::Transform),
            _ => Err(bad()),
        };
        let mut samples = Vec::with_capacity(count);
        for k in 0..count {
            let outcome = match self.status[k] {
                0 => ProbeOutcome::Complete,
                1 => ProbeOutcome::Threw { stage: stage(self.stage[k])?, message: self.messages[k].clone() },
                2 => ProbeOutcome::ThrewNullish { stage: stage(self.stage[k])? },
                3 => ProbeOutcome::TransformReturnedNullish { null: false },
                4 => ProbeOutcome::TransformReturnedNullish { null: true },
                5 => ProbeOutcome::InvalidGeometry {
                    message: self.geo_error[k].clone(),
                    svg_error: self.svg_err[k].clone(),
                },
                _ => return Err(bad()),
            };
            let range = |from: i32, to: i32| {
                let (from, to) = (usize::try_from(from).map_err(|_| bad())?, usize::try_from(to).map_err(|_| bad())?);
                (from <= to && to <= self.points.len()).then_some(from..to).ok_or_else(bad)
            };
            let span = range(self.point_offsets[k], self.point_offsets[k + 1])?;
            let geometry = match usize::try_from(self.topology[k]) {
                Ok(index) => Some(ProbeGeometry {
                    topology: self.topologies.get(index).ok_or_else(bad)?.clone(),
                    points: self.points[span.clone()].to_vec(),
                }),
                Err(_) => None,
            };
            let flags = self.vis_flags[k];
            let visibility = (flags & 1 == 1).then(|| ProbeVisibility {
                value: self.visibility[k],
                is_number: flags & 2 == 2,
                nullish: flags & 4 == 4,
            });
            let pairs = usize::try_from(self.posed_pairs[k]).map_err(|_| bad())?;
            if 2 * pairs > span.len() {
                return Err(bad());
            }
            let posed = (0..pairs)
                .map(|p| {
                    let at = span.start + 2 * p;
                    PosedPoint { x: self.posed[at], y: self.posed[at + 1], ok: self.posed_ok[at / 2] == 1 }
                })
                .collect();
            let tr = self.tr_flags[k];
            let transform = (tr & 2 == 2).then(|| {
                let three = |v: &[f64]| [v[3 * k], v[3 * k + 1], v[3 * k + 2]];
                let pivot = [self.pivot[2 * k], self.pivot[2 * k + 1]];
                ProbeTransform {
                    nullish: three(&self.tr_nullish),
                    pivot: pivot.iter().all(|v| v.is_finite()).then_some(pivot),
                    ok: tr & 4 == 4,
                    pivot_json: self.pivot_json[k].clone(),
                    or: three(&self.tr_or),
                }
            });
            samples.push(ProbeSample { time: times[k], outcome, geometry, visibility, posed, transform });
        }
        Ok(ProbeResult { callbacks: self.callbacks, static_d: self.static_d, samples })
    }
}
