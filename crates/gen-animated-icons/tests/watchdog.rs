//! One-shot commands install gai-js's watchdog, so icon code stuck in native code that
//! never checks its time limit ends the command with `SOURCE_ERROR` instead of hanging.
//! That ends the process, so the stuck case runs in a child: this test binary run again
//! with `GAI_CLI_WATCHDOG_CHILD` set.

use gai_js::{Limits, SetRuntime, SetSource, run_on_worker};
use serde_json::Value;
use std::process::Command;
use std::time::{Duration, Instant};

const CHILD: &str = "GAI_CLI_WATCHDOG_CHILD";

/// The child: runs a command line (which installs the watchdog), then a call stuck in
/// native code past its 200 ms limit.
#[test]
fn child() {
    if std::env::var_os(CHILD).is_none() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let set = dir.path().join("99-test");
    std::fs::create_dir_all(&set).unwrap();
    std::fs::create_dir_all(dir.path().join("library")).unwrap();
    std::fs::write(set.join("definitions.mjs"), "export const icons = [];\n").unwrap();
    // Any one-shot command protects the process, whatever it then does.
    let cwd = dir.path().display().to_string();
    let args = ["gen-animated-icons", "--json", "--cwd", cwd.as_str(), "diff"];
    let status = gen_animated_icons::run(args.map(std::ffi::OsString::from));
    println!("COMMAND {status}");
    assert!(gai_js::watchdog::installed());
    let limits =
        Limits { call_timeout: Duration::from_millis(200), grace: Duration::from_millis(300), ..Limits::default() };
    let source = SetSource::library_set(&set).unwrap();
    run_on_worker(&limits.clone(), move || {
        let runtime = SetRuntime::load(&source, &limits).unwrap();
        let _ = runtime.with_icons(|_| {
            std::thread::sleep(Duration::from_secs(60));
            Ok(())
        });
    });
    println!("NOT STOPPED");
}

#[test]
fn icon_code_stuck_in_native_code_ends_the_command_with_a_source_error() {
    let started = Instant::now();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["child", "--exact", "--nocapture", "--test-threads=1"])
        .env(CHILD, "1")
        .output()
        .unwrap();
    let (stdout, stderr) = (String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(stdout.contains("COMMAND 0") && !stdout.contains("NOT STOPPED"), "{stdout}\n{stderr}");
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    let line = stderr.lines().rev().find(|line| line.starts_with("{\"error\"")).expect(&stderr);
    let error: Value = serde_json::from_str(line).unwrap();
    assert_eq!(error["error"]["code"], "SOURCE_ERROR", "{line}");
    assert_eq!(error["error"]["message"], "99-test: stopped after 200ms", "{line}");
    assert!(error["error"]["hint"].as_str().unwrap().starts_with("The icon code ran too long"), "{line}");
    // 200 ms plus 300 ms of grace, not the stuck call's 60 s.
    assert!(started.elapsed() < Duration::from_secs(30), "{:?}", started.elapsed());
}
