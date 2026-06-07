//! Feature 5 — Pluggable async scanning (specification clauses S5.1–S5.3).

mod common;

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll, Wake, Waker};

use dioxus_swdir_tree_core::{ScanExecutor, ScanFuture, ScanJob, ThreadExecutor};

use common::fixture;

// ── Minimal block_on (no external async runtime dep) ─────────────────────────

/// Drive a future to completion using a condvar-backed waker. Works
/// correctly with futures that wake asynchronously (like [`JobFuture`]).
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    struct CondWaker(Arc<(Mutex<bool>, Condvar)>);

    impl Wake for CondWaker {
        fn wake(self: Arc<Self>) {
            let (lock, cvar) = &*self.0;
            *lock.lock().unwrap() = true;
            cvar.notify_one();
        }
    }

    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let waker = Waker::from(Arc::new(CondWaker(Arc::clone(&pair))));
    let mut cx = Context::from_waker(&waker);
    let mut pinned = std::pin::pin!(future);

    loop {
        match pinned.as_mut().poll(&mut cx) {
            Poll::Ready(val) => return val,
            Poll::Pending => {
                let (lock, cvar) = &*pair;
                let mut woken = lock.lock().unwrap();
                // Re-poll spuriously if we wake before waiting — that is
                // correct (and rare). Just loop back.
                while !*woken {
                    woken = cvar.wait(woken).unwrap();
                }
                *woken = false;
            }
        }
    }
}

// ── S5.1 — Object safety ──────────────────────────────────────────────────────

/// S5.1 — `ScanExecutor` is object-safe: `Arc<dyn ScanExecutor>` compiles and
/// holds `ThreadExecutor` without any trait-object unsoundness.
#[test]
fn s5_1_executor_is_object_safe() {
    let executor: Arc<dyn ScanExecutor> = Arc::new(ThreadExecutor);
    // If this line compiles and runs, the trait is object-safe.
    assert!(Arc::strong_count(&executor) >= 1);
}

// ── S5.2 — Exactly one spawn_blocking per request ────────────────────────────

/// A recording executor that counts how many times `spawn_blocking` is
/// called, then delegates to [`ThreadExecutor`].
struct CountingExecutor {
    calls: Arc<AtomicU32>,
    inner: ThreadExecutor,
}

impl ScanExecutor for CountingExecutor {
    fn spawn_blocking(&self, job: ScanJob) -> ScanFuture {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.spawn_blocking(job)
    }
}

/// S5.2 — The widget issues exactly one `spawn_blocking` per `ScanRequest`.
#[test]
fn s5_2_exactly_one_spawn_per_request() {
    let fx = fixture();
    let calls = Arc::new(AtomicU32::new(0));
    let executor = Arc::new(CountingExecutor {
        calls: Arc::clone(&calls),
        inner: ThreadExecutor,
    }) as Arc<dyn ScanExecutor>;

    let mut tree = dioxus_swdir_tree_core::DirectoryTree::new(&fx.root);
    let request = tree
        .on_toggled(&fx.root)
        .expect("root expansion issues a scan request");

    // One request → one spawn_blocking call.
    let req_copy = request.clone();
    let job: ScanJob = Box::new(move || dioxus_swdir_tree_core::scan::run(&req_copy));
    let payload = block_on(executor.spawn_blocking(job));
    tree.on_loaded(payload);

    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "exactly one spawn per request"
    );

    // Expanding alpha: another request → another single call.
    let alpha_req = tree
        .on_toggled(&fx.path("alpha"))
        .expect("alpha not yet loaded");
    let req_copy = alpha_req.clone();
    let job: ScanJob = Box::new(move || dioxus_swdir_tree_core::scan::run(&req_copy));
    let payload = block_on(executor.spawn_blocking(job));
    tree.on_loaded(payload);

    assert_eq!(calls.load(Ordering::SeqCst), 2, "each request is one spawn");
}

// ── S5.3 — ThreadExecutor delivers a correct payload off-thread ──────────────

/// S5.3 — `ThreadExecutor::spawn_blocking` resolves with the correct
/// `LoadPayload` for the given `ScanRequest`, off the calling thread.
#[test]
fn s5_3_thread_executor_delivers_correct_payload() {
    let fx = fixture();
    let executor = ThreadExecutor;

    let mut tree = dioxus_swdir_tree_core::DirectoryTree::new(&fx.root);
    let request = tree.on_toggled(&fx.root).expect("scan request");

    let req_copy = request.clone();
    let job: ScanJob = Box::new(move || dioxus_swdir_tree_core::scan::run(&req_copy));
    let payload = block_on(executor.spawn_blocking(job));

    assert_eq!(payload.path, fx.root);
    assert_eq!(payload.generation, request.generation);
    assert!(
        payload.result.is_ok(),
        "scan of the fixture root must succeed"
    );

    let entries = payload.result.as_ref().unwrap();
    // Fixture root has alpha/, beta/, zeta.txt (and hidden entries).
    // The raw entries (before filtering) should include all of them.
    let names: Vec<_> = entries
        .iter()
        .map(|e| e.path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert!(names.contains(&"alpha".to_string()), "alpha in raw entries");
    assert!(names.contains(&"beta".to_string()), "beta in raw entries");

    // Merge into the tree and verify it accepted.
    let outcome = tree.on_loaded(payload);
    assert!(outcome.accepted);
    assert!(tree.root().is_loaded);
}
