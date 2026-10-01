//! Helpers shared by the integration tests: running the binary, project and library
//! fixtures, and a tiny HTTP server.

// Each test binary uses a different subset.
#![allow(dead_code)]

use gen_animated_icons::weight::{Entry, write_zip};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Version of the fixture library.
pub const FIXTURE_VERSION: &str = "7.1.0-fixture";

/// The repository root (two levels above this crate).
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

/// The built `@lottiefiles/generative-icons` dist in this checkout, when present.
pub fn packages_dist() -> Option<PathBuf> {
    let dist = repo_root().join("packages/icons/dist");
    dist.join("catalog.json").is_file().then_some(dist)
}

/// Skips an oracle test whose inputs are missing.
#[macro_export]
macro_rules! require {
    ($value:expr, $what:literal) => {
        match $value {
            Some(value) => value,
            None => {
                eprintln!("skipped: {} not found", $what);
                return;
            }
        }
    };
}

/// A finished run of the binary.
#[derive(Debug)]
pub struct Run {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Run {
    /// Parsed stdout.
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.stdout).unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {self:?}"))
    }

    /// Parsed `error` object from stderr.
    pub fn error(&self) -> Value {
        let line = self.stderr.lines().last().unwrap_or_default();
        let value: Value = serde_json::from_str(line).unwrap_or_else(|e| panic!("stderr is not JSON ({e}): {self:?}"));
        value["error"].clone()
    }

    /// Asserts success and returns parsed stdout.
    pub fn ok(&self) -> Value {
        assert_eq!(self.code, 0, "{self:?}");
        self.json()
    }

    /// Asserts a failure with this exit status and error code.
    pub fn fails(&self, code: i32, error: &str) -> Value {
        assert_eq!(self.code, code, "{self:?}");
        let value = self.error();
        assert_eq!(value["code"], error, "{self:?}");
        value
    }
}

/// Runs the binary in `dir` with an isolated cache and credentials folder, and no
/// inherited library or engine settings.
pub fn run(dir: &Path, args: &[&str]) -> Run {
    run_with(dir, args, &[])
}

/// The binary, isolated like [`run`] does, to add arguments to.
pub fn command(dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_gen-animated-icons"));
    command
        .current_dir(dir)
        .env("GEN_ANIMATED_ICONS_CACHE_DIR", dir.join(".test-cache"))
        .env("GEN_ANIMATED_ICONS_HOME", dir.join(".test-home"))
        .env("NO_COLOR", "1");
    for name in [
        "GEN_ANIMATED_ICONS_LIBRARY",
        "GAI_LIBRARY_ROOT",
        "GAI_ENGINE_URL",
        "GAI_ENGINE_TOKEN",
        "GEN_ANIMATED_ICONS_ENGINE_URL",
        "CI",
    ] {
        command.env_remove(name);
    }
    command
}

/// Like [`run`], with extra environment variables.
pub fn run_with(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> Run {
    let mut command = command(dir);
    command.args(args);
    for (key, value) in env {
        command.env(key, value);
    }
    let output = command.output().expect("binary runs");
    Run {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

/// Writes `content` to `root/relative`, creating folders.
pub fn write(root: &Path, relative: &str, content: impl AsRef<[u8]>) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// Reads `root/relative` as text.
pub fn read(root: &Path, relative: &str) -> String {
    std::fs::read_to_string(root.join(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"))
}

/// sha256 of `root/relative`.
pub fn sha(root: &Path, relative: &str) -> String {
    gai_core::sha256_hex(std::fs::read(root.join(relative)).unwrap_or_else(|e| panic!("{relative}: {e}")))
}

/// A React app, as in the legacy CLI goldens.
pub fn web_app(root: &Path) {
    write(root, "package.json", r#"{"name":"consumer","private":true,"dependencies":{"react":"^19"}}"#);
    std::fs::create_dir_all(root.join("src")).unwrap();
}

/// A SwiftUI app: `Demo.xcodeproj` beside its `Demo` target folder.
pub fn swiftui_app(root: &Path) {
    write(root, "Demo.xcodeproj/project.pbxproj", "// !$*UTF8*$!\n{}\n");
    write(
        root,
        "Demo/DemoApp.swift",
        "import SwiftUI\n\n@main\nstruct DemoApp: App {\n  var body: some Scene { WindowGroup { ContentView() } }\n}\n",
    );
    write(
        root,
        "Demo/ContentView.swift",
        "import SwiftUI\n\nstruct ContentView: View {\n  var body: some View { Text(\"Hi\") }\n}\n",
    );
}

/// A Compose app with one application module, `app`.
pub fn android_app(root: &Path) {
    write(root, "settings.gradle.kts", "rootProject.name = \"Demo\"\ninclude(\":app\")\n");
    write(root, "build.gradle.kts", "plugins { alias(libs.plugins.android.application) apply false }\n");
    write(
        root,
        "app/build.gradle.kts",
        "plugins { alias(libs.plugins.android.application)\n  alias(libs.plugins.kotlin.compose) }\nandroid { buildFeatures { compose = true } }\n",
    );
    write(
        root,
        "app/src/main/AndroidManifest.xml",
        "<manifest xmlns:android=\"http://schemas.android.com/apk/res/android\" />\n",
    );
}

/// A minimal `.lottie` whose Regular stroke carries the stroke-width token.
pub fn fixture_lottie(actions: &[&str]) -> Vec<u8> {
    let animation = |name: &str| {
        json!({"nm": name, "w": 32, "h": 32, "layers": [{"ty": 4, "shapes": [{"ty": "gr", "it": [
            {"ty": "sh", "ks": {"a": 0, "k": {"c": false, "v": [[8, 16], [24, 16]], "i": [[0, 0], [0, 0]], "o": [[0, 0], [0, 0]]}}},
            {"ty": "st", "w": {"a": 0, "k": 2, "sid": "style.outline.width"}, "lc": 2, "lj": 2}
        ]}]}], "slots": {"style.outline.width": {"p": {"a": 0, "k": 2}}}})
    };
    let manifest = json!({"version": "2", "initial": {"animation": actions[actions.len() - 1]},
        "animations": actions.iter().map(|a| json!({"id": a})).collect::<Vec<_>>()});
    let mut entries = vec![Entry { name: "manifest.json".into(), data: manifest.to_string().into_bytes() }];
    for action in actions {
        entries.push(Entry { name: format!("a/{action}.json"), data: animation(action).to_string().into_bytes() });
    }
    entries.push(Entry {
        name: "t/light.json".into(),
        data: br#"{"rules":[{"id":"style.outline.width","type":"Scalar","value":2}]}"#.to_vec(),
    });
    write_zip(&entries)
}

fn motion(name: &str, motion_id: &str, stateful: bool) -> Value {
    let mut action = json!({"direction": "forward", "startState": "rest", "endState": "rest", "name": name,
        "motionId": motion_id, "fps": 60, "frames": 144, "active": 0.42, "rest": 31.200000000000003});
    if stateful {
        action["startState"] = json!("prepared");
        action["endState"] = json!("result");
        action["reverse"] = json!({"direction": "reverse", "startState": "result", "endState": "prepared",
            "name": format!("{name}-reset"), "motionId": motion_id, "fps": 60, "frames": 144, "active": 0.3, "rest": 21});
    }
    action
}

/// Writes a small library dist (catalog plus assets) into `dir` and returns it.
pub fn fixture_library(dir: &Path) -> PathBuf {
    let icons = [
        ("heart", "Heart", "Essentials", "The heart gives one short acknowledgement beat.", false),
        ("bell-ring", "Bell ring", "Notifications", "The bell swings and rings once.", false),
        ("menu", "Menu", "Interface", "Three lines slide into a compact stack.", false),
        ("date-range", "Date range", "Scheduling & time tools", "A range bar sweeps to the end date and stays.", true),
    ];
    let mut descriptors = Vec::new();
    for (id, label, category, intent, stateful) in icons {
        let strong = format!("{id}-strong");
        let actions: Vec<String> = if stateful {
            vec![id.into(), format!("{id}-reset"), strong.clone(), format!("{strong}-reset")]
        } else {
            vec![id.into(), strong.clone()]
        };
        let bytes = fixture_lottie(&actions.iter().map(String::as_str).collect::<Vec<_>>());
        write(dir, &format!("assets/{id}.lottie"), &bytes);
        descriptors.push(json!({
            "schemaVersion": 1, "id": id, "name": id, "label": label, "intent": intent,
            "category": category, "categoryId": category.to_lowercase().replace([' ', '&'], "-"),
            "family": "generative-icons", "stateful": stateful, "previewReset": stateful,
            "startState": if stateful { "prepared" } else { "rest" }, "endState": if stateful { "result" } else { "rest" },
            "width": 32, "height": 32, "styles": ["outline", "solid", "duotone"], "weights": [1.5, 2, 2.5],
            "motions": [motion(id, "light", stateful), motion(&strong, "strong", stateful)],
            "license": "MIT", "aliases": [], "source": {"set": "99-fixture", "manifest": "99/assets/set.json", "sha256": "0".repeat(64)},
            "asset": {"path": format!("assets/{id}.lottie"), "sha256": gai_core::sha256_hex(&bytes), "bytes": bytes.len()}
        }));
    }
    let catalog = json!({"schemaVersion": 1, "version": FIXTURE_VERSION,
        "categories": [{"id": "essentials", "label": "Essentials"}], "icons": descriptors});
    write(dir, "catalog.json", format!("{}\n", serde_json::to_string_pretty(&catalog).unwrap()));
    dir.to_path_buf()
}

/// Serves files under `root` over HTTP on 127.0.0.1 until the test process exits.
/// Returns the base URL.
pub fn serve(root: PathBuf) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}/", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request = String::new();
            if reader.read_line(&mut request).is_err() {
                continue;
            }
            let mut header = String::new();
            while reader.read_line(&mut header).is_ok_and(|n| n > 2) {
                header.clear();
            }
            let path = request.split_whitespace().nth(1).unwrap_or("/").trim_start_matches('/').to_string();
            let mut stream = stream;
            let response = match std::fs::read(root.join(&path)) {
                Ok(body) if !path.contains("..") => {
                    let mut head =
                        format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len())
                            .into_bytes();
                    head.extend(body);
                    head
                }
                _ => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(),
            };
            let _ = stream.write_all(&response);
        }
    });
    base
}

/// Copies the library checkout at `repo` into `dest` for tests that write set outputs:
/// everything but VCS data, build output and other crates, with only `sets` (all when
/// `None`) among `motion/icon-sets/NN-*` and `prototype/icons/NN`; the other asset
/// folders keep just their `set.json`, which catalog readers need. Files are cloned
/// where the file system supports it.
pub fn copy_checkout(repo: &Path, dest: &Path, sets: Option<&[&str]>) {
    let wanted = |set: &str| sets.is_none_or(|sets| sets.contains(&set));
    let numbers: Option<Vec<&str>> = sets.map(|sets| sets.iter().map(|set| &set[..2]).collect());
    let keep = |relative: &Path| -> bool {
        let parts: Vec<String> = relative.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
        let part = |i: usize| parts.get(i).map(String::as_str);
        if parts.iter().any(|p| p == "node_modules") {
            return false;
        }
        match (part(0), part(1), part(2), part(3), part(4)) {
            (Some(".git" | "target" | "crates" | "vendor" | ".local" | "dist" | "npm"), ..) => false,
            (Some("packages"), Some(_), Some("dist"), ..) => false,
            (Some("motion"), Some("icon-sets"), Some(set), ..) if set.as_bytes()[0].is_ascii_digit() => wanted(set),
            (Some("prototype"), Some("icons"), Some(number), Some("assets"), file) if number.len() == 2 => {
                numbers.as_ref().is_none_or(|numbers| numbers.contains(&number)) || file.is_none_or(|f| f == "set.json")
            }
            _ => true,
        }
    };
    copy_tree(repo, dest, Path::new(""), &keep);
}

fn copy_tree(from: &Path, to: &Path, relative: &Path, keep: &dyn Fn(&Path) -> bool) {
    std::fs::create_dir_all(to.join(relative)).unwrap();
    for entry in std::fs::read_dir(from.join(relative)).unwrap().flatten() {
        let child = relative.join(entry.file_name());
        if !keep(&child) {
            continue;
        }
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            copy_tree(from, to, &child, keep);
        } else if kind.is_symlink() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(std::fs::read_link(entry.path()).unwrap(), to.join(&child)).unwrap();
        } else {
            std::fs::copy(entry.path(), to.join(&child)).unwrap();
        }
    }
}

/// Every file below `root`, relative and sorted.
pub fn files_below(root: &Path) -> Vec<PathBuf> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                out.push(path.strip_prefix(root).unwrap().to_path_buf());
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}
