//! Live export on the hosted engine. Needs a prior `generative-icons login` (or
//! `GAI_ENGINE_TOKEN`) and the gitignored scene and native oracles:
//! `cargo test -p gai-engine --test live -- --ignored`.

mod common;

use common::{HOSTED_FIXTURES, library_root, native_oracle, with_hosted_path_names};
use gai_core::export::Exporter;
use gai_engine::{EngineConfig, HostedExporter, status, whoami};

#[test]
#[ignore = "talks to the hosted engine; needs a prior login"]
fn live_exports_match_the_committed_natives_except_path_names() {
    let config = EngineConfig::from_env().unwrap();
    if status(&config).unwrap().is_none() {
        eprintln!(
            "skipped: not signed in to {} (run `generative-icons login`)",
            config.url
        );
        return;
    }
    let Some(root) = library_root() else {
        eprintln!("skipped: no library checkout");
        return;
    };
    let me = whoami(&config).unwrap();
    eprintln!(
        "{} {} as {:?}",
        me.server_name, me.server_version, me.status.account
    );

    let exporter = HostedExporter::new(&config).unwrap();
    for (set, name) in HOSTED_FIXTURES {
        let scene_path = root
            .join("motion/icon-sets")
            .join(set)
            .join("scenes")
            .join(format!("{name}.scene.json"));
        let (Ok(scene), Some(native)) = (
            std::fs::read_to_string(&scene_path),
            native_oracle(set, name),
        ) else {
            eprintln!("skipped {set}/{name}: scene or native oracle missing");
            continue;
        };
        let exported = exporter.export(name, &scene).unwrap();
        assert_eq!(
            exported.text,
            with_hosted_path_names(&native, &exported.text),
            "{set}/{name}"
        );
        eprintln!(
            "{set}/{name}: {} bytes, sha256 {}",
            exported.text.len(),
            exported.provenance["sha256"]
        );
    }
}
