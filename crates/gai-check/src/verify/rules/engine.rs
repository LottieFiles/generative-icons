//! The rules interpreter. Order is fixed: icon → action → style → frame (145 samples at
//! `t = f/60`) → canvas margin → rules in document order, or icon → action → rule → frame
//! for rule-major documents. Actions run in parallel and merge in action order, which
//! reproduces the sequential key order, counters, aggregates and errors.

use super::expr::{Clock, Env, Part, PartValue, Val};
use super::template::Vars;
use super::{
    At, Field, Kind, Op, Program, Recording, Rule, Style, TailOutcome, Trim, VisibleCheck,
};
use crate::contracts::geom::{Point, bounds, canvas_margin, inside_poly, pad_box, radius};
use crate::contracts::jsnum::{hypot2, max2, min_of, min2, sum};
use crate::contracts::rules::{Failure, GapRadius, VisibleTest};
use crate::contracts::scene::Scene;
use crate::verify::geom::{Flattener, Pick, pick_index, poly_distance};
use crate::verify::js::{same_numbers, strict_equals, to_number};
use crate::verify::manifest::Record;
use crate::verify::{Evidence, SetContext, VerifyError, evidence_json, generic::visibility};
use gai_core::parallel::ordered_map;
use indexmap::IndexMap;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::rc::Rc;

/// Frames per action.
const FRAMES: u32 = 145;

/// What one action contributes; merged in action order.
#[derive(Debug, Default)]
struct ActionOut {
    counters: HashMap<String, u64>,
    aggs: HashMap<String, f64>,
    margin: Option<f64>,
    map: IndexMap<String, (bool, f64)>,
    errors: Vec<String>,
    contracts: Vec<String>,
}

impl ActionOut {
    fn count(&mut self, name: &str, n: u64) {
        *self.counters.entry(name.to_string()).or_default() += n;
    }
}

fn fold(max: bool, a: f64, b: f64) -> f64 {
    if max { max2(a, b) } else { min2(a, b) }
}

fn identity(max: bool) -> f64 {
    if max {
        f64::NEG_INFINITY
    } else {
        f64::INFINITY
    }
}

/// Runs a program over every action of the set and assembles its evidence.
pub(super) fn run(program: &Program, set: &SetContext<'_>) -> Result<TailOutcome, VerifyError> {
    let flattener = Flattener::new(program.dialect.flatten);
    let aggs: HashMap<&str, bool> = program
        .evidence
        .fields
        .iter()
        .chain(
            program
                .evidence
                .embed
                .iter()
                .flat_map(|e| e.fields.iter().flatten()),
        )
        .filter_map(|(name, f)| match f {
            Field::Agg {
                max, margin: false, ..
            } => Some((name.as_str(), *max)),
            _ => None,
        })
        .collect();
    let mut jobs: Vec<(&str, &[Rule], Record<'_>)> = Vec::new();
    for icon in set.manifest.icons()? {
        let name = icon.name()?;
        let rules = program.icons.get(name).map(Vec::as_slice);
        if rules.is_none() {
            if let Some(message) = &program.require_icon {
                return Err(VerifyError::Assertion(message.render(&Vars {
                    icon: name,
                    ..Vars::default()
                })));
            }
            if program.skip_unlisted {
                continue;
            }
        }
        for action in icon.list("animations")? {
            jobs.push((name, rules.unwrap_or_default(), action));
        }
    }
    let outs = ordered_map(jobs.len(), |i| {
        let (icon, rules, action) = jobs[i];
        let scene = set.scene(action.name()?)?;
        ActionRun::new(program, &flattener, &aggs, &scene, icon, action, rules).run()
    })?;

    let mut total = ActionOut::default();
    for out in outs {
        for (k, n) in out.counters {
            total.count(&k, n);
        }
        for (k, v) in out.aggs {
            let max = aggs[k.as_str()];
            let slot = total.aggs.entry(k).or_insert(identity(max));
            *slot = fold(max, *slot, v);
        }
        total.margin = match (total.margin, out.margin) {
            (Some(a), Some(b)) => Some(min2(a, b)),
            (a, b) => a.or(b),
        };
        for (k, (max, v)) in out.map {
            let slot = total.map.entry(k).or_insert((max, v));
            slot.1 = fold(max, slot.1, v);
        }
        total.errors.extend(out.errors);
        for c in out.contracts {
            if !total.contracts.contains(&c) {
                total.contracts.push(c);
            }
        }
    }
    let actions: Vec<&str> = jobs
        .iter()
        .map(|(_, _, a)| a.name())
        .collect::<Result<_, _>>()?;
    let evidence = &program.evidence;
    let main = build(&evidence.fields, &total, &actions, None);
    let embed = evidence.embed.as_ref().map(|e| {
        let value = match &e.fields {
            Some(fields) => Value::Object(build(fields, &total, &actions, Some(&main))),
            None => Value::Object(main.clone()),
        };
        (e.key.clone(), value, e.before_loop_checks)
    });
    Ok(TailOutcome {
        file: evidence.file.as_ref().map(|path| Evidence {
            path: path.clone(),
            contents: evidence_json(&Value::Object(main.clone())),
        }),
        embed,
        fail_directed: evidence.embed.as_ref().is_some_and(|e| e.fail_directed),
        contracts: total.contracts.len(),
        errors: total.errors,
    })
}

/// Evaluates evidence fields in order; `main` resolves `copy` sources.
fn build(
    fields: &[(String, Field)],
    total: &ActionOut,
    actions: &[&str],
    main: Option<&Map<String, Value>>,
) -> Map<String, Value> {
    let mut out = Map::new();
    for (name, field) in fields {
        let value = match field {
            Field::Status => Value::from(if total.errors.is_empty() {
                "PASS"
            } else {
                "FAIL"
            }),
            Field::Counter(c) => Value::from(total.counters.get(c).copied().unwrap_or(0)),
            Field::Agg { max, init, margin } => {
                let partial = if *margin {
                    total.margin
                } else {
                    total.aggs.get(name).copied()
                };
                Value::from(partial.map_or(*init, |p| fold(*max, *init, p)))
            }
            Field::RuleMap => Value::Object(
                total
                    .map
                    .iter()
                    .map(|(k, (_, v))| (k.clone(), Value::from(*v)))
                    .collect(),
            ),
            Field::Errors => Value::from(total.errors.clone()),
            Field::PerAction(template) => {
                Value::Array(actions.iter().map(|a| expand(template, a)).collect())
            }
            Field::Contracts => Value::from(total.contracts.clone()),
            Field::Const(v) => v.clone(),
            Field::Copy(source) => main
                .and_then(|m| m.get(source))
                .cloned()
                .unwrap_or(Value::Null),
        };
        out.insert(name.clone(), value);
    }
    out
}

/// A `perAction` template: `"{action}"` strings become the action name.
fn expand(template: &Value, action: &str) -> Value {
    match template {
        Value::String(s) if s == "{action}" => Value::from(action),
        Value::Object(m) => Value::Object(
            m.iter()
                .map(|(k, v)| (k.clone(), expand(v, action)))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// One action's evaluation state. Caches are per frame and pure.
struct ActionRun<'a> {
    program: &'a Program,
    flattener: &'a Flattener,
    aggs: &'a HashMap<&'a str, bool>,
    scene: &'a Scene,
    icon: &'a str,
    action: Record<'a>,
    action_name: &'a str,
    rules: &'a [Rule],
    keys: Vec<String>,
    frame: u32,
    t: f64,
    style: Option<&'a Style>,
    each: Option<usize>,
    picks: HashMap<(String, Pick, Option<usize>), usize>,
    coords: Vec<Option<Rc<[f64]>>>,
    flat: Vec<Option<Rc<[Point]>>>,
    out: ActionOut,
}

/// A measured rule: value, pass, the threshold the message prints, and a second value.
struct Measure {
    value: f64,
    pass: bool,
    shown: f64,
    other: f64,
    /// Aggregate of the rule-map entry: `true` for a running maximum from 0 (`touch`).
    max_from_zero: bool,
}

impl<'a> ActionRun<'a> {
    fn new(
        program: &'a Program,
        flattener: &'a Flattener,
        aggs: &'a HashMap<&'a str, bool>,
        scene: &'a Scene,
        icon: &'a str,
        action: Record<'a>,
        rules: &'a [Rule],
    ) -> ActionRun<'a> {
        let keys = rules
            .iter()
            .map(|r| {
                program.evidence.key.render(&Vars {
                    icon,
                    label: &r.label,
                    ..Vars::default()
                })
            })
            .collect();
        ActionRun {
            program,
            flattener,
            aggs,
            scene,
            icon,
            action,
            action_name: action.str("name").unwrap_or_default(),
            rules,
            keys,
            frame: 0,
            t: 0.0,
            style: None,
            each: None,
            picks: HashMap::new(),
            coords: vec![None; scene.nodes.len()],
            flat: vec![None; scene.nodes.len()],
            out: ActionOut::default(),
        }
    }

    fn set_frame(&mut self, f: u32) {
        self.frame = f;
        self.t = f64::from(f) / 60.0;
        self.coords.iter_mut().for_each(|c| *c = None);
        self.flat.iter_mut().for_each(|c| *c = None);
    }

    fn run(mut self) -> Result<ActionOut, VerifyError> {
        let rules = self.rules;
        self.out.count("rules", rules.len() as u64);
        if self.program.dialect.rule_major {
            for (i, rule) in rules.iter().enumerate() {
                for f in 0..FRAMES {
                    self.set_frame(f);
                    self.eval_rule(i, rule)?;
                }
            }
            return Ok(self.out);
        }
        let styles: Vec<Option<&'a Style>> = if self.program.styles.is_empty() {
            vec![None]
        } else {
            self.program.styles.iter().map(Some).collect()
        };
        for style in styles {
            self.style = style;
            for f in 0..FRAMES {
                self.set_frame(f);
                self.out.count("frames", 1);
                self.margins()?;
                for (i, rule) in rules.iter().enumerate() {
                    self.eval_rule(i, rule)?;
                }
            }
        }
        Ok(self.out)
    }

    fn sampler(&self) -> crate::contracts::geom::Sampler {
        self.program.dialect.sampler
    }

    fn vis(&self, i: usize) -> Result<f64, VerifyError> {
        Ok(self
            .sampler()
            .scalar(visibility(&self.scene.nodes[i])?, self.t))
    }

    fn radius(&self, i: usize) -> f64 {
        radius(&self.scene.nodes[i], self.sampler(), self.t)
    }

    fn coords(&mut self, i: usize) -> Rc<[f64]> {
        if let Some(c) = &self.coords[i] {
            return c.clone();
        }
        let c: Rc<[f64]> = self
            .sampler()
            .coords(&self.scene.nodes[i].points, self.t)
            .into();
        self.coords[i] = Some(c.clone());
        c
    }

    fn flat(&mut self, i: usize) -> Result<Rc<[Point]>, VerifyError> {
        if let Some(p) = &self.flat[i] {
            return Ok(p.clone());
        }
        let coords = self.coords(i);
        let node = &self.scene.nodes[i];
        let p: Rc<[Point]> = self
            .flattener
            .points(&node.name, &node.commands, &coords)?
            .into();
        self.flat[i] = Some(p.clone());
        Ok(p)
    }

    /// Resolves a part to a node index with the dialect picker and the current style.
    fn resolve(&mut self, part: &Part) -> Result<usize, VerifyError> {
        if part.name == "@" {
            return self
                .each
                .ok_or_else(|| VerifyError::Rules("@ is only defined inside an each rule".into()));
        }
        let dialect = &self.program.dialect;
        let how = part.pick.unwrap_or(dialect.pick);
        let name = if dialect.part_prefix {
            format!("{}-{}", self.icon, part.name)
        } else {
            part.name.clone()
        };
        let style_index = self
            .style
            .and_then(|s| self.program.styles.iter().position(|x| std::ptr::eq(x, s)));
        let key = (name, how, style_index);
        if let Some(&i) = self.picks.get(&key) {
            return Ok(i);
        }
        let suffixes: &[String] = match self.style {
            Some(style) if how == Pick::Exact => std::slice::from_ref(&style.suffix),
            _ => &dialect.suffixes,
        };
        let i = pick_index(self.scene, how, &key.0, suffixes).ok_or_else(|| {
            VerifyError::Assertion(format!("{} missing {}", self.scene.name, key.0))
        })?;
        self.picks.insert(key, i);
        Ok(i)
    }

    fn margins(&mut self) -> Result<(), VerifyError> {
        let Some(spec) = &self.program.margin else {
            return Ok(());
        };
        for i in 0..self.scene.nodes.len() {
            let node = &self.scene.nodes[i];
            if !spec
                .name_ends_with
                .iter()
                .any(|s| node.name.ends_with(s.as_str()))
                || !spec.visible.visible(self.vis(i)?)
            {
                continue;
            }
            let (points, r) = (self.flat(i)?, self.radius(i));
            let m = if spec.per_point {
                min_of(
                    points
                        .iter()
                        .map(|q| min_of([q[0], q[1], 32.0 - q[0], 32.0 - q[1]])),
                ) - r
            } else {
                canvas_margin(pad_box(bounds(points.iter().copied()), r))
            };
            self.out.margin = Some(self.out.margin.map_or(m, |x| min2(x, m)));
            self.out.count("checks", 1);
            if spec
                .failure
                .fails(m, crate::contracts::rules::Bound::AtLeast, spec.min)
            {
                let vars = Vars {
                    action: self.action_name,
                    node: &self.scene.nodes[i].name,
                    frame: self.frame,
                    value: m,
                    ..Vars::default()
                };
                self.fail(spec.failure, spec.message.render(&vars))?;
            }
        }
        Ok(())
    }

    fn fail(&mut self, failure: Failure, message: String) -> Result<(), VerifyError> {
        match failure {
            Failure::Collect => {
                self.out.errors.push(message);
                Ok(())
            }
            Failure::Assert => Err(VerifyError::Assertion(message)),
        }
    }

    /// Whether the rule's frame gates let it run now.
    fn live(&mut self, rule: &Rule) -> Result<bool, VerifyError> {
        if let (Some(allowed), Some(style)) = (&rule.styles, self.style)
            && !allowed.contains(&style.id)
        {
            return Ok(false);
        }
        let action = self.action;
        let differs = |want: &Option<String>, key: &str| {
            want.as_ref()
                .is_some_and(|w| !strict_equals(action.get(key), Some(&Value::from(w.as_str()))))
        };
        if differs(&rule.direction, "direction") || differs(&rule.profile, "motionId") {
            return Ok(false);
        }
        if rule.from.is_some_and(|from| self.t < from) || rule.to.is_some_and(|to| self.t > to) {
            return Ok(false);
        }
        let f = self.frame;
        let forward = action.str("direction") == Some("forward");
        let at = match rule.at {
            None => true,
            Some(At::StartOrRest) => self.t == 0.0 || f64::from(f) == to_number(action.get("rest")),
            Some(At::Complete) => f == if forward { 144 } else { 0 },
            Some(At::Closed) => f == if forward { 0 } else { 144 },
            Some(At::Settled) => {
                forward
                    && f64::from(f) >= gai_core::js_round(to_number(action.get("active")) * 60.0)
            }
        };
        if !at {
            return Ok(false);
        }
        match &rule.when {
            Some(when) => when.truthy(self, None),
            None => Ok(true),
        }
    }

    fn eval_rule(&mut self, index: usize, rule: &'a Rule) -> Result<(), VerifyError> {
        if !self.live(rule)? {
            return Ok(());
        }
        let dialect = &self.program.dialect;
        let gate = VisibleTest {
            op: dialect.visible.op,
            value: rule.min_visibility.unwrap_or(dialect.visible.value),
        };
        let a = rule.a.as_ref().map(|p| self.resolve(p)).transpose()?;
        if rule.visible_check != VisibleCheck::None
            && let Some(a) = a
            && !gate.visible(self.vis(a)?)
        {
            return Ok(());
        }
        let b = rule.b.as_ref().map(|p| self.resolve(p)).transpose()?;
        if rule.visible_check == VisibleCheck::Both
            && let Some(b) = b
            && !gate.visible(self.vis(b)?)
        {
            return Ok(());
        }
        if let Kind::Each {
            name_ends_with,
            visible,
            value,
            op,
            limit,
        } = &rule.kind
        {
            for i in 0..self.scene.nodes.len() {
                if !name_ends_with
                    .iter()
                    .any(|s| self.scene.nodes[i].name.ends_with(s.as_str()))
                    || !visible.visible(self.vis(i)?)
                {
                    continue;
                }
                self.each = Some(i);
                let (v, l) = (value.number(self, None)?, limit.number(self, None)?);
                self.each = None;
                let pass = self.compare(v, *op, l, rule);
                self.conclude(
                    index,
                    rule,
                    Measure {
                        value: v,
                        pass,
                        shown: l,
                        other: f64::NAN,
                        max_from_zero: false,
                    },
                )?;
            }
            return Ok(());
        }
        let m = self.measure(rule, a, b)?;
        self.conclude(index, rule, m)
    }

    /// Whether `value op limit` passes, with the rule's slack on `>=` and `<=`.
    fn compare(&self, value: f64, op: Op, limit: f64, rule: &Rule) -> bool {
        self.passes(
            value,
            op,
            limit,
            rule.slack.unwrap_or(self.program.dialect.slack),
            rule,
        )
    }

    /// Whether `value op limit` passes with no slack (kinds whose legacy test had none).
    fn strict(&self, value: f64, op: Op, limit: f64, rule: &Rule) -> bool {
        self.passes(value, op, limit, 0.0, rule)
    }

    /// A NaN measurement fails under a positive `ok` test (assert mode, `nanFails`) and
    /// passes under `if (value < limit)`.
    fn passes(&self, value: f64, op: Op, limit: f64, slack: f64, rule: &Rule) -> bool {
        let nan_fails = match rule.failure {
            Some(f) => f == Failure::Assert,
            None => self.program.dialect.nan_fails,
        };
        let holds = |ok: Option<bool>| ok.unwrap_or(!nan_fails);
        let ord = |a: f64, b: f64| a.partial_cmp(&b);
        match op {
            Op::AtLeast => holds(ord(value, limit - slack).map(|o| o.is_ge())),
            Op::AtMost => holds(ord(value, limit + slack).map(|o| o.is_le())),
            Op::Above => holds(ord(value, limit).map(|o| o.is_gt())),
            Op::Below => holds(ord(value, limit).map(|o| o.is_lt())),
            Op::Close(tol) => {
                let d = (value - limit).abs();
                holds((!d.is_nan()).then_some(d < tol))
            }
        }
    }

    /// Endpoint `end` (0 first, 1 last) of a polyline.
    fn endpoint(points: &[Point], end: &Value, label: &str) -> Result<Point, VerifyError> {
        let p = match end.as_f64() {
            Some(0.0) => points.first(),
            Some(1.0) => points.last(),
            _ => None,
        };
        p.copied()
            .ok_or_else(|| VerifyError::Geometry(format!("{label}: no endpoint {end}")))
    }

    fn measure(
        &mut self,
        rule: &Rule,
        a: Option<usize>,
        b: Option<usize>,
    ) -> Result<Measure, VerifyError> {
        let sp = self.program.dialect.single_point;
        let dist = |p: Point, line: &[Point]| poly_distance(p, line, sp);
        let parts = || -> Result<(usize, usize), VerifyError> {
            a.zip(b)
                .ok_or_else(|| VerifyError::Rules(format!("{} needs parts a and b", rule.label)))
        };
        let only_a = || a.ok_or_else(|| VerifyError::Rules(format!("{} needs part a", rule.label)));
        let first = |p: &[Point]| {
            p.first()
                .copied()
                .ok_or_else(|| VerifyError::Geometry(format!("{}: empty path", rule.label)))
        };
        let at_least = |run: &Self, value: f64, limit: f64| Measure {
            value,
            pass: run.compare(value, Op::AtLeast, limit, rule),
            shown: limit,
            other: f64::NAN,
            max_from_zero: false,
        };
        Ok(match &rule.kind {
            Kind::Gap { min, trim } => {
                let (a, b) = parts()?;
                let (mut pa, pb) = (self.flat(a)?, self.flat(b)?);
                if let Some(trim) = trim {
                    let n = pa.len() as f64;
                    pa = match trim {
                        Trim::Start => pa[((n * 0.12).ceil() as usize).min(pa.len())..].into(),
                        Trim::End => pa[..((n * 0.88).floor() as usize).min(pa.len())].into(),
                    };
                }
                let m = pa.iter().fold(f64::INFINITY, |m, &p| min2(m, dist(p, &pb)));
                let m = pb.iter().fold(m, |m, &p| min2(m, dist(p, &pa)));
                let (ra, rb) = (self.radius(a), self.radius(b));
                let value = match self.program.dialect.gap_radius {
                    GapRadius::Sequential => (m - ra) - rb,
                    GapRadius::Summed => m - (ra + rb),
                };
                at_least(self, value, *min)
            }
            Kind::AttachPaint { end } => {
                let (a, b) = parts()?;
                let (pa, pb) = (self.flat(a)?, self.flat(b)?);
                let e = if crate::verify::js::truthy(Some(end)) {
                    pa.last().copied()
                } else {
                    pa.first().copied()
                };
                let e =
                    e.ok_or_else(|| VerifyError::Geometry(format!("{}: empty path", rule.label)))?;
                at_least(self, (self.radius(b) + self.radius(a)) - dist(e, &pb), 0.0)
            }
            Kind::AttachTouch { end, through } => {
                let (a, b) = parts()?;
                let (pa, pb) = (self.flat(a)?, self.flat(b)?);
                let (stroke, rb) = (self.scene.nodes[b].is_stroke(), self.radius(b));
                let touches = |p: Point| {
                    let d = dist(p, &pb);
                    if stroke {
                        rb - d
                    } else if inside_poly(p, &pb) {
                        0.0
                    } else {
                        -d
                    }
                };
                let value = if *through {
                    let mut best = f64::NEG_INFINITY;
                    for (i, &p) in pa.iter().enumerate() {
                        if i == 0 {
                            best = max2(best, touches(p));
                            continue;
                        }
                        let q = pa[i - 1];
                        for j in 0..12 {
                            let j = f64::from(j);
                            best = max2(
                                best,
                                touches([0, 1].map(|k| q[k] + (p[k] - q[k]) * j / 12.0)),
                            );
                        }
                    }
                    best
                } else {
                    let p = if end.as_f64() == Some(0.0) {
                        pa.first()
                    } else {
                        pa.last()
                    };
                    touches(*p.ok_or_else(|| {
                        VerifyError::Geometry(format!("{}: empty path", rule.label))
                    })?)
                };
                at_least(self, value, -0.001)
            }
            Kind::ContainedIn => {
                let (a, b) = parts()?;
                let ba = pad_box(bounds(self.flat(a)?.iter().copied()), self.radius(a));
                let bb = pad_box(bounds(self.flat(b)?.iter().copied()), self.radius(b));
                at_least(
                    self,
                    min_of([ba[0] - bb[0], ba[1] - bb[1], bb[2] - ba[2], bb[3] - ba[3]]),
                    0.0,
                )
            }
            Kind::InsideBox {
                bounds: [l, top, r, bottom],
                min,
            } => {
                let a = only_a()?;
                let b = pad_box(bounds(self.flat(a)?.iter().copied()), self.radius(a));
                at_least(
                    self,
                    min_of([b[0] - l, b[1] - top, r - b[2], bottom - b[3]]),
                    *min,
                )
            }
            Kind::InsidePoints {
                bounds: [l, top, r, bottom],
            } => {
                let a = only_a()?;
                let p = self.flat(a)?;
                let m = min_of(
                    p.iter()
                        .map(|q| min_of([q[0] - l, q[1] - top, r - q[0], bottom - q[1]])),
                ) - self.radius(a);
                at_least(self, m, 0.0)
            }
            Kind::Route { route, min } => {
                let a = only_a()?;
                let b = pad_box(bounds(self.flat(a)?.iter().copied()), self.radius(a));
                let centre = [(b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0];
                at_least(self, -dist(centre, route), *min)
            }
            Kind::Touch { max } => {
                let (a, b) = parts()?;
                let tip = first(&self.flat(a)?)?;
                let v = dist(tip, &self.flat(b)?);
                Measure {
                    value: v,
                    pass: self.strict(v, Op::AtMost, *max, rule),
                    shown: *max,
                    other: f64::NAN,
                    max_from_zero: true,
                }
            }
            Kind::OutsideCircle { r, tol } => {
                let (a, b) = parts()?;
                let tip = first(&self.flat(a)?)?;
                let pb = self.flat(b)?;
                let n = pb.len() as f64;
                let c = [0, 1].map(|i| sum(pb.iter().map(|q| q[i])) / n);
                let v = hypot2(tip[0] - c[0], tip[1] - c[1]);
                let pass = self.strict(v, Op::AtLeast, r - tol, rule);
                Measure {
                    value: v,
                    pass,
                    shown: *r,
                    other: f64::NAN,
                    max_from_zero: false,
                }
            }
            Kind::Exclusive => {
                let (a, b) = parts()?;
                let (va, vb) = (self.vis(a)?, self.vis(b)?);
                let gate = self.program.dialect.visible;
                Measure {
                    value: va,
                    pass: !(gate.visible(va) && gate.visible(vb)),
                    shown: f64::NAN,
                    other: vb,
                    max_from_zero: false,
                }
            }
            Kind::Joint { end, max } => {
                let (a, b) = parts()?;
                let e = Self::endpoint(&self.flat(a)?, end, &rule.label)?;
                let d = dist(e, &self.flat(b)?);
                Measure {
                    value: d,
                    pass: self.strict(d, Op::AtMost, *max, rule),
                    shown: *max,
                    other: f64::NAN,
                    max_from_zero: false,
                }
            }
            Kind::Contact { max } => {
                let (a, b) = parts()?;
                let (pa, pb) = (self.flat(a)?, self.flat(b)?);
                let d = pa.iter().fold(f64::INFINITY, |m, &p| min2(m, dist(p, &pb)));
                Measure {
                    value: d,
                    pass: self.compare(d, Op::AtMost, *max, rule),
                    shown: *max,
                    other: f64::NAN,
                    max_from_zero: false,
                }
            }
            Kind::Buried { end, min } => {
                let (a, b) = parts()?;
                let e = Self::endpoint(&self.flat(a)?, end, &rule.label)?;
                let poly = self.flat(b)?;
                let (inside, d) = (inside_poly(e, &poly), dist(e, &poly));
                Measure {
                    value: if inside { d } else { -d },
                    pass: inside && d >= *min,
                    shown: *min,
                    other: f64::NAN,
                    max_from_zero: false,
                }
            }
            Kind::Frozen => {
                let a = only_a()?;
                let now = self.coords(a);
                let still = same_numbers(&now, &self.scene.nodes[a].points[0].1);
                Measure {
                    value: f64::NAN,
                    pass: still,
                    shown: f64::NAN,
                    other: f64::NAN,
                    max_from_zero: false,
                }
            }
            Kind::Compare { value, op, limit } => {
                let (v, l) = (value.number(self, None)?, limit.number(self, None)?);
                Measure {
                    value: v,
                    pass: self.compare(v, *op, l, rule),
                    shown: l,
                    other: f64::NAN,
                    max_from_zero: false,
                }
            }
            Kind::Each { .. } => {
                return Err(VerifyError::Rules(format!(
                    "{}: each rules run per node",
                    rule.label
                )));
            }
        })
    }

    /// Records the measurement, counts it and reports a failure.
    fn conclude(&mut self, index: usize, rule: &Rule, m: Measure) -> Result<(), VerifyError> {
        match &rule.record {
            Recording::Map => {
                let init = if m.max_from_zero { 0.0 } else { f64::INFINITY };
                let slot = self
                    .out
                    .map
                    .entry(self.keys[index].clone())
                    .or_insert((m.max_from_zero, init));
                slot.1 = fold(m.max_from_zero, slot.1, m.value);
            }
            Recording::Field(name) => {
                let max = self.aggs[name.as_str()];
                let slot = self.out.aggs.entry(name.clone()).or_insert(identity(max));
                *slot = fold(max, *slot, m.value);
            }
            Recording::None => {}
        }
        self.out.count("checks", 1);
        for (name, n) in &rule.count {
            self.out.count(name, *n);
        }
        if let Some(c) = &rule.contract
            && !self.out.contracts.contains(c)
        {
            self.out.contracts.push(c.clone());
        }
        if !m.pass {
            let vars = Vars {
                icon: self.icon,
                action: self.action_name,
                label: &rule.label,
                key: &self.keys[index],
                frame: self.frame,
                value: m.value,
                min: m.shown,
                other: m.other,
                ..Vars::default()
            };
            let message = rule
                .message
                .as_ref()
                .unwrap_or(&self.program.evidence.message)
                .render(&vars);
            self.fail(
                rule.failure.unwrap_or(self.program.dialect.failure),
                message,
            )?;
        }
        Ok(())
    }
}

impl Env for ActionRun<'_> {
    fn part(&mut self, what: PartValue, part: &Part) -> Result<Val, VerifyError> {
        let i = self.resolve(part)?;
        Ok(match what {
            PartValue::Pts => Val::Nums(self.coords(i)),
            PartValue::Flat => Val::Points(self.flat(i)?),
            PartValue::Vis => Val::Num(self.vis(i)?),
            PartValue::Width => Val::Num(self.sampler().scalar(&self.scene.nodes[i].width, self.t)),
            PartValue::Radius => Val::Num(self.radius(i)),
        })
    }

    fn clock(&self, which: Clock) -> f64 {
        match which {
            Clock::Time => self.t,
            Clock::Frame => f64::from(self.frame),
            Clock::Rest => to_number(self.action.get("rest")),
        }
    }

    fn single_point(&self) -> bool {
        self.program.dialect.single_point
    }
}

#[cfg(test)]
mod tests {
    use super::super::RuleDoc;
    use crate::verify::{SetContext, SetManifest, SetPaths, VerifyOptions};
    use serde_json::{Value, json};

    /// A stroke node of width 2 (radius 1) held still for the whole action.
    fn node(part: &str, commands: &[u8], coords: &[f64]) -> Value {
        json!({"name": format!("cup-{part}-bold__color.primary__outline.bold"), "commands": commands, "type": "stroke",
               "points": [[0, coords], [2.4, coords]], "width": [[0, 2], [2.4, 2]], "visibility": [[0, 1], [2.4, 1]]})
    }

    /// Runs `rules` for icon `cup` over one action whose scene has:
    /// `a` (10,10)–(10,20), `b` (15,10)–(15,20), `c` (15,20)–(15,25), `sq` the closed square
    /// (0,0)–(30,30), and `mover`, which slides right by 1 over the action.
    fn run(rules: Value) -> (Value, Vec<String>) {
        let mover = json!({"name": "cup-mover-bold__color.primary__outline.bold", "commands": [0, 1], "type": "fill",
                           "points": [[0, [0, 0, 0, 5]], [2.4, [1, 0, 1, 5]]], "width": [[0, 2]], "visibility": [[0, 1]]});
        let scene = json!({"name": "cup", "nodes": [
            node("a", &[0, 1], &[10., 10., 10., 20.]), node("b", &[0, 1], &[15., 10., 15., 20.]),
            node("c", &[0, 1], &[15., 20., 15., 25.]), node("sq", &[0, 1, 1, 1, 3], &[0., 0., 30., 0., 30., 30., 0., 30.]), mover]});
        let doc = json!({"format": "gai-verify-rules", "version": 2,
            "dialect": {"sampler": "ge", "flatten": {"cubicSteps": 4, "close": "first", "unknown": "throw"}, "pick": "bold",
                        "visible": {"op": ">=", "value": 0.01}, "slack": 0.015, "gapRadius": "sequential", "failure": "collect"},
            "icons": {"cup": rules},
            "evidence": {"file": "out.json", "fields": [["gaps", {"map": "rules"}], ["best", {"agg": "max", "init": 0}], ["checks", {"counter": "checks"}], ["errors", {"errors": true}]],
                         "key": "{label}", "message": "{label} {f}: {value|fixed3} vs {min}"}});
        let rules = RuleDoc::from_slice(&serde_json::to_vec(&doc).unwrap()).unwrap();
        let paths = SetPaths::catalog("/nonexistent/00-test");
        let options = VerifyOptions::default();
        let manifest = SetManifest {
            doc: json!({"id": "00-test", "icons": [{"name": "cup", "animations": [{"name": "cup", "direction": "forward", "rest": 30}]}]}),
        };
        let set = SetContext::new(&paths, manifest, &options);
        set.cache_scene("cup", &serde_json::to_vec(&scene).unwrap())
            .unwrap();
        let outcome = rules.run(&set).unwrap();
        (
            serde_json::from_str(&outcome.file.unwrap().contents).unwrap(),
            outcome.errors,
        )
    }

    /// Runs one rule and returns its recorded value and whether any frame failed.
    fn measure(rule: Value) -> (Value, bool) {
        let (evidence, errors) = run(json!([rule]));
        (evidence["gaps"]["r"].clone(), !errors.is_empty())
    }

    #[test]
    fn distance_kinds() {
        let r = |kind: &str, extra: Value| {
            let mut rule = json!({"kind": kind, "label": "r"});
            rule.as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            measure(rule)
        };
        // a and b are 5 apart, less both radii.
        assert_eq!(
            r("gap", json!({"a": "a", "b": "b", "min": 1})),
            (json!(3), false)
        );
        assert_eq!(
            r("gap", json!({"a": "a", "b": "b", "min": 4})),
            (json!(3), true)
        );
        // Trimming a's start keeps the end at (10, 20): still 5 from b.
        assert_eq!(
            r("gap", json!({"a": "a", "b": "b", "trim": "start"})),
            (json!(3), false)
        );
        // c starts on b: (rB + rA) - 0; a is 5 away.
        assert_eq!(
            r("attach-paint", json!({"a": "c", "b": "b", "end": 0})),
            (json!(2), false)
        );
        assert_eq!(
            r("attach-paint", json!({"a": "a", "b": "b", "end": 1})),
            (json!(-3), true)
        );
        // Stroke b: rB - distance, at a's last point or along its length.
        assert_eq!(
            r("attach-touch", json!({"a": "c", "b": "b", "end": 0})),
            (json!(1), false)
        );
        assert_eq!(
            r("attach-touch", json!({"a": "a", "b": "b", "through": true})),
            (json!(-4), true)
        );
        assert_eq!(
            r("contained-in", json!({"a": "a", "b": "sq"})),
            (Value::Null, false)
        );
        assert_eq!(
            r(
                "inside-box",
                json!({"a": "a", "box": [8, 8, 12, 22], "min": 0.5})
            ),
            (json!(1), false)
        );
        assert_eq!(
            r(
                "inside-box",
                json!({"a": "a", "box": [8, 8, 12, 22], "min": 2})
            ),
            (json!(1), true)
        );
        assert_eq!(
            r("inside-points", json!({"a": "a", "box": [8, 8, 12, 22]})),
            (json!(1), false)
        );
        assert_eq!(
            r(
                "on-route",
                json!({"a": "a", "route": [[10, 0], [10, 30]], "min": -0.05})
            ),
            (json!(0), false)
        );
        assert_eq!(
            r(
                "hop",
                json!({"a": "a", "route": [[0, 15], [1, 15]], "min": -2.25})
            ),
            (json!(-9), true)
        );
        assert_eq!(
            r(
                "touch",
                json!({"a": "c", "b": "b", "max": 0.05, "at": "start-or-rest"})
            ),
            (json!(0), false)
        );
        assert_eq!(
            r("touch", json!({"a": "a", "b": "b", "max": 0.05})),
            (json!(5), true)
        );
        let diagonal = 50f64.sqrt();
        assert_eq!(
            r(
                "outside-circle",
                json!({"a": "a", "b": "b", "r": 7, "tol": 0})
            ),
            (json!(diagonal), false)
        );
        assert_eq!(
            r(
                "outside-circle",
                json!({"a": "a", "b": "b", "r": 8, "tol": 0.05})
            ),
            (json!(diagonal), true)
        );
        assert_eq!(
            r("joint", json!({"a": "c", "b": "b", "end": 0})),
            (json!(0), false)
        );
        assert_eq!(
            r("joint", json!({"a": "a", "b": "b", "end": 1})),
            (json!(5), true)
        );
        assert_eq!(
            r("contact", json!({"a": "a", "b": "b", "max": 6})),
            (json!(5), false)
        );
        assert_eq!(
            r("buried", json!({"a": "a", "b": "sq", "end": 0})),
            (json!(10), false)
        );
        assert_eq!(
            r("buried", json!({"a": "a", "b": "b", "end": 0})),
            (json!(-5), true)
        );
    }

    #[test]
    fn state_kinds_and_gates() {
        let (_, failed) = measure(json!({"kind": "exclusive", "label": "r", "a": "a", "b": "b"}));
        assert!(failed);
        let (_, failed) = measure(json!({"kind": "frozen", "label": "r", "a": "a"}));
        assert!(!failed);
        let (_, failed) = measure(json!({"kind": "frozen", "label": "r", "a": "mover"}));
        assert!(failed);
        // Only frame 0 of a forward action is "closed"; the rule runs once.
        let (evidence, errors) = run(
            json!([{"kind": "joint", "label": "r", "a": "a", "b": "b", "end": 1, "at": "closed"}]),
        );
        assert_eq!((evidence["checks"].clone(), errors.len()), (json!(1), 1));
        assert_eq!(errors[0], "r 0: 5.000 vs 0.02");
        // Direction and profile gates skip the whole action.
        let (evidence, _) =
            run(json!([{"kind": "gap", "label": "r", "a": "a", "b": "b", "direction": "reverse"}]));
        assert_eq!(evidence["checks"], json!(0));
    }

    #[test]
    fn compare_records_counts_and_each() {
        let rules = json!([
            {"kind": "compare", "label": "r", "value": {"sub": [{"at": [{"pts": "a"}, 3]}, 5]}, "op": ">=", "limit": 10},
            {"kind": "compare", "label": "moved", "value": {"at": [{"pts": "mover"}, 0]}, "op": "close", "limit": 0, "record": "best"},
            {"kind": "each", "label": "inside", "nodes": {"nameEndsWith": ["__outline.bold"]}, "visible": {"op": ">", "value": 0},
             "value": {"add": [{"maxOf": {"pts": "@"}}, {"radius": "@"}]}, "op": "<=", "limit": 30}
        ]);
        let (evidence, errors) = run(rules);
        // The second rule records into its field, not the map.
        assert_eq!(evidence["gaps"], json!({"r": 15}));
        assert_eq!(evidence["best"], json!(1));
        // The mover leaves x = 0 after frame 0; the square's 30 plus its radius exceeds 30.
        assert_eq!(
            errors.iter().filter(|e| e.starts_with("moved")).count(),
            144
        );
        assert_eq!(
            errors.iter().filter(|e| e.starts_with("inside")).count(),
            145
        );
        // Two compare rules plus the each rule once per outline node (five), 145 frames.
        assert_eq!(evidence["checks"], json!(145 * 7));
    }
}
