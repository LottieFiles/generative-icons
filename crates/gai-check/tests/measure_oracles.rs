//! Byte parity of `evidence/motion-measurements.json` (sets 22–39).
//!
//! The measurements need authoring-code samples (unrounded, unexpanded parts), which the
//! gai-js `probe` will provide. Until then this test drives the same [`ProbeSource`]
//! contract with Node, running `measure-motion.mjs`'s verbatim `points()` helper on the
//! set's own `definitions.mjs`, so everything after sampling (RMS, folds, ratios, the
//! JSON writer) is checked against the committed oracles. Skipped when `node` or the
//! repo's `motion/node_modules` is unavailable; the repo is only read.

use gai_check::measure::{
    EVIDENCE_FILE, PartProbe, ProbeError, ProbeIcon, ProbeSource, builtin_spec, measure_motion,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Dumps the probe samples of every icon's main part. Times: `0`, then `f/60`.
const DUMP: &str = r#"
import {pathToFileURL} from 'node:url';
const [setDir, svgpathFile, primaryJson] = process.argv.slice(2);
const svgpath = (await import(pathToFileURL(svgpathFile))).default;
const {icons} = await import(pathToFileURL(setDir + '/definitions.mjs'));
const primary = JSON.parse(primaryJson);
// Verbatim from measure-motion.mjs.
function points(p,t,s){
 const path=svgpath(typeof p.d==='function'?p.d(t,s,false,2):p.d).abs().unarc().unshort();
 const out=[];let last=[0,0];
 for(const [command,...a] of path.segments){
  if(command==='Z')continue;
  if(command==='H'){a.unshift(last[0]);a[0]=a[1];a[1]=last[1];}
  if(command==='V')a.unshift(last[0]);
  for(let k=0;k<a.length;k+=2)out.push(...(p.pose?p.pose(a[k],a[k+1],t,s,false):[a[k],a[k+1]]));
  last=a.slice(-2);
 }
 return out;
}
const times = [0, ...Array.from({length: 145}, (_, f) => f / 60)];
const out = {icons: [], probes: {}};
icons.forEach((icon, i) => {
  out.icons.push({name: icon.name, intent: icon.intent, active: [icon.timings.light.active, icon.timings.strong.active], parts: icon.parts.map(p => p.name)});
  const part = icon.parts.find(p => p.name === icon.name + '-' + primary[i]);
  if (!part) return;
  for (const strong of [false, true]) out.probes[i + ':' + strong] = {
    points: times.map(t => points(part, t, strong)),
    alpha: times.map(t => part.visibility?.(t, strong, false) ?? 1),
    transform: part.pose?.transform ? times.map(t => {const r = part.pose.transform(t, strong, false); return [r.angle || 0, r.dx || 0, r.dy || 0];}) : null,
  };
});
process.stdout.write(JSON.stringify(out));
"#;

#[derive(Deserialize)]
struct Dump {
    icons: Vec<DumpIcon>,
    probes: HashMap<String, DumpProbe>,
}

#[derive(Deserialize)]
struct DumpIcon {
    name: String,
    intent: Option<String>,
    active: [Option<f64>; 2],
    parts: Vec<String>,
}

#[derive(Deserialize)]
struct DumpProbe {
    points: Vec<Vec<f64>>,
    alpha: Vec<Option<f64>>,
    transform: Option<Vec<[f64; 3]>>,
}

/// Samples recorded by Node for one set.
struct NodeProbe(Dump);

impl ProbeSource for NodeProbe {
    fn icons(&mut self) -> Result<Vec<ProbeIcon>, ProbeError> {
        Ok(self
            .0
            .icons
            .iter()
            .map(|i| ProbeIcon {
                name: i.name.clone(),
                intent: i.intent.clone(),
                active_seconds: i.active,
                parts: i.parts.clone(),
            })
            .collect())
    }

    fn probe(
        &mut self,
        icon: usize,
        _part: usize,
        strong: bool,
        times: &[f64],
    ) -> Result<PartProbe, ProbeError> {
        let recorded = self
            .0
            .probes
            .get(&format!("{icon}:{strong}"))
            .ok_or_else(|| ProbeError::new("part was not recorded"))?;
        assert_eq!(
            times.len(),
            recorded.points.len(),
            "the dump samples the same times"
        );
        Ok(PartProbe {
            points: recorded.points.clone(),
            alpha: recorded
                .alpha
                .iter()
                .map(|a| a.unwrap_or(f64::NAN))
                .collect(),
            transform: recorded.transform.clone(),
        })
    }
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Measures one set through the Node probe and compares with the committed evidence.
/// Returns `None` when the inputs are unavailable.
fn matches_oracle(set: &str) -> Option<bool> {
    let set_dir = repo().join("motion/icon-sets").join(set);
    let svgpath = repo().join("motion/node_modules/svgpath/index.js");
    let node = Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    if !node || !svgpath.is_file() {
        eprintln!("{set}: skipped (node or motion/node_modules unavailable)");
        return None;
    }
    let spec = builtin_spec(set).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let script = scratch.path().join("dump.mjs");
    std::fs::write(&script, DUMP).unwrap();
    let out = Command::new("node")
        .arg(&script)
        .arg(&set_dir)
        .arg(&svgpath)
        .arg(serde_json::to_string(&spec.primary).unwrap())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{set}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let mut probe = NodeProbe(serde_json::from_slice(&out.stdout).unwrap());
    let definitions = std::fs::read(set_dir.join("definitions.mjs")).unwrap();
    let measured =
        measure_motion(&spec, &definitions, &mut probe).unwrap_or_else(|e| panic!("{set}: {e}"));
    let flagged: Vec<&str> = measured
        .review_at_native_speed()
        .map(|r| r.icon.as_str())
        .collect();
    let same = measured.to_json() == std::fs::read_to_string(set_dir.join(EVIDENCE_FILE)).unwrap();
    eprintln!(
        "{set}: {} (review at native speed: {flagged:?})",
        if same { "byte-identical" } else { "DIFFERENT" }
    );
    if !same {
        let path = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("measure")
            .join(format!("{set}.json"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, measured.to_json()).unwrap();
        eprintln!("  wrote {}", path.display());
    }
    Some(same)
}

/// One raw-ratio set (with the Strong-to-Light flag) and one visible-ratio set.
#[test]
fn representative_sets_match_oracles_via_node_probe() {
    for set in ["23-get-help", "39-food-drink"] {
        assert_ne!(matches_oracle(set), Some(false), "{set} differs");
    }
}

#[test]
#[ignore = "whole catalogue (22–39); needs node and motion/node_modules"]
fn every_measurement_matches_oracle_via_node_probe() {
    let sets: Vec<String> = std::fs::read_dir(repo().join("motion/icon-sets"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|name| builtin_spec(name).is_some())
        .collect();
    let mut sets = sets;
    sets.sort();
    assert_eq!(sets.len(), 18);
    let different: Vec<&String> = sets
        .iter()
        .filter(|s| matches_oracle(s) == Some(false))
        .collect();
    assert!(
        different.is_empty(),
        "measurements differ for {different:?}"
    );
}
