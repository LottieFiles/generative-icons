//! Parsing of rules documents: strict field sets, defaults per rule kind, and templates
//! checked against the variables their context provides.

use super::expr::{Expr, Part};
use super::template::{ICON_VARS, KEY_VARS, MARGIN_VARS, MESSAGE_VARS, Template};
use super::{
    At, Count, Dialect, Embed, EvidenceSpec, Field, Kind, Margin, Op, Program, RULES_FORMAT,
    Recording, Rule, RuleDoc, SetAssert, SetAssertKind, Style, Trim, VisibleCheck,
};
use crate::contracts::geom::Sampler;
use crate::contracts::rules::{Failure, GapRadius, VisibleTest};
use crate::verify::VerifyError;
use crate::verify::geom::{FlattenSpec, Pick};
use crate::verify::profile::Stage;
use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::{Map, Value};

fn bad(message: impl Into<String>) -> VerifyError {
    VerifyError::Rules(message.into())
}

fn from_value<T: serde::de::DeserializeOwned>(v: &Value, what: &str) -> Result<T, VerifyError> {
    serde_json::from_value(v.clone()).map_err(|e| bad(format!("{what}: {e}")))
}

fn only_keys(map: &Map<String, Value>, allowed: &[&str], what: &str) -> Result<(), VerifyError> {
    match map.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(k) => Err(bad(format!("unknown {what} field {k}"))),
        None => Ok(()),
    }
}

/// Parses a rules document.
pub(super) fn document(bytes: &[u8]) -> Result<RuleDoc, VerifyError> {
    let doc: Value = serde_json::from_slice(bytes).map_err(|e| bad(e.to_string()))?;
    let obj = doc
        .as_object()
        .ok_or_else(|| bad("a rules document is an object"))?;
    only_keys(
        obj,
        &[
            "format",
            "version",
            "set",
            "setAsserts",
            "dialect",
            "margin",
            "requireIcon",
            "unlisted",
            "styles",
            "icons",
            "evidence",
        ],
        "document",
    )?;
    if obj.get("format").and_then(Value::as_str) != Some(RULES_FORMAT) {
        return Err(bad(format!("format must be {RULES_FORMAT:?}")));
    }
    if obj.get("version").and_then(Value::as_u64) != Some(2) {
        return Err(bad("unsupported rules version"));
    }
    let set_asserts = match obj.get("setAsserts") {
        None => Vec::new(),
        Some(v) => v
            .as_array()
            .ok_or_else(|| bad("setAsserts is an array"))?
            .iter()
            .map(parse_set_assert)
            .collect::<Result<_, _>>()?,
    };
    let program = if obj.contains_key("icons") {
        Some(parse_program(obj)?)
    } else {
        None
    };
    if program.is_none()
        && ["dialect", "margin", "evidence"]
            .iter()
            .any(|k| obj.contains_key(*k))
    {
        return Err(bad("dialect, margin and evidence need icons"));
    }
    Ok(RuleDoc {
        set: obj.get("set").and_then(Value::as_str).map(str::to_string),
        set_asserts,
        program,
    })
}

fn parse_set_assert(v: &Value) -> Result<SetAssert, VerifyError> {
    let obj = v
        .as_object()
        .ok_or_else(|| bad("a set assert is an object"))?;
    let kind = obj
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| bad("a set assert needs a kind"))?;
    let message = obj
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or(kind)
        .to_string();
    let count = || -> Result<Count, VerifyError> {
        let mut rest = obj.clone();
        rest.remove("kind");
        rest.remove("message");
        from_value(&Value::Object(rest), kind)
    };
    let kind = match kind {
        "iconCount" => SetAssertKind::IconCount(count()?),
        "uniqueIconIds" => SetAssertKind::UniqueIconIds(count()?),
        "statefulCount" => SetAssertKind::StatefulCount(count()?),
        "contractCount" => SetAssertKind::ContractCount(count()?),
        "category" => {
            only_keys(obj, &["kind", "eq", "message"], "category assert")?;
            SetAssertKind::Category(
                obj.get("eq")
                    .and_then(Value::as_str)
                    .ok_or_else(|| bad("category needs eq"))?
                    .to_string(),
            )
        }
        other => return Err(bad(format!("unknown set assert {other}"))),
    };
    Ok(SetAssert { kind, message })
}

fn parse_program(obj: &Map<String, Value>) -> Result<Program, VerifyError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct DialectDoc {
        sampler: Sampler,
        flatten: FlattenSpec,
        pick: Pick,
        #[serde(default)]
        suffixes: Vec<String>,
        #[serde(default)]
        part_prefix: Option<String>,
        visible: VisibleTest,
        slack: f64,
        gap_radius: GapRadius,
        #[serde(default)]
        single_point_distance: bool,
        failure: Failure,
        #[serde(default)]
        nan_fails: Option<bool>,
        #[serde(default)]
        order: Option<String>,
    }
    let d: DialectDoc = from_value(
        obj.get("dialect")
            .ok_or_else(|| bad("rules need a dialect"))?,
        "dialect",
    )?;
    let dialect = Dialect {
        sampler: d.sampler,
        flatten: d.flatten,
        pick: d.pick,
        suffixes: d.suffixes,
        part_prefix: match d.part_prefix.as_deref() {
            None | Some("icon") => true,
            Some("none") => false,
            Some(other) => return Err(bad(format!("unknown partPrefix {other}"))),
        },
        visible: d.visible,
        slack: d.slack,
        gap_radius: d.gap_radius,
        single_point: d.single_point_distance,
        nan_fails: d.nan_fails.unwrap_or(d.failure == Failure::Assert),
        failure: d.failure,
        rule_major: match d.order.as_deref() {
            None | Some("frame-major") => false,
            Some("rule-major") => true,
            Some(other) => return Err(bad(format!("unknown order {other}"))),
        },
    };
    let margin = obj.get("margin").map(parse_margin).transpose()?;
    let require_icon = match obj.get("requireIcon") {
        None | Some(Value::Bool(false)) => None,
        Some(Value::Bool(true)) => Some(Template::parse(
            "{icon} missing supervisor geometry contract",
            ICON_VARS,
        )?),
        Some(Value::String(m)) => Some(Template::parse(m, ICON_VARS)?),
        Some(other) => {
            return Err(bad(format!(
                "requireIcon must be a boolean or a message, got {other}"
            )));
        }
    };
    let skip_unlisted = match obj.get("unlisted").map(Value::as_str) {
        None | Some(Some("run")) => false,
        Some(Some("skip")) => true,
        _ => return Err(bad("unlisted must be \"run\" or \"skip\"")),
    };
    let styles: Vec<Style> = obj
        .get("styles")
        .map(|v| from_value(v, "styles"))
        .transpose()?
        .unwrap_or_default();
    if !styles.is_empty() && margin.is_some() {
        return Err(bad("a margin pass cannot be combined with styles"));
    }
    if dialect.rule_major && (margin.is_some() || !styles.is_empty()) {
        return Err(bad("rule-major order has no margin pass or styles"));
    }
    let evidence = parse_evidence(
        obj.get("evidence")
            .ok_or_else(|| bad("rules need evidence"))?,
    )?;
    let mut icons = IndexMap::new();
    for (icon, rules) in obj["icons"]
        .as_object()
        .ok_or_else(|| bad("icons is an object"))?
    {
        let rules = rules
            .as_array()
            .ok_or_else(|| bad(format!("rules of {icon} are not an array")))?;
        let parsed = rules
            .iter()
            .map(|r| parse_rule(r, &styles).map_err(|e| bad(format!("{icon}: {e}"))))
            .collect::<Result<Vec<_>, _>>()?;
        icons.insert(icon.clone(), parsed);
    }
    let declared = |name: &str| {
        let has = |fields: &[(String, Field)]| {
            fields
                .iter()
                .any(|(n, f)| n == name && matches!(f, Field::Agg { margin: false, .. }))
        };
        has(&evidence.fields)
            || evidence
                .embed
                .as_ref()
                .and_then(|e| e.fields.as_deref())
                .is_some_and(has)
    };
    for rule in icons.values().flatten() {
        if let Recording::Field(name) = &rule.record
            && !declared(name)
        {
            return Err(bad(format!(
                "{}: records into undeclared field {name}",
                rule.label
            )));
        }
    }
    Ok(Program {
        dialect,
        margin,
        require_icon,
        skip_unlisted,
        styles,
        icons,
        evidence,
    })
}

fn parse_margin(v: &Value) -> Result<Margin, VerifyError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct NodesDoc {
        name_ends_with: Vec<String>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct MarginDoc {
        nodes: NodesDoc,
        visible: VisibleTest,
        method: String,
        min: f64,
        failure: Failure,
        message: String,
    }
    let m: MarginDoc = from_value(v, "margin")?;
    Ok(Margin {
        name_ends_with: m.nodes.name_ends_with,
        visible: m.visible,
        per_point: match m.method.as_str() {
            "box" => false,
            "points" => true,
            other => return Err(bad(format!("unknown margin method {other}"))),
        },
        min: m.min,
        failure: m.failure,
        message: Template::parse(&m.message, MARGIN_VARS)?,
    })
}

fn parse_fields(v: &Value, embedded: bool) -> Result<Vec<(String, Field)>, VerifyError> {
    let items = v
        .as_array()
        .ok_or_else(|| bad("evidence fields are an array"))?;
    let mut fields = Vec::new();
    for item in items {
        let (name, source): (String, Map<String, Value>) =
            from_value(item, "an evidence field is [name, source]")?;
        let only = |key: &str| source.len() == 1 && source.contains_key(key);
        let field = if only("status") && source["status"] == true {
            Field::Status
        } else if only("errors") && source["errors"] == true {
            Field::Errors
        } else if only("contracts") && source["contracts"] == true {
            Field::Contracts
        } else if only("counter") {
            Field::Counter(
                source["counter"]
                    .as_str()
                    .ok_or_else(|| bad("counter names a counter"))?
                    .to_string(),
            )
        } else if only("map") && source["map"] == "rules" {
            Field::RuleMap
        } else if only("const") {
            Field::Const(source["const"].clone())
        } else if only("perAction") {
            Field::PerAction(source["perAction"].clone())
        } else if only("copy") && embedded {
            Field::Copy(
                source["copy"]
                    .as_str()
                    .ok_or_else(|| bad("copy names a field"))?
                    .to_string(),
            )
        } else if let Some(agg) = source.get("agg") {
            only_keys(&source, &["agg", "init", "source"], "aggregate")?;
            let max = match agg.as_str() {
                Some("min") => false,
                Some("max") => true,
                _ => return Err(bad("agg is \"min\" or \"max\"")),
            };
            let margin = match source.get("source").map(Value::as_str) {
                None => false,
                Some(Some("margin")) => true,
                _ => return Err(bad("an aggregate source can only be \"margin\"")),
            };
            let identity = if max {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
            let init = match source.get("init") {
                None => identity,
                Some(v) => v.as_f64().ok_or_else(|| bad("init is a number"))?,
            };
            Field::Agg { max, init, margin }
        } else {
            return Err(bad(format!(
                "unsupported evidence field source {}",
                Value::Object(source)
            )));
        };
        fields.push((name, field));
    }
    Ok(fields)
}

fn parse_evidence(v: &Value) -> Result<EvidenceSpec, VerifyError> {
    let obj = v.as_object().ok_or_else(|| bad("evidence is an object"))?;
    only_keys(
        obj,
        &["stage", "file", "fields", "embed", "key", "message"],
        "evidence",
    )?;
    let stage = match obj.get("stage").map(Value::as_str) {
        None | Some(Some("afterDirected")) => Stage::AfterDirected,
        Some(Some("beforeDirected")) => Stage::BeforeDirected,
        _ => return Err(bad("stage must be \"beforeDirected\" or \"afterDirected\"")),
    };
    let fields = parse_fields(
        obj.get("fields")
            .ok_or_else(|| bad("evidence needs fields"))?,
        false,
    )?;
    let embed = obj
        .get("embed")
        .map(|e| -> Result<Embed, VerifyError> {
            let e = e.as_object().ok_or_else(|| bad("embed is an object"))?;
            only_keys(e, &["key", "position", "fields", "failDirected"], "embed")?;
            Ok(Embed {
                key: e
                    .get("key")
                    .and_then(Value::as_str)
                    .ok_or_else(|| bad("embed needs a key"))?
                    .to_string(),
                before_loop_checks: match e.get("position").map(Value::as_str) {
                    None | Some(Some("end")) => false,
                    Some(Some("beforeLoopChecks")) => true,
                    _ => {
                        return Err(bad(
                            "embed position must be \"end\" or \"beforeLoopChecks\"",
                        ));
                    }
                },
                fields: e.get("fields").map(|f| parse_fields(f, true)).transpose()?,
                fail_directed: e
                    .get("failDirected")
                    .map(|b| b.as_bool().ok_or_else(|| bad("failDirected is a boolean")))
                    .transpose()?
                    .unwrap_or(false),
            })
        })
        .transpose()?;
    if embed.is_some() && stage == Stage::AfterDirected {
        return Err(bad(
            "only a beforeDirected stage can embed into the directed report",
        ));
    }
    let template = |key: &str, vars| -> Result<Template, VerifyError> {
        Template::parse(
            obj.get(key)
                .and_then(Value::as_str)
                .ok_or_else(|| bad(format!("evidence needs a {key}")))?,
            vars,
        )
    };
    Ok(EvidenceSpec {
        stage,
        file: obj
            .get("file")
            .map(|f| {
                f.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| bad("file is a path"))
            })
            .transpose()?,
        fields,
        embed,
        key: template("key", KEY_VARS)?,
        message: template("message", MESSAGE_VARS)?,
    })
}

const RULE_FIELDS: &[&str] = &[
    "kind",
    "label",
    "a",
    "b",
    "min",
    "max",
    "tol",
    "r",
    "box",
    "route",
    "end",
    "through",
    "trim",
    "direction",
    "profile",
    "from",
    "to",
    "at",
    "when",
    "styles",
    "minVisibility",
    "visibleCheck",
    "record",
    "count",
    "contract",
    "slack",
    "failure",
    "message",
    "value",
    "op",
    "limit",
    "nodes",
    "visible",
];

fn parse_rule(v: &Value, styles: &[Style]) -> Result<Rule, String> {
    let obj = v.as_object().ok_or("a rule is an object")?;
    if let Some(k) = obj.keys().find(|k| !RULE_FIELDS.contains(&k.as_str())) {
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
    let part = |k: &str| {
        get(k)
            .map(|v| Part::parse(v).map_err(|e| e.to_string()))
            .transpose()
    };
    let expr = |k: &str| {
        get(k)
            .map(|v| Expr::parse(v).map_err(|e| e.to_string()))
            .transpose()
    };
    let bounds = || -> Result<[f64; 4], String> {
        serde_json::from_value(get("box").cloned().unwrap_or(Value::Null))
            .map_err(|_| "box must be [left, top, right, bottom]".to_string())
    };
    let end = || get("end").cloned().unwrap_or(Value::Null);
    let op = || -> Result<Op, String> {
        Ok(match get("op").and_then(Value::as_str) {
            Some(">=") => Op::AtLeast,
            Some("<=") => Op::AtMost,
            Some(">") => Op::Above,
            Some("<") => Op::Below,
            Some("close") => Op::Close(number("tol")?.unwrap_or(3e-5)),
            _ => return Err("op must be one of >=, <=, >, <, close".into()),
        })
    };
    let kind_name = string("kind")?.ok_or("a rule needs a kind")?;
    let (a, b) = (part("a")?, part("b")?);
    let (needs_a, needs_b) = match kind_name.as_str() {
        "compare" | "each" => (false, false),
        "inside-box" | "inside-points" | "on-route" | "hop" | "frozen" => (true, false),
        _ => (true, true),
    };
    if needs_a != a.is_some() || needs_b != b.is_some() {
        return Err(format!(
            "{kind_name} takes {}",
            match (needs_a, needs_b) {
                (true, true) => "parts a and b",
                (true, false) => "part a only",
                _ => "no a/b parts (name them in expressions)",
            }
        ));
    }
    let kind = match kind_name.as_str() {
        "gap" => Kind::Gap {
            min: number("min")?.unwrap_or(1.0),
            trim: match string("trim")?.as_deref() {
                None => None,
                Some("start") => Some(Trim::Start),
                Some("end") => Some(Trim::End),
                Some(other) => return Err(format!("unknown trim {other}")),
            },
        },
        "attach-paint" => Kind::AttachPaint { end: end() },
        "attach-touch" => Kind::AttachTouch {
            end: end(),
            through: get("through").and_then(Value::as_bool).unwrap_or(false),
        },
        "contained-in" => Kind::ContainedIn,
        "inside-box" => Kind::InsideBox {
            bounds: bounds()?,
            min: number("min")?.unwrap_or(0.0),
        },
        "inside-points" => Kind::InsidePoints { bounds: bounds()? },
        "on-route" | "hop" => Kind::Route {
            route: serde_json::from_value(get("route").cloned().unwrap_or(Value::Null))
                .map_err(|_| "route must be [[x, y], …]")?,
            min: number("min")?.ok_or("on-route and hop need min")?,
        },
        "touch" => Kind::Touch {
            max: number("max")?.ok_or("touch needs max")?,
        },
        "outside-circle" => Kind::OutsideCircle {
            r: number("r")?.ok_or("outside-circle needs r")?,
            tol: number("tol")?.unwrap_or(0.0),
        },
        "exclusive" => Kind::Exclusive,
        "joint" => Kind::Joint {
            end: end(),
            max: number("max")?.unwrap_or(0.02),
        },
        "contact" => Kind::Contact {
            max: number("max")?.ok_or("contact needs max")?,
        },
        "buried" => Kind::Buried {
            end: end(),
            min: number("min")?.unwrap_or(0.25),
        },
        "frozen" => Kind::Frozen,
        "compare" => Kind::Compare {
            value: expr("value")?.ok_or("compare needs a value")?,
            op: op()?,
            limit: expr("limit")?.ok_or("compare needs a limit")?,
        },
        "each" => {
            let nodes = get("nodes")
                .and_then(|n| n.get("nameEndsWith"))
                .ok_or("each needs nodes.nameEndsWith")?;
            Kind::Each {
                name_ends_with: serde_json::from_value(nodes.clone())
                    .map_err(|_| "nameEndsWith is a list of strings")?,
                visible: serde_json::from_value(
                    get("visible").cloned().ok_or("each needs visible")?,
                )
                .map_err(|e| e.to_string())?,
                value: expr("value")?.ok_or("each needs a value")?,
                op: op()?,
                limit: expr("limit")?.ok_or("each needs a limit")?,
            }
        }
        other => return Err(format!("unknown rule kind {other}")),
    };
    let default_visible = match kind {
        Kind::Gap { .. }
        | Kind::AttachPaint { .. }
        | Kind::ContainedIn
        | Kind::Joint { .. }
        | Kind::Contact { .. }
        | Kind::Buried { .. } => VisibleCheck::Both,
        Kind::AttachTouch { .. }
        | Kind::InsideBox { .. }
        | Kind::InsidePoints { .. }
        | Kind::Route { .. }
        | Kind::OutsideCircle { .. } => VisibleCheck::A,
        _ => VisibleCheck::None,
    };
    let default_record = match kind {
        Kind::ContainedIn | Kind::Exclusive | Kind::Frozen | Kind::Each { .. } => Recording::None,
        _ => Recording::Map,
    };
    let rule_styles: Option<Vec<String>> = get("styles")
        .map(|s| {
            serde_json::from_value(s.clone())
                .map_err(|_| "styles is a list of style ids".to_string())
        })
        .transpose()?;
    if let Some(unknown) = rule_styles
        .iter()
        .flatten()
        .find(|s| !styles.iter().any(|st| &st.id == *s))
    {
        return Err(format!("unknown style {unknown}"));
    }
    Ok(Rule {
        label: string("label")?.ok_or("a rule needs a label")?,
        a,
        b,
        direction: string("direction")?.filter(|s| !s.is_empty()),
        profile: string("profile")?.filter(|s| !s.is_empty()),
        from: number("from")?,
        to: number("to")?,
        at: match string("at")?.as_deref() {
            None => None,
            Some("start-or-rest") => Some(At::StartOrRest),
            Some("complete") => Some(At::Complete),
            Some("closed") => Some(At::Closed),
            Some("settled") => Some(At::Settled),
            Some(other) => return Err(format!("unknown at {other}")),
        },
        when: expr("when")?,
        styles: rule_styles,
        min_visibility: number("minVisibility")?,
        visible_check: match string("visibleCheck")?.as_deref() {
            None => default_visible,
            Some("both") => VisibleCheck::Both,
            Some("a") => VisibleCheck::A,
            Some("none") => VisibleCheck::None,
            Some(other) => return Err(format!("unknown visibleCheck {other}")),
        },
        record: match get("record") {
            None => default_record,
            Some(Value::Bool(true)) => Recording::Map,
            Some(Value::Bool(false)) => Recording::None,
            Some(Value::String(field)) => Recording::Field(field.clone()),
            Some(_) => return Err("record is a boolean or a field name".into()),
        },
        count: match get("count") {
            None => Vec::new(),
            Some(Value::Object(m)) => m
                .iter()
                .map(|(k, v)| {
                    v.as_u64()
                        .map(|n| (k.clone(), n))
                        .ok_or(format!("count {k} must be a whole number"))
                })
                .collect::<Result<_, _>>()?,
            Some(_) => return Err("count maps counter names to increments".into()),
        },
        contract: string("contract")?,
        slack: number("slack")?,
        failure: get("failure")
            .map(|v| {
                serde_json::from_value(v.clone())
                    .map_err(|_| "failure is collect or assert".to_string())
            })
            .transpose()?,
        message: string("message")?
            .map(|m| Template::parse(&m, MESSAGE_VARS).map_err(|e| e.to_string()))
            .transpose()?,
        kind,
    })
}
