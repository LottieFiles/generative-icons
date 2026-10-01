//! Reads the definitions' `icons` export with JS semantics and runs every check that needs
//! no sampling (`preflight.mjs:47-87`): names, labels, parts, paints, callbacks, Duotone and
//! state policy, timing and key poses. The result is the diagnostics of each icon, in the
//! order `add` saw them, and what to sample.

use super::{
    contract::FamilyContract,
    report::{Code, Detail, Diagnostic},
};
use gai_js::{
    JsScope, ProbeWidth,
    rquickjs::{self, Function, IntoJs, Object, Value},
};
use serde_json::Value as Json;
use std::collections::HashSet;

/// V8's TypeError texts for `width / 2` (`preflight.mjs:114`), which QuickJS words differently.
const BIGINT_MIX: &str = "Cannot mix BigInt and other types, use explicit conversions";
const SYMBOL_TO_NUMBER: &str = "Cannot convert a Symbol value to a number";

pub(super) enum Plan {
    /// `icons` is not an array, or is empty.
    EmptySet,
    Icons {
        /// `icons.length`.
        count: usize,
        icons: Vec<IconPlan>,
    },
}

pub(super) struct IconPlan {
    /// `icon?.name` as `JSON.stringify` sees it.
    pub name: Option<Json>,
    /// The icon- and part-level diagnostics, in `add` order.
    pub checks: Vec<Diagnostic>,
    /// `None` when the icon stopped at `ICON_NAME` or `EMPTY_PARTS`.
    pub sampling: Option<Sampling>,
}

pub(super) struct Sampling {
    /// Forward then reverse (stateful icons), each over the contract's profiles.
    pub actions: Vec<Action>,
    /// `icon.parts.filter(p => p && typeof p.name === 'string')`.
    pub parts: Vec<SampledPart>,
}

pub(super) struct Action {
    pub profile: String,
    pub strong: bool,
    pub reverse: bool,
    /// The sorted, unique sample times, or the timing code that stops the action.
    pub times: Result<Vec<f64>, Code>,
}

pub(super) struct SampledPart {
    /// Index into the icon's authored `parts`.
    pub index: usize,
    pub name: String,
    /// The contract's outline weights for `style: 'outline'`, else `authored`.
    pub weights: Vec<Weight>,
    /// `typeof` of each callback is `'function'`: when the JS called something else,
    /// the diagnostic carries V8's TypeError text.
    pub visibility_callable: bool,
    pub pose_callable: bool,
    pub transform_callable: bool,
    /// `d` is a string and `pose` and `visibility` are falsy: no author code runs, so every
    /// time yields the same sample.
    pub constant: bool,
}

pub(super) struct Weight {
    /// `light`, `regular`, `bold` or `authored`.
    pub name: String,
    pub width: ProbeWidth,
    pub radius: Radius,
}

/// `part.type === 'stroke' ? width / 2 : 0`.
pub(super) enum Radius {
    Number(f64),
    /// `width / 2` throws this TypeError (a BigInt or Symbol width) at every sample.
    Throws(&'static str),
}

/// Inspects every icon of the scope's `icons` export.
pub(super) fn inspect(scope: &JsScope<'_>, contract: &FamilyContract) -> rquickjs::Result<Plan> {
    let js = Js::new(scope)?;
    let icons = scope.icons();
    if !js.is_array(icons)? {
        return Ok(Plan::EmptySet);
    }
    let count = js.length(icons)?;
    if count == 0 {
        return Ok(Plan::EmptySet);
    }
    let frames = contract.frame_times();
    let icons = (0..count)
        .map(|index| js.get(icons, index).and_then(|icon| inspect_icon(&js, &icon, contract, &frames)))
        .collect::<rquickjs::Result<_>>()?;
    Ok(Plan::Icons { count: count as usize, icons })
}

/// `icon?.name` for each icon when it is a string, for the catalog collision check
/// (`preflight.mjs:137-138`); `None` when `icons` is not an array.
pub(super) fn icon_names(scope: &JsScope<'_>) -> rquickjs::Result<Option<Vec<Option<String>>>> {
    let js = Js::new(scope)?;
    let icons = scope.icons();
    if !js.is_array(icons)? {
        return Ok(None);
    }
    let names = (0..js.length(icons)?)
        .map(|index| js.get(icons, index).and_then(|icon| js.string(&js.get_opt(&icon, "name")?)))
        .collect::<rquickjs::Result<_>>()?;
    Ok(Some(names))
}

fn inspect_icon<'js>(
    js: &Js<'_, 'js>,
    icon: &Value<'js>,
    contract: &FamilyContract,
    frames: &[f64],
) -> rquickjs::Result<IconPlan> {
    let raw_name = js.get_opt(icon, "name")?;
    let name = js.scope.project(raw_name.clone())?;
    let mut checks = Vec::new();
    let mut add = |code: Code, part: Option<&str>, message: Option<String>| {
        let message = message.or_else(|| code.fixed_message().map(str::to_owned)).map(Json::from);
        checks.push(Diagnostic {
            code,
            icon: name.clone(),
            part: part.map(str::to_owned),
            message,
            detail: Detail::None,
        });
    };
    if !js.truthy(icon)? || !js.kebab(&raw_name)? {
        add(Code::IconName, None, None);
        return Ok(IconPlan { name, checks, sampling: None });
    }
    if js.seen(&raw_name)? {
        add(Code::DuplicateIcon, None, None);
    }
    if js.string(&js.get(icon, "label")?)?.is_none_or(|label| is_blank(&label)) {
        add(Code::Label, None, None);
    }
    let parts = js.get(icon, "parts")?;
    if !js.is_array(&parts)? || js.length(&parts)? == 0 {
        add(Code::EmptyParts, None, None);
        return Ok(IconPlan { name, checks, sampling: None });
    }

    let mut names = HashSet::new();
    let mut sampled = Vec::new();
    for index in 0..js.length(&parts)? {
        let part = js.get(&parts, index)?;
        let truthy = js.truthy(&part)?;
        let part_name = js.string(&js.get_opt(&part, "name")?)?;
        if let Some(part_name) = part_name.as_ref().filter(|_| truthy) {
            sampled.push(sampled_part(js, index as usize, part_name, &part, contract)?);
        }
        let Some(checked) = part_name.filter(|name| truthy && !is_blank(name)) else {
            add(Code::PartName, None, None);
            continue;
        };
        let part_name = Some(checked.as_str());
        if names.contains(&checked) {
            add(Code::AmbiguousLayerName, part_name, None);
        }
        let style = js.string(&js.get(&part, "style")?)?;
        if !style.as_ref().is_some_and(|style| contract.source_styles.contains(style)) {
            add(Code::PaintStyle, part_name, Some(format!("Expected {}", contract.source_styles.join(", "))));
        }
        let kind = js.get(&part, "type")?;
        let kind_name = js.string(&kind)?;
        if !kind.is_undefined() && !matches!(kind_name.as_deref(), Some("fill" | "stroke")) {
            add(Code::PaintType, part_name, None);
        }
        let stroke = kind_name.as_deref() == Some("stroke");
        let width = js.get(&part, "width")?;
        if stroke && !width.as_number().is_some_and(|width| width.is_finite() && width > 0.0) {
            add(Code::StrokeWidth, part_name, None);
        }
        // `p.width !== contract.outlineWeights.regular`, strictly.
        let regular = match contract.regular_width() {
            Some(regular) => width.as_number() == Some(regular),
            None => width.is_undefined(),
        };
        if stroke && style.as_deref() == Some("outline") && !regular {
            add(Code::OutlineWidth, part_name, None);
        }
        for callback in ["pose", "visibility"] {
            let value = js.get(&part, callback)?;
            if !value.is_undefined() && !js.is_function(&value)? {
                add(Code::Callback, part_name, Some(format!("{callback} must be a function")));
            }
        }
        names.insert(checked);
    }

    let duotone = js.get(icon, "duotoneFill")?;
    if duotone.is_undefined() {
        add(Code::DuotoneUndeclared, None, None);
    } else if !matches!(duotone.as_bool(), Some(declared) if declared == has_tint(js, &parts)?) {
        add(Code::DuotonePolicy, None, None);
    }
    let stateful = js.truthy(&js.get(icon, "stateful")?)?;
    if stateful && (!js.truthy(&js.get(icon, "initialState")?)? || !js.truthy(&js.get(icon, "finalState")?)?) {
        add(Code::StatePolicy, None, None);
    }
    if stateful && !js.truthy(&js.get(icon, "reverseSuffix")?)? {
        add(Code::LegacyReverseName, None, None);
    }

    let directions: &[bool] = if stateful { &[false, true] } else { &[false] };
    let mut actions = Vec::new();
    for &reverse in directions {
        for profile in &contract.profiles {
            actions.push(Action {
                profile: profile.clone(),
                strong: profile == "strong",
                reverse,
                times: action_times(js, icon, reverse, profile, contract, frames)?,
            });
        }
    }
    Ok(IconPlan { name, checks, sampling: Some(Sampling { actions, parts: sampled }) })
}

/// `icon.parts.some(p => p?.style === 'tint')`.
fn has_tint<'js>(js: &Js<'_, 'js>, parts: &Value<'js>) -> rquickjs::Result<bool> {
    for index in 0..js.length(parts)? {
        let part = js.get(parts, index)?;
        if js.string(&js.get_opt(&part, "style")?)?.as_deref() == Some("tint") {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The TIMING and KEY_POSES checks of one action (`preflight.mjs:78-87`), then its sample
/// times: every frame, the key poses, `active` and `rest`, unique and ascending.
fn action_times<'js>(
    js: &Js<'_, 'js>,
    icon: &Value<'js>,
    reverse: bool,
    profile: &str,
    contract: &FamilyContract,
    frames: &[f64],
) -> rquickjs::Result<Result<Vec<f64>, Code>> {
    let duration = contract.duration_seconds;
    let timings = js.get(icon, if reverse { "reverseTimings" } else { "timings" })?;
    let timing = js.get_opt(&timings, profile)?;
    if !js.truthy(&timing)? {
        return Ok(Err(Code::Timing));
    }
    let Some(active) = finite(&js.get(&timing, "active")?).filter(|active| *active > 0.0 && *active <= duration) else {
        return Ok(Err(Code::Timing));
    };
    let Some(rest) = finite(&js.get(&timing, "rest")?).filter(|rest| *rest >= 0.0 && *rest <= duration) else {
        return Ok(Err(Code::Timing));
    };
    if rest < active && !js.truthy(&js.get(icon, "continuous")?)? {
        return Ok(Err(Code::Timing));
    }
    let keys = js.get(&timing, "keyPoses")?;
    if !js.is_array(&keys)? {
        return Ok(Err(Code::KeyPoses));
    }
    let count = js.length(&keys)?;
    let mut poses = Vec::with_capacity(count as usize);
    for index in 0..count {
        // `some` skips holes, and spreading one would sample at `undefined`; a hole fails here.
        if !js.has(&keys, index)? {
            return Ok(Err(Code::KeyPoses));
        }
        let ordered = |t: &f64| *t >= 0.0 && *t <= duration && poses.last().is_none_or(|last| t >= last);
        let Some(time) = finite(&js.get(&keys, index)?).filter(ordered) else {
            return Ok(Err(Code::KeyPoses));
        };
        poses.push(time);
    }
    if poses.is_empty() {
        return Ok(Err(Code::KeyPoses));
    }
    // `[...new Set(times)].sort((a, b) => a - b)`: the Set stores -0 as +0.
    let mut times: Vec<f64> = frames.iter().chain(&poses).chain([&active, &rest]).map(|t| t + 0.0).collect();
    times.sort_by(f64::total_cmp);
    times.dedup();
    Ok(Ok(times))
}

/// How to sample a truthy part whose `name` is a string (`preflight.mjs:88-90`), including
/// parts whose blank name or paint failed the checks.
fn sampled_part<'js>(
    js: &Js<'_, 'js>,
    index: usize,
    name: &str,
    part: &Value<'js>,
    contract: &FamilyContract,
) -> rquickjs::Result<SampledPart> {
    let outline = js.string(&js.get(part, "style")?)?.as_deref() == Some("outline");
    let stroke = js.string(&js.get(part, "type")?)?.as_deref() == Some("stroke");
    let weights = if outline {
        let weight = |(name, width): &(String, f64)| Weight {
            name: name.clone(),
            width: ProbeWidth::Value(*width),
            radius: Radius::Number(if stroke { width / 2.0 } else { 0.0 }),
        };
        contract.outline_weights.iter().map(weight).collect()
    } else {
        let radius = if stroke { js.half(&js.get(part, "width")?)? } else { Radius::Number(0.0) };
        vec![Weight { name: "authored".into(), width: ProbeWidth::Authored, radius }]
    };
    let (pose, visibility) = (js.get(part, "pose")?, js.get(part, "visibility")?);
    let constant = js.get(part, "d")?.is_string() && !js.truthy(&pose)? && !js.truthy(&visibility)?;
    Ok(SampledPart {
        index,
        name: name.to_owned(),
        weights,
        visibility_callable: js.is_function(&visibility)?,
        pose_callable: js.is_function(&pose)?,
        transform_callable: js.is_function(&js.get_opt(&pose, "transform")?)?,
        constant,
    })
}

/// `Number.isFinite(value) ? value : None`.
fn finite(value: &Value<'_>) -> Option<f64> {
    value.as_number().filter(|number| number.is_finite())
}

/// `!s.trim()`: every code point is ECMAScript WhiteSpace or LineTerminator. That set has
/// U+FEFF and not U+0085 or U+180E, unlike Rust's `trim`.
fn is_blank(s: &str) -> bool {
    s.chars().all(|c| {
        matches!(c, '\u{9}'..='\u{D}' | ' ' | '\u{A0}' | '\u{1680}' | '\u{2000}'..='\u{200A}')
            || matches!(c, '\u{2028}' | '\u{2029}' | '\u{202F}' | '\u{205F}' | '\u{3000}' | '\u{FEFF}')
    })
}

/// The JS operations of `helpers.js`, bound to one scope.
struct Js<'a, 'js> {
    scope: &'a JsScope<'js>,
    get: Function<'js>,
    type_of: Function<'js>,
    truthy: Function<'js>,
    is_array: Function<'js>,
    has: Function<'js>,
    to_number: Function<'js>,
    kebab: Function<'js>,
    seen: Function<'js>,
}

impl<'a, 'js> Js<'a, 'js> {
    fn new(scope: &'a JsScope<'js>) -> rquickjs::Result<Self> {
        let helpers: Object<'js> = scope.ctx().eval(include_str!("helpers.js"))?;
        Ok(Js {
            scope,
            get: helpers.get("get")?,
            type_of: helpers.get("typeOf")?,
            truthy: helpers.get("truthy")?,
            is_array: helpers.get("isArray")?,
            has: helpers.get("has")?,
            to_number: helpers.get("toNumber")?,
            kebab: helpers.get("kebab")?,
            seen: helpers.get("seen")?,
        })
    }

    /// `value[key]` on a value that is not `null` or `undefined`.
    fn get(&self, value: &Value<'js>, key: impl IntoJs<'js>) -> rquickjs::Result<Value<'js>> {
        self.get.call((value.clone(), key))
    }

    /// `value?.[key]`.
    fn get_opt(&self, value: &Value<'js>, key: impl IntoJs<'js>) -> rquickjs::Result<Value<'js>> {
        if value.is_undefined() || value.is_null() {
            return Ok(Value::new_undefined(self.scope.ctx().clone()));
        }
        self.get(value, key)
    }

    fn type_of(&self, value: &Value<'js>) -> rquickjs::Result<String> {
        self.type_of.call((value.clone(),))
    }

    fn is_function(&self, value: &Value<'js>) -> rquickjs::Result<bool> {
        Ok(self.type_of(value)? == "function")
    }

    fn truthy(&self, value: &Value<'js>) -> rquickjs::Result<bool> {
        self.truthy.call((value.clone(),))
    }

    fn is_array(&self, value: &Value<'js>) -> rquickjs::Result<bool> {
        self.is_array.call((value.clone(),))
    }

    /// `value.length` of an array.
    fn length(&self, array: &Value<'js>) -> rquickjs::Result<u32> {
        Ok(self.get(array, "length")?.as_number().unwrap_or(0.0) as u32)
    }

    /// `index in array`: false for a hole.
    fn has(&self, array: &Value<'js>, index: u32) -> rquickjs::Result<bool> {
        self.has.call((array.clone(), index))
    }

    /// The value when `typeof value === 'string'`.
    fn string(&self, value: &Value<'js>) -> rquickjs::Result<Option<String>> {
        value.as_string().map(|s| s.to_string()).transpose()
    }

    /// `/^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/.test(name)`, which applies ToString.
    fn kebab(&self, name: &Value<'js>) -> rquickjs::Result<bool> {
        self.kebab.call((name.clone(),))
    }

    /// `ids.has(name)`, then `ids.add(name)`.
    fn seen(&self, name: &Value<'js>) -> rquickjs::Result<bool> {
        self.seen.call((name.clone(),))
    }

    /// `width / 2`: ToNumber, except that a BigInt or Symbol throws V8's TypeError.
    fn half(&self, width: &Value<'js>) -> rquickjs::Result<Radius> {
        Ok(match self.type_of(width)?.as_str() {
            "bigint" => Radius::Throws(BIGINT_MIX),
            "symbol" => Radius::Throws(SYMBOL_TO_NUMBER),
            _ => Radius::Number(self.to_number.call::<_, f64>((width.clone(),))? / 2.0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::is_blank;

    #[test]
    fn blank_follows_js_trim() {
        assert!(is_blank(""));
        assert!(is_blank(" \t\n\u{B}\u{C}\r\u{A0}\u{2028}\u{3000}\u{FEFF}"));
        assert!(!is_blank("\u{85}"));
        assert!(!is_blank("\u{180E}"));
        assert!(!is_blank(" a "));
    }
}
