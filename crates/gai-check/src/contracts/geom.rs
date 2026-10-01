//! Geometry primitives shared by every clearance dialect, evaluated exactly as the JS
//! wrote them: left to right, no FMA, `Math.hypot`/`Math.min` semantics.

use super::ContractError;
use super::jsnum::{hypot2, max2, min2, slice_range};
use super::scene::{Scene, SceneNode, Track, cmd};
use serde::Deserialize;
use std::borrow::Cow;

/// A 2D point `[x, y]`.
pub type Point = [f64; 2];

/// How a keyframed track is read between keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sampler {
    /// Early-returns the first and last keys, then interpolates from the first key with
    /// `t > time` (13–39 clearance). An exact key hit returns `a + (b - a) * 0`.
    Gt,
    /// Interpolates from the first key with `t >= time`, with no early return for the
    /// last key (12, 45). An exact key hit returns `a + (b - a) * 1`.
    Ge,
}

/// Where a sample time falls on a track.
enum Hit {
    /// Return this key's value as is.
    Key(usize),
    /// Interpolate from key `a` to key `b` by `u`.
    Between(usize, usize, f64),
}

impl Sampler {
    fn locate<T>(self, keys: &Track<T>, t: f64) -> Hit {
        let last = keys.len() - 1;
        if t <= keys[0].0 {
            return Hit::Key(0);
        }
        let next = match self {
            Sampler::Gt if t >= keys[last].0 => return Hit::Key(last),
            Sampler::Gt => keys.iter().position(|k| k.0 > t),
            Sampler::Ge => keys.iter().position(|k| k.0 >= t),
        };
        match next {
            // `next >= 1`, since the first key is before `t`.
            Some(b) => Hit::Between(b - 1, b, (t - keys[b - 1].0) / (keys[b].0 - keys[b - 1].0)),
            None => Hit::Key(last),
        }
    }

    /// Samples a scalar track at `t`.
    pub fn scalar(self, keys: &Track<f64>, t: f64) -> f64 {
        match self.locate(keys, t) {
            Hit::Key(k) => keys[k].1,
            Hit::Between(a, b, u) => {
                let (x, y) = (keys[a].1, keys[b].1);
                x + (y - x) * u
            }
        }
    }

    /// Samples a coordinate-array track at `t`; the length comes from the earlier key.
    pub fn coords(self, keys: &Track<Vec<f64>>, t: f64) -> Cow<'_, [f64]> {
        match self.locate(keys, t) {
            Hit::Key(k) => Cow::Borrowed(&keys[k].1),
            Hit::Between(a, b, u) => {
                let (xs, ys) = (&keys[a].1, &keys[b].1);
                let lerp =
                    |(j, &v): (usize, &f64)| v + (ys.get(j).copied().unwrap_or(f64::NAN) - v) * u;
                Cow::Owned(xs.iter().enumerate().map(lerp).collect())
            }
        }
    }
}

/// What `Z` contributes to a flattened polyline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Close {
    /// Push the subpath's first point (every set except 45).
    First,
    /// Nothing (45).
    Ignore,
}

/// What an unknown command code does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unknown {
    /// Fail with `Unknown path command`.
    Throw,
    /// Skip it.
    Ignore,
}

/// The flattening dialect (`flatten` in the rules format).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FlattenSpec {
    /// Uniform parameter steps per cubic (`u = j/steps`, `j = 1..=steps`).
    pub cubic_steps: u32,
    /// Skip cubics whose four points are equal. Measured to change no evidence.
    pub skip_degenerate: bool,
    /// `Z` handling.
    pub close: Close,
    /// Unknown command handling.
    pub unknown: Unknown,
}

/// Flattens path views into polylines with a fixed dialect.
#[derive(Debug, Clone)]
pub struct Flattener {
    spec: FlattenSpec,
    /// Bernstein weights per step, `((v*v)*v, ((3*v)*v)*u, ((3*v)*u)*u, (u*u)*u)`.
    weights: Vec<[f64; 4]>,
}

impl Flattener {
    /// Precomputes the cubic weights for a dialect.
    pub fn new(spec: FlattenSpec) -> Flattener {
        let n = f64::from(spec.cubic_steps);
        let weights = (1..=spec.cubic_steps)
            .map(|j| {
                let u = f64::from(j) / n;
                let v = 1.0 - u;
                [v * v * v, 3.0 * v * v * u, 3.0 * v * u * u, u * u * u]
            })
            .collect();
        Flattener { spec, weights }
    }

    /// The `curvePoints` polyline of a view at `t`. Subpaths are concatenated, so the
    /// bridge from one subpath's end to the next `M` is a real segment.
    pub fn points(
        &self,
        view: &PathView<'_>,
        sampler: Sampler,
        t: f64,
    ) -> Result<Vec<Point>, ContractError> {
        let p = view.coords(sampler, t);
        let short = || {
            ContractError::Geometry(format!(
                "{} has fewer coordinates than its commands use",
                view.node.name
            ))
        };
        let at = |k: usize| -> Result<Point, ContractError> {
            match (p.get(k), p.get(k + 1)) {
                (Some(&x), Some(&y)) => Ok([x, y]),
                _ => Err(short()),
            }
        };
        let mut out = Vec::with_capacity(view.commands.len() * 4);
        let (mut k, mut last, mut first): (usize, Option<Point>, Option<Point>) = (0, None, None);
        for &c in view.commands.iter() {
            match c {
                cmd::MOVE | cmd::LINE => {
                    let q = at(k)?;
                    k += 2;
                    out.push(q);
                    last = Some(q);
                    if c == cmd::MOVE {
                        first = Some(q);
                    }
                }
                cmd::CUBIC => {
                    let a = last.ok_or_else(|| {
                        ContractError::Geometry(format!("{} starts with a cubic", view.node.name))
                    })?;
                    let (b, cc, d) = (at(k)?, at(k + 2)?, at(k + 4)?);
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
                        ContractError::Geometry(format!("{} closes before a move", view.node.name))
                    })?;
                    out.push(f);
                    last = Some(f);
                }
                cmd::CLOSE => {}
                _ if self.spec.unknown == Unknown::Throw => {
                    return Err(ContractError::UnknownPathCommand);
                }
                _ => {}
            }
        }
        Ok(out)
    }

    /// Painted bounds `[left, top, right, bottom]`: the polyline dilated by half the stroke
    /// width (round caps and joins). An empty polyline gives `[+∞, +∞, -∞, -∞]`.
    pub fn painted_box(
        &self,
        view: &PathView<'_>,
        sampler: Sampler,
        t: f64,
    ) -> Result<[f64; 4], ContractError> {
        let p = self.points(view, sampler, t)?;
        Ok(pad_box(
            bounds(p.iter().copied()),
            radius(view.node, sampler, t),
        ))
    }
}

/// `[minX, minY, maxX, maxY]` with `Math.min`/`Math.max` semantics.
pub fn bounds(points: impl Iterator<Item = Point>) -> [f64; 4] {
    points.fold(
        [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ],
        |b, p| {
            [
                min2(b[0], p[0]),
                min2(b[1], p[1]),
                max2(b[2], p[0]),
                max2(b[3], p[1]),
            ]
        },
    )
}

/// Grows bounds by `r` on every side.
pub fn pad_box(b: [f64; 4], r: f64) -> [f64; 4] {
    [b[0] - r, b[1] - r, b[2] + r, b[3] + r]
}

/// Paint radius: half the sampled width for strokes, 0 for fills.
pub fn radius(node: &SceneNode, sampler: Sampler, t: f64) -> f64 {
    if node.is_stroke() {
        sampler.scalar(&node.width, t) / 2.0
    } else {
        0.0
    }
}

/// `JSMIN(b0, b1, 32 - b2, 32 - b3)`: the painted margin to the 32-unit canvas.
pub fn canvas_margin(b: [f64; 4]) -> f64 {
    min2(min2(min2(b[0], b[1]), 32.0 - b[2]), 32.0 - b[3])
}

/// Distance from `p` to segment `ab` (`lineDistance`). A zero or NaN length uses 1 (`|| 1`).
#[inline]
pub fn seg_dist(p: Point, a: Point, b: Point) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let mut den = dx * dx + dy * dy;
    if den == 0.0 || den.is_nan() {
        den = 1.0;
    }
    let u = max2(
        0.0,
        min2(1.0, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / den),
    );
    hypot2(p[0] - a[0] - u * dx, p[1] - a[1] - u * dy)
}

/// Minimum distance from `p` to the polyline; `+∞` below two points.
pub fn poly_dist(p: Point, line: &[Point]) -> f64 {
    line.windows(2)
        .fold(f64::INFINITY, |m, s| min2(m, seg_dist(p, s[0], s[1])))
}

/// Symmetric vertex-to-polyline distance: every vertex of each polyline to the other.
/// Two crossing segments whose vertices are far apart still report a positive distance.
pub fn polyline_gap(pa: &[Point], pb: &[Point]) -> f64 {
    let m = pa
        .iter()
        .fold(f64::INFINITY, |m, &p| min2(m, poly_dist(p, pb)));
    pb.iter().fold(m, |m, &p| min2(m, poly_dist(p, pa)))
}

/// Even-odd point-in-polygon over the polyline (bridges included).
pub fn inside_poly(p: Point, poly: &[Point]) -> bool {
    let mut c = false;
    let n = poly.len();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + n - 1) % n]);
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            c = !c;
        }
    }
    c
}

/// `PICK_BOLD`: the first node whose base is `name-bold`, otherwise the first whose base is `name`.
pub fn pick_bold<'a>(scene: &'a Scene, name: &str) -> Result<&'a SceneNode, ContractError> {
    let bold = format!("{name}-bold");
    scene
        .nodes
        .iter()
        .find(|n| n.base() == bold)
        .or_else(|| scene.nodes.iter().find(|n| n.base() == name))
        .ok_or_else(|| ContractError::MissingPart(name.to_string()))
}

/// `PICK_BASE`: the first node whose base is exactly `name`.
pub fn pick_base<'a>(scene: &'a Scene, name: &str) -> Result<&'a SceneNode, ContractError> {
    scene
        .nodes
        .iter()
        .find(|n| n.base() == name)
        .ok_or_else(|| ContractError::MissingPart(format!("Missing {name}")))
}

/// A path derived from a node: a command subrange, a single segment or a command prefix,
/// over the node's sampled coordinates with JS `slice` windows applied in order.
#[derive(Debug, Clone)]
pub struct PathView<'a> {
    /// The scene node the coordinates come from (its width and visibility apply unchanged).
    pub node: &'a SceneNode,
    /// The view's commands.
    pub commands: Cow<'a, [u8]>,
    windows: Vec<(i64, Option<i64>)>,
}

impl<'a> PathView<'a> {
    /// The whole node.
    pub fn whole(node: &'a SceneNode) -> PathView<'a> {
        PathView {
            node,
            commands: Cow::Borrowed(&node.commands),
            windows: Vec::new(),
        }
    }

    /// Whether the view is the node's full path (so its flattening can be shared).
    pub fn is_whole(&self) -> bool {
        self.windows.is_empty() && *self.commands == *self.node.commands
    }

    /// Sampled coordinates at `t` with the view's windows applied.
    pub fn coords(&self, sampler: Sampler, t: f64) -> Cow<'a, [f64]> {
        let mut p = sampler.coords(&self.node.points, t);
        for &(start, end) in &self.windows {
            let r = slice_range(p.len(), start, end);
            p = match p {
                Cow::Borrowed(s) => Cow::Borrowed(&s[r]),
                Cow::Owned(v) => Cow::Owned(v[r].to_vec()),
            };
        }
        p
    }

    fn offset(commands: &[u8], end: usize) -> usize {
        commands[..end].iter().map(|&c| cmd::coords(c)).sum()
    }

    /// `sceneSubpath`: commands `[start, end)` with the first forced to `M`.
    /// `range` is the raw rule value; `null` keeps the view unchanged.
    pub fn subpath(self, range: &serde_json::Value) -> Result<PathView<'a>, ContractError> {
        if range.is_null() {
            return Ok(self);
        }
        let bounds: Option<Vec<f64>> = range
            .as_array()
            .and_then(|a| a.iter().map(serde_json::Value::as_f64).collect());
        let [start, end] = match bounds.as_deref() {
            Some(&[s, e]) if super::jsnum::is_integer(s) && super::jsnum::is_integer(e) => [s, e],
            _ => return Err(ContractError::InvalidSubpath),
        };
        if start < 0.0 || end > self.commands.len() as f64 || start >= end {
            return Err(ContractError::InvalidSubpath);
        }
        let (start, end) = (start as usize, end as usize);
        if !matches!(self.commands[start], cmd::MOVE | cmd::LINE) {
            return Err(ContractError::SubpathStart);
        }
        let offset = Self::offset(&self.commands, start);
        let length: usize = self.commands[start..end]
            .iter()
            .map(|&c| cmd::coords(c))
            .sum();
        let mut commands = self.commands[start..end].to_vec();
        commands[0] = cmd::MOVE;
        let mut windows = self.windows;
        windows.push((offset as i64, Some((offset + length) as i64)));
        Ok(PathView {
            node: self.node,
            commands: Cow::Owned(commands),
            windows,
        })
    }

    /// `segmentNode`: command `j` alone, starting from the previous command's end point.
    pub fn segment(self, j: i64) -> Result<PathView<'a>, ContractError> {
        let j = usize::try_from(j)
            .ok()
            .filter(|&j| j < self.commands.len())
            .ok_or(ContractError::UnknownPathCommand)?;
        let c = self.commands[j];
        let offset = Self::offset(&self.commands, j) as i64;
        let mut windows = self.windows;
        windows.push((offset - 2, Some(offset + cmd::coords(c) as i64)));
        Ok(PathView {
            node: self.node,
            commands: Cow::Owned(vec![cmd::MOVE, c]),
            windows,
        })
    }

    /// `commands.slice(0, n)`: the flattener stops early; coordinates are untouched.
    pub fn truncate(self, n: i64) -> PathView<'a> {
        let r = slice_range(self.commands.len(), 0, Some(n));
        let commands = match self.commands {
            Cow::Borrowed(c) => Cow::Borrowed(&c[r]),
            Cow::Owned(c) => Cow::Owned(c[r].to_vec()),
        };
        PathView {
            node: self.node,
            commands,
            windows: self.windows,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn node(commands: Vec<u8>, points: Track<Vec<f64>>) -> SceneNode {
        SceneNode {
            name: "n-a__color.primary__outline.bold".into(),
            commands,
            points,
            base_points: None,
            transform: false,
            kind: Some("stroke".into()),
            width: vec![(0.0, 2.5), (2.4, 2.5)],
            visibility: Some(vec![(0.0, 1.0), (2.4, 1.0)]),
        }
    }

    fn subpath_fixture() -> SceneNode {
        let a = vec![
            0., 0., 30., 0., 0., 0., 10., 10., 10., 10., 11., 10., 12., 11., 12., 12., 12., 13.,
            11., 14., 10., 10.,
        ];
        let b = vec![
            0., 0., 30., 0., 0., 0., 10., 11., 10., 11., 11., 11., 12., 12., 12., 13., 12., 14.,
            11., 15., 10., 11.,
        ];
        node(vec![0, 1, 1, 1, 1, 2, 2, 3], vec![(0.0, a), (1.0, b)])
    }

    #[test]
    fn subpath_matches_geometry_subpath_test() {
        let n = subpath_fixture();
        let view = PathView::whole(&n).subpath(&json!([4, 7])).unwrap();
        assert_eq!(&*view.commands, &[0, 2, 2]);
        let expect0 = [
            10., 10., 11., 10., 12., 11., 12., 12., 12., 13., 11., 14., 10., 10.,
        ];
        let expect1 = [
            10., 11., 11., 11., 12., 12., 12., 13., 12., 14., 11., 15., 10., 11.,
        ];
        assert_eq!(&*view.coords(Sampler::Gt, 0.0), &expect0);
        assert_eq!(&*view.coords(Sampler::Gt, 1.0), &expect1);
        let same = PathView::whole(&n)
            .subpath(&serde_json::Value::Null)
            .unwrap();
        assert_eq!(&*same.commands, &n.commands[..]);
        assert!(matches!(
            PathView::whole(&n).subpath(&json!([5, 7])),
            Err(ContractError::SubpathStart)
        ));
        for bad in [
            json!([2]),
            json!([1.5, 3]),
            json!([-1, 2]),
            json!([3, 3]),
            json!([0, 99]),
            json!("0,2"),
        ] {
            assert!(
                matches!(
                    PathView::whole(&n).subpath(&bad),
                    Err(ContractError::InvalidSubpath)
                ),
                "{bad}"
            );
        }
        assert!(PathView::whole(&n).subpath(&json!([4.0, 7.0])).is_ok());
    }

    #[test]
    fn segment_window_starts_at_previous_end() {
        let n = subpath_fixture();
        let seg = PathView::whole(&n).segment(5).unwrap();
        assert_eq!(&*seg.commands, &[0, 2]);
        assert_eq!(
            &*seg.coords(Sampler::Gt, 0.0),
            &[10., 10., 11., 10., 12., 11., 12., 12.]
        );
        // A segment of a subpath indexes the subpath's re-based commands and coordinates.
        let sub = PathView::whole(&n)
            .subpath(&json!([4, 7]))
            .unwrap()
            .segment(2)
            .unwrap();
        assert_eq!(
            &*sub.coords(Sampler::Gt, 0.0),
            &[12., 12., 12., 13., 11., 14., 10., 10.]
        );
        assert!(matches!(
            PathView::whole(&n).segment(8),
            Err(ContractError::UnknownPathCommand)
        ));
        assert_eq!(
            &*PathView::whole(&n).truncate(-1).commands,
            &[0, 1, 1, 1, 1, 2, 2]
        );
        assert!(PathView::whole(&n).truncate(0).commands.is_empty());
    }

    #[test]
    fn samplers_differ_on_exact_key_hits() {
        let keys: Track<f64> = vec![(0.0, 16.0), (1.0, 0.1), (2.0, 5.0)];
        assert_eq!(Sampler::Gt.scalar(&keys, 1.0), 0.1);
        assert_eq!(Sampler::Ge.scalar(&keys, 1.0), 0.09999999999999964);
        assert_eq!(Sampler::Gt.scalar(&keys, 0.5), 8.05);
        assert_eq!(Sampler::Ge.scalar(&keys, 2.0), 5.0);
        assert_eq!(Sampler::Gt.scalar(&keys, 9.0), 5.0);
        assert_eq!(Sampler::Ge.scalar(&keys, -1.0), 16.0);
    }

    #[test]
    fn flattening_closes_and_subdivides() {
        let n = node(
            vec![0, 2, 3],
            vec![(0.0, vec![0., 0., 0., 4., 4., 4., 4., 0.])],
        );
        let spec = FlattenSpec {
            cubic_steps: 4,
            skip_degenerate: true,
            close: Close::First,
            unknown: Unknown::Throw,
        };
        let p = Flattener::new(spec)
            .points(&PathView::whole(&n), Sampler::Gt, 0.0)
            .unwrap();
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
        let open = Flattener::new(FlattenSpec {
            close: Close::Ignore,
            ..spec
        });
        assert_eq!(
            open.points(&PathView::whole(&n), Sampler::Gt, 0.0)
                .unwrap()
                .len(),
            5
        );
        let bad = node(vec![0, 7], vec![(0.0, vec![0., 0.])]);
        assert!(matches!(
            Flattener::new(spec).points(&PathView::whole(&bad), Sampler::Gt, 0.0),
            Err(ContractError::UnknownPathCommand)
        ));
        let lenient = Flattener::new(FlattenSpec {
            unknown: Unknown::Ignore,
            ..spec
        });
        assert_eq!(
            lenient
                .points(&PathView::whole(&bad), Sampler::Gt, 0.0)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn distances_follow_the_js_helpers() {
        assert_eq!(seg_dist([3., 4.], [0., 0.], [0., 0.]), 5.0);
        assert_eq!(seg_dist([1., 1.], [0., 0.], [2., 0.]), 1.0);
        assert_eq!(poly_dist([0., 0.], &[[1., 1.]]), f64::INFINITY);
        // Crossing segments with far-apart vertices still report a positive gap.
        assert!(polyline_gap(&[[0., -5.], [0., 5.]], &[[-5., 0.], [5., 0.]]) > 4.0);
        let square = [[0., 0.], [4., 0.], [4., 4.], [0., 4.]];
        assert!(inside_poly([2., 2.], &square) && !inside_poly([5., 2.], &square));
        assert_eq!(canvas_margin([3.0, 2.0, 29.0, 31.5]), 0.5);
    }
}
