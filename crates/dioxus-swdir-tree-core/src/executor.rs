//! The async executor seam: who runs a [`crate::scan::ScanRequest`] and
//! how the result re-enters the reactive cycle.
//!
//! [`DirectoryTree`][crate::DirectoryTree] transitions stay pure — they
//! never spawn tasks, never touch a runtime. When a scan is needed,
//! [`DirectoryTree::on_toggled`][crate::DirectoryTree::on_toggled]
//! returns a [`crate::scan::ScanRequest`] as data. The embedding layer
//! (a Dioxus coroutine, the [`ThreadExecutor`] default, or a test)
//! passes that request to an executor and feeds the resulting
//! [`crate::scan::LoadPayload`] back through
//! [`DirectoryTree::on_loaded`][crate::DirectoryTree::on_loaded].
//!
//! # Object safety and pluggability (S5.1)
//!
//! `ScanExecutor` is object-safe and usable behind `Arc<dyn ScanExecutor>`.
//! High-throughput applications (heavy prefetch, RFC 009) can plug in a
//! `tokio::task::spawn_blocking`-based or rayon-based executor without
//! touching the tree state machine.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use crate::scan::LoadPayload;

// ── Public types ──────────────────────────────────────────────────────────────

/// A heap-allocated blocking job: no arguments, returns a [`LoadPayload`].
pub type ScanJob = Box<dyn FnOnce() -> LoadPayload + Send>;

/// The future returned by [`ScanExecutor::spawn_blocking`].
pub type ScanFuture = Pin<Box<dyn Future<Output = LoadPayload> + Send + 'static>>;

/// The executor seam — pluggable off-thread scan execution (S5.1).
///
/// Implement this trait to redirect scan work to a custom thread pool,
/// `tokio::task::spawn_blocking`, smol, or any other executor. The
/// default is [`ThreadExecutor`].
///
/// # Object safety
///
/// The trait is object-safe; wrap in `Arc<dyn ScanExecutor>` to store
/// it alongside a `Signal<DirectoryTree>`.
pub trait ScanExecutor: Send + Sync {
    /// Schedule `job` to run off the UI thread and return a future that
    /// resolves to its [`LoadPayload`] result.
    ///
    /// The widget issues exactly one `spawn_blocking` call per
    /// [`crate::scan::ScanRequest`] (S5.2).
    fn spawn_blocking(&self, job: ScanJob) -> ScanFuture;
}

// ── ThreadExecutor ────────────────────────────────────────────────────────────

/// Default executor: spawns one `std::thread::spawn` per scan (S5.3).
///
/// Correct everywhere; one OS thread per concurrent expansion. For
/// heavy workloads (many prefetches) swap in a pooled executor by
/// implementing [`ScanExecutor`] over a thread-pool or async runtime.
#[derive(Debug, Default, Clone, Copy)]
pub struct ThreadExecutor;

impl ScanExecutor for ThreadExecutor {
    fn spawn_blocking(&self, job: ScanJob) -> ScanFuture {
        let state = Arc::new(Mutex::new(JobFutureState {
            result: None,
            waker: None,
        }));
        let state_for_thread = Arc::clone(&state);

        std::thread::spawn(move || {
            let result = job();
            let mut locked = state_for_thread.lock().unwrap();
            locked.result = Some(result);
            if let Some(waker) = locked.waker.take() {
                waker.wake();
            }
        });

        Box::pin(JobFuture { state })
    }
}

// ── Private future wiring ─────────────────────────────────────────────────────

struct JobFutureState {
    result: Option<LoadPayload>,
    waker: Option<Waker>,
}

/// A [`Future`] that becomes ready when a background thread deposits its
/// result and notifies the waker. No async runtime dependency.
struct JobFuture {
    state: Arc<Mutex<JobFutureState>>,
}

// SAFETY: Arc<Mutex<…>> is Send + Sync; the contained LoadPayload is Send.
unsafe impl Send for JobFuture {}

impl Future for JobFuture {
    type Output = LoadPayload;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        if let Some(result) = state.result.take() {
            Poll::Ready(result)
        } else {
            // Register (or replace) the waker so the thread can wake us.
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
