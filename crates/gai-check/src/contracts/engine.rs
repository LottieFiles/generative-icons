//! The rules interpreter (the 32–39 `verify-clearance.mjs` feature set, plus the §8
//! extensions the converted 16–20 rules need).
//!
//! Order is fixed: icon → action → frame (145 samples at `t = f/60`) → canvas margin →
//! rules in document order. Actions run in parallel and merge in action order, which
//! reproduces the sequential `gaps` insertion order, `checks`, `errors` and minimum.

use super::ContractError;
use super::evidence::{ClearanceReport, js_number};
use super::expr::Env;
use super::geom::{
    Flattener, PathView, Point, Sampler, bounds, canvas_margin, pad_box, poly_dist, polyline_gap,
    radius,
};
use super::jsnum::{hypot2, min_of, min2, slice_range};
use super::rules::{Bound, Failure, Field, GapRadius, Pick, Rule, RuleKind, RuleSet, Vars};
use super::scene::{Scene, SceneNode, SceneSource, SetAction, SetManifest};
use gai_core::parallel::ordered_map;
use indexmap::IndexMap;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::rc::Rc;

/// Frames per action: `f = 0..=144`, the full 2.4 s at 60 fps.
pub const FRAMES: u32 = 145;

/// What one action contributes to the report.
#[derive(Debug, Default)]
struct ActionOutcome {
    checks: u64,
    errors: Vec<String>,
    margin: f64,
    gaps: IndexMap<String, f64>,
}

/// Runs a rules document over every action of a set.
pub fn run(
    rules: &RuleSet,
    manifest: &SetManifest,
    scenes: &dyn SceneSource,
) -> Result<ClearanceReport, ContractError> {
    if let Some(set) = rules.set.as_ref().filter(|s| **s != manifest.id) {
        return Err(ContractError::Rules(format!(
            "rules are for {set}, not {}",
            manifest.id
        )));
    }
    let flattener = Flattener::new(rules.dialect.flatten);
    let jobs: Vec<(&str, &SetAction)> = manifest
        .icons
        .iter()
        .flat_map(|icon| icon.animations.iter().map(move |a| (icon.name.as_str(), a)))
        .collect();
    let outcomes = ordered_map(jobs.len(), |i| {
        let (icon, action) = jobs[i];
        let scene = scenes.load(&action.name)?;
        let icon_rules = match rules.icons.get(icon) {
            Some(r) => r.as_slice(),
            None if rules.require_icon => {
                return Err(ContractError::MissingContract(icon.to_string()));
            }
            None => &[],
        };
        ActionRun::new(rules, &flattener, &scene, icon, action, icon_rules).run()
    })?;

    let (mut checks, mut errors, mut margin, mut gaps) = (
        0u64,
        Vec::new(),
        f64::INFINITY,
        IndexMap::<String, f64>::new(),
    );
    for o in outcomes {
        checks += o.checks;
        errors.extend(o.errors);
        margin = min2(margin, o.margin);
        for (k, v) in o.gaps {
            let slot = gaps.entry(k).or_insert(f64::INFINITY);
            *slot = min2(*slot, v);
        }
    }
    let mut evidence = Map::new();
    for (name, field) in &rules.evidence.fields {
        let value = match field {
            Field::Status => Value::from(if errors.is_empty() { "PASS" } else { "FAIL" }),
            Field::PerAction(template) => Value::Array(
                jobs.iter()
                    .map(|(_, a)| per_action(template, &a.name))
                    .collect(),
            ),
            Field::MarginMin { init } => js_number(min2(*init, margin)),
            Field::RuleMap => Value::Object(
                gaps.iter()
                    .map(|(k, &v)| (k.clone(), js_number(v)))
                    .collect(),
            ),
            Field::Checks => Value::from(checks),
            Field::Errors => Value::from(errors.clone()),
            Field::Const(v) => v.clone(),
        };
        evidence.insert(name.clone(), value);
    }
    Ok(ClearanceReport {
        evidence: Value::Object(evidence),
        errors,
    })
}

/// One `perAction` entry: `"{action}"` becomes the action name; everything else is copied.
fn per_action(template: &Map<String, Value>, action: &str) -> Value {
    let expand = |v: &Value| {
        if v == "{action}" {
            Value::from(action)
        } else {
            v.clone()
        }
    };
    Value::Object(
        template
            .iter()
            .map(|(k, v)| (k.clone(), expand(v)))
            .collect(),
    )
}

/// Per-action evaluation state; caches are per frame and pure, so they never change a result.
struct ActionRun<'a> {
    rules: &'a RuleSet,
    flattener: &'a Flattener,
    scene: &'a Scene,
    icon: &'a str,
    action: &'a SetAction,
    icon_rules: &'a [Rule],
    keys: Vec<String>,
    frame: u32,
    t: f64,
    picks: HashMap<String, usize>,
    flat: Vec<Option<Rc<[Point]>>>,
    out: ActionOutcome,
}

impl<'a> ActionRun<'a> {
    fn new(
        rules: &'a RuleSet,
        flattener: &'a Flattener,
        scene: &'a Scene,
        icon: &'a str,
        action: &'a SetAction,
        icon_rules: &'a [Rule],
    ) -> Self {
        let keys = icon_rules
            .iter()
            .map(|r| {
                rules.evidence.key.render(&Vars {
                    icon,
                    label: &r.label,
                    ..Vars::default()
                })
            })
            .collect();
        ActionRun {
            rules,
            flattener,
            scene,
            icon,
            action,
            icon_rules,
            keys,
            frame: 0,
            t: 0.0,
            picks: HashMap::new(),
            flat: vec![None; scene.nodes.len()],
            out: ActionOutcome {
                margin: f64::INFINITY,
                ..ActionOutcome::default()
            },
        }
    }

    fn sampler(&self) -> Sampler {
        self.rules.dialect.sampler
    }

    fn run(mut self) -> Result<ActionOutcome, ContractError> {
        let margin_nodes: Vec<usize> = match &self.rules.margin {
            Some(m) => (0..self.scene.nodes.len())
                .filter(|&i| {
                    m.name_ends_with
                        .iter()
                        .any(|s| self.scene.nodes[i].name.ends_with(s.as_str()))
                })
                .collect(),
            None => Vec::new(),
        };
        for f in 0..FRAMES {
            self.frame = f;
            self.t = f64::from(f) / 60.0;
            self.flat.iter_mut().for_each(|c| *c = None);
            for &i in &margin_nodes {
                self.margin_check(i)?;
            }
            let icon_rules = self.icon_rules;
            for (i, rule) in icon_rules.iter().enumerate() {
                self.eval_rule(i, rule)?;
            }
        }
        Ok(self.out)
    }

    fn node_visibility(&self, node: &SceneNode) -> Result<f64, ContractError> {
        Ok(self.sampler().scalar(node.visibility()?, self.t))
    }

    /// Flattened polyline of a whole node, cached for the frame.
    fn flat_node(&mut self, i: usize) -> Result<Rc<[Point]>, ContractError> {
        if let Some(p) = &self.flat[i] {
            return Ok(p.clone());
        }
        let p: Rc<[Point]> = self
            .flattener
            .points(
                &PathView::whole(&self.scene.nodes[i]),
                self.sampler(),
                self.t,
            )?
            .into();
        self.flat[i] = Some(p.clone());
        Ok(p)
    }

    fn points(&mut self, i: usize, view: &PathView<'_>) -> Result<Rc<[Point]>, ContractError> {
        if view.is_whole() {
            self.flat_node(i)
        } else {
            Ok(self.flattener.points(view, self.sampler(), self.t)?.into())
        }
    }

    fn margin_check(&mut self, i: usize) -> Result<(), ContractError> {
        let (rules, scene) = (self.rules, self.scene);
        let Some(spec) = &rules.margin else {
            return Ok(());
        };
        let node = &scene.nodes[i];
        if !spec.visible.visible(self.node_visibility(node)?) {
            return Ok(());
        }
        let b = pad_box(
            bounds(self.flat_node(i)?.iter().copied()),
            radius(node, self.sampler(), self.t),
        );
        let m = canvas_margin(b);
        self.out.margin = min2(self.out.margin, m);
        self.out.checks += 1;
        if spec.failure.fails(m, Bound::AtLeast, spec.min) {
            let vars = Vars {
                action: &self.action.name,
                node: &node.name,
                frame: self.frame,
                value: m,
                ..Vars::default()
            };
            self.fail(spec.failure, spec.message.render(&vars))?;
        }
        Ok(())
    }

    fn fail(&mut self, failure: Failure, message: String) -> Result<(), ContractError> {
        match failure {
            Failure::Collect => {
                self.out.errors.push(message);
                Ok(())
            }
            Failure::Assert => Err(ContractError::Assertion(message)),
        }
    }

    /// Resolves `<icon>-<part>` with the dialect picker (a missing name reads `undefined`, as in JS).
    fn resolve(&mut self, part: Option<&str>) -> Result<usize, ContractError> {
        let name = format!("{}-{}", self.icon, part.unwrap_or("undefined"));
        if let Some(&i) = self.picks.get(&name) {
            return Ok(i);
        }
        let nodes = &self.scene.nodes;
        let found = match self.rules.dialect.pick {
            Pick::Bold => {
                let bold = format!("{name}-bold");
                nodes
                    .iter()
                    .position(|n| n.base() == bold)
                    .or_else(|| nodes.iter().position(|n| n.base() == name))
            }
            Pick::Base => nodes.iter().position(|n| n.base() == name),
        };
        let i = found.ok_or_else(|| ContractError::MissingPart(name.clone()))?;
        self.picks.insert(name, i);
        Ok(i)
    }

    /// Resolves a part, applies its subpath and tests its visibility; `None` when hidden.
    fn gated(
        &mut self,
        part: Option<&str>,
        segments: &Value,
    ) -> Result<Option<(usize, PathView<'a>)>, ContractError> {
        let i = self.resolve(part)?;
        let scene: &'a Scene = self.scene;
        let view = PathView::whole(&scene.nodes[i]).subpath(segments)?;
        let shown = self
            .rules
            .dialect
            .visible
            .visible(self.node_visibility(view.node)?);
        Ok(shown.then_some((i, view)))
    }

    fn eval_rule(&mut self, index: usize, rule: &'a Rule) -> Result<(), ContractError> {
        let action = self.action;
        let gated_off = |want: &Option<String>, have: &Option<String>| {
            want.as_ref().is_some_and(|w| have.as_ref() != Some(w))
        };
        if gated_off(&rule.direction, &action.direction)
            || gated_off(&rule.profile, &action.motion_id)
        {
            return Ok(());
        }
        if rule.from.is_some_and(|from| self.t < from) || rule.to.is_some_and(|to| self.t > to) {
            return Ok(());
        }
        if let Some(when) = &rule.when
            && !when.truthy(self)?
        {
            return Ok(());
        }
        let sampler = self.sampler();
        let (a, b) = (rule.a.as_deref(), rule.b.as_deref());
        match &rule.kind {
            RuleKind::Compare {
                value,
                bound,
                limit,
            } => {
                let value = value.number(self)?;
                let limit = limit.number(self)?;
                self.record(index, rule, value, *bound, limit)
            }
            RuleKind::Gap | RuleKind::ArrowCurveGap { .. } | RuleKind::SelfGap { .. } => {
                let Some((ai, a)) = self.gated(a, &rule.a_segments)? else {
                    return Ok(());
                };
                let Some((bi, b)) = self.gated(b, &rule.b_segments)? else {
                    return Ok(());
                };
                let curved = match rule.kind {
                    RuleKind::ArrowCurveGap {
                        curve_commands: Some(n),
                    } => b.clone().truncate(n),
                    _ => b.clone(),
                };
                let (pa, pb) = match rule.kind {
                    RuleKind::SelfGap {
                        a_command,
                        b_command,
                    } => {
                        let missing = || ContractError::UnknownPathCommand;
                        (
                            a.clone().segment(a_command.ok_or_else(missing)?)?,
                            b.clone().segment(b_command.ok_or_else(missing)?)?,
                        )
                    }
                    _ => (
                        rule.a_commands
                            .map_or_else(|| a.clone(), |n| a.clone().truncate(n)),
                        rule.b_commands
                            .map_or_else(|| curved.clone(), |n| curved.clone().truncate(n)),
                    ),
                };
                let (pa, pb) = (self.points(ai, &pa)?, self.points(bi, &pb)?);
                let (ra, rb) = (
                    radius(a.node, sampler, self.t),
                    radius(b.node, sampler, self.t),
                );
                let m = polyline_gap(&pa, &pb);
                let value = match self.rules.dialect.gap_radius {
                    GapRadius::Summed => m - (ra + rb),
                    GapRadius::Sequential => (m - ra) - rb,
                };
                self.record(index, rule, value, Bound::AtLeast, rule.min.unwrap_or(1.0))
            }
            RuleKind::Attachment { end } => {
                let Some((ai, a)) = self.gated(a, &rule.a_segments)? else {
                    return Ok(());
                };
                // B is neither subpathed nor visibility-gated.
                let bi = self.resolve(b)?;
                let pa = self.points(ai, &a)?;
                let p = match end {
                    Some(0) => pa.first(),
                    Some(1) => pa.last(),
                    _ => {
                        return Err(ContractError::Rules(format!(
                            "{}: attachment end must be 0 or 1",
                            rule.label
                        )));
                    }
                }
                .copied()
                .ok_or_else(|| {
                    ContractError::Geometry(format!("{} has an empty path", a.node.name))
                })?;
                let line = self.flat_node(bi)?;
                self.record(index, rule, -poly_dist(p, &line), Bound::AtLeast, -0.001)
            }
            RuleKind::Opaque => {
                let Some((_, a)) = self.gated(a, &rule.a_segments)? else {
                    return Ok(());
                };
                let v = self.node_visibility(a.node)?;
                self.record(index, rule, v, Bound::AtLeast, 1.0)
            }
            RuleKind::InsideBox { bounds: frame } => {
                let Some((ai, a)) = self.gated(a, &rule.a_segments)? else {
                    return Ok(());
                };
                let [l, top, r, bottom] = frame.ok_or_else(|| {
                    ContractError::Rules(format!("{}: inside needs a box", rule.label))
                })?;
                let pa = self.points(ai, &a)?;
                let b = pad_box(bounds(pa.iter().copied()), radius(a.node, sampler, self.t));
                let value = min_of([b[0] - l, b[1] - top, r - b[2], bottom - b[3]]);
                self.record(index, rule, value, Bound::AtLeast, rule.min.unwrap_or(0.0))
            }
            RuleKind::PointJoint { a_offset, b_offset } => {
                let Some((_, a)) = self.gated(a, &rule.a_segments)? else {
                    return Ok(());
                };
                let Some((_, b)) = self.gated(b, &rule.b_segments)? else {
                    return Ok(());
                };
                // `sample(points).slice(i, i + 2)`; missing coordinates read as NaN.
                let t = self.t;
                let pair = |view: &PathView<'_>, offset: &Option<i64>| -> [f64; 2] {
                    let c = view.coords(sampler, t);
                    let s = &c[offset.map_or(0..0, |o| slice_range(c.len(), o, Some(o + 2)))];
                    [
                        s.first().copied().unwrap_or(f64::NAN),
                        s.get(1).copied().unwrap_or(f64::NAN),
                    ]
                };
                let (pa, pb) = (pair(&a, a_offset), pair(&b, b_offset));
                self.record(
                    index,
                    rule,
                    -hypot2(pa[0] - pb[0], pa[1] - pb[1]),
                    Bound::AtLeast,
                    -0.001,
                )
            }
            RuleKind::Unknown(kind) => {
                // Like the JS, an unknown kind only fails once its part is visible.
                match self.gated(a, &rule.a_segments)? {
                    Some(_) => Err(ContractError::UnknownRule(kind.clone())),
                    None => Ok(()),
                }
            }
        }
    }

    /// `metric(value, min)`: record the minimum under the rule key, count the check and
    /// test it against the slack-adjusted threshold.
    fn record(
        &mut self,
        index: usize,
        rule: &Rule,
        value: f64,
        bound: Bound,
        limit: f64,
    ) -> Result<(), ContractError> {
        let key = &self.keys[index];
        if rule.record {
            let slot = self.out.gaps.entry(key.clone()).or_insert(f64::INFINITY);
            *slot = min2(*slot, value);
        }
        self.out.checks += 1;
        let dialect = &self.rules.dialect;
        let slack = rule.slack.unwrap_or(dialect.slack);
        let failure = rule.failure.unwrap_or(dialect.failure);
        let adjusted = match bound {
            Bound::AtLeast => limit - slack,
            Bound::AtMost => limit + slack,
        };
        if failure.fails(value, bound, adjusted) {
            let vars = Vars {
                icon: self.icon,
                action: &self.action.name,
                label: &rule.label,
                key,
                frame: self.frame,
                value,
                min: limit,
                ..Vars::default()
            };
            let message = rule
                .message
                .as_ref()
                .unwrap_or(&self.rules.evidence.message)
                .render(&vars);
            self.fail(failure, message)?;
        }
        Ok(())
    }
}

impl Env for ActionRun<'_> {
    fn coords(&mut self, part: &str) -> Result<Rc<[f64]>, ContractError> {
        let i = self.resolve(Some(part))?;
        Ok(self
            .sampler()
            .coords(&self.scene.nodes[i].points, self.t)
            .into())
    }

    fn flat(&mut self, part: &str) -> Result<Rc<[Point]>, ContractError> {
        let i = self.resolve(Some(part))?;
        self.flat_node(i)
    }

    fn visibility(&mut self, part: &str) -> Result<f64, ContractError> {
        let i = self.resolve(Some(part))?;
        self.node_visibility(&self.scene.nodes[i])
    }

    fn radius(&mut self, part: &str) -> Result<f64, ContractError> {
        let i = self.resolve(Some(part))?;
        Ok(radius(&self.scene.nodes[i], self.sampler(), self.t))
    }
}
