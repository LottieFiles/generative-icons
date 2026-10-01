//! `{var}` templates for rule keys and messages: the contracts syntax plus the
//! `|fixed2`/`|fixed3` filters (JS `toFixed`) and `{other}`.

use crate::verify::VerifyError;
use crate::verify::js::to_fixed;
use std::fmt::Write as _;

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
    /// Scene node name (margin messages).
    Node,
    /// Frame number.
    Frame,
    /// Measured value.
    Value,
    /// The threshold as the legacy message printed it.
    Min,
    /// A second measured value (the other part of an `exclusive` rule).
    Other,
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
            "other" => Var::Other,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Piece {
    Text(String),
    /// A variable and its `toFixed` digits, if filtered.
    Var(Var, Option<usize>),
}

/// A parsed template.
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
    /// `{other}`
    pub other: f64,
}

/// Variables of a rule key.
pub const KEY_VARS: &[Var] = &[Var::Icon, Var::Label];
/// Variables of a rule message.
pub const MESSAGE_VARS: &[Var] = &[
    Var::Icon,
    Var::Action,
    Var::Label,
    Var::Key,
    Var::Frame,
    Var::Value,
    Var::Min,
    Var::Other,
];
/// Variables of a margin message.
pub const MARGIN_VARS: &[Var] = &[Var::Action, Var::Node, Var::Frame, Var::Value];
/// Variables of a set-level message.
pub const ICON_VARS: &[Var] = &[Var::Icon];

impl Template {
    /// Parses `text`, allowing only `allowed` variables.
    pub fn parse(text: &str, allowed: &[Var]) -> Result<Template, VerifyError> {
        let bad = |m: String| VerifyError::Rules(m);
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
            let (name, filter) = match rest[open + 1..close].split_once('|') {
                Some((name, "fixed2")) => (name, Some(2)),
                Some((name, "fixed3")) => (name, Some(3)),
                Some((_, f)) => {
                    return Err(bad(format!("unknown filter {f} in template {text:?}")));
                }
                None => (&rest[open + 1..close], None),
            };
            let var = Var::parse(name)
                .filter(|v| allowed.contains(v))
                .ok_or_else(|| bad(format!("unknown variable {{{name}}} in template {text:?}")))?;
            pieces.push(Piece::Var(var, filter));
            rest = &rest[close + 1..];
        }
        if !rest.is_empty() {
            pieces.push(Piece::Text(rest.to_string()));
        }
        Ok(Template(pieces))
    }

    /// Expands the template; numbers print as JS `Number::toString` or `toFixed`.
    pub fn render(&self, vars: &Vars<'_>) -> String {
        let mut out = String::new();
        let number = |out: &mut String, x: f64, filter: Option<usize>| match filter {
            Some(d) => out.push_str(&to_fixed(x, d)),
            None => out.push_str(&gai_core::js_number_to_string(x)),
        };
        for piece in &self.0 {
            match *piece {
                Piece::Text(ref s) => out.push_str(s),
                Piece::Var(v, filter) => match v {
                    Var::Icon => out.push_str(vars.icon),
                    Var::Action => out.push_str(vars.action),
                    Var::Label => out.push_str(vars.label),
                    Var::Key => out.push_str(vars.key),
                    Var::Node => out.push_str(vars.node),
                    Var::Frame => {
                        let _ = write!(out, "{}", vars.frame);
                    }
                    Var::Value => number(&mut out, vars.value, filter),
                    Var::Min => number(&mut out, vars.min, filter),
                    Var::Other => number(&mut out, vars.other, filter),
                },
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_js_numbers_and_to_fixed() {
        let t = Template::parse(
            "{action} {key} frame {f}: {value|fixed3} < {min}",
            MESSAGE_VARS,
        )
        .unwrap();
        let vars = Vars {
            action: "cup",
            key: "cup gap a/b",
            frame: 7,
            value: 0.9844999,
            min: 1.0,
            ..Vars::default()
        };
        assert_eq!(t.render(&vars), "cup cup gap a/b frame 7: 0.984 < 1");
        let t = Template::parse("{value|fixed2} and {other|fixed2}", MESSAGE_VARS).unwrap();
        assert_eq!(
            t.render(&Vars {
                value: 0.125,
                other: -0.0,
                ..vars
            }),
            "0.13 and 0.00"
        );
        assert!(Template::parse("{node}", KEY_VARS).is_err());
        assert!(Template::parse("{value|fixed9}", MESSAGE_VARS).is_err());
        assert!(Template::parse("{icon", KEY_VARS).is_err());
    }
}
