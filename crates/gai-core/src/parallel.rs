//! Order-preserving parallel maps over scoped threads on all available cores.

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

fn workers(count: usize) -> usize {
    std::thread::available_parallelism().map_or(1, usize::from).min(count)
}

/// Applies `f` to every item and returns the results in input order.
pub fn parallel_map<T, R, F>(items: Vec<T>, f: F) -> Vec<R>
where
    T: Send,
    R: Send,
    F: Fn(T) -> R + Sync,
{
    if workers(items.len()) <= 1 {
        return items.into_iter().map(f).collect();
    }
    let queue: Vec<Mutex<Option<T>>> = items.into_iter().map(|item| Mutex::new(Some(item))).collect();
    let take = |i: usize| queue[i].lock().unwrap_or_else(|e| e.into_inner()).take().expect("each item is taken once");
    match ordered_map(queue.len(), |i| Ok::<R, std::convert::Infallible>(f(take(i)))) {
        Ok(results) => results,
        Err(never) => match never {},
    }
}

/// Maps `job` over `0..count`. On failure, returns the error of the lowest failing index,
/// as a sequential run would; jobs after it are skipped once it is known.
pub fn ordered_map<T: Send, E: Send>(count: usize, job: impl Fn(usize) -> Result<T, E> + Sync) -> Result<Vec<T>, E> {
    let workers = workers(count);
    if workers <= 1 {
        return (0..count).map(job).collect();
    }
    let next = AtomicUsize::new(0);
    let first_error = AtomicUsize::new(usize::MAX);
    let slots: Mutex<Vec<Option<Result<T, E>>>> = Mutex::new((0..count).map(|_| None).collect());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    if i >= count || i > first_error.load(Ordering::Relaxed) {
                        break;
                    }
                    let result = job(i);
                    if result.is_err() {
                        first_error.fetch_min(i, Ordering::Relaxed);
                    }
                    slots.lock().unwrap_or_else(|e| e.into_inner())[i] = Some(result);
                }
            });
        }
    });
    // Every index below the first error ran; indices after it may not have.
    slots.into_inner().unwrap_or_else(|e| e.into_inner()).into_iter().map_while(|slot| slot).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_order_and_reports_the_first_error() {
        let squares = ordered_map(100, |i| Ok::<_, usize>(i * i)).unwrap();
        assert_eq!(squares, (0..100).map(|i| i * i).collect::<Vec<_>>());
        assert_eq!(ordered_map(100, |i| if i % 30 == 29 { Err(i) } else { Ok(i) }), Err(29));
        assert_eq!(ordered_map(0, Ok::<_, ()>), Ok(vec![]));
    }

    #[test]
    fn parallel_map_keeps_order() {
        let out = parallel_map((0..1000).collect(), |x: u64| x * x);
        assert_eq!(out, (0..1000).map(|x| x * x).collect::<Vec<_>>());
        assert_eq!(parallel_map(Vec::<u8>::new(), |x| x), Vec::<u8>::new());
    }
}
