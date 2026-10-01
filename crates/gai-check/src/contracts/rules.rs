//! Rule documents: `SET/geometry-contract.json` v1 (22–39) and the unified
//! `gai-verify-rules` format of verify.md §8, which the built-in 16–20 rules use.
//!
//! A v1 contract is read natively with the 22–39 dialect. Its rule fields keep the JS
//! truthiness of the interpreter (`aCommands: 0`, `direction: ""` and `min: null` mean
//! "absent"), and its errors stay lazy: an unknown `kind`, a bad subpath range or a
//! missing part fails only on a frame where the rule is live.

use super::ContractError;
use super::evidence::js_normalize;
use super::expr::Expr;
use super::geom::{Close, FlattenSpec, Sampler, Unknown};
use gai_core::js::truthy;
use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::cmp::Ordering;
use std::fmt::Write as _;

/// Format marker of the unified rules document.
pub const RULES_FORMAT: &str = "gai-verify-rules";

/// A complete clearance program: numeric dialect, canvas margin, rules per icon and the
/// evidence layout.
#[derive(Debug, Clone)]
pub struct RuleSet {
    /// Set id the rules belong to (`set.json` `id`), when the document names one.
    pub set: Option<String>,
    /// Numeric knobs.
    pub dialect: Dialect,
    /// The canvas-margin pass, if any.
    pub margin: Option<MarginSpec>,
    /// Whether every `set.json` icon needs an `icons` entry (`<icon> missing supervisor
    /// geometry contract`).
    pub require_icon: bool,
    /// Rules per icon, evaluated in order.
    pub icons: IndexMap<String, Vec<Rule>>,
    /// Evidence layout.
    pub evidence: EvidenceSpec,
}

/// Numeric knobs shared by every rule of a document.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Dialect {
    /// Track sampler.
    pub sampler: Sampler,
    /// Curve flattening.
    pub flatten: FlattenSpec,
    /// Part picker.
    pub pick: Pick,
    /// Visibility gate for rule parts.
    pub visible: VisibleTest,
    /// Tolerance subtracted from minimums (added to maximums).
    pub slack: f64,
    /// How gap rules subtract the two paint radii.
    pub gap_radius: GapRadius,
    /// What a failed check does.
    pub failure: Failure,
}

/// Part pickers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Pick {
    /// `PICK_BOLD`: prefer `<name>-bold`, else `<name>`.
    Bold,
    /// `PICK_BASE`: exactly `<name>`.
    Base,
}

/// "Skip a part unless `visibility <op> value`".
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisibleTest {
    /// Comparison.
    pub op: VisibleOp,
    /// Threshold.
    pub value: f64,
}

/// Visibility comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum VisibleOp {
    /// Skip when `v < value` (NaN counts as visible).
    #[serde(rename = ">=")]
    AtLeast,
    /// Skip unless `v > value` (NaN counts as hidden).
    #[serde(rename = ">")]
    Above,
}

impl VisibleTest {
    /// Whether a part with this sampled visibility is checked.
    pub fn visible(self, v: f64) -> bool {
        match self.op {
            VisibleOp::AtLeast => v.partial_cmp(&self.value) != Some(Ordering::Less),
            VisibleOp::Above => v > self.value,
        }
    }
}

/// Radius subtraction order for gap rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GapRadius {
    /// `d - (rA + rB)` (16–39).
    Summed,
    /// `(d - rA) - rB`.
    Sequential,
}

/// What a failed check does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Failure {
    /// Record the message; the evidence says FAIL (`if (v < min) errors.push(…)`).
    Collect,
    /// Abort without evidence (`assert.ok(v >= min, …)`; NaN fails).
    Assert,
}

/// Direction of a threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    /// Pass when `value >= limit - slack`.
    AtLeast,
    /// Pass when `value <= limit + slack`.
    AtMost,
}

impl Failure {
    /// Whether `value` fails `bound` against the slack-adjusted `limit`. A NaN comparison
    /// passes a collected check (`if (v < min)`) and fails an assert (`ok(v >= min)`).
    pub fn fails(self, value: f64, bound: Bound, limit: f64) -> bool {
        let wrong_side = match bound {
            Bound::AtLeast => Ordering::Less,
            Bound::AtMost => Ordering::Greater,
        };
        match value.partial_cmp(&limit) {
            Some(order) => order == wrong_side,
            None => self == Failure::Assert,
        }
    }
}

/// The canvas-margin pass: painted bounds of selected nodes must stay `min` inside the
/// 32-unit canvas on every frame.
#[derive(Debug, Clone)]
pub struct MarginSpec {
    /// Node names must end with one of these.
    pub name_ends_with: Vec<String>,
    /// Visibility gate.
    pub visible: VisibleTest,
    /// Threshold (`1.99` = 2 units less a 0.01 tolerance).
    pub min: f64,
    /// Failure mode.
    pub failure: Failure,
    /// Failure message (`{action} {node} {f} {value}`).
    pub message: Template,
}

/// Evidence layout: ordered fields, the gap key and the failure message.
#[derive(Debug, Clone)]
pub struct EvidenceSpec {
    /// Top-level fields in output order.
    pub fields: Vec<(String, Field)>,
    /// Key of a rule's recorded minimum (`{icon} {label}`).
    pub key: Template,
    /// Rule failure message (`{action} {key} {label} {icon} {f} {value} {min}`).
    pub message: Template,
}

/// Sources of evidence fields.
#[derive(Debug, Clone, PartialEq)]
pub enum Field {
    /// `"PASS"` or `"FAIL"`.
    Status,
    /// One object per action; string values expand `{action}`, others are copied.
    PerAction(Map<String, Value>),
    /// Minimum margin over the run, folded into `init`.
    MarginMin {
        /// Initial value (32 = the canvas).
        init: f64,
    },
    /// Recorded rule minimums by key, in first-evaluation order.
    RuleMap,
    /// Checks performed.
    Checks,
    /// Collected failure messages.
    Errors,
    /// A constant value.
    Const(Value),
}

/// One rule. Kind-specific parameters live in [`RuleKind`].
#[derive(Debug, Clone)]
pub struct Rule {
    /// Kind and its parameters.
    pub kind: RuleKind,
    /// Label (key and messages).
    pub label: String,
    /// First part, without the icon prefix.
    pub a: Option<String>,
    /// Second part, without the icon prefix.
    pub b: Option<String>,
    /// Minimum; `None` takes the kind's default.
    pub min: Option<f64>,
    /// Only for actions with this direction.
    pub direction: Option<String>,
    /// Only for actions with this motion profile.
    pub profile: Option<String>,
    /// Only for `t >= from`.
    pub from: Option<f64>,
    /// Only for `t <= to`.
    pub to: Option<f64>,
    /// Only on frames where this is truthy.
    pub when: Option<Expr>,
    /// Command range of `a` (raw; validated when used).
    pub a_segments: Value,
    /// Command range of `b` (raw; validated when used).
    pub b_segments: Value,
    /// Command prefix of `a` for gap kinds (`slice(0, n)`).
    pub a_commands: Option<i64>,
    /// Command prefix of `b` for gap kinds.
    pub b_commands: Option<i64>,
    /// Per-rule slack.
    pub slack: Option<f64>,
    /// Per-rule failure mode.
    pub failure: Option<Failure>,
    /// Record the value under the rule key (default true).
    pub record: bool,
    /// Per-rule failure message.
    pub message: Option<Template>,
}

/// Rule kinds and their own parameters.
#[derive(Debug, Clone)]
pub enum RuleKind {
    /// Symmetric vertex-to-polyline distance less both paint radii.
    Gap,
    /// A gap against `b` cut to its first `curve_commands` commands (`None` keeps all).
    ArrowCurveGap {
        /// Command count; JSON `null` is 0.
        curve_commands: Option<i64>,
    },
    /// A gap between single commands `a_command` of `a` and `b_command` of `b`.
    SelfGap {
        /// Command index into `a`.
        a_command: Option<i64>,
        /// Command index into `b`.
        b_command: Option<i64>,
    },
    /// Whenever `a` is visible it must be fully opaque.
    Opaque,
    /// Painted bounds of `a` inside `bounds`.
    InsideBox {
        /// `[left, top, right, bottom]`.
        bounds: Option<[f64; 4]>,
    },
    /// Raw control points of `a` and `b` coincide.
    PointJoint {
        /// Flat coordinate offset into `a`.
        a_offset: Option<i64>,
        /// Flat coordinate offset into `b`.
        b_offset: Option<i64>,
    },
    /// An end of `a` lies on `b`'s centerline.
    Attachment {
        /// 0 = first point, 1 = last point.
        end: Option<i64>,
    },
    /// An expression against a limit.
    Compare {
        /// The measured value.
        value: Expr,
        /// Threshold direction.
        bound: Bound,
        /// The threshold.
        limit: Expr,
    },
    /// Any other kind; evaluating it fails with `Unknown geometry rule <kind>`.
    Unknown(String),
}

/// Template variables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Var {
    /// Icon name.
    Icon,
    /// Action name.
    Action,
    /// Rule label.
    Label,
    /// Rendered rule key.
    Key,
    /// Scene node name.
    Node,
    /// Frame number.
    Frame,
    /// Measured value (JS `Number::toString`).
    Value,
    /// Threshold (JS `Number::toString`).
    Min,
}

impl Var {
    fn parse(name: &str) -> Option<Var> {
        Some(match name {
            "icon" => Var::Icon,
            "action" => Var::Action,
            "label" => Var::Label,
            "key" => Var::Key,
            "node" => Var::Node,
            "f" => Var::Frame,
            "value" => Var::Value,
            "min" => Var::Min,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Piece {
    Text(String),
    Var(Var),
}

/// A `{var}` template, validated against the variables its context provides.
#[derive(Debug, Clone, PartialEq)]
pub struct Template(Vec<Piece>);

/// Values for template variables.
#[derive(Debug, Default, Clone, Copy)]
pub struct Vars<'a> {
    /// `{icon}`
    pub icon: &'a str,
    /// `{action}`
    pub action: &'a str,
    /// `{label}`
    pub label: &'a str,
    /// `{key}`
    pub key: &'a str,
    /// `{node}`
    pub node: &'a str,
    /// `{f}`
    pub frame: u32,
    /// `{value}`
    pub value: f64,
    /// `{min}`
    pub min: f64,
}

impl Template {
    /// Parses `text`, allowing only `allowed` variables.
    pub fn parse(text: &str, allowed: &[Var]) -> Result<Template, ContractError> {
        let mut pieces = Vec::new();
        let mut rest = text;
        while let Some(open) = rest.find('{') {
            if open > 0 {
                pieces.push(Piece::Text(rest[..open].to_string()));
            }
            let close = rest[open..]
                .find('}')
                .ok_or_else(|| bad(format!("unclosed {{ in template {text:?}")))?
                + open;
            let name = &rest[open + 1..close];
            match Var::parse(name).filter(|v| allowed.contains(v)) {
                Some(v) => pieces.push(Piece::Var(v)),
                None => {
                    return Err(bad(format!(
                        "unknown variable {{{name}}} in template {text:?}"
                    )));
                }
            }
            rest = &rest[close + 1..];
        }
        if !rest.is_empty() {
            pieces.push(Piece::Text(rest.to_string()));
        }
        Ok(Template(pieces))
    }

    /// Expands the template; numbers print as JS `Number::toString`.
    pub fn render(&self, vars: &Vars<'_>) -> String {
        let mut out = String::new();
        for piece in &self.0 {
            match piece {
                Piece::Text(s) => out.push_str(s),
                Piece::Var(v) => match v {
                    Var::Icon => out.push_str(vars.icon),
                    Var::Action => out.push_str(vars.action),
                    Var::Label => out.push_str(vars.label),
                    Var::Key => out.push_str(vars.key),
                    Var::Node => out.push_str(vars.node),
                    Var::Frame => {
                        let _ = write!(out, "{}", vars.frame);
                    }
                    Var::Value => out.push_str(&gai_core::js_number_to_string(vars.value)),
                    Var::Min => out.push_str(&gai_core::js_number_to_string(vars.min)),
                },
            }
        }
        out
    }
}

/// Variables of a rule key.
pub const KEY_VARS: &[Var] = &[Var::Icon, Var::Label];
/// Variables of a rule failure message.
pub const MESSAGE_VARS: &[Var] = &[
    Var::Icon,
    Var::Action,
    Var::Label,
    Var::Key,
    Var::Frame,
    Var::Value,
    Var::Min,
];
/// Variables of a margin failure message.
pub const MARGIN_VARS: &[Var] = &[Var::Action, Var::Node, Var::Frame, Var::Value];

fn bad(message: impl Into<String>) -> ContractError {
    ContractError::Rules(message.into())
}

impl RuleSet {
    /// Parses a rules document: `{"format": "gai-verify-rules", …}` or a v1 geometry contract.
    pub fn from_slice(bytes: &[u8]) -> Result<RuleSet, ContractError> {
        let doc: Value = serde_json::from_slice(bytes).map_err(|e| bad(e.to_string()))?;
        match doc.get("format") {
            Some(f) if f == RULES_FORMAT => RuleSet::from_rules_doc(&doc),
            Some(f) => Err(bad(format!("unknown rules format {f}"))),
            None => RuleSet::from_contract(&doc),
        }
    }

    /// The 22–39 dialect: S_GT, 64-step flattening that skips degenerate cubics, `PICK_BOLD`,
    /// visibility `>= .01`, slack `.015`, summed radii, collected failures.
    pub fn contract_dialect() -> Dialect {
        Dialect {
            sampler: Sampler::Gt,
            flatten: FlattenSpec {
                cubic_steps: 64,
                skip_degenerate: true,
                close: Close::First,
                unknown: Unknown::Throw,
            },
            pick: Pick::Bold,
            visible: VisibleTest {
                op: VisibleOp::AtLeast,
                value: 0.01,
            },
            slack: 0.015,
            gap_radius: GapRadius::Summed,
            failure: Failure::Collect,
        }
    }

    /// Reads a v1 `geometry-contract.json` with the 22–39 dialect and evidence layout.
    /// `version` and `scope` are ignored; `exceptions` is copied into the evidence.
    pub fn from_contract(doc: &Value) -> Result<RuleSet, ContractError> {
        let obj = doc
            .as_object()
            .ok_or_else(|| bad("a geometry contract is a JSON object"))?;
        let icons = parse_icons(
            obj.get("icons")
                .ok_or_else(|| bad("a geometry contract needs \"icons\""))?,
            false,
        )?;
        let exceptions = match obj.get("exceptions") {
            Some(v) if truthy(Some(v)) => js_normalize(v.clone()),
            _ => Value::Object(Map::new()),
        };
        let margin = MarginSpec {
            name_ends_with: vec!["__outline.bold".into(), "__solid".into()],
            visible: VisibleTest {
                op: VisibleOp::AtLeast,
                value: 0.01,
            },
            min: 1.99,
            failure: Failure::Collect,
            message: Template::parse("{action} {node} frame {f}: margin {value}", MARGIN_VARS)?,
        };
        let fields = vec![
            ("result".into(), Field::Status),
            ("actions".into(), Field::PerAction(per_action_frames())),
            (
                "minimumPaintedMargin".into(),
                Field::MarginMin { init: 32.0 },
            ),
            ("gaps".into(), Field::RuleMap),
            ("checks".into(), Field::Checks),
            ("errors".into(), Field::Errors),
            ("exceptions".into(), Field::Const(exceptions)),
        ];
        Ok(RuleSet {
            set: None,
            dialect: RuleSet::contract_dialect(),
            margin: Some(margin),
            require_icon: true,
            icons,
            evidence: EvidenceSpec {
                fields,
                key: Template::parse("{icon} {label}", KEY_VARS)?,
                message: Template::parse(
                    "{action} {label} frame {f}: {value} < {min}",
                    MESSAGE_VARS,
                )?,
            },
        })
    }

    fn from_rules_doc(doc: &Value) -> Result<RuleSet, ContractError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Doc {
            #[serde(rename = "format")]
            _format: String,
            version: u32,
            set: Option<String>,
            dialect: Dialect,
            margin: Option<MarginDoc>,
            #[serde(default)]
            require_icon: bool,
            icons: Value,
            evidence: EvidenceDoc,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct MarginDoc {
            nodes: NodesDoc,
            visible: VisibleTest,
            method: MarginMethod,
            min: f64,
            failure: Failure,
            message: String,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct NodesDoc {
            name_ends_with: Vec<String>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "lowercase")]
        enum MarginMethod {
            Box,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct EvidenceDoc {
            fields: Vec<(String, Value)>,
            key: String,
            message: String,
        }

        let d: Doc = serde_json::from_value(doc.clone()).map_err(|e| bad(e.to_string()))?;
        if d.version != 2 {
            return Err(bad(format!("unsupported rules version {}", d.version)));
        }
        let margin = d
            .margin
            .map(|m| -> Result<MarginSpec, ContractError> {
                let MarginMethod::Box = m.method;
                Ok(MarginSpec {
                    name_ends_with: m.nodes.name_ends_with,
                    visible: m.visible,
                    min: m.min,
                    failure: m.failure,
                    message: Template::parse(&m.message, MARGIN_VARS)?,
                })
            })
            .transpose()?;
        let fields = d
            .evidence
            .fields
            .iter()
            .map(|(name, v)| Ok((name.clone(), parse_field(v)?)))
            .collect::<Result<_, ContractError>>()?;
        Ok(RuleSet {
            set: d.set,
            dialect: d.dialect,
            margin,
            require_icon: d.require_icon,
            icons: parse_icons(&d.icons, true)?,
            evidence: EvidenceSpec {
                fields,
                key: Template::parse(&d.evidence.key, KEY_VARS)?,
                message: Template::parse(&d.evidence.message, MESSAGE_VARS)?,
            },
        })
    }
}

fn per_action_frames() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("name".into(), Value::from("{action}"));
    m.insert("frames".into(), Value::from(145));
    m
}

fn parse_field(v: &Value) -> Result<Field, ContractError> {
    let obj = v
        .as_object()
        .ok_or_else(|| bad("an evidence field source is an object"))?;
    let only = |key: &str| obj.len() == 1 && obj.contains_key(key);
    Ok(if only("status") && obj["status"] == true {
        Field::Status
    } else if only("errors") && obj["errors"] == true {
        Field::Errors
    } else if only("counter") && obj["counter"] == "checks" {
        Field::Checks
    } else if only("map") && obj["map"] == "rules" {
        Field::RuleMap
    } else if only("const") {
        Field::Const(js_normalize(obj["const"].clone()))
    } else if only("perAction") {
        let template = obj["perAction"]
            .as_object()
            .ok_or_else(|| bad("perAction is an object"))?;
        if let Some(s) = template
            .values()
            .filter_map(Value::as_str)
            .find(|s| s.contains('{') && *s != "{action}")
        {
            return Err(bad(format!(
                "perAction strings support only the \"{{action}}\" variable, got {s:?}"
            )));
        }
        Field::PerAction(template.clone())
    } else if obj.len() == 3
        && obj.get("agg") == Some(&Value::from("min"))
        && obj.get("source") == Some(&Value::from("margin"))
    {
        Field::MarginMin {
            init: obj
                .get("init")
                .and_then(Value::as_f64)
                .ok_or_else(|| bad("margin agg needs a numeric init"))?,
        }
    } else {
        return Err(bad(format!("unsupported evidence field source {v}")));
    })
}

fn parse_icons(v: &Value, strict: bool) -> Result<IndexMap<String, Vec<Rule>>, ContractError> {
    let obj = v.as_object().ok_or_else(|| bad("\"icons\" is an object"))?;
    let mut icons = IndexMap::new();
    for (icon, rules) in obj {
        // A falsy entry is a missing contract; the interpreter reports it per action.
        if !truthy(Some(rules)) {
            continue;
        }
        let list = rules
            .as_array()
            .ok_or_else(|| bad(format!("rules of {icon} are not an array")))?;
        let parsed = list
            .iter()
            .map(|r| parse_rule(r, strict).map_err(|e| bad(format!("{icon}: {e}"))))
            .collect::<Result<Vec<_>, _>>()?;
        icons.insert(icon.clone(), parsed);
    }
    Ok(icons)
}

const RULE_FIELDS: &[&str] = &[
    "kind",
    "label",
    "a",
    "b",
    "min",
    "direction",
    "profile",
    "from",
    "to",
    "when",
    "aSegments",
    "bSegments",
    "aCommands",
    "bCommands",
    "curveCommands",
    "ai",
    "bi",
    "end",
    "box",
    "slack",
    "failure",
    "record",
    "message",
    "value",
    "op",
    "limit",
];

fn parse_rule(v: &Value, strict: bool) -> Result<Rule, String> {
    let obj = v.as_object().ok_or("a rule is an object")?;
    let unknown = obj.keys().find(|k| !RULE_FIELDS.contains(&k.as_str()));
    if let Some(k) = unknown.filter(|_| strict) {
        return Err(format!("unknown rule field {k}"));
    }
    let get = |k: &str| obj.get(k).filter(|v| !v.is_null());
    let string = |k: &str| -> Result<Option<String>, String> {
        get(k)
            .map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .ok_or(format!("{k} must be a string"))
            })
            .transpose()
    };
    let number = |k: &str| -> Result<Option<f64>, String> {
        get(k)
            .map(|v| v.as_f64().ok_or(format!("{k} must be a number")))
            .transpose()
    };
    let integer = |k: &str| -> Result<Option<i64>, String> {
        get(k)
            .map(|v| {
                v.as_f64()
                    .filter(|x| super::jsnum::is_integer(*x))
                    .map(|x| x as i64)
                    .ok_or(format!("{k} must be an integer"))
            })
            .transpose()
    };
    // JS truthiness: "" and 0 mean absent.
    let nonempty = |s: Option<String>| s.filter(|s| !s.is_empty());
    let nonzero = |n: Option<i64>| n.filter(|&n| n != 0);
    let expr = |k: &str| {
        get(k)
            .map(|v| Expr::parse(v).map_err(|e| e.to_string()))
            .transpose()
    };

    let kind_name = match obj.get("kind") {
        None | Some(Value::Null) => "undefined".to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    };
    let kind = match kind_name.as_str() {
        "gap" => RuleKind::Gap,
        "arrow-curve-gap" => RuleKind::ArrowCurveGap {
            // `slice(0, undefined)` keeps every command; `slice(0, null)` keeps none.
            curve_commands: match obj.get("curveCommands") {
                None => None,
                Some(Value::Null) => Some(0),
                Some(_) => integer("curveCommands")?,
            },
        },
        "self-gap" => RuleKind::SelfGap {
            a_command: integer("ai")?,
            b_command: integer("bi")?,
        },
        "opaque" => RuleKind::Opaque,
        "inside" | "inside-box" => RuleKind::InsideBox {
            bounds: get("box")
                .map(|v| {
                    serde_json::from_value::<[f64; 4]>(v.clone())
                        .map_err(|_| "box must be [left, top, right, bottom]".to_string())
                })
                .transpose()?,
        },
        "point-joint" => RuleKind::PointJoint {
            a_offset: integer("ai")?,
            b_offset: integer("bi")?,
        },
        "attachment" => RuleKind::Attachment {
            end: integer("end")?,
        },
        "compare" => {
            if obj.contains_key("a") || obj.contains_key("b") {
                return Err("compare rules name parts inside their expressions, not in a/b".into());
            }
            RuleKind::Compare {
                value: expr("value")?.ok_or("compare needs a value")?,
                bound: match get("op").and_then(Value::as_str) {
                    Some(">=") => Bound::AtLeast,
                    Some("<=") => Bound::AtMost,
                    _ => return Err("compare op must be \">=\" or \"<=\"".into()),
                },
                limit: expr("limit")?.ok_or("compare needs a limit")?,
            }
        }
        _ => RuleKind::Unknown(kind_name),
    };
    let failure = get("failure")
        .map(|v| {
            serde_json::from_value::<Failure>(v.clone())
                .map_err(|_| "failure must be \"collect\" or \"assert\"".to_string())
        })
        .transpose()?;
    let record = match get("record") {
        None => true,
        Some(v) => v.as_bool().ok_or("record must be a boolean")?,
    };
    let message = string("message")?
        .map(|m| Template::parse(&m, MESSAGE_VARS).map_err(|e| e.to_string()))
        .transpose()?;
    Ok(Rule {
        kind,
        label: string("label")?.ok_or("a rule needs a label")?,
        a: string("a")?,
        b: string("b")?,
        min: number("min")?,
        direction: nonempty(string("direction")?),
        profile: nonempty(string("profile")?),
        from: number("from")?,
        to: number("to")?,
        when: expr("when")?,
        a_segments: obj.get("aSegments").cloned().unwrap_or(Value::Null),
        b_segments: obj.get("bSegments").cloned().unwrap_or(Value::Null),
        a_commands: nonzero(integer("aCommands")?),
        b_commands: nonzero(integer("bCommands")?),
        slack: number("slack")?,
        failure,
        record,
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn contract(rules: Value) -> RuleSet {
        RuleSet::from_contract(&json!({"icons": {"cup": rules}})).unwrap()
    }

    #[test]
    fn v1_fields_follow_js_truthiness() {
        let set = contract(json!([
            {"kind": "gap", "a": "x", "b": "y", "label": "l", "min": null, "direction": "", "aCommands": 0, "bCommands": -2},
            {"kind": "arrow-curve-gap", "a": "x", "b": "y", "label": "c", "curveCommands": null},
            {"kind": "arrow-curve-gap", "a": "x", "b": "y", "label": "d"},
            {"kind": "wobble", "a": "x", "label": "w", "extra": true}
        ]));
        let r = &set.icons["cup"];
        assert_eq!(
            (
                r[0].min,
                r[0].direction.as_deref(),
                r[0].a_commands,
                r[0].b_commands
            ),
            (None, None, None, Some(-2))
        );
        assert!(matches!(
            r[1].kind,
            RuleKind::ArrowCurveGap {
                curve_commands: Some(0)
            }
        ));
        assert!(matches!(
            r[2].kind,
            RuleKind::ArrowCurveGap {
                curve_commands: None
            }
        ));
        assert!(matches!(&r[3].kind, RuleKind::Unknown(k) if k == "wobble"));
    }

    #[test]
    fn v1_contract_defaults() {
        let set = RuleSet::from_contract(
            &json!({"version": 1, "icons": {"cup": [], "mug": null}, "exceptions": null}),
        )
        .unwrap();
        assert!(
            set.require_icon && set.icons.contains_key("cup") && !set.icons.contains_key("mug")
        );
        assert_eq!(
            set.evidence.fields.last(),
            Some(&("exceptions".to_string(), Field::Const(json!({}))))
        );
        assert!(RuleSet::from_contract(&json!({"icons": {"cup": [{"kind": "self-gap", "a": "x", "b": "y", "label": "l", "ai": 1.5}]}})).is_err());
    }

    #[test]
    fn templates_render_js_numbers() {
        let t =
            Template::parse("{action} {label} frame {f}: {value} < {min}", MESSAGE_VARS).unwrap();
        let vars = Vars {
            action: "cup-strong",
            label: "rim",
            frame: 7,
            value: -0.0,
            min: -0.001,
            ..Vars::default()
        };
        assert_eq!(t.render(&vars), "cup-strong rim frame 7: 0 < -0.001");
        let nan = Vars {
            value: f64::NAN,
            min: 1.0,
            ..vars
        };
        assert_eq!(t.render(&nan), "cup-strong rim frame 7: NaN < 1");
        assert!(Template::parse("{node}", KEY_VARS).is_err());
        assert!(Template::parse("{icon", KEY_VARS).is_err());
    }

    #[test]
    fn rules_docs_are_strict() {
        let doc = |extra: Value| {
            let mut d = json!({
                "format": RULES_FORMAT, "version": 2,
                "dialect": {"sampler": "gt", "flatten": {"cubicSteps": 20, "skipDegenerate": false, "close": "first", "unknown": "throw"},
                            "pick": "bold", "visible": {"op": ">=", "value": 0.05}, "slack": 0.015, "gapRadius": "summed", "failure": "collect"},
                "icons": {"cup": [{"kind": "compare", "label": "c", "value": 1, "op": ">=", "limit": 0}]},
                "evidence": {"fields": [["result", {"status": true}], ["gaps", {"map": "rules"}]], "key": "{label}", "message": "{action} {key}"}
            });
            d.as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            serde_json::to_vec(&d).unwrap()
        };
        assert!(RuleSet::from_slice(&doc(json!({}))).is_ok());
        assert!(RuleSet::from_slice(&doc(json!({"order": "rule-major"}))).is_err());
        assert!(RuleSet::from_slice(&doc(json!({"icons": {"cup": [{"kind": "gap", "a": "x", "b": "y", "label": "l", "trim": "start"}]}}))).is_err());
        assert!(RuleSet::from_slice(&doc(json!({"icons": {"cup": [{"kind": "compare", "label": "c", "a": "x", "value": 1, "op": ">=", "limit": 0}]}}))).is_err());
    }

    #[test]
    fn failure_modes_differ_on_nan() {
        assert!(!Failure::Collect.fails(f64::NAN, Bound::AtLeast, 1.0));
        assert!(Failure::Assert.fails(f64::NAN, Bound::AtLeast, 1.0));
        assert!(Failure::Collect.fails(0.5, Bound::AtLeast, 1.0));
        assert!(
            Failure::Collect.fails(0.02, Bound::AtMost, 0.015)
                && !Failure::Collect.fails(0.015, Bound::AtMost, 0.015)
        );
    }
}
