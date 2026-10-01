//! `sampleVariant()` results, copied out of the host's typed arrays.

use crate::{describe::FRAMES, runtime::Fail};

/// Every scene node of one animation, sampled on all [`FRAMES`] frames.
#[derive(Debug, Clone, PartialEq)]
pub struct VariantSamples {
    /// One entry per [`Icon::parts`](crate::Icon::parts) entry, in order.
    pub parts: Vec<PartSamples>,
}

/// One scene node's samples.
#[derive(Debug, Clone, PartialEq)]
pub struct PartSamples {
    /// The first frame's command codes: 0 `M`, 1 `L`, 2 `C`, 3 `Z`.
    pub commands: Vec<u8>,
    /// Coordinates per frame (`x, y` pairs).
    pub point_count: usize,
    /// Posed points rounded to 1e-5, frame-major: frame `f` is `f * point_count ..`.
    pub points: Vec<f64>,
    /// Rounded visibility per frame (absent for legacy sets).
    pub visibility: Option<Vec<f64>>,
    /// The native transform of a rigid part (a `pose.transform` on a static path).
    pub rigid: Option<Rigid>,
}

impl PartSamples {
    /// The posed, rounded points of one frame.
    pub fn frame(&self, frame: usize) -> &[f64] {
        &self.points[frame * self.point_count..(frame + 1) * self.point_count]
    }
}

/// A rigid part's native transform samples (`build-set.mjs:112-123`).
#[derive(Debug, Clone, PartialEq)]
pub struct Rigid {
    /// The authored pivot.
    pub anchor: [f64; 2],
    /// The unposed first-frame points, rounded.
    pub base_points: Vec<f64>,
    /// `anchor + (dx, dy)` per frame, unrounded.
    pub position: Vec<[f64; 2]>,
    /// `-angle * PI / 180` per frame, unrounded.
    pub rotation: Vec<f64>,
}

/// Unpacks the layout documented at `sampleVariant` in `js/host.mjs`.
pub(crate) fn unpack(
    header: &[i32],
    commands: &[u8],
    data: &[f64],
    with_visibility: bool,
) -> Result<VariantSamples, Fail> {
    let bad = || Fail::Protocol("sampleVariant(): malformed result".into());
    let count = usize::try_from(*header.first().ok_or_else(bad)?).map_err(|_| bad())?;
    if header.len() != 1 + 5 * count {
        return Err(bad());
    }
    let mut parts = Vec::with_capacity(count);
    for fields in header[1..].chunks_exact(5) {
        let [command_count, n, flags, command_offset, data_offset] =
            [fields[0], fields[1], fields[2], fields[3], fields[4]].map(|v| usize::try_from(v).map_err(|_| bad()));
        let (n, data_offset, command_offset) = (n?, data_offset?, command_offset?);
        let commands = commands.get(command_offset..command_offset + command_count?).ok_or_else(bad)?.to_vec();
        let mut cursor = data_offset;
        let mut take = |len: usize| {
            let slice = data.get(cursor..cursor + len).ok_or_else(bad);
            cursor += len;
            slice
        };
        let points = take(FRAMES * n)?.to_vec();
        let visibility = if with_visibility { Some(take(FRAMES)?.to_vec()) } else { None };
        let rigid = if flags? & 1 == 1 {
            let anchor = take(2)?;
            let anchor = [anchor[0], anchor[1]];
            let base_points = take(n)?.to_vec();
            let position = take(2 * FRAMES)?.chunks_exact(2).map(|p| [p[0], p[1]]).collect();
            let rotation = take(FRAMES)?.to_vec();
            Some(Rigid { anchor, base_points, position, rotation })
        } else {
            None
        };
        parts.push(PartSamples { commands, point_count: n, points, visibility, rigid });
    }
    Ok(VariantSamples { parts })
}
