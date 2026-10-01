//! Raster primitives of `verify.html`, on straight (non-premultiplied) RGBA bytes.
//!
//! Every function reproduces the page's JavaScript exactly, including the
//! floating-point operation order of [`raster_difference`].

/// FNV-1a (32-bit) over the RGBA bytes: the page's `hash(buffer)`.
pub fn fnv1a(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .fold(2_166_136_261u32, |n, &b| (n ^ u32::from(b)).wrapping_mul(16_777_619))
}

/// Visible pixel count and bounding box: the page's `pixels(buffer, width)`.
///
/// A pixel is visible when its alpha is above 8. An empty raster reports
/// `left = top = size` and `right = bottom = -1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    /// Number of visible pixels.
    pub n: u32,
    /// Leftmost visible column.
    pub left: i64,
    /// Topmost visible row.
    pub top: i64,
    /// Rightmost visible column.
    pub right: i64,
    /// Bottommost visible row.
    pub bottom: i64,
}

impl Bounds {
    /// The smallest distance from the visible box to the raster edge. Empty
    /// rasters count `size` on every side, as the page's arithmetic does.
    pub fn edge_clearance(&self, size: u32) -> i64 {
        let last = i64::from(size) - 1;
        self.left.min(self.top).min(last - self.right).min(last - self.bottom)
    }
}

/// Visible pixel count and bounding box of a `width`-wide square raster.
pub fn bounds(rgba: &[u8], width: u32) -> Bounds {
    let w = i64::from(width);
    let mut out = Bounds { n: 0, left: w, top: w, right: -1, bottom: -1 };
    for (p, px) in rgba.chunks_exact(4).enumerate() {
        if px[3] > 8 {
            let (x, y) = (p as i64 % w, p as i64 / w);
            out.n += 1;
            out.left = out.left.min(x);
            out.top = out.top.min(y);
            out.right = out.right.max(x);
            out.bottom = out.bottom.max(y);
        }
    }
    out
}

/// Enclosed transparent regions (alpha < 32, 4-connected) that do not touch the
/// raster edge and cover at least `minimum_area` pixels: the page's `counters()`.
pub fn counters(rgba: &[u8], size: u32, minimum_area: usize) -> u32 {
    let size = size as usize;
    let count = size * size;
    let transparent = |p: usize| rgba[p * 4 + 3] < 32;
    let mut seen = vec![false; count];
    let mut queue = vec![0usize; count];
    let mut total = 0;
    for first in 0..count {
        if seen[first] || !transparent(first) {
            continue;
        }
        let (mut head, mut tail, mut edge) = (0, 1, false);
        queue[0] = first;
        seen[first] = true;
        while head < tail {
            let p = queue[head];
            head += 1;
            let (x, y) = (p % size, p / size);
            if x == 0 || y == 0 || x == size - 1 || y == size - 1 {
                edge = true;
            }
            // Same neighbour order as the page: left, right, up, down.
            let neighbours = [
                (x > 0).then(|| p - 1),
                (x < size - 1).then(|| p + 1),
                (y > 0).then(|| p - size),
                (y < size - 1).then(|| p + size),
            ];
            for next in neighbours.into_iter().flatten() {
                if !seen[next] && transparent(next) {
                    seen[next] = true;
                    queue[tail] = next;
                    tail += 1;
                }
            }
        }
        if !edge && tail >= minimum_area {
            total += 1;
        }
    }
    total
}

/// Premultiplied mean raster error between two equally sized rasters: the
/// page's `rasterDifference(a, b)`.
///
/// The sum order and the per-term division by 255 are load-bearing: the
/// evidence records this value bit for bit.
pub fn raster_difference(a: &[u8], b: &[u8]) -> f64 {
    let mut sum = 0.0f64;
    for (pa, pb) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
        let (aa, ba) = (f64::from(pa[3]), f64::from(pb[3]));
        sum += (aa - ba).abs();
        for c in 0..3 {
            sum += (f64::from(pa[c]) * aa - f64::from(pb[c]) * ba).abs() / 255.0;
        }
    }
    sum / (255.0 * a.len() as f64)
}

/// The page's `equivalent(a, b, cyclic)`: identical hashes, or, for icons that
/// declare a rotation cycle, a raster difference below 0.002.
pub fn equivalent(a: &[u8], b: &[u8], cyclic: bool) -> bool {
    fnv1a(a) == fnv1a(b) || (cyclic && raster_difference(a, b) < 0.002)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raster(size: usize, alpha: impl Fn(usize, usize) -> u8) -> Vec<u8> {
        let mut out = vec![0u8; size * size * 4];
        for y in 0..size {
            for x in 0..size {
                out[(y * size + x) * 4 + 3] = alpha(x, y);
            }
        }
        out
    }

    #[test]
    fn fnv_matches_reference() {
        assert_eq!(fnv1a(&[]), 2_166_136_261);
        assert_eq!(fnv1a(b"a"), 0xe40c_292c);
        assert_eq!(fnv1a(b"foobar"), 0xbf9c_f968);
    }

    #[test]
    fn bounds_use_the_visibility_threshold() {
        let empty = raster(4, |_, _| 8);
        assert_eq!(bounds(&empty, 4), Bounds { n: 0, left: 4, top: 4, right: -1, bottom: -1 });
        assert_eq!(bounds(&empty, 4).edge_clearance(4), 4);
        let dot = raster(5, |x, y| if (x, y) == (1, 2) || (x, y) == (3, 3) { 9 } else { 0 });
        let b = bounds(&dot, 5);
        assert_eq!(b, Bounds { n: 2, left: 1, top: 2, right: 3, bottom: 3 });
        assert_eq!(b.edge_clearance(5), 1);
    }

    #[test]
    fn counters_find_enclosed_holes_only() {
        // A 7x7 ring with a 3x3 hole; the outside touches the edge.
        let ring = raster(7, |x, y| if (1..=5).contains(&x) && (1..=5).contains(&y) && !((2..=4).contains(&x) && (2..=4).contains(&y)) { 255 } else { 0 });
        assert_eq!(counters(&ring, 7, 4), 1);
        assert_eq!(counters(&ring, 7, 10), 0);
        // A one-pixel hole counts only with minimum area 1.
        let pin = raster(5, |x, y| if (1..=3).contains(&x) && (1..=3).contains(&y) && (x, y) != (2, 2) { 255 } else { 0 });
        assert_eq!(counters(&pin, 5, 1), 1);
        assert_eq!(counters(&pin, 5, 4), 0);
        // Alpha 31 is transparent, 32 is not.
        let soft = raster(5, |x, y| if (x, y) == (2, 2) { 31 } else if (1..=3).contains(&x) && (1..=3).contains(&y) { 32 } else { 0 });
        assert_eq!(counters(&soft, 5, 1), 1);
    }

    #[test]
    fn raster_difference_divides_each_term() {
        let a = [10u8, 20, 30, 200, 0, 0, 0, 0];
        let b = [10u8, 25, 30, 100, 0, 0, 0, 0];
        let mut sum = 100.0f64;
        sum += (10.0f64 * 200.0 - 10.0 * 100.0).abs() / 255.0;
        sum += (20.0f64 * 200.0 - 25.0 * 100.0).abs() / 255.0;
        sum += (30.0f64 * 200.0 - 30.0 * 100.0).abs() / 255.0;
        assert_eq!(raster_difference(&a, &b), sum / (255.0 * 8.0));
        assert_eq!(raster_difference(&a, &a), 0.0);
    }

    #[test]
    fn equivalence_tolerance_is_for_cyclic_icons_only() {
        let a = raster(10, |_, _| 255);
        let b = raster(10, |x, y| if (x, y) == (0, 0) { 200 } else { 255 });
        assert!(raster_difference(&a, &b) < 0.002);
        assert!(!equivalent(&a, &b, false));
        assert!(equivalent(&a, &b, true));
        assert!(equivalent(&a, &a, false));
    }
}
