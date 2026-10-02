//! `context` against `generation-context.mjs` (cli.md §4.3): the `--search` goldens and
//! the packet fingerprint in library mode, a byte differential with Node when it is
//! installed, project-mode packets (package plus sidecar) equal to library-mode packets
//! apart from `inputs`, and built local references read within a player's archive limits.

mod common;

use common::*;
use serde_json::{Map, Value};
use std::process::Command;

fn repo_arg() -> String {
    repo_root().display().to_string()
}

/// Python `json.dumps(value, sort_keys=True)`.
fn sorted_py_json(value: &Value) -> String {
    fn sort(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                Value::Object(keys.into_iter().map(|key| (key.clone(), sort(&map[key]))).collect::<Map<_, _>>())
            }
            Value::Array(items) => Value::Array(items.iter().map(sort).collect()),
            other => other.clone(),
        }
    }
    gai_core::to_py_json(&sort(value), None, true)
}

#[test]
fn library_mode_matches_the_generation_context_goldens() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo_arg();
    for (query, golden) in [
        ("notification bell", "4dc3ca333bfc1c33715e7041460adacd4c5d0265a36ca30baaf2c2489c35f786"),
        ("package return", "f4204788f9a6e1a82c539b308b6ef8c9facd3504b715b44cb503a0e23a6f527c"),
        ("heart", "3cd3cc8129f42680b55f8c15daa7562ef713713b87aa0d87cfcce74fa9ef3dff"),
        ("loading spinner", "742e4f4f84fc495d31b5e7b56c10aab090b0a14aba7c66ef54aca30c0494e9fd"),
        ("create a new icon for the payment", "53e53d7f21f60058cb256d75639b0550d0ab7997f410c28ddf72b27548ed19a4"),
        ("zzzz", "37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570"),
        ("Café İstanbul key", "179bea1f58b086a9f4d740d69de33dde64d621b48d1463c9ebaf2f4e3b2b9423"),
    ] {
        let run = run(dir.path(), &["--json", "--library-root", &repo, "context", "--search", query]);
        assert_eq!(gai_core::sha256_hex(run.stdout.as_bytes()), golden, "{query}: {}", run.stdout);
    }
    let mut packet =
        run(dir.path(), &["--library-root", &repo, "context", "notification feedback", "--neighbor", "bell"]).ok();
    let inputs = packet.as_object_mut().unwrap().remove("inputs").unwrap();
    assert_eq!(inputs.as_array().unwrap().len(), 63);
    assert_eq!(
        gai_core::sha256_hex(sorted_py_json(&packet)),
        "f0521027be419c9ad307f91924940e4538402949bce8cbd6e4c15f69d0b7d70d"
    );

    // The prompt embeds the packet after the skill's rules.
    let output = dir.path().join("proposal");
    let brief = dir.path().join("brief.md");
    std::fs::write(&brief, "A bell that rings once for a new message.\n").unwrap();
    let prompted = run(
        dir.path(),
        &["--library-root", &repo, "context", "--prompt", "--brief", "brief.md", "--output", "proposal"],
    )
    .ok();
    let prompt = std::fs::read_to_string(output.join("prompt.md")).unwrap();
    let packet_text = std::fs::read_to_string(output.join("generation-context.json")).unwrap();
    assert_eq!(prompted["prompt"], prompt);
    assert!(prompt.starts_with("You are providing a bounded icon design proposal through an optional CLI adapter.\n"));
    assert!(prompt.contains("\n\n## User brief: brief.md\n\nA bell that rings once for a new message.\n\n\n## Project workflow and design rules: SKILL.md\n\n"));
    assert!(
        prompt.ends_with(&format!("## Current local reference packet (data, not a design approval)\n\n{packet_text}"))
    );
    run(dir.path(), &["--library-root", &repo, "context", "bell", "--output", "proposal"]).fails(3, "EXISTS");
    run(dir.path(), &["--library-root", &repo, "context", "bell", "--neighbor", "no-such"]).fails(3, "UNKNOWN_ICON");
}

#[test]
fn library_mode_packets_equal_node_but_for_the_self_input() {
    let node = Command::new("node").arg("--version").output();
    if !node.is_ok_and(|output| output.status.success()) {
        eprintln!("skipped: node not found");
        return;
    }
    let repo = repo_root();
    let dir = tempfile::tempdir().unwrap();
    for neighbors in [vec![], vec!["bell"], vec!["turn-taking", "folder-open", "orbit-comet"]] {
        let mut ours = vec!["--library-root".to_string(), repo_arg(), "context".into(), "notification feedback".into()];
        let mut theirs = vec![
            "motion/icon-sets/library/generation-context.mjs".to_string(),
            "--query".into(),
            "notification feedback".into(),
        ];
        for neighbor in &neighbors {
            ours.extend(["--neighbor".to_string(), neighbor.to_string()]);
            theirs.extend(["--neighbor".to_string(), neighbor.to_string()]);
        }
        let ours = run(dir.path(), &ours.iter().map(String::as_str).collect::<Vec<_>>());
        let node = Command::new("node").args(&theirs).current_dir(&repo).output().unwrap();
        let theirs = String::from_utf8(node.stdout).unwrap();
        let (ours, theirs): (Vec<&str>, Vec<&str>) = (ours.stdout.lines().collect(), theirs.lines().collect());
        assert_eq!(ours.len(), theirs.len(), "{neighbors:?}");
        let differing: Vec<usize> = (0..ours.len()).filter(|&i| ours[i] != theirs[i]).collect();
        // Only the self input's path and sha256 lines differ.
        assert_eq!(differing.len(), 2, "{neighbors:?}: {differing:?}");
        assert!(ours[differing[0]].contains("\"path\": \"gen-animated-icons@"), "{}", ours[differing[0]]);
        assert!(theirs[differing[0]].ends_with("generation-context.mjs\","));
    }
}

#[test]
fn project_packets_equal_library_packets_but_for_inputs() {
    require!(packages_dist(), "packages/icons/dist");
    let dir = tempfile::tempdir().unwrap();
    let repo = repo_arg();
    let packages = dir.path().join("packages");
    run(dir.path(), &["--library-root", &repo, "library", "packages", "--output", packages.to_str().unwrap()]).ok();
    let dist = packages.join("icons/dist").display().to_string();
    // bell, heart and search are the three icons whose Strong intent differs from the
    // icon's; bell (set 01) also lacks states and a brief.
    for neighbors in
        [vec!["bell", "heart", "search"], vec!["turn-taking", "folder-open", "loading-spinner", "orbit-comet"], vec![]]
    {
        let mut args = vec!["context", "notification feedback"];
        for neighbor in &neighbors {
            args.extend(["--neighbor", neighbor]);
        }
        let library = [["--library-root", repo.as_str()].as_slice(), &args].concat();
        let project = [["--library", dist.as_str()].as_slice(), &args].concat();
        let mut library = run(dir.path(), &library).ok();
        let mut project = run(dir.path(), &project).ok();
        let project_inputs = project.as_object_mut().unwrap().remove("inputs").unwrap();
        library.as_object_mut().unwrap().remove("inputs");
        assert_eq!(gai_core::to_js_json(&project), gai_core::to_js_json(&library), "{neighbors:?}");
        let references = project["references"].as_array().unwrap().len();
        assert_eq!(project_inputs.as_array().unwrap().len(), 3 + references);
    }
}

#[test]
fn built_references_are_read_within_a_players_archive_limits() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("app");
    std::fs::create_dir(&app).unwrap();
    web_app(&app);
    let library = fixture_library(&dir.path().join("library")).display().to_string();
    run(&app, &["init"]).ok();
    // A project controls its build folder: a Strong archive with more members than a
    // player reads, each pointing at the same tiny stream, and a set.json that pins it.
    let entries: Vec<gai_package::zip::ZipEntry> = (0..4097)
        .map(|i| gai_package::zip::ZipEntry::deflate(format!("a/{i}.json"), b"{}", gai_package::zip::ZipDate::SETS))
        .collect();
    let lottie = gai_package::zip::write_zip(&entries).unwrap();
    let build = ".gen-animated-icons/build/zz-bomb";
    let strong =
        serde_json::json!({"motionId": "strong", "name": "zz-bomb-strong", "sha256": gai_core::sha256_hex(&lottie)});
    let set = serde_json::json!({"id": "00-local", "icons": [{"name": "zz-bomb", "motions": [strong]}]});
    write(&app, &format!("{build}/set.json"), set.to_string());
    write(&app, &format!("{build}/zz-bomb-strong.json"), "{}");
    write(&app, &format!("{build}/zz-bomb-strong.lottie"), &lottie);

    let error =
        run(&app, &["context", "bell", "--neighbor", "zz-bomb", "--library", &library]).fails(6, "CORRUPT_ARCHIVE");
    let message = error["message"].as_str().unwrap();
    assert!(message.ends_with("archive exceeds the reader's limits: 4097 members, more than 4096"), "{message}");
}

#[test]
fn text_output_lists_candidates_and_references() {
    let dir = tempfile::tempdir().unwrap();
    let repo = repo_arg();
    let text = |args: &[&str]| {
        let run = run(dir.path(), &[&["--library-root", repo.as_str(), "--format", "text", "context"], args].concat());
        assert_eq!(run.code, 0, "{run:?}");
        run.stdout
    };
    // The candidate list reads like `search`: id, label, category and score.
    assert_eq!(
        text(&["--search", "--limit", "3", "notification bell"]),
        "notification-settings        Notification settings · Notifications  [19]\nbell-off                     Bell off · Notifications  [16]\nbell-ring                    Bell ring · Notifications  [16]\n"
    );
    assert_eq!(
        text(&["--search", "zzzz"]),
        "No library icons match \"zzzz\". Try a synonym or the underlying object.\n"
    );
    // The packet: its candidates, each reference's timing and what the JSON adds.
    let packet = text(&["notification feedback", "--neighbor", "bell"]);
    let lines: Vec<&str> = packet.lines().collect();
    assert_eq!(lines[0], "Context for \"notification feedback\" (Agent-selected neighbors)", "{packet}");
    for line in [
        "Contract: 32-unit grid, outline weights 1.5, 2, 2.5 (light, regular, bold; 1 to 3), 60 fps, 2.4 s timeline",
        "Candidates (of 1005 library icons):",
        "References:",
        "  bell                         Bell · Essentials, set 01-heart-bell",
        "    intent: The bell rings. Light: the bell swings out from its mount, the clapper trails it and strikes, a ring arc flicks out on each side and fades as it spreads, and the swing decays back to rest in smaller beats. Strong: a wider swing that rings twice, once at each end of the swing, and the clapper chases the bell through a longer decay. The mount never moves.",
        "    light: active 0.56 s, rest frame 37, states rest to rest",
        "    strong: active 0.76 s, rest frame 48, states rest to rest",
        "    initial outline geometry: 6 layers",
        "The whole packet (the contract, and each reference's initial outline geometry) is JSON: add --json.",
    ] {
        assert!(lines.contains(&line), "{line:?} in\n{packet}");
    }
    assert!(!packet.trim_start().starts_with('{'), "{packet}");
}
