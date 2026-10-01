//! The watchdog: a call stuck past its deadline in code that never polls ends the process.
//! That would end the test process too, so each stuck case runs in a child: this test
//! binary run again with `GAI_JS_WATCHDOG_CHILD` naming the case.

use gai_js::{Error, Limits, SetRuntime, SetSource, run_on_worker, watchdog};
use std::{
    fs,
    process::Command,
    time::{Duration, Instant},
};
use tempfile::TempDir;

const CHILD: &str = "GAI_JS_WATCHDOG_CHILD";
const STUCK_STATUS: i32 = 86;

/// A set whose only icon's `d` never returns.
fn set() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let definitions = "export const icons = [{ name: 'hang', label: 'hang', heading: 'hang', description: 'hang', intent: 'hang',
      timings: { light: { active: .5, rest: .54, keyPoses: [0, .5] }, strong: { active: .5, rest: .54, keyPoses: [0, .5] } },
      parts: [{ name: 'hang-part', style: 'solid', d: t => { for (;;) {} } }] }];";
    let path = dir.path().join("99-test/definitions.mjs");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, definitions).unwrap();
    fs::create_dir_all(dir.path().join("library")).unwrap();
    dir
}

/// Code the interrupt handler cannot stop: a Rust closure inside a call.
fn stuck(runtime: &SetRuntime) -> gai_js::Result<()> {
    runtime.with_icons(|_| {
        std::thread::sleep(Duration::from_secs(60));
        Ok(())
    })
}

/// Runs `case` in a child and returns its exit status, stdout and run time.
fn run_child(case: &str) -> (Option<i32>, String, Duration) {
    let started = Instant::now();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["child", "--exact", "--nocapture", "--test-threads=1"])
        .env(CHILD, case)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    (output.status.code(), stdout, started.elapsed())
}

/// The child: does nothing unless `GAI_JS_WATCHDOG_CHILD` names a case.
#[test]
fn child() {
    let Ok(case) = std::env::var(CHILD) else { return };
    let blocked = case == "blocked";
    watchdog::install(move |error: Error| {
        println!("STUCK {error}");
        if blocked {
            // Reporting that never finishes, as on a pipe nobody reads.
            std::thread::sleep(Duration::from_secs(60));
        }
        STUCK_STATUS as u8
    })
    .unwrap();
    assert!(watchdog::installed());
    let dir = set();
    let source = SetSource::library_set(dir.path().join("99-test")).unwrap();
    let millis = Duration::from_millis;
    let limits = match case.as_str() {
        "timeout" | "blocked" => Limits { call_timeout: millis(200), grace: millis(300), ..Limits::default() },
        "budget" => Limits { call_timeout: millis(5000), budget: millis(200), grace: millis(300), ..Limits::default() },
        other => panic!("unknown case {other}"),
    };
    run_on_worker(&limits.clone(), move || {
        let runtime = SetRuntime::load(&source, &limits).unwrap();
        if case == "timeout" {
            // Calls that stop at their deadline, or finish, return as usual.
            let request = gai_js::ProbeRequest {
                icon: 0,
                part: 0,
                strong: false,
                reverse: false,
                width: gai_js::ProbeWidth::Authored,
                times: &[0.0],
            };
            println!("SOFT {}", runtime.probe(&request).unwrap_err());
            runtime.with_icons(|_| Ok(())).unwrap();
            println!("DONE");
        }
        let _ = stuck(&runtime);
    });
    println!("NOT STOPPED");
}

#[test]
fn a_call_stuck_past_its_deadline_ends_the_process() {
    let (status, stdout, elapsed) = run_child("timeout");
    assert_eq!(status, Some(STUCK_STATUS), "{stdout}");
    let soft = stdout.find("SOFT 99-test: stopped after 200ms").expect(&stdout);
    let done = stdout.find("DONE").expect(&stdout);
    let stopped = stdout.find("STUCK 99-test: stopped after 200ms").expect(&stdout);
    assert!(soft < done && done < stopped && !stdout.contains("NOT STOPPED"), "{stdout}");
    // 200 ms + 300 ms of grace, not the stuck call's 60 s.
    assert!(elapsed < Duration::from_secs(20), "{elapsed:?}");
}

#[test]
fn a_call_stuck_past_the_sets_budget_reports_the_budget() {
    let (status, stdout, elapsed) = run_child("budget");
    assert_eq!(status, Some(STUCK_STATUS), "{stdout}");
    assert!(stdout.contains("STUCK 99-test: stopped when the set's calls had used their 200ms budget"), "{stdout}");
    assert!(elapsed < Duration::from_secs(20), "{elapsed:?}");
}

#[test]
fn the_process_ends_even_when_reporting_blocks() {
    let (status, stdout, elapsed) = run_child("blocked");
    assert_eq!(status, Some(watchdog::UNREPORTED.into()), "{stdout}");
    assert!(stdout.contains("STUCK 99-test: stopped after 200ms") && !stdout.contains("NOT STOPPED"), "{stdout}");
    // 200 ms + 300 ms of grace + REPORT_TIMEOUT, not the report's 60 s.
    assert!(watchdog::REPORT_TIMEOUT < Duration::from_secs(10));
    assert!(elapsed < Duration::from_secs(20), "{elapsed:?}");
}

#[test]
fn without_the_watchdog_native_code_runs_to_completion() {
    let dir = set();
    let source = SetSource::library_set(dir.path().join("99-test")).unwrap();
    let limits =
        Limits { call_timeout: Duration::from_millis(50), grace: Duration::from_millis(50), ..Limits::default() };
    let started = Instant::now();
    run_on_worker(&limits.clone(), move || {
        let runtime = SetRuntime::load(&source, &limits).unwrap();
        runtime
            .with_icons(|_| {
                std::thread::sleep(Duration::from_millis(300));
                Ok(())
            })
            .unwrap();
    });
    assert!(!watchdog::installed());
    assert!(started.elapsed() >= Duration::from_millis(300));
}
