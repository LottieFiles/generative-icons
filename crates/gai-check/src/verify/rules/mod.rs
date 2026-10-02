//! Drawing rules (`verify.rules.json`, verify.md §8): the per-set checks the legacy
//! `verify.mjs` files carried after the generic checks, as data.
//!
//! The format is the contracts `gai-verify-rules` v2 document (same dialect, margin,
//! rule, expression and evidence vocabulary) extended with what the tails of 42–63 need:
//! more pickers and rule kinds, a style dimension, rule-major order, named counters and
//! aggregates, set asserts, and the placement of the evidence relative to the directed
//! report. The contracts' built-in 16–20 documents run unchanged here and reproduce their
//! evidence; the command-subrange kinds of v1 geometry contracts (`self-gap`,
//! `arrow-curve-gap`, `point-joint`, `attachment`, `opaque`) stay with the clearance stage.
//!
//! Top level (every key but `format` and `version` optional):
//!
//! ```text
//! {"format": "gai-verify-rules", "version": 2, "set": "42-maps-navigation",
//!  "setAsserts": [{"kind": "iconCount"|"uniqueIconIds"|"statefulCount"|"category"|"contractCount",
//!                  "eq"|"min"|"max": …, "message"?: …}],
//!  "dialect": {"sampler": "ge"|"gt", "flatten": {…}, "pick": …, "suffixes"?: […], "partPrefix"?: "icon"|"none",
//!              "visible": {"op": ">="|">", "value": …}, "slack": …, "gapRadius": "sequential"|"summed",
//!              "singlePointDistance"?: bool, "failure": "collect"|"assert", "nanFails"?: bool,
//!              "order"?: "frame-major"|"rule-major"},
//!  "margin": {"nodes": {"nameEndsWith": […]}, "visible": {…}, "method": "box"|"points", "min": …,
//!             "failure": …, "message": "…"},
//!  "requireIcon": bool | "message", "unlisted": "run"|"skip",
//!  "styles": [{"id": "outline", "suffix": "-bold__color.primary__outline.bold"}, …],
//!  "icons": {"<icon>": [RULE, …]},
//!  "evidence": {"stage": "beforeDirected"|"afterDirected", "file"?: "evidence/….json", "fields": [[name, SOURCE], …],
//!               "embed"?: {"key": …, "position": "end"|"beforeLoopChecks", "fields"?: […], "failDirected"?: bool},
//!               "key": "{icon} {label}", "message": "…"}}
//! ```
//!
//! Evidence field sources: `{"status": true}`, `{"counter": name}` (`checks`, `frames` and
//! `rules` count themselves), `{"agg": "min"|"max", "init"?: n, "source"?: "margin"}`,
//! `{"map": "rules"}`, `{"errors": true}`, `{"perAction": template}`, `{"contracts": true}`,
//! `{"const": value}` and, in an embed, `{"copy": field}`.

mod engine;
pub mod expr;
mod parse;
pub mod template;

use super::geom::{FlattenSpec, Pick};
use super::profile::Stage;
use super::{Evidence, SetContext, SetManifest, VerifyError, ensure};
use crate::contracts::geom::{Point, Sampler};
use crate::contracts::rules::{Failure, GapRadius, VisibleTest};
use expr::{Expr, Part};
use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::Value;
use template::Template;

/// Format marker shared with the contracts rules documents.
pub const RULES_FORMAT: &str = "gai-verify-rules";

/// A parsed rules document.
#[derive(Debug, Clone)]
pub struct RuleDoc {
    /// The set id the rules belong to.
    pub set: Option<String>,
    /// Cardinality and category gates.
    pub set_asserts: Vec<SetAssert>,
    /// The drawing rules, when the document has any.
    pub program: Option<Program>,
}

/// A set-level gate.
#[derive(Debug, Clone)]
pub struct SetAssert {
    /// What is counted or compared.
    pub kind: SetAssertKind,
    /// Failure message.
    pub message: String,
}

/// Set-level gates.
#[derive(Debug, Clone)]
pub enum SetAssertKind {
    /// `set.icons.length`.
    IconCount(Count),
    /// `new Set(set.icons.map(i => i.icon)).size`.
    UniqueIconIds(Count),
    /// Icons with a truthy `stateful`.
    StatefulCount(Count),
    /// `set.category`.
    Category(String),
    /// Distinct `contract` strings recorded by the rules (checked after they run).
    ContractCount(Count),
}

/// An exact count or an inclusive range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Count {
    /// Exactly this many.
    pub eq: Option<usize>,
    /// At least this many.
    pub min: Option<usize>,
    /// At most this many.
    pub max: Option<usize>,
}

impl Count {
    fn holds(self, n: usize) -> bool {
        self.eq.is_none_or(|e| n == e)
            && self.min.is_none_or(|m| n >= m)
            && self.max.is_none_or(|m| n <= m)
    }
}

/// The drawing rules and how they report.
#[derive(Debug, Clone)]
pub struct Program {
    /// Numeric knobs.
    pub dialect: Dialect,
    /// The canvas-margin pass.
    pub margin: Option<Margin>,
    /// The failure message when a set icon has no rules; `None` allows it.
    pub require_icon: Option<Template>,
    /// Skip the actions of icons without rules entirely (53).
    pub skip_unlisted: bool,
    /// The style dimension (53, 63): every rule runs once per style.
    pub styles: Vec<Style>,
    /// Rules per icon, in order.
    pub icons: IndexMap<String, Vec<Rule>>,
    /// The evidence layout.
    pub evidence: EvidenceSpec,
}

/// A style of the style dimension.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Style {
    /// Name used by rule `styles` filters.
    pub id: String,
    /// Node-name suffix for the `exact` picker.
    pub suffix: String,
}

/// Numeric knobs of a document.
#[derive(Debug, Clone)]
pub struct Dialect {
    /// Track sampler.
    pub sampler: Sampler,
    /// Curve flattening.
    pub flatten: FlattenSpec,
    /// Default picker.
    pub pick: Pick,
    /// Suffixes the `exact` picker tries in order (without a style dimension).
    pub suffixes: Vec<String>,
    /// Prefix part names with `<icon>-`.
    pub part_prefix: bool,
    /// Visibility gate of rule parts.
    pub visible: VisibleTest,
    /// Tolerance subtracted from minimums (added to maximums).
    pub slack: f64,
    /// Radius subtraction order of `gap`.
    pub gap_radius: GapRadius,
    /// One-point polylines measure to that point (49) instead of reading `+∞`.
    pub single_point: bool,
    /// What a failed check does.
    pub failure: Failure,
    /// Whether a NaN measurement fails (a positive `ok` test) or passes (`if (v < min)`).
    pub nan_fails: bool,
    /// icon → action → rule → frame (50) instead of icon → action → frame → rules.
    pub rule_major: bool,
}

/// The canvas-margin pass.
#[derive(Debug, Clone)]
pub struct Margin {
    /// Node names must end with one of these.
    pub name_ends_with: Vec<String>,
    /// Visibility gate.
    pub visible: VisibleTest,
    /// Painted bounds (`box`) or per-point margins less the radius (`points`, 42 and 58).
    pub per_point: bool,
    /// Threshold.
    pub min: f64,
    /// Failure mode.
    pub failure: Failure,
    /// Failure message.
    pub message: Template,
}

/// Frame gates of a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
    /// `t === 0 || f === action.rest` (50 `touch`).
    StartOrRest,
    /// Forward frame 144, reverse frame 0 (58 `from: "complete"`).
    Complete,
    /// Forward frame 0, reverse frame 144 (58 `from: "closed"`).
    Closed,
    /// Forward actions from `Math.round(active * 60)` (49 `fromEnd`).
    Settled,
}

/// Which parts must be visible for a rule to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibleCheck {
    /// Both `a` and `b` (when `b` is given).
    Both,
    /// Only `a`.
    A,
    /// Neither.
    None,
}

/// Where a rule's value is recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recording {
    /// Under the rule key in the rule map.
    Map,
    /// Nowhere.
    None,
    /// Into a named aggregate field.
    Field(String),
}

/// Threshold comparisons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    /// `value >= limit - slack`.
    AtLeast,
    /// `value <= limit + slack`.
    AtMost,
    /// `value > limit`.
    Above,
    /// `value < limit`.
    Below,
    /// `|value - limit| < tol`.
    Close(f64),
}

/// `curvePoints` trimmed around a deliberately attached end (58 `skipStart`/`skipEnd`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trim {
    /// Drop the first `ceil(n * 0.12)` points.
    Start,
    /// Keep the first `floor(n * 0.88)` points.
    End,
}

/// Rule kinds (verify.md §8.4) and their parameters.
#[derive(Debug, Clone)]
pub enum Kind {
    /// Painted clearance between two parts.
    Gap {
        /// Minimum.
        min: f64,
        /// Trim of `a`'s polyline.
        trim: Option<Trim>,
    },
    /// An end of `a` lies inside `b`'s paint (42): `(rB + rA) - dist`.
    AttachPaint {
        /// Truthy: last point; otherwise first.
        end: Value,
    },
    /// An end (or the whole length) of `a` touches `b`'s paint (44).
    AttachTouch {
        /// `0`: first point; otherwise last.
        end: Value,
        /// Take the maximum over the densified polyline.
        through: bool,
    },
    /// `a`'s painted box stays inside `b`'s (43).
    ContainedIn,
    /// `a`'s painted box stays inside a region.
    InsideBox {
        /// `[left, top, right, bottom]`.
        bounds: [f64; 4],
        /// Minimum inset.
        min: f64,
    },
    /// Every flattened point of `a` stays inside a region, less the paint radius (58).
    InsidePoints {
        /// `[left, top, right, bottom]`.
        bounds: [f64; 4],
    },
    /// The centre of `a`'s painted box against a polyline (49 `on-route`, `hop`).
    Route {
        /// The polyline.
        route: Vec<Point>,
        /// Minimum of `-distance`.
        min: f64,
    },
    /// The first point of `a` sits on `b` (50).
    Touch {
        /// Maximum distance.
        max: f64,
    },
    /// The first point of `a` stays outside a circle around `b`'s centroid (50).
    OutsideCircle {
        /// Circle radius.
        r: f64,
        /// Tolerance.
        tol: f64,
    },
    /// `a` and `b` are never visible together (50).
    Exclusive,
    /// An end of `a` lies on `b` (58).
    Joint {
        /// `0` or `1`.
        end: Value,
        /// Maximum distance.
        max: f64,
    },
    /// Some point of `a` comes within `max` of `b` (58).
    Contact {
        /// Maximum distance.
        max: f64,
    },
    /// An end of `a` is inside `b` by at least `min` (58).
    Buried {
        /// `0` or `1`.
        end: Value,
        /// Minimum depth.
        min: f64,
    },
    /// `a`'s sampled coordinates equal its first key (54).
    Frozen,
    /// An expression against a limit.
    Compare {
        /// Measured value.
        value: Expr,
        /// Comparison.
        op: Op,
        /// Threshold.
        limit: Expr,
    },
    /// A comparison for every visible node matching a name filter; `@` names the node (60).
    Each {
        /// Node names must end with one of these.
        name_ends_with: Vec<String>,
        /// Visibility gate.
        visible: VisibleTest,
        /// Measured value.
        value: Expr,
        /// Comparison.
        op: Op,
        /// Threshold.
        limit: Expr,
    },
}

/// One rule.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Kind and parameters.
    pub kind: Kind,
    /// Label (keys and messages).
    pub label: String,
    /// First part.
    pub a: Option<Part>,
    /// Second part.
    pub b: Option<Part>,
    /// Only for actions with this direction.
    pub direction: Option<String>,
    /// Only for actions with this motion profile.
    pub profile: Option<String>,
    /// Only for `t >= from`.
    pub from: Option<f64>,
    /// Only for `t <= to`.
    pub to: Option<f64>,
    /// Only on these frames.
    pub at: Option<At>,
    /// Only where this is truthy.
    pub when: Option<Expr>,
    /// Only in these styles.
    pub styles: Option<Vec<String>>,
    /// Per-rule visibility threshold.
    pub min_visibility: Option<f64>,
    /// Which parts are visibility-gated.
    pub visible_check: VisibleCheck,
    /// Where the value goes.
    pub record: Recording,
    /// Extra counters to bump per evaluation.
    pub count: Vec<(String, u64)>,
    /// A contract string recorded when the rule runs.
    pub contract: Option<String>,
    /// Per-rule slack.
    pub slack: Option<f64>,
    /// Per-rule failure mode.
    pub failure: Option<Failure>,
    /// Per-rule failure message.
    pub message: Option<Template>,
}

/// Where the rule stage's evidence goes.
#[derive(Debug, Clone)]
pub struct EvidenceSpec {
    /// Before or after the directed write.
    pub stage: Stage,
    /// A file of its own, relative to the set folder.
    pub file: Option<String>,
    /// Fields of that object, in order.
    pub fields: Vec<(String, Field)>,
    /// A key added to the directed report.
    pub embed: Option<Embed>,
    /// Rule key template.
    pub key: Template,
    /// Rule failure message.
    pub message: Template,
}

/// A key the rules add to the directed report.
#[derive(Debug, Clone)]
pub struct Embed {
    /// Key name.
    pub key: String,
    /// Insert before `loopChecks` (53) instead of at the end.
    pub before_loop_checks: bool,
    /// The embedded fields; `None` embeds the evidence object itself.
    pub fields: Option<Vec<(String, Field)>>,
    /// Mark the directed report `FAIL` when a rule failed (50).
    pub fail_directed: bool,
}

/// Evidence field sources.
#[derive(Debug, Clone, PartialEq)]
pub enum Field {
    /// `"PASS"` or `"FAIL"`.
    Status,
    /// A counter.
    Counter(String),
    /// A running minimum or maximum from `init`; the margin pass or rules recording into it.
    Agg {
        /// `true`: maximum.
        max: bool,
        /// Initial value.
        init: f64,
        /// Fed by the margin pass rather than by rules.
        margin: bool,
    },
    /// The rule map.
    RuleMap,
    /// Collected failure messages.
    Errors,
    /// One entry per action; strings `"{action}"` expand to the action name.
    PerAction(Value),
    /// Recorded contract strings in first-seen order.
    Contracts,
    /// A constant.
    Const(Value),
    /// The value of a field of the main object.
    Copy(String),
}

impl RuleDoc {
    /// Parses a rules document.
    pub fn from_slice(bytes: &[u8]) -> Result<RuleDoc, VerifyError> {
        parse::document(bytes)
    }

    /// Where the rules run; `None` when the document only has set asserts.
    pub fn stage(&self) -> Option<Stage> {
        self.program.as_ref().map(|p| p.evidence.stage)
    }

    /// Runs the set asserts checked before anything else.
    pub(crate) fn check_set(&self, manifest: &SetManifest) -> Result<(), VerifyError> {
        let icons = manifest.icons()?;
        for a in &self.set_asserts {
            let ok = match &a.kind {
                SetAssertKind::IconCount(c) => c.holds(icons.len()),
                SetAssertKind::UniqueIconIds(c) => {
                    let ids = super::js::same_value_zero_set(
                        icons.iter().map(|i| i.get("icon").cloned()),
                    );
                    c.holds(ids.len())
                }
                SetAssertKind::StatefulCount(c) => {
                    c.holds(icons.iter().filter(|i| i.flag("stateful")).count())
                }
                SetAssertKind::Category(want) => {
                    manifest.doc.get("category").and_then(Value::as_str) == Some(want.as_str())
                }
                SetAssertKind::ContractCount(_) => true,
            };
            ensure(ok, || a.message.clone())?;
        }
        Ok(())
    }

    /// Runs the drawing rules over every action of the set.
    pub(crate) fn run(&self, set: &SetContext<'_>) -> Result<TailOutcome, VerifyError> {
        let Some(program) = &self.program else {
            return Ok(TailOutcome::default());
        };
        let outcome = engine::run(program, set)?;
        // Contracts only icons adopted from spec v2 hold: those icons are checked against
        // their specs instead, so the count holds them as met.
        let held = |adopted: bool| -> std::collections::HashSet<&String> {
            program
                .icons
                .iter()
                .filter(|(id, _)| set.is_adopted(id) == adopted)
                .flat_map(|(_, rules)| rules.iter().filter_map(|r| r.contract.as_ref()))
                .collect()
        };
        let kept = held(false);
        let adopted = held(true).into_iter().filter(|c| !kept.contains(c)).count();
        for a in &self.set_asserts {
            if let SetAssertKind::ContractCount(c) = a.kind {
                ensure(c.holds(outcome.contracts + adopted), || a.message.clone())?;
            }
        }
        Ok(outcome)
    }
}

/// What the rule stage produced.
#[derive(Debug, Clone, Default)]
pub(crate) struct TailOutcome {
    /// Its own evidence file.
    pub file: Option<Evidence>,
    /// The key to add to the directed report: name, value, before `loopChecks`.
    pub embed: Option<(String, Value, bool)>,
    /// Mark the directed report `FAIL`.
    pub fail_directed: bool,
    /// Collected failures.
    pub errors: Vec<String>,
    /// Distinct contract strings recorded.
    pub contracts: usize,
}

impl TailOutcome {
    /// Adds the embedded key to the directed report.
    pub fn embed_into(&self, report: &mut super::generic::DirectedReport) {
        if let Some((key, value, before)) = &self.embed {
            let slot = if *before {
                &mut report.before_loop_checks
            } else {
                &mut report.extra
            };
            slot.push((key.clone(), value.clone()));
        }
        report.failed |= self.fail_directed && !self.errors.is_empty();
    }

    /// The stage's own evidence file.
    pub fn file(&self) -> Option<Evidence> {
        self.file.clone()
    }

    /// The collected failures, if any.
    pub fn failure(&self) -> Option<VerifyError> {
        (!self.errors.is_empty()).then(|| VerifyError::Collected {
            errors: self.errors.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn doc(extra: Value) -> Vec<u8> {
        let mut d = json!({
            "format": RULES_FORMAT, "version": 2,
            "dialect": {"sampler": "ge", "flatten": {"cubicSteps": 48, "close": "first", "unknown": "ignore"},
                        "pick": "bold", "visible": {"op": ">=", "value": 0.05}, "slack": 0.015, "gapRadius": "sequential", "failure": "collect"},
            "icons": {"cup": [{"kind": "gap", "label": "gap a/b", "a": "a", "b": "b", "min": 1}]},
            "evidence": {"fields": [["result", {"status": true}], ["gaps", {"map": "rules"}], ["best", {"agg": "max", "init": 0}]],
                         "key": "{icon} {label}", "message": "{action} {key} frame {f}: {value|fixed3} < {min}"}
        });
        d.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        serde_json::to_vec(&d).unwrap()
    }

    #[test]
    fn parses_and_validates() {
        let parsed = RuleDoc::from_slice(&doc(json!({}))).unwrap();
        assert_eq!(parsed.stage(), Some(Stage::AfterDirected));
        let p = parsed.program.unwrap();
        assert!(!p.dialect.nan_fails && p.dialect.part_prefix);
        assert!(matches!(p.icons["cup"][0].kind, Kind::Gap { min, trim: None } if min == 1.0));
        for bad in [
            json!({"order": "sideways"}),
            json!({"icons": {"cup": [{"kind": "gap", "label": "x", "a": "a"}]}}),
            json!({"icons": {"cup": [{"kind": "gap", "label": "x", "a": "a", "b": "b", "trim": "middle"}]}}),
            json!({"icons": {"cup": [{"kind": "compare", "label": "x", "value": 1, "op": "~", "limit": 0}]}}),
            json!({"icons": {"cup": [{"kind": "compare", "label": "x", "value": 1, "op": ">=", "limit": 0, "record": "nowhere"}]}}),
            json!({"styles": [{"id": "solid", "suffix": "-solid"}], "margin": {"nodes": {"nameEndsWith": []}, "visible": {"op": ">=", "value": 0}, "method": "box", "min": 1, "failure": "collect", "message": "{node}"}}),
            json!({"surprise": 1}),
        ] {
            assert!(RuleDoc::from_slice(&doc(bad.clone())).is_err(), "{bad}");
        }
        let ok = json!({"icons": {"cup": [{"kind": "compare", "label": "x", "value": 1, "op": ">=", "limit": 0, "record": "best"}]}});
        assert!(RuleDoc::from_slice(&doc(ok)).is_ok());
    }

    #[test]
    fn asserts_only_documents() {
        let d = json!({"format": RULES_FORMAT, "version": 2, "set": "45-chat-messaging",
                       "setAsserts": [{"kind": "iconCount", "eq": 2, "message": "two icons"}, {"kind": "statefulCount", "min": 1, "max": 1}]});
        let parsed = RuleDoc::from_slice(&serde_json::to_vec(&d).unwrap()).unwrap();
        assert!(parsed.program.is_none() && parsed.stage().is_none());
        let manifest = SetManifest {
            doc: json!({"icons": [{"name": "a", "stateful": true}, {"name": "b"}]}),
        };
        parsed.check_set(&manifest).unwrap();
        let manifest = SetManifest {
            doc: json!({"icons": [{"name": "a"}]}),
        };
        assert!(
            matches!(parsed.check_set(&manifest), Err(VerifyError::Assertion(m)) if m == "two icons")
        );
    }
}
