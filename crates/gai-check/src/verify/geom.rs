//! Geometry for the verify stages, on top of the clearance primitives: a flattener with
//! the extra per-set knobs (`cubicPow`, lazy NaN for short paths), the node pickers of
//! verify.md §2.6, control-point boxes and the 49 polyline distance.

use super::VerifyError;
use crate::contracts::geom::{Close, Point, Unknown, poly_dist};
use crate::contracts::jsnum::{hypot2, max_of, min_of};
use crate::contracts::scene::{Scene, SceneNode, cmd};
use serde::Deserialize;

/// Curve flattening knobs (verify.md §2.5).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FlattenSpec {
    /// Uniform parameter steps per cubic (`u = j/steps`).
    pub cubic_steps: u32,
    /// Skip cubics whose four points are equal.
    #[serde(default)]
    pub skip_degenerate: bool,
    /// `Z` handling.
    pub close: Close,
    /// Unknown command handling.
    pub unknown: Unknown,
    /// Weight the end points with `v**3` and `u**3` (62) instead of `v*v*v`, `u*u*u`.
    #[serde(default)]
    pub cubic_pow: bool,
}

/// Flattens paths with a fixed dialect.
#[derive(Debug, Clone)]
pub struct Flattener {
    spec: FlattenSpec,
    weights: Vec<[f64; 4]>,
}

impl Flattener {
    /// Precomputes the Bernstein weights, left-associated as the JS wrote them.
    pub fn new(spec: FlattenSpec) -> Flattener {
        let n = f64::from(spec.cubic_steps);
        let weights = (1..=spec.cubic_steps)
            .map(|j| {
                let u = f64::from(j) / n;
                let v = 1.0 - u;
                let (vvv, uuu) = if spec.cubic_pow {
                    (v.powf(3.0), u.powf(3.0))
                } else {
                    (v * v * v, u * u * u)
                };
                [vvv, 3.0 * v * v * u, 3.0 * v * u * u, uuu]
            })
            .collect();
        Flattener { spec, weights }
    }

    /// The polyline of `commands` over the sampled coordinates `p`. Missing coordinates
    /// read as NaN, as `p.slice(k, k + 2)` would leave them undefined.
    pub fn points(
        &self,
        name: &str,
        commands: &[u8],
        p: &[f64],
    ) -> Result<Vec<Point>, VerifyError> {
        let at = |k: usize| {
            [
                p.get(k).copied().unwrap_or(f64::NAN),
                p.get(k + 1).copied().unwrap_or(f64::NAN),
            ]
        };
        let mut out = Vec::with_capacity(commands.len() * 4);
        let (mut k, mut last, mut first): (usize, Option<Point>, Option<Point>) = (0, None, None);
        for &c in commands {
            match c {
                cmd::MOVE | cmd::LINE => {
                    let q = at(k);
                    k += 2;
                    out.push(q);
                    last = Some(q);
                    if c == cmd::MOVE {
                        first = Some(q);
                    }
                }
                cmd::CUBIC => {
                    let a = last.ok_or_else(|| {
                        VerifyError::Geometry(format!("{name} draws a cubic before a move"))
                    })?;
                    let (b, cc, d) = (at(k), at(k + 2), at(k + 4));
                    k += 6;
                    let degenerate = (0..2).all(|i| a[i] == b[i] && a[i] == cc[i] && a[i] == d[i]);
                    if !(self.spec.skip_degenerate && degenerate) {
                        out.extend(self.weights.iter().map(|w| {
                            [0, 1].map(|i| w[0] * a[i] + w[1] * b[i] + w[2] * cc[i] + w[3] * d[i])
                        }));
                    }
                    last = Some(d);
                }
                cmd::CLOSE if self.spec.close == Close::First => {
                    let f = first.ok_or_else(|| {
                        VerifyError::Geometry(format!("{name} closes before a move"))
                    })?;
                    out.push(f);
                    last = Some(f);
                }
                cmd::CLOSE => {}
                other if self.spec.unknown == Unknown::Throw => {
                    return Err(VerifyError::Geometry(format!(
                        "Unknown path command {other} in {name}"
                    )));
                }
                _ => {}
            }
        }
        Ok(out)
    }
}

/// How a rule names a scene node (verify.md §2.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum Pick {
    /// `PICK_BOLD`: base `<name>-bold`, else base `<name>`.
    #[serde(rename = "bold")]
    Bold,
    /// `PICK_BASE`: base exactly `<name>`.
    #[serde(rename = "base")]
    Base,
    /// `PICK_PREFIX`: the first node named `<name>__…` or `<name>-regular__…`.
    #[serde(rename = "prefix")]
    Prefix,
    /// Base `<name>-bold`, else `PICK_PREFIX`.
    #[serde(rename = "boldOrPrefix")]
    BoldOrPrefix,
    /// The full node name `<name><suffix>`, trying each suffix in order.
    #[serde(rename = "exact")]
    Exact,
}

/// Finds a node; `suffixes` apply to [`Pick::Exact`] (none means the bare name).
pub fn pick<'s>(
    scene: &'s Scene,
    how: Pick,
    name: &str,
    suffixes: &[String],
) -> Option<&'s SceneNode> {
    pick_index(scene, how, name, suffixes).map(|i| &scene.nodes[i])
}

/// The index of the node [`pick`] finds.
pub fn pick_index(scene: &Scene, how: Pick, name: &str, suffixes: &[String]) -> Option<usize> {
    let base = |want: &str| scene.nodes.iter().position(|n| n.base() == want);
    let prefix = || {
        let (plain, regular) = (format!("{name}__"), format!("{name}-regular__"));
        scene
            .nodes
            .iter()
            .position(|n| n.name.starts_with(&plain) || n.name.starts_with(&regular))
    };
    match how {
        Pick::Bold => base(&format!("{name}-bold")).or_else(|| base(name)),
        Pick::Base => base(name),
        Pick::Prefix => prefix(),
        Pick::BoldOrPrefix => base(&format!("{name}-bold")).or_else(prefix),
        Pick::Exact if suffixes.is_empty() => scene.nodes.iter().position(|n| n.name == name),
        Pick::Exact => suffixes.iter().find_map(|s| {
            let full = format!("{name}{s}");
            scene.nodes.iter().position(|n| n.name == full)
        }),
    }
}

/// `[minX, minY, maxX, maxY]` of flat control coordinates (`Math.min(...xy(p, 0))`, …).
pub fn coords_box(p: &[f64]) -> [f64; 4] {
    let xs = p.iter().step_by(2).copied();
    let ys = p.iter().skip(1).step_by(2).copied();
    [
        min_of(xs.clone()),
        min_of(ys.clone()),
        max_of(xs),
        max_of(ys),
    ]
}

/// Distance from `p` to a polyline. The 49 dialect measures to the point itself when the
/// polyline has exactly one point; the others read `+∞` below two points.
pub fn poly_distance(p: Point, line: &[Point], single_point: bool) -> f64 {
    match line {
        [only] if single_point => hypot2(p[0] - only[0], p[1] - only[1]),
        _ => poly_dist(p, line),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(name: &str, commands: Vec<u8>, coords: Vec<f64>) -> SceneNode {
        SceneNode {
            name: name.into(),
            commands,
            points: vec![(0.0, coords)],
            base_points: None,
            transform: false,
            kind: Some("stroke".into()),
            width: vec![(0.0, 2.0)],
            visibility: Some(vec![(0.0, 1.0)]),
        }
    }

    #[test]
    fn pickers_follow_the_js_helpers() {
        let scene = Scene {
            name: "s".into(),
            nodes: vec![
                node(
                    "cup-rim-regular__color.primary__outline.regular",
                    vec![0],
                    vec![0., 0.],
                ),
                node(
                    "cup-rim-bold__color.primary__outline.bold",
                    vec![0],
                    vec![0., 0.],
                ),
                node("cup-rim-solid__color.primary__solid", vec![0], vec![0., 0.]),
                node("cup-body__color.primary__solid", vec![0], vec![0., 0.]),
            ],
        };
        let name = |n: Option<&SceneNode>| n.map(|n| n.name.clone()).unwrap_or_default();
        assert!(name(pick(&scene, Pick::Prefix, "cup-rim", &[])).contains("-regular__"));
        assert!(name(pick(&scene, Pick::Bold, "cup-rim", &[])).contains("-bold__"));
        assert!(name(pick(&scene, Pick::BoldOrPrefix, "cup-body", &[])).starts_with("cup-body__"));
        assert!(pick(&scene, Pick::Base, "cup-rim", &[]).is_none());
        let suffixes = [
            "-bold__color.primary__outline.bold".to_string(),
            "__color.primary__solid".to_string(),
        ];
        assert!(name(pick(&scene, Pick::Exact, "cup-body", &suffixes)).starts_with("cup-body__"));
        assert!(name(pick(&scene, Pick::Exact, "cup-rim", &suffixes)).contains("-bold__"));
    }

    fn flat(f: &Flattener, n: &SceneNode) -> Result<Vec<Point>, VerifyError> {
        f.points(&n.name, &n.commands, &n.points[0].1)
    }

    #[test]
    fn flattening_variants() {
        let spec = FlattenSpec {
            cubic_steps: 4,
            skip_degenerate: false,
            close: Close::First,
            unknown: Unknown::Ignore,
            cubic_pow: false,
        };
        let n = node("n", vec![0, 2, 3, 9], vec![0., 0., 0., 4., 4., 4., 4., 0.]);
        let p = flat(&Flattener::new(spec), &n).unwrap();
        assert_eq!(
            p,
            vec![
                [0., 0.],
                [0.625, 2.25],
                [2., 3.],
                [3.375, 2.25],
                [4., 0.],
                [0., 0.]
            ]
        );
        let pow = flat(
            &Flattener::new(FlattenSpec {
                cubic_pow: true,
                ..spec
            }),
            &n,
        )
        .unwrap();
        assert_eq!(pow, p);
        assert!(
            flat(
                &Flattener::new(FlattenSpec {
                    unknown: Unknown::Throw,
                    ..spec
                }),
                &n
            )
            .is_err()
        );
        let short = node("s", vec![0, 1], vec![1., 2., 3.]);
        let p = flat(&Flattener::new(spec), &short).unwrap();
        assert!(p[1][0] == 3.0 && p[1][1].is_nan());
    }

    #[test]
    fn boxes_and_distances() {
        assert_eq!(coords_box(&[1., 5., 3., -2.]), [1., -2., 3., 5.]);
        assert_eq!(poly_distance([3., 4.], &[[0., 0.]], true), 5.0);
        assert_eq!(poly_distance([3., 4.], &[[0., 0.]], false), f64::INFINITY);
    }
}
