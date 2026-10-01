//! 13-fintech-finance: a 3-unit painted margin plus four named aggregates, assert-mode.

use super::{Frame, ensure, flattener20, frame_times};
use crate::contracts::ContractError;
use crate::contracts::evidence::js_number;
use crate::contracts::jsnum::{max2, min2};
use crate::contracts::scene::{SceneSource, SetManifest};
use gai_core::js_number_to_string as js;
use gai_core::parallel::ordered_map;
use serde_json::{Value, json};

struct Outcome {
    checked: u64,
    margin: f64,
    contactless: f64,
    transfer: f64,
    door: f64,
    handle: f64,
}

pub(super) fn run(
    manifest: &SetManifest,
    scenes: &dyn SceneSource,
) -> Result<Value, ContractError> {
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
            checked: 0,
            margin: f64::INFINITY,
            contactless: f64::INFINITY,
            transfer: f64::INFINITY,
            door: f64::INFINITY,
            handle: f64::NEG_INFINITY,
        };
        for (f, t) in frame_times() {
            let fr = Frame {
                scene: &scene,
                flattener: &flattener,
                t,
            };
            fr.margins(|n, m| {
                o.margin = min2(o.margin, m);
                ensure(m >= 3.0 - 0.025, || {
                    format!(
                        "{} {} at frame {f}: painted margin {}",
                        action.name,
                        n.name,
                        js(m)
                    )
                })?;
                o.checked += 1;
                Ok(())
            })?;
            match icon {
                "contactless-payment" => {
                    let phone = fr.painted_box("contactless-payment-phone-frame")?;
                    let signal = fr.painted_box("contactless-payment-signal-0")?;
                    let gap = signal[0] - phone[2];
                    o.contactless = min2(o.contactless, gap);
                    ensure(gap >= 1.0 - 0.025, || {
                        format!("Contactless painted phone/signal gap {}", js(gap))
                    })?;
                }
                "bank-transfer" => {
                    for arrow in ["outgoing", "incoming"] {
                        let a = fr.painted_box(&format!("bank-transfer-{arrow}"))?;
                        let l = fr.painted_box("bank-transfer-account-left-frame")?;
                        let r = fr.painted_box("bank-transfer-account-right-frame")?;
                        let gap = min2(a[0] - l[2], r[0] - a[2]);
                        o.transfer = min2(o.transfer, gap);
                        ensure(gap >= 1.0 - 0.025, || {
                            format!("Transfer cap touches account {}", js(gap))
                        })?;
                    }
                }
                "bank" => {
                    let b = fr.painted_box("bank-door-frame")?;
                    let opening = b[2] - b[0] - 5.0;
                    o.door = min2(o.door, opening);
                    ensure(opening >= 1.5, || "Bank door counter closed".into())?;
                }
                "portfolio" => {
                    let h = fr.raw(fr.node("portfolio-handle")?);
                    let edge = fr.painted_box("portfolio-case-frame")?[1] + 1.25;
                    let at = |i: usize| h.get(i).copied().unwrap_or(f64::NAN);
                    let error = max2(
                        (at(1) - edge).abs(),
                        (h.last().copied().unwrap_or(f64::NAN) - edge).abs(),
                    );
                    o.handle = max2(o.handle, error);
                    ensure(error < 3e-5, || "Handle detaches from case".into())?;
                }
                _ => {}
            }
        }
        Ok(o)
    })?;

    let (mut margin, mut contactless, mut transfer, mut door, mut handle) =
        (32.0, 32.0, 32.0, 32.0, 0.0);
    let mut actions = Vec::new();
    for ((_, action), o) in jobs.iter().zip(&outcomes) {
        actions.push(json!({"name": action.name, "paintedBoundsChecked": o.checked}));
        margin = min2(margin, o.margin);
        contactless = min2(contactless, o.contactless);
        transfer = min2(transfer, o.transfer);
        door = min2(door, o.door);
        handle = max2(handle, o.handle);
    }
    let mut report = serde_json::Map::new();
    report.insert("result".into(), "PASS".into());
    report.insert("actions".into(), Value::Array(actions));
    report.insert("minimumPaintedMargin".into(), js_number(margin));
    report.insert("minimumContactlessGap".into(), js_number(contactless));
    report.insert("minimumTransferGap".into(), js_number(transfer));
    report.insert("minimumDoorOpening".into(), js_number(door));
    report.insert("maximumHandleAttachmentError".into(), js_number(handle));
    Ok(Value::Object(report))
}
