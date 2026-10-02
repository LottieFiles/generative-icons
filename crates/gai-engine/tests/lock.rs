//! The sign-in lock held by another process: the wait is said and bounded.

mod common;

use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::{FakeBrowser, MockEngine};
use gai_engine::config::{LockWait, Notice};
use gai_engine::{
    Error, ErrorKind, LoginMode, LoginOptions, SessionState, check_session, login, logout,
};

/// Names the lock file the helper holds.
const HOLD_ENV: &str = "GAI_TEST_HOLD_SIGN_IN_LOCK";
/// What the helper prints once it holds the lock.
const HOLDING: &str = "HOLDING-THE-SIGN-IN-LOCK";

/// Run in a child process by [`Holder`]: holds the lock until its stdin closes.
#[test]
#[ignore = "helper: run in a child process by the tests in this file"]
fn hold_the_sign_in_lock() {
    let Some(path) = std::env::var_os(HOLD_ENV) else {
        return;
    };
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .unwrap();
    file.lock().unwrap();
    println!("{HOLDING}");
    std::io::stdout().flush().unwrap();
    let _ = std::io::stdin().read_to_end(&mut Vec::new());
}

/// Another process holding `<home>/refresh.lock`, until dropped.
struct Holder(Child);

impl Holder {
    fn start(home: &Path) -> Holder {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "hold_the_sign_in_lock",
                "--exact",
                "--ignored",
                "--nocapture",
            ])
            .env(HOLD_ENV, home.join("refresh.lock"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut out = BufReader::new(child.stdout.take().unwrap());
        let mut holder = Holder(child);
        loop {
            let mut line = String::new();
            if out.read_line(&mut line).unwrap() == 0 {
                drop(holder.0.stdin.take());
                panic!("the helper exited before it held the lock");
            }
            if line.contains(HOLDING) {
                return holder;
            }
        }
    }
}

impl Drop for Holder {
    fn drop(&mut self) {
        drop(self.0.stdin.take());
        let _ = self.0.wait();
    }
}

#[test]
fn another_process_holding_the_sign_in_lock_is_waited_for_then_reported() {
    let engine = MockEngine::start("{}");
    let home = tempfile::tempdir().unwrap();
    let mut config = engine.config(home.path());
    let options = LoginOptions {
        mode: LoginMode::Loopback,
        open_browser: false,
        timeout: Duration::from_secs(20),
    };
    login(
        &config,
        &options,
        &mut FakeBrowser::new(engine.state.clone(), false),
    )
    .unwrap();
    let heard = Arc::new(Mutex::new(Vec::new()));
    let sink = heard.clone();
    config.notices = Some(Arc::new(move |notice| sink.lock().unwrap().push(notice)));
    config.lock_wait = LockWait {
        notice_after: Duration::from_millis(100),
        give_up_after: Duration::from_millis(600),
    };

    let holder = Holder::start(home.path());
    let started = Instant::now();
    let err = check_session(&config, false).unwrap_err();
    assert!(started.elapsed() >= Duration::from_millis(600));
    assert!(matches!(err, Error::SignInBusy { .. }), "{err}");
    assert_eq!(err.kind(), ErrorKind::SignInBusy);
    assert_eq!(*heard.lock().unwrap(), [Notice::WaitingForSignInLock]);
    let err = logout(&config).unwrap_err();
    assert_eq!(err.code(), "SIGN_IN_BUSY");

    // Once that process ends, the same calls go through.
    drop(holder);
    heard.lock().unwrap().clear();
    let check = check_session(&config, false).unwrap();
    assert_eq!(check.state, SessionState::NotVerified);
    assert!(logout(&config).unwrap().was_signed_in);
    assert!(heard.lock().unwrap().is_empty());
}
