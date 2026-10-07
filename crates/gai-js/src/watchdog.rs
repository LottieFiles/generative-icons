//! Deadlines for calls into a set: a prompt to poll at the deadline, and a hard stop past it.
//!
//! QuickJS stops a call at its deadline by polling an interrupt handler (see the crate
//! docs) every so many steps, and a builtin call counts as one step, however long it runs.
//! So a timer thread asks each call's runtime to poll as soon as its deadline passes
//! (`JS_RequestInterrupt`, from the patched engine): a loop of slow builtins then stops
//! within one builtin call, not thousands. This runs in every process.
//!
//! Code that never polls runs on past the deadline: libbf's BigInt arithmetic (`toString`
//! of a 10^8-bit BigInt takes seconds, of a 10^9-bit one minutes), a Rust closure passed
//! to [`SetRuntime::with_icons`](crate::SetRuntime::with_icons), or a builtin nobody has
//! found yet. Once [`install`] has run, the same thread ends the process when a call runs
//! [`Limits::grace`](crate::Limits::grace) past its deadline. It hands `on_stuck` the error
//! the call would have returned ([`Error::Timeout`], or [`Error::Budget`] when the set's
//! budget set the deadline), flushes stdout and stderr, then exits with the status
//! `on_stuck` returns. If that report panics, or has not finished after [`REPORT_TIMEOUT`]
//! (a pipe nobody reads, a lock the stuck call holds), the process exits with
//! [`UNREPORTED`] instead. Without `install`, such a call runs to completion.
//!
//! The watchdog ends the whole process, so install it only in a process that runs one
//! job, such as a CLI command. A server stays up by running each job in a child process
//! that installs it, as `generative-icons mcp` runs each tool call as a one-shot
//! command of the same binary.

use crate::Error;
use rquickjs::{Ctx, qjs};
use std::{
    io::{self, Write},
    panic::{self, AssertUnwindSafe},
    ptr::NonNull,
    sync::{
        Arc, Condvar, Mutex, MutexGuard, OnceLock, PoisonError,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

/// How long reporting a stuck call (`on_stuck`, then flushing stdout and stderr) may take
/// before the process exits without it.
pub const REPORT_TIMEOUT: Duration = Duration::from_secs(2);

/// The exit status when `on_stuck` panics or does not return within [`REPORT_TIMEOUT`]
/// (`EX_SOFTWARE`).
pub const UNREPORTED: u8 = 70;

type OnStuck = dyn Fn(Error) -> u8 + Send + Sync;

static INSTALLED: AtomicBool = AtomicBool::new(false);
static ON_STUCK: Mutex<Option<Arc<OnStuck>>> = Mutex::new(None);
static STARTED: OnceLock<io::Result<()>> = OnceLock::new();
static CALLS: Mutex<Calls> = Mutex::new(Calls { next: 0, open: Vec::new(), wake_at: None });
static WAKE: Condvar = Condvar::new();

/// Ends this process when any call into a set runs [`Limits::grace`](crate::Limits::grace)
/// past its deadline: `on_stuck` reports the call's error (it runs on the watchdog thread,
/// while the stuck call still runs) and returns the exit status, or the process exits with
/// [`UNREPORTED`] once it has taken [`REPORT_TIMEOUT`]. Calls that start after this are
/// watched. Calling it again replaces `on_stuck`.
///
/// Fails only if the watchdog thread cannot be started; nothing is watched then.
pub fn install(on_stuck: impl Fn(Error) -> u8 + Send + Sync + 'static) -> io::Result<()> {
    *lock(&ON_STUCK) = Some(Arc::new(on_stuck));
    if let Err(error) = start() {
        return Err(io::Error::new(error.kind(), format!("cannot start the gai-js watchdog: {error}")));
    }
    INSTALLED.store(true, Ordering::Release);
    Ok(())
}

/// Whether [`install`] has run, so calls are watched.
pub fn installed() -> bool {
    INSTALLED.load(Ordering::Acquire)
}

/// Starts the timer thread once.
fn start() -> &'static io::Result<()> {
    STARTED.get_or_init(|| thread::Builder::new().name("gai-js-watchdog".into()).spawn(run).map(drop))
}

/// The runtime of a call, which the timer thread asks to poll its interrupt handler.
#[derive(Clone, Copy)]
pub(crate) struct Runtime(NonNull<qjs::JSRuntime>);

// SAFETY: the pointer only reaches `JS_RequestInterrupt`, which any thread may call while
// the runtime is alive, and only while its call is open: the call's `Watch` removes it
// (under `CALLS`, which the timer thread holds while it uses it) before the call returns,
// and a runtime outlives its calls.
unsafe impl Send for Runtime {}

impl Runtime {
    pub(crate) fn of(ctx: &Ctx<'_>) -> Self {
        // SAFETY: `ctx` is a live context, and a context always has a runtime.
        let runtime = unsafe { qjs::JS_GetRuntime(ctx.as_raw().as_ptr()) };
        Runtime(NonNull::new(runtime).expect("a context has a runtime"))
    }

    /// Makes the runtime's next poll call its interrupt handler.
    fn interrupt(self) {
        unsafe extern "C" {
            fn JS_RequestInterrupt(runtime: *mut qjs::JSRuntime);
        }
        // SAFETY: see `impl Send`; the function only sets an atomic flag.
        unsafe { JS_RequestInterrupt(self.0.as_ptr()) }
    }
}

/// One watched call; dropping it ends the watch.
pub(crate) struct Watch(Option<u64>);

/// Watches a call on `runtime` that must return by `deadline`. Once the deadline passes,
/// the runtime is asked to poll its interrupt handler. If the watchdog is installed and the
/// call is still open `grace` later, `stuck` builds the error to report and the process
/// ends.
pub(crate) fn watch(runtime: Runtime, deadline: Instant, grace: Duration, stuck: impl FnOnce() -> Error) -> Watch {
    // Without the thread, calls stop when the interrupt handler's own polling finds them late.
    if start().is_err() {
        return Watch(None);
    }
    let until = deadline.checked_add(grace).filter(|_| installed());
    let open = Open { id: 0, runtime, nudge: Some(deadline), until, error: until.map(|_| stuck()) };
    let mut calls = lock(&CALLS);
    let id = calls.next;
    calls.next += 1;
    let at = open.next();
    calls.open.push(Open { id, ..open });
    // The thread sleeps until `wake_at`, so only an earlier time needs to wake it.
    if at.is_some_and(|at| calls.wake_at.is_none_or(|wake_at| at < wake_at)) {
        WAKE.notify_one();
    }
    Watch(Some(id))
}

impl Drop for Watch {
    fn drop(&mut self) {
        if let Some(id) = self.0 {
            lock(&CALLS).open.retain(|open| open.id != id);
        }
    }
}

struct Calls {
    next: u64,
    open: Vec<Open>,
    /// When the timer thread wakes next; `None` while it waits for a call or runs.
    wake_at: Option<Instant>,
}

struct Open {
    id: u64,
    runtime: Runtime,
    /// The deadline, until the runtime has been asked to poll.
    nudge: Option<Instant>,
    /// When the process ends if the call is still open; `None` unless installed.
    until: Option<Instant>,
    /// The error `on_stuck` reports, if `until` passes.
    error: Option<Error>,
}

impl Open {
    fn next(&self) -> Option<Instant> {
        self.nudge.into_iter().chain(self.until).min()
    }
}

/// The timer thread: asks each call's runtime to poll once its deadline has passed, and
/// stops the process once a watched call is still open past its grace.
fn run() {
    let mut calls = lock(&CALLS);
    loop {
        let now = Instant::now();
        // Strictly past: the interrupt handler finds a call late only after its deadline.
        for open in &mut calls.open {
            if open.nudge.is_some_and(|deadline| deadline < now) {
                open.nudge = None;
                open.runtime.interrupt();
            }
        }
        if let Some(i) = calls.open.iter().position(|open| open.until.is_some_and(|until| until <= now)) {
            let open = calls.open.swap_remove(i);
            drop(calls);
            stop(open.error.expect("a watched call has its error"))
        }
        let next = calls.open.iter().filter_map(Open::next).min();
        calls.wake_at = next;
        calls = match next {
            None => WAKE.wait(calls).unwrap_or_else(PoisonError::into_inner),
            Some(at) => {
                WAKE.wait_timeout(calls, at.saturating_duration_since(now)).unwrap_or_else(PoisonError::into_inner).0
            }
        };
        calls.wake_at = None;
    }
}

fn stop(error: Error) -> ! {
    // The report can block; the process ends regardless. If this thread cannot start,
    // the report runs unguarded.
    let _ = thread::Builder::new().name("gai-js-watchdog-exit".into()).spawn(|| {
        thread::sleep(REPORT_TIMEOUT);
        exit(UNREPORTED)
    });
    let on_stuck = lock(&ON_STUCK).clone();
    let status = on_stuck.and_then(|on_stuck| panic::catch_unwind(AssertUnwindSafe(|| on_stuck(error))).ok());
    let _ = io::stdout().flush();
    let _ = io::stderr().flush();
    exit(status.unwrap_or(UNREPORTED))
}

/// Exits without running atexit handlers or C++ static destructors, which other threads
/// (the stuck call among them) may still be using.
#[cfg(unix)]
fn exit(status: u8) -> ! {
    unsafe extern "C" {
        safe fn _exit(status: std::ffi::c_int) -> !;
    }
    _exit(status.into())
}

#[cfg(not(unix))]
fn exit(status: u8) -> ! {
    std::process::exit(status.into())
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
