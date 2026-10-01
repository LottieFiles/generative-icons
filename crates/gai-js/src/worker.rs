//! Threads with stacks large enough for a [`SetRuntime`](crate::SetRuntime).
//!
//! QuickJS enforces its stack limit against the stack top rquickjs records on entry; it
//! does not know the thread's real stack. A Rust thread defaults to 2 MiB, so an 8 MiB
//! QuickJS limit would let deep recursion overflow the native stack, which aborts the
//! process instead of throwing a `RangeError`. Runtimes created elsewhere therefore get
//! at most [`UNSIZED_THREAD_STACK`] of QuickJS stack.

use crate::Limits;
use std::{cell::Cell, sync::Mutex, thread};

/// The QuickJS stack limit on threads not started here: the QuickJS default, which the
/// whole corpus fits in, with room to spare on a 2 MiB thread.
pub const UNSIZED_THREAD_STACK: usize = 1 << 20;

thread_local! {
    /// The native stack size of a thread started here.
    static WORKER_STACK: Cell<Option<usize>> = const { Cell::new(None) };
}

/// The QuickJS stack limit a runtime created on this thread can use safely.
pub(crate) fn usable_stack(limits: &Limits) -> usize {
    match WORKER_STACK.with(Cell::get) {
        Some(size) if size >= worker_stack_size(limits) => limits.stack,
        _ => limits.stack.min(UNSIZED_THREAD_STACK),
    }
}

fn spawn<'scope, T: Send + 'scope>(
    scope: &'scope thread::Scope<'scope, '_>,
    name: String,
    limits: &Limits,
    f: impl FnOnce() -> T + Send + 'scope,
) -> thread::ScopedJoinHandle<'scope, T> {
    let size = worker_stack_size(limits);
    thread::Builder::new()
        .name(name)
        .stack_size(size)
        .spawn_scoped(scope, move || {
            WORKER_STACK.with(|stack| stack.set(Some(size)));
            f()
        })
        .expect("spawn a gai-js worker")
}

/// The native stack a runtime's thread needs: the QuickJS limit plus 8 MiB for the Rust
/// and C frames around it.
pub fn worker_stack_size(limits: &Limits) -> usize {
    limits.stack + (8 << 20)
}

/// Runs `f` on a new thread sized for `limits` and returns its result. Panics in `f`
/// propagate.
pub fn run_on_worker<T: Send>(limits: &Limits, f: impl FnOnce() -> T + Send) -> T {
    thread::scope(|scope| {
        spawn(scope, "gai-js".into(), limits, f).join().unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
}

/// Applies `f` to every item on up to `workers` threads sized for `limits`, and returns the
/// results in input order. Items are taken in order, so listing slow items first shortens
/// the wall time. Each call to `f` should create (and drop) its own runtime.
pub fn map_parallel<I: Send, T: Send>(
    items: Vec<I>,
    workers: usize,
    limits: &Limits,
    f: impl Fn(I) -> T + Sync,
) -> Vec<T> {
    let count = items.len();
    let queue = Mutex::new(items.into_iter().enumerate());
    let results: Mutex<Vec<Option<T>>> = Mutex::new((0..count).map(|_| None).collect());
    thread::scope(|scope| {
        let handles: Vec<_> = (0..workers.clamp(1, count.max(1)))
            .map(|n| {
                spawn(scope, format!("gai-js-{n}"), limits, || {
                    loop {
                        let next = queue.lock().unwrap_or_else(|poison| poison.into_inner()).next();
                        let Some((index, item)) = next else { break };
                        let result = f(item);
                        results.lock().unwrap_or_else(|poison| poison.into_inner())[index] = Some(result);
                    }
                })
            })
            .collect();
        for handle in handles {
            if let Err(panic) = handle.join() {
                std::panic::resume_unwind(panic);
            }
        }
    });
    results
        .into_inner()
        .unwrap_or_else(|poison| poison.into_inner())
        .into_iter()
        .map(|result| result.expect("every item was processed"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_parallel_keeps_input_order() {
        let out = map_parallel((0..50).collect(), 4, &Limits::default(), |n: u64| n * n);
        assert_eq!(out, (0..50).map(|n| n * n).collect::<Vec<_>>());
        assert!(map_parallel(Vec::<u8>::new(), 4, &Limits::default(), |n| n).is_empty());
    }

    #[test]
    fn workers_get_room_for_the_quickjs_stack() {
        let limits = Limits::default();
        assert_eq!(worker_stack_size(&limits), 16 << 20);
        assert_eq!(run_on_worker(&limits, || usable_stack(&limits)), limits.stack);
        assert_eq!(usable_stack(&limits), UNSIZED_THREAD_STACK);
        let small = Limits { stack: 256 << 10, ..Limits::default() };
        assert_eq!(usable_stack(&small), 256 << 10);
    }
}
