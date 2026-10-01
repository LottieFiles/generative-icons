//! 45-chat-messaging: pen and fold contacts, smile, caret and waveform paint gaps, sampled
//! with S_GE and flattened as open curves (`Z` and unknown commands ignored). Assert-mode.

use super::ensure;
use crate::contracts::ContractError;
use crate::contracts::evidence::js_number;
use crate::contracts::geom::{
    Close, FlattenSpec, Flattener, PathView, Sampler, Unknown, pick_base, poly_dist,
};
use crate::contracts::jsnum::{hypot2, max2, min2};
use crate::contracts::scene::{SceneSource, SetManifest};
use gai_core::js_number_to_string as js;
use serde_json::Value;

const STYLES: [&str; 2] = ["-bold", "-solid"];

fn at(p: &[f64], i: i64) -> f64 {
    crate::contracts::jsnum::at_index(p.len(), i).map_or(f64::NAN, |k| p[k])
}

pub(super) fn run(
    manifest: &SetManifest,
    scenes: &dyn SceneSource,
) -> Result<Value, ContractError> {
    let flattener = Flattener::new(FlattenSpec {
        cubic_steps: 64,
        skip_degenerate: false,
        close: Close::Ignore,
        unknown: Unknown::Ignore,
    });
    let (mut checks, mut frames) = (0u64, 0u64);
    let (mut wave, mut pen, mut fold_error, mut smile, mut caret) =
        (f64::INFINITY, 0.0, 0.0, f64::INFINITY, f64::INFINITY);
    for icon in &manifest.icons {
        let id = icon.icon.as_deref().unwrap_or(&icon.name);
        for action in &icon.animations {
            let scene = scenes.load(&action.name)?;
            let name = action.name.as_str();
            let sample = |part: &str, t: f64| -> Result<Vec<f64>, ContractError> {
                Ok(Sampler::Ge
                    .coords(&pick_base(&scene, part)?.points, t)
                    .into_owned())
            };
            for f in 0..=144u32 {
                let t = f64::from(f) / 60.0;
                frames += 1;
                if id == "edit-message" {
                    for s in STYLES {
                        let (ink, p) = (
                            sample(&format!("edit-message-ink{s}"), t)?,
                            sample(&format!("edit-message-pen{s}"), t)?,
                        );
                        let d = hypot2(at(&ink, -2) - at(&p, 0), at(&ink, -1) - at(&p, 1));
                        pen = max2(pen, d);
                        ensure(d < 0.00003, || format!("{name} pen detached from ink"))?;
                        checks += 1;
                    }
                }
                if id == "sticker" {
                    for s in STYLES {
                        let (paper, fold) = (
                            sample(&format!("sticker-paper{s}"), t)?,
                            sample(&format!("sticker-fold{s}"), t)?,
                        );
                        // Paper's diagonal endpoints follow its first cubic: bottom = [12..14], top = [10..12].
                        let d = max2(
                            hypot2(at(&fold, 0) - at(&paper, 12), at(&fold, 1) - at(&paper, 13)),
                            hypot2(
                                at(&fold, -2) - at(&paper, 10),
                                at(&fold, -1) - at(&paper, 11),
                            ),
                        );
                        fold_error = max2(fold_error, d);
                        ensure(d < 0.00003, || {
                            format!("{name} fold detached from paper diagonal")
                        })?;
                        checks += 1;
                    }
                    for s in STYLES {
                        let (a, b) = (
                            pick_base(&scene, &format!("sticker-smile{s}"))?,
                            pick_base(&scene, &format!("sticker-fold{s}"))?,
                        );
                        let pa = flattener.points(&PathView::whole(a), Sampler::Ge, t)?;
                        let pb = flattener.points(&PathView::whole(b), Sampler::Ge, t)?;
                        let (wa, wb) = (
                            Sampler::Ge.scalar(&a.width, t),
                            Sampler::Ge.scalar(&b.width, t),
                        );
                        let gap = pa
                            .iter()
                            .fold(f64::INFINITY, |m, &p| min2(m, poly_dist(p, &pb)))
                            - (wa + wb) / 2.0;
                        smile = min2(smile, gap);
                        ensure(gap >= 1.0, || {
                            format!("{} smile crowds peeling fold: {}", a.name, js(gap))
                        })?;
                        checks += 1;
                    }
                }
                if id == "composing-message" {
                    for s in STYLES {
                        let n = pick_base(&scene, &format!("composing-message-caret{s}"))?;
                        let p = Sampler::Ge.coords(&n.points, t);
                        let r = Sampler::Ge.scalar(&n.width, t) / 2.0;
                        let gap = 27.0 - 1.3 - max2(at(&p, 0), at(&p, 2)) - r;
                        caret = min2(caret, gap);
                        ensure(gap >= 1.15, || {
                            format!("{name} caret crowds right chat border")
                        })?;
                        checks += 1;
                    }
                }
                if id == "voice-note" {
                    for s in STYLES {
                        for j in 0..3 {
                            let n = pick_base(&scene, &format!("voice-note-wave-{j}{s}"))?;
                            let p = Sampler::Ge.coords(&n.points, t);
                            let radius = Sampler::Ge.scalar(&n.width, t) / 2.0;
                            let (p0, p1, p3) = (at(&p, 0), at(&p, 1), at(&p, 3));
                            let gap = min2(
                                min2(min2(p1, p3) - radius - 9.3, 22.7 - max2(p1, p3) - radius),
                                26.7 - p0 - radius,
                            );
                            wave = min2(wave, gap);
                            ensure(gap >= 1.15, || {
                                format!("{name} waveform crowds clip border: {}", js(gap))
                            })?;
                            checks += 1;
                        }
                    }
                }
            }
        }
    }
    let mut report = serde_json::Map::new();
    report.insert("result".into(), "PASS".into());
    report.insert("checks".into(), checks.into());
    report.insert("frames".into(), frames.into());
    report.insert("minimumWavePaintGap".into(), js_number(wave));
    report.insert("maximumPenContactError".into(), js_number(pen));
    report.insert("maximumFoldContactError".into(), js_number(fold_error));
    report.insert("minimumStickerSmileGap".into(), js_number(smile));
    report.insert("minimumCaretBorderGap".into(), js_number(caret));
    Ok(Value::Object(report))
}
