//! Frozen ports of the hand-coded clearance scripts that no rule kind expresses:
//! 12 (dynamic node sets, jump detection), 13–15 (3-unit margin, named aggregates,
//! 15's `atan2` summary), 21 (`spans` flattening) and 45 (S_GE, open curves).
//!
//! Each is a literal transcription of `motion/icon-sets/<set>/verify-clearance.mjs`,
//! including its assert messages, aggregate initial values and evaluation order.

mod set12;
mod set13;
mod set14;
mod set15;
mod set21;
mod set45;

use super::ContractError;
use super::evidence::ClearanceReport;
use super::geom::{
    Close, FlattenSpec, Flattener, PathView, Point, Sampler, Unknown, canvas_margin, pick_bold,
};
use super::scene::{Scene, SceneNode, SceneSource, SetManifest};

/// The frozen legacy clearance checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacySet {
    /// 12-loading-indicators.
    LoadingIndicators,
    /// 13-fintech-finance.
    FintechFinance,
    /// 14-travel.
    Travel,
    /// 15-commerce-delivery.
    CommerceDelivery,
    /// 21-annotation-arrows.
    AnnotationArrows,
    /// 45-chat-messaging.
    ChatMessaging,
}

impl LegacySet {
    /// The frozen check for a set id, if it has one.
    pub fn for_set(set_id: &str) -> Option<LegacySet> {
        Some(match set_id {
            "12-loading-indicators" => LegacySet::LoadingIndicators,
            "13-fintech-finance" => LegacySet::FintechFinance,
            "14-travel" => LegacySet::Travel,
            "15-commerce-delivery" => LegacySet::CommerceDelivery,
            "21-annotation-arrows" => LegacySet::AnnotationArrows,
            "45-chat-messaging" => LegacySet::ChatMessaging,
            _ => return None,
        })
    }

    /// Runs the check. Assert-mode sets (all but 21) return an error on the first failure,
    /// so a report always passes; 21 collects failures into its report.
    pub fn run(
        self,
        manifest: &SetManifest,
        scenes: &dyn SceneSource,
    ) -> Result<ClearanceReport, ContractError> {
        let evidence = match self {
            LegacySet::LoadingIndicators => set12::run(scenes)?,
            LegacySet::FintechFinance => set13::run(manifest, scenes)?,
            LegacySet::Travel => set14::run(manifest, scenes)?,
            LegacySet::CommerceDelivery => set15::run(manifest, scenes)?,
            LegacySet::AnnotationArrows => return set21::run(manifest, scenes),
            LegacySet::ChatMessaging => set45::run(manifest, scenes)?,
        };
        Ok(ClearanceReport {
            evidence,
            errors: Vec::new(),
        })
    }
}

/// `curvePoints` of 13–21: 20 steps per cubic, no degenerate skip, `Z` closes, unknown throws.
fn flattener20() -> Flattener {
    Flattener::new(FlattenSpec {
        cubic_steps: 20,
        skip_degenerate: false,
        close: Close::First,
        unknown: Unknown::Throw,
    })
}

/// `assert.ok(ok, message)`.
fn ensure(ok: bool, message: impl FnOnce() -> String) -> Result<(), ContractError> {
    if ok {
        Ok(())
    } else {
        Err(ContractError::Assertion(message()))
    }
}

/// One frame of a 13–21 script: S_GT sampling, 20-step flattening, `PICK_BOLD`.
#[derive(Clone, Copy)]
struct Frame<'s> {
    scene: &'s Scene,
    flattener: &'s Flattener,
    t: f64,
}

impl<'s> Frame<'s> {
    fn node(&self, name: &str) -> Result<&'s SceneNode, ContractError> {
        pick_bold(self.scene, name)
    }

    fn points_of(&self, node: &SceneNode) -> Result<Vec<Point>, ContractError> {
        self.flattener
            .points(&PathView::whole(node), Sampler::Gt, self.t)
    }

    /// `curvePoints(pick(name))`.
    fn points(&self, name: &str) -> Result<Vec<Point>, ContractError> {
        self.points_of(self.node(name)?)
    }

    /// `box(pick(name))`.
    fn painted_box(&self, name: &str) -> Result<[f64; 4], ContractError> {
        self.flattener
            .painted_box(&PathView::whole(self.node(name)?), Sampler::Gt, self.t)
    }

    /// `ends(pick(name))`: the first and last flattened points.
    fn ends(&self, name: &str) -> Result<[Point; 2], ContractError> {
        let p = self.points(name)?;
        match (p.first(), p.last()) {
            (Some(&a), Some(&b)) => Ok([a, b]),
            _ => Err(ContractError::Geometry(format!("{name} has an empty path"))),
        }
    }

    /// `sample(node.points)`.
    fn raw(&self, node: &SceneNode) -> Vec<f64> {
        Sampler::Gt.coords(&node.points, self.t).into_owned()
    }

    /// `sample(node.visibility)`.
    fn vis(&self, node: &SceneNode) -> Result<f64, ContractError> {
        Ok(Sampler::Gt.scalar(node.visibility()?, self.t))
    }

    /// The canvas-margin pass: calls `check(node, margin)` for each visible Bold or Solid node.
    fn margins(
        &self,
        mut check: impl FnMut(&'s SceneNode, f64) -> Result<(), ContractError>,
    ) -> Result<(), ContractError> {
        for n in &self.scene.nodes {
            if !(n.name.ends_with("__outline.bold") || n.name.ends_with("__solid"))
                || self.vis(n)? < 0.01
            {
                continue;
            }
            let b = self
                .flattener
                .painted_box(&PathView::whole(n), Sampler::Gt, self.t)?;
            check(n, canvas_margin(b))?;
        }
        Ok(())
    }
}

/// `t = f/60` for `f = 0..=144`.
fn frame_times() -> impl Iterator<Item = (u32, f64)> {
    (0..super::engine::FRAMES).map(|f| (f, f64::from(f) / 60.0))
}
