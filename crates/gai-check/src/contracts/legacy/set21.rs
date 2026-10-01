//! 21-annotation-arrows: head clearances over curved route spans, head and branch
//! attachment, and the loop's counter, checked once for Bold and once for Solid.

use super::{Frame, flattener20, frame_times};
use crate::contracts::ContractError;
use crate::contracts::evidence::{ClearanceReport, js_number};
use crate::contracts::geom::{Point, Sampler, poly_dist};
use crate::contracts::jsnum::{hypot2, min_of, min2, slice_range};
use crate::contracts::scene::{SceneNode, SceneSource, SetManifest, cmd};
use gai_core::js_number_to_string as js;
use gai_core::parallel::ordered_map;
use indexmap::IndexMap;
use serde_json::json;

/// Route-head tips per icon (`tips`, `21:56`).
const TIPS: &[(&str, Point)] = &[
    ("annotation-curve-right", [26.0, 9.0]),
    ("annotation-curve-left", [6.0, 9.0]),
    ("annotation-s-curve", [26.0, 9.0]),
    ("annotation-swoop-up", [25.5, 8.0]),
    ("annotation-swoop-down", [25.5, 24.0]),
    ("annotation-loop", [26.95, 17.13]),
    ("annotation-elbow", [25.0, 8.0]),
    ("annotation-callout", [25.5, 25.5]),
];

const EXCEPTIONS: [&str; 3] = [
    "Opaque source-route and route-head tip contacts",
    "Branch junction contacts within 4 grid units",
    "Curled pointer has one intentional stem crossover; its counter remains protected",
];

/// `spans`: one `[last, end]` pair per line and 65 points (`u = 0..=1`) per cubic.
/// `M` only moves; `Z` and unknown commands are ignored.
fn spans(node: &SceneNode, t: f64) -> Result<Vec<Vec<Point>>, ContractError> {
    let p = Sampler::Gt.coords(&node.points, t);
    let at = |k: usize| -> Result<Point, ContractError> {
        match (p.get(k), p.get(k + 1)) {
            (Some(&x), Some(&y)) => Ok([x, y]),
            _ => Err(ContractError::Geometry(format!(
                "{} has fewer coordinates than its commands use",
                node.name
            ))),
        }
    };
    let missing = || ContractError::Geometry(format!("{} draws before a move", node.name));
    let (mut out, mut k, mut last): (Vec<Vec<Point>>, usize, Option<Point>) = (Vec::new(), 0, None);
    for &c in &node.commands {
        match c {
            cmd::MOVE => {
                last = Some(at(k)?);
                k += 2;
            }
            cmd::LINE => {
                let end = at(k)?;
                out.push(vec![last.ok_or_else(missing)?, end]);
                last = Some(end);
                k += 2;
            }
            cmd::CUBIC => {
                let (a, b, cc, d) = (last.ok_or_else(missing)?, at(k)?, at(k + 2)?, at(k + 4)?);
                k += 6;
                out.push(
                    (0..=64)
                        .map(|j| {
                            let u = f64::from(j) / 64.0;
                            let v = 1.0 - u;
                            [0, 1].map(|i| {
                                v * v * v * a[i]
                                    + 3.0 * v * v * u * b[i]
                                    + 3.0 * v * u * u * cc[i]
                                    + u * u * u * d[i]
                            })
                        })
                        .collect(),
                );
                last = Some(d);
            }
            _ => {}
        }
    }
    Ok(out)
}

#[derive(Default)]
struct Outcome {
    checks: u64,
    errors: Vec<String>,
    margin: f64,
    gaps: IndexMap<String, f64>,
}

struct Style<'s, 'o> {
    fr: Frame<'s>,
    icon: &'s str,
    action: &'s str,
    frame: u32,
    solid: bool,
    out: &'o mut Outcome,
}

impl<'s> Style<'s, '_> {
    fn node(&self, name: &str) -> Result<&'s SceneNode, ContractError> {
        self.fr.node(&format!(
            "{}-{name}{}",
            self.icon,
            if self.solid { "-solid" } else { "" }
        ))
    }

    fn vis(&self, n: &SceneNode) -> Result<bool, ContractError> {
        Ok(self.fr.vis(n)? > 0.01)
    }

    fn width(&self, n: &SceneNode) -> f64 {
        Sampler::Gt.scalar(&n.width, self.fr.t)
    }

    fn metric(&mut self, label: &str, value: f64, min: f64) {
        let key = format!(
            "{} {}{label}",
            self.icon,
            if self.solid { "Solid " } else { "Bold " }
        );
        let slot = self.out.gaps.entry(key.clone()).or_insert(f64::INFINITY);
        *slot = min2(*slot, value);
        self.out.checks += 1;
        if value < min - 0.015 {
            self.out.errors.push(format!(
                "{} {key} f{}: {}",
                self.action,
                self.frame,
                js(value)
            ));
        }
    }

    fn head_gap(
        &mut self,
        head: &str,
        body: &str,
        skip_start: i64,
        skip_end: i64,
    ) -> Result<(), ContractError> {
        let (h, b) = (self.node(head)?, self.node(body)?);
        if !self.vis(h)? || !self.vis(b)? {
            return Ok(());
        }
        let all = spans(b, self.fr.t)?;
        let end = (skip_end != 0).then_some(-skip_end);
        let portions: Vec<&Vec<Point>> = all[slice_range(all.len(), skip_start, end)]
            .iter()
            .filter(|q| hypot2(q[0][0] - q[q.len() - 1][0], q[0][1] - q[q.len() - 1][1]) > 0.02)
            .collect();
        if portions.is_empty() {
            return Ok(());
        }
        let hp = self.fr.points_of(h)?;
        let bp: Vec<Point> = portions.into_iter().flatten().copied().collect();
        let m = hp
            .iter()
            .fold(f64::INFINITY, |m, &p| min2(m, poly_dist(p, &bp)));
        let m = bp.iter().fold(m, |m, &p| min2(m, poly_dist(p, &hp)));
        let d = m - self.width(h) / 2.0 - self.width(b) / 2.0;
        self.metric(&format!("{head} clears {body} curved section"), d, 1.0);
        Ok(())
    }

    /// `attach` and `branchWait`: the end of `body` meets `tip`, and `body` is visible
    /// whenever `head` is.
    fn joined(
        &mut self,
        head: &str,
        body: &str,
        tip: Point,
        labels: [String; 2],
    ) -> Result<(), ContractError> {
        let (h, b) = (self.node(head)?, self.node(body)?);
        if !self.vis(h)? {
            return Ok(());
        }
        let p = self.fr.points_of(b)?;
        let end = p
            .last()
            .copied()
            .ok_or_else(|| ContractError::Geometry(format!("{body} has an empty path")))?;
        let [waits, visible] = labels;
        self.metric(&waits, -hypot2(end[0] - tip[0], end[1] - tip[1]), -0.001);
        let shown = if self.vis(b)? { 1.0 } else { 0.0 };
        self.metric(&visible, shown, 1.0);
        Ok(())
    }

    fn attach(&mut self, head: &str, body: &str, tip: Point) -> Result<(), ContractError> {
        self.joined(
            head,
            body,
            tip,
            [
                format!("{head} waits for full {body}"),
                format!("{head} has visible {body}"),
            ],
        )
    }

    fn branch_wait(
        &mut self,
        child: &str,
        parent: &str,
        junction: Point,
    ) -> Result<(), ContractError> {
        self.joined(
            child,
            parent,
            junction,
            [
                format!("{child} waits for {parent}"),
                format!("{child} parent visible"),
            ],
        )
    }

    fn loop_counter(&mut self) -> Result<(), ContractError> {
        let n = self.node("route")?;
        if !self.vis(n)? {
            return Ok(());
        }
        let qs: Vec<Vec<Point>> = spans(n, self.fr.t)?.into_iter().skip(1).collect();
        let w = self.width(n);
        let nearest = |c: Point| {
            min_of(
                qs.iter()
                    .flatten()
                    .map(|p| hypot2(p[0] - c[0], p[1] - c[1])),
            )
        };
        let d = nearest([13.0, 14.0]) - w / 2.0;
        self.metric("loop counter radius", d, 3.7);
        let d2 = nearest([17.65, 23.94]) - 2.0 - w / 2.0;
        // Collapsed future segments are ignored while the initial stem is still writing.
        let first = qs.first().ok_or_else(|| {
            ContractError::Geometry("annotation-loop route has no second span".into())
        })?;
        let (a, z) = (first[0], first[first.len() - 1]);
        if hypot2(a[0] - z[0], a[1] - z[1]) > 0.02 {
            self.metric("return stroke clears anchor", d2, 1.0);
        }
        Ok(())
    }

    fn check(&mut self) -> Result<(), ContractError> {
        let icon = self.icon;
        if let Some(&(_, tip)) = TIPS.iter().find(|(id, _)| *id == icon) {
            self.head_gap("head", "route", 0, 1)?;
            self.attach("head", "route", tip)?;
        }
        match icon {
            "annotation-loop" => self.loop_counter()?,
            "annotation-bracket" => {
                for c in ["upper", "lower"] {
                    self.branch_wait(c, "leader", [11.0, 16.0])?;
                }
            }
            "annotation-split" => {
                for (side, other, y) in [("up", "down", 9.5), ("down", "up", 22.5)] {
                    self.branch_wait(&format!("branch-{side}"), "trunk", [12.0, 16.0])?;
                    self.attach(
                        &format!("head-{side}"),
                        &format!("branch-{side}"),
                        [26.0, y],
                    )?;
                    self.head_gap(&format!("head-{side}"), &format!("branch-{side}"), 0, 1)?;
                    self.head_gap(&format!("head-{side}"), &format!("branch-{other}"), 0, 0)?;
                }
            }
            "annotation-merge" => {
                for side in ["up", "down"] {
                    self.branch_wait("trunk", &format!("branch-{side}"), [20.0, 16.0])?;
                    self.head_gap("head", &format!("branch-{side}"), 0, 0)?;
                }
                self.attach("head", "trunk", [26.0, 16.0])?;
            }
            "annotation-double-ended" => {
                self.head_gap("head-source", "body", 1, 1)?;
                self.head_gap("head-target", "body", 1, 1)?;
                self.attach("head-target", "body", [25.5, 22.0])?;
            }
            _ => {}
        }
        Ok(())
    }
}

pub(super) fn run(
    manifest: &SetManifest,
    scenes: &dyn SceneSource,
) -> Result<ClearanceReport, ContractError> {
    let flattener = flattener20();
    let jobs: Vec<_> = manifest
        .icons
        .iter()
        .flat_map(|i| i.animations.iter().map(move |a| (i.name.as_str(), a)))
        .collect();
    let outcomes = ordered_map(jobs.len(), |j| {
        let (icon, action) = jobs[j];
        let scene = scenes.load(&action.name)?;
        let mut o = Outcome {
            margin: f64::INFINITY,
            ..Outcome::default()
        };
        for (f, t) in frame_times() {
            let fr = Frame {
                scene: &scene,
                flattener: &flattener,
                t,
            };
            fr.margins(|n, m| {
                o.margin = min2(o.margin, m);
                o.checks += 1;
                if m < 1.99 {
                    o.errors
                        .push(format!("{} {} f{f}: margin {}", action.name, n.name, js(m)));
                }
                Ok(())
            })?;
            for solid in [false, true] {
                Style {
                    fr,
                    icon,
                    action: &action.name,
                    frame: f,
                    solid,
                    out: &mut o,
                }
                .check()?;
            }
        }
        Ok(o)
    })?;

    let (mut checks, mut errors, mut margin, mut gaps) =
        (0u64, Vec::new(), 32.0, IndexMap::<String, f64>::new());
    for o in outcomes {
        checks += o.checks;
        errors.extend(o.errors);
        margin = min2(margin, o.margin);
        for (k, v) in o.gaps {
            let slot = gaps.entry(k).or_insert(f64::INFINITY);
            *slot = min2(*slot, v);
        }
    }
    let evidence = json!({
        "result": if errors.is_empty() { "PASS" } else { "FAIL" },
        "actions": jobs.iter().map(|(_, a)| json!({"name": a.name, "frames": 145})).collect::<Vec<_>>(),
        "minimumPaintedMargin": js_number(margin),
        "gaps": gaps.iter().map(|(k, &v)| (k.clone(), js_number(v))).collect::<serde_json::Map<_, _>>(),
        "checks": checks,
        "errors": errors.clone(),
        "exceptions": EXCEPTIONS,
    });
    Ok(ClearanceReport { evidence, errors })
}
