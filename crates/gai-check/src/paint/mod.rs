//! The catalog paint contract (`verify-paint.mjs`, verify.md §3.14, packaging.md §5.2):
//! every exported stroke is slot-bound with one of the three outline weights, Regular
//! strokes carry the stroke-width token, nothing is scaled, open icons have no tint, and
//! `duotoneFill` metadata matches the tint paints. The weights, token and open-icon list
//! are data ([`PaintContract`]), not code.
//!
//! ```no_run
//! # use gai_check::paint::{PaintContract, audit_catalog, EVIDENCE_FILE};
//! let report = audit_catalog("prototype/icons".as_ref(), PaintContract::builtin())?;
//! std::fs::write(EVIDENCE_FILE, report.to_json())?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::verify::js::{default_sort, is_number, same_value, strict_equals, truthy};
use gai_core::parallel::ordered_map;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Where the catalog audit writes its evidence, relative to the repository root.
pub const EVIDENCE_FILE: &str = "motion/icon-sets/library/evidence/paint-verification.json";

/// The built-in contract of the catalog.
const BUILTIN: &str = include_str!("../../rules/verify/paint-contract.json");

/// Paint-contract failures. The first failed check aborts the audit.
#[derive(Debug, thiserror::Error)]
pub enum PaintError {
    /// A file could not be read.
    #[error("cannot read {}: {source}", path.display())]
    Io {
        /// The file.
        path: PathBuf,
        /// The cause.
        source: std::io::Error,
    },
    /// A file is not valid JSON.
    #[error("cannot parse {}: {source}", path.display())]
    Json {
        /// The file.
        path: PathBuf,
        /// The cause.
        source: serde_json::Error,
    },
    /// A contract document is malformed.
    #[error("invalid paint contract: {0}")]
    Contract(String),
    /// A check failed; the message names it.
    #[error("{0}")]
    Assertion(String),
}

/// The paint contract: outline weights, the Regular stroke-width token and the icons with
/// no enclosed surface to tint.
#[derive(Debug, Clone, PartialEq)]
pub struct PaintContract {
    /// Slot of the Regular stroke width.
    pub stroke_width_token: String,
    /// Outline presets and their stroke widths, in report order (`light`, `regular`, `bold`).
    pub weights: Vec<(String, f64)>,
    /// The `units` text of the report.
    pub units: String,
    /// Icons whose Outline is filled dots rather than strokes (`more-options`).
    pub filled_ellipsis: Vec<String>,
    /// Icons that must have no tint paint, in report order.
    pub open_icons: Vec<String>,
}

impl PaintContract {
    /// The catalog's contract (`rules/verify/paint-contract.json`).
    pub fn builtin() -> &'static PaintContract {
        static CONTRACT: OnceLock<PaintContract> = OnceLock::new();
        CONTRACT.get_or_init(|| {
            PaintContract::from_slice(BUILTIN.as_bytes()).expect("built-in paint contract is valid")
        })
    }

    /// Parses a contract document (`{"format": "gai-paint-contract", "version": 1, …}`).
    pub fn from_slice(bytes: &[u8]) -> Result<PaintContract, PaintError> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Doc {
            format: String,
            version: u32,
            stroke_width_token: String,
            weights: Map<String, Value>,
            units: String,
            filled_ellipsis: Vec<String>,
            open_icons: Vec<String>,
        }
        let bad = |m: String| PaintError::Contract(m);
        let d: Doc = serde_json::from_slice(bytes).map_err(|e| bad(e.to_string()))?;
        if d.format != "gai-paint-contract" || d.version != 1 {
            return Err(bad(format!(
                "unsupported format {} v{}",
                d.format, d.version
            )));
        }
        let weights = d
            .weights
            .into_iter()
            .map(|(k, v)| {
                v.as_f64()
                    .map(|w| (k.clone(), w))
                    .ok_or_else(|| bad(format!("weight {k} is not a number")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !weights.iter().any(|(k, _)| k == "regular") {
            return Err(bad("weights need a regular preset".into()));
        }
        let mut seen = HashSet::new();
        if let Some(dup) = d.open_icons.iter().find(|i| !seen.insert(i.as_str())) {
            return Err(bad(format!("open icon {dup} is listed twice")));
        }
        Ok(PaintContract {
            stroke_width_token: d.stroke_width_token,
            weights,
            units: d.units,
            filled_ellipsis: d.filled_ellipsis,
            open_icons: d.open_icons,
        })
    }

    /// The stroke width of a preset.
    pub fn weight(&self, preset: &str) -> Option<f64> {
        self.weights
            .iter()
            .find(|(k, _)| k == preset)
            .map(|(_, w)| *w)
    }

    /// The Regular weight the token slot and theme rules carry.
    pub fn regular_weight(&self) -> f64 {
        self.weight("regular").unwrap_or(f64::NAN)
    }

    /// `verifyFilledEllipsis(data, label)`: three closed dots per weight, each exactly one
    /// stroke width across, and no dummy strokes.
    pub fn verify_filled_ellipsis(&self, data: &Value, label: &str) -> Result<(), PaintError> {
        let layers = array(data.get("layers"), "data.layers")?;
        for (preset, width) in &self.weights {
            let sid = format!("style.outline.{preset}");
            let mut dots = Vec::new();
            for layer in layers {
                let shapes = array(layer.get("shapes"), "layer.shapes")?;
                if shapes
                    .iter()
                    .any(|s| ty(s) == Some("fl") && sid_of(s.get("o")) == Some(sid.as_str()))
                {
                    dots.push(shapes);
                }
            }
            ensure(dots.len() == 3, || {
                format!("{label} must contain three {preset} dots")
            })?;
            for shapes in dots {
                ensure(!shapes.iter().any(|s| ty(s) == Some("st")), || {
                    format!("{label} must not use dummy strokes")
                })?;
                let paths: Vec<&Value> = shapes.iter().filter(|s| ty(s) == Some("sh")).collect();
                ensure(paths.len() == 1, || {
                    format!("{label} each dot is one contour")
                })?;
                let ks = paths[0]
                    .get("ks")
                    .ok_or_else(|| PaintError::Assertion(format!("{label} dot has no ks")))?;
                let geometry: Vec<&Value> = if truthy(ks.get("a")) {
                    let mut all = Vec::new();
                    for k in array(ks.get("k"), "ks.k")? {
                        for key in ["s", "e"] {
                            if truthy(k.get(key)) {
                                all.extend(array(k.get(key), key)?);
                            }
                        }
                    }
                    all
                } else {
                    ks.get("k").into_iter().collect()
                };
                ensure(!geometry.is_empty(), || {
                    format!("{label} missing dot geometry")
                })?;
                for shape in geometry {
                    ensure(truthy(shape.get("c")), || {
                        format!("{label} dot must be closed")
                    })?;
                    for axis in 0..2 {
                        let values: Vec<f64> = array(shape.get("v"), "shape.v")?
                            .iter()
                            .map(|v| v.get(axis).and_then(Value::as_f64).unwrap_or(f64::NAN))
                            .collect();
                        let span = crate::contracts::jsnum::max_of(values.iter().copied())
                            - crate::contracts::jsnum::min_of(values.iter().copied());
                        ensure((span - 2.0 * width).abs() < 0.00005, || {
                            format!("{label} incorrect {preset} dot diameter")
                        })?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// `verifyFilledEllipsis` with the built-in contract's weights.
pub fn verify_filled_ellipsis(data: &Value, label: &str) -> Result<(), PaintError> {
    PaintContract::builtin().verify_filled_ellipsis(data, label)
}

fn ensure(ok: bool, message: impl FnOnce() -> String) -> Result<(), PaintError> {
    if ok {
        Ok(())
    } else {
        Err(PaintError::Assertion(message()))
    }
}

fn array<'a>(v: Option<&'a Value>, what: &str) -> Result<&'a Vec<Value>, PaintError> {
    v.and_then(Value::as_array)
        .ok_or_else(|| PaintError::Assertion(format!("{what} is not an array")))
}

fn ty(shape: &Value) -> Option<&str> {
    shape.get("ty").and_then(Value::as_str)
}

/// `o?.sid` when it is a string.
fn sid_of(o: Option<&Value>) -> Option<&str> {
    o.and_then(|o| o.get("sid")).and_then(Value::as_str)
}

fn read(path: &Path) -> Result<Vec<u8>, PaintError> {
    std::fs::read(path).map_err(|source| PaintError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn read_json(path: &Path) -> Result<Value, PaintError> {
    serde_json::from_slice(&read(path)?).map_err(|source| PaintError::Json {
        path: path.to_path_buf(),
        source,
    })
}

/// `values(p)`: a property's static value, or every truthy keyframe `s`/`e`.
fn values(p: &Value) -> Result<Vec<&Value>, PaintError> {
    if truthy(p.get("a")) {
        let mut out = Vec::new();
        for k in array(p.get("k"), "k")? {
            out.extend(
                [k.get("s"), k.get("e")]
                    .into_iter()
                    .flatten()
                    .filter(|v| truthy(Some(v))),
            );
        }
        Ok(out)
    } else {
        Ok(vec![p.get("k").unwrap_or(&Value::Null)])
    }
}

/// Every component of every scale value is exactly 100.
fn unscaled(p: &Value, message: impl Fn() -> String) -> Result<(), PaintError> {
    for scale in values(p)? {
        let components = scale
            .as_array()
            .ok_or_else(|| PaintError::Assertion(message()))?;
        ensure(
            components
                .iter()
                .all(|v| strict_equals(Some(v), Some(&json!(100)))),
            &message,
        )?;
    }
    Ok(())
}

/// One action's audit: its record and the number of outline stroke paints.
struct ActionAudit {
    record: Value,
    stroke_paints: u64,
}

/// What a walk over one action's shapes found.
#[derive(Default)]
struct Found {
    tint: u64,
    presets: Vec<String>,
    stroke_paints: u64,
}

impl PaintContract {
    fn walk(&self, items: Option<&Value>, name: &str, found: &mut Found) -> Result<(), PaintError> {
        let items = match items {
            None | Some(Value::Null) => return Ok(()),
            Some(v) => array(Some(v), "shapes")?,
        };
        for s in items {
            let t = ty(s);
            if t == Some("gr") {
                self.walk(s.get("it"), name, found)?;
            }
            if t == Some("tr") && truthy(s.get("s")) {
                unscaled(&s["s"], || {
                    format!("Scaled group changes stroke weight: {name}")
                })?;
            }
            let sid = s.get("o").and_then(|o| o.get("sid"));
            if matches!(t, Some("st" | "fl")) && sid == Some(&json!("style.tint")) {
                found.tint += 1;
            }
            if t != Some("st") {
                continue;
            }
            let outline = match sid {
                None | Some(Value::Null) => false,
                Some(Value::String(s)) => s.starts_with("style.outline."),
                Some(_) => {
                    return Err(PaintError::Assertion(format!(
                        "{name}: o.sid.startsWith is not a function"
                    )));
                }
            };
            if !outline {
                continue;
            }
            let preset = sid
                .and_then(Value::as_str)
                .and_then(|s| s.rsplit('.').next())
                .unwrap_or_default();
            let weight = self
                .weight(preset)
                .ok_or_else(|| PaintError::Assertion(format!("{name}: unknown preset {preset}")))?;
            let w = s
                .get("w")
                .ok_or_else(|| PaintError::Assertion(format!("{name}: stroke has no width")))?;
            for v in values(w)? {
                let first = if let Value::Array(items) = v {
                    items.first()
                } else {
                    Some(v)
                };
                ensure(
                    first
                        .and_then(Value::as_f64)
                        .is_some_and(|x| same_value(x, weight)),
                    || format!("{name} {preset} stroke"),
                )?;
            }
            let token = (preset == "regular").then(|| json!(self.stroke_width_token));
            ensure(w.get("sid") == token.as_ref(), || {
                format!("{name} {preset} stroke-width token")
            })?;
            ensure(is_number(s.get("lc"), 2.0), || format!("{name} round cap"))?;
            ensure(is_number(s.get("lj"), 2.0), || format!("{name} round join"))?;
            found.stroke_paints += 1;
            if !found.presets.iter().any(|p| p == preset) {
                found.presets.push(preset.to_string());
            }
        }
        Ok(())
    }

    fn audit_action(
        &self,
        folder: &Path,
        icon: &Value,
        action: &Value,
    ) -> Result<ActionAudit, PaintError> {
        let icon_name = icon.get("name").and_then(Value::as_str).unwrap_or_default();
        let name = action
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| PaintError::Assertion("an action has no name".into()))?;
        let bytes = read(&folder.join(format!("{name}.json")))?;
        let data: Value = serde_json::from_slice(&bytes).map_err(|source| PaintError::Json {
            path: folder.join(format!("{name}.json")),
            source,
        })?;
        let regular = self.regular_weight();
        ensure(
            is_number(data.get("w"), 32.0) && is_number(data.get("h"), 32.0),
            || format!("{name}: not a 32-unit canvas"),
        )?;
        let slot = data
            .get("slots")
            .and_then(|s| s.get(&self.stroke_width_token))
            .and_then(|s| s.get("p"))
            .and_then(|p| p.get("k"));
        ensure(is_number(slot, regular), || {
            format!("{name} stroke-width token slot")
        })?;
        for theme in ["dark", "light"] {
            let file = read_json(&folder.join(format!("{name}.theme-{theme}.json")))?;
            let has_rule = array(file.get("rules"), "theme.rules")?.iter().any(|r| {
                r.get("id").and_then(Value::as_str) == Some(self.stroke_width_token.as_str())
                    && r.get("type") == Some(&json!("Scalar"))
                    && strict_equals(r.get("value"), Some(&json!(regular)))
            });
            ensure(has_rule, || {
                format!("{name} {theme} theme stroke-width rule")
            })?;
        }
        let mut found = Found::default();
        for layer in array(data.get("layers"), "data.layers")? {
            if let Some(scale) = layer
                .get("ks")
                .and_then(|ks| ks.get("s"))
                .filter(|s| truthy(Some(s)))
            {
                unscaled(scale, || {
                    format!("Scaled layer changes stroke weight: {name}")
                })?;
            }
            self.walk(layer.get("shapes"), name, &mut found)?;
        }
        if self.filled_ellipsis.iter().any(|i| i == icon_name) {
            ensure(found.presets.is_empty(), || {
                format!("{name}: filled ellipsis has outline strokes")
            })?;
            self.verify_filled_ellipsis(&data, name)?;
        } else {
            let mut got: Vec<Option<Value>> =
                found.presets.iter().map(|p| Some(json!(p))).collect();
            let mut want: Vec<Option<Value>> =
                self.weights.iter().map(|(k, _)| Some(json!(k))).collect();
            default_sort(&mut got);
            default_sort(&mut want);
            ensure(got == want, || format!("{name}: outline presets {got:?}"))?;
        }
        let tinted = found.tint > 0;
        if self.open_icons.iter().any(|i| i == icon_name) {
            ensure(!tinted, || {
                format!("{icon_name} has no enclosed surface to tint")
            })?;
        }
        if let Some(fill) = action.get("duotoneFill") {
            ensure(fill == &json!(tinted), || {
                format!("{name} stale Duotone metadata")
            })?;
        }
        let record = json!({"icon": icon_name, "action": name, "sha256": gai_core::sha256_hex(&bytes), "duotoneFill": tinted});
        Ok(ActionAudit {
            record,
            stroke_paints: found.stroke_paints,
        })
    }
}

/// The audit report, keys in output order.
#[derive(Debug, Clone, PartialEq)]
pub struct PaintReport {
    /// The report object.
    pub report: Value,
    /// Distinct icons audited.
    pub icons: usize,
    /// Actions audited.
    pub actions: usize,
    /// Outline stroke paints checked.
    pub stroke_paints: u64,
}

impl PaintReport {
    /// The evidence file bytes.
    pub fn to_json(&self) -> String {
        crate::verify::evidence_json(&self.report)
    }

    /// The PASS line the script prints.
    pub fn summary(&self) -> String {
        format!(
            "PASS {} icons / {} actions: {} constant outline stroke paints; no scaled layers/groups; open icons have no tint.",
            self.icons, self.actions, self.stroke_paints
        )
    }
}

/// Audits the sets listed in `<icons_dir>/catalog.json` (`sets`: `set.json` paths relative
/// to `icons_dir`), in catalog order.
pub fn audit_catalog(
    icons_dir: &Path,
    contract: &PaintContract,
) -> Result<PaintReport, PaintError> {
    let catalog = read_json(&icons_dir.join("catalog.json"))?;
    let sets = array(catalog.get("sets"), "catalog.sets")?
        .iter()
        .map(|s| {
            s.as_str()
                .map(|s| icons_dir.join(s))
                .ok_or_else(|| PaintError::Assertion("a catalog set is not a path".into()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    audit(&sets, contract)
}

/// Audits the actions of each `set.json`, in order. Action files sit next to their manifest.
pub fn audit(manifests: &[PathBuf], contract: &PaintContract) -> Result<PaintReport, PaintError> {
    let documents = manifests
        .iter()
        .map(|m| read_json(m))
        .collect::<Result<Vec<_>, _>>()?;
    let mut jobs = Vec::new();
    for (path, doc) in manifests.iter().zip(&documents) {
        let folder = path.parent().unwrap_or(Path::new("."));
        for icon in array(doc.get("icons"), "set.icons")? {
            let actions = match icon.get("animations").filter(|a| truthy(Some(a))) {
                Some(a) => a,
                None => icon.get("motions").unwrap_or(&Value::Null),
            };
            for action in array(Some(actions), "icon.animations")? {
                jobs.push((folder, icon, action));
            }
        }
    }
    let audits = ordered_map(jobs.len(), |i| {
        contract.audit_action(jobs[i].0, jobs[i].1, jobs[i].2)
    })?;
    let actions = audits.len();
    let stroke_paints = audits.iter().map(|a| a.stroke_paints).sum();
    let mut icons = HashSet::new();
    for a in &audits {
        icons.insert(a.record["icon"].as_str().unwrap_or_default().to_string());
    }
    let mut report = Map::new();
    report.insert("result".into(), json!("PASS"));
    report.insert("icons".into(), json!(icons.len()));
    report.insert("actions".into(), json!(actions));
    report.insert("strokePaints".into(), json!(stroke_paints));
    report.insert(
        "weights".into(),
        Value::Object(
            contract
                .weights
                .iter()
                .map(|(k, w)| (k.clone(), json!(w)))
                .collect(),
        ),
    );
    report.insert(
        "strokeWidthToken".into(),
        json!(contract.stroke_width_token),
    );
    report.insert("units".into(), json!(contract.units));
    report.insert("openIcons".into(), json!(contract.open_icons));
    report.insert(
        "records".into(),
        Value::Array(audits.into_iter().map(|a| a.record).collect()),
    );
    Ok(PaintReport {
        icons: icons.len(),
        actions,
        stroke_paints,
        report: Value::Object(report),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_contract_keeps_the_legacy_list() {
        let c = PaintContract::builtin();
        assert_eq!(c.open_icons.len(), 180);
        assert_eq!(c.open_icons.first().map(String::as_str), Some("download"));
        assert_eq!(c.open_icons.last().map(String::as_str), Some("rss-feed"));
        assert_eq!(
            c.weights,
            [
                ("light".to_string(), 1.5),
                ("regular".to_string(), 2.0),
                ("bold".to_string(), 2.5)
            ]
        );
        assert_eq!(c.regular_weight(), 2.0);
        assert!(PaintContract::from_slice(br#"{"format":"gai-paint-contract","version":1,"strokeWidthToken":"t","weights":{"bold":2.5},"units":"u","filledEllipsis":[],"openIcons":[]}"#).is_err());
    }

    fn stroke(preset: &str, width: Value, sid: Option<&str>) -> Value {
        let mut w = json!({"a": 0, "k": width});
        if let Some(sid) = sid {
            w["sid"] = json!(sid);
        }
        json!({"ty": "st", "c": {"sid": "color.primary"}, "o": {"sid": format!("style.outline.{preset}")}, "w": w, "lc": 2, "lj": 2})
    }

    #[test]
    fn walk_checks_weights_tokens_and_scale() {
        let c = PaintContract::builtin();
        let shapes = json!([
            {"ty": "gr", "it": [stroke("light", json!(1.5), None), {"ty": "tr", "s": {"a": 0, "k": [100, 100]}}]},
            stroke("regular", json!(2), Some("style.outline.width")),
            stroke("bold", json!([2.5]), None),
            {"ty": "fl", "c": {"sid": "color.primary"}, "o": {"sid": "style.tint"}}
        ]);
        let mut found = Found::default();
        c.walk(Some(&shapes), "cup", &mut found).unwrap();
        assert_eq!(
            (found.tint, found.stroke_paints, found.presets.len()),
            (1, 3, 3)
        );
        for bad in [
            json!([stroke("regular", json!(2), None)]),
            json!([stroke("bold", json!(2.5), Some("style.outline.width"))]),
            json!([stroke("light", json!(2), None)]),
            json!([{"ty": "tr", "s": {"a": 0, "k": [100, 99]}}]),
        ] {
            assert!(
                c.walk(Some(&bad), "cup", &mut Found::default()).is_err(),
                "{bad}"
            );
        }
    }
}
