//! The async scan driver: a `use_coroutine`-backed hook that runs
//! [`ScanRequest`]s off the UI thread and merges results back through the
//! [`Signal<DirectoryTree>`].

use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_swdir_tree_core::{DirectoryTree, ScanExecutor, ScanRequest, scan};
use futures_util::StreamExt;

/// Set up the scan coroutine for a [`Signal<DirectoryTree>`].
///
/// Returns a [`Coroutine<ScanRequest>`] handle. Send any `ScanRequest`
/// returned by `tree.write().on_toggled(path)` or (RFC 009) by
/// `LoadedOutcome::prefetch_requests` through this handle.
///
/// # Example
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use dioxus_swdir_tree::{DirectoryTreeView, DirectoryTreeEvent, use_scan_driver};
/// # use dioxus_swdir_tree_core::{DirectoryTree, ThreadExecutor};
/// # use std::sync::Arc;
/// fn app() -> Element {
///     let mut tree = use_signal(|| DirectoryTree::new("/home"));
///     let scans = use_scan_driver(tree, Arc::new(ThreadExecutor));
///
///     let on_event = move |ev: DirectoryTreeEvent| {
///         if let DirectoryTreeEvent::Toggled(path) = ev {
///             if let Some(req) = tree.write().on_toggled(&path) {
///                 scans.send(req);
///             }
///         }
///     };
///
///     rsx! { DirectoryTreeView { tree, on_event } }
/// }
/// ```
pub fn use_scan_driver(
    mut tree: Signal<DirectoryTree>,
    executor: Arc<dyn ScanExecutor>,
) -> Coroutine<ScanRequest> {
    // `use_coroutine` takes an `FnMut` init closure; clone the Arc inside
    // the body so `executor` is borrowed (not moved) across calls.
    use_coroutine(move |mut rx: UnboundedReceiver<ScanRequest>| {
        let exec = Arc::clone(&executor);
        async move {
            while let Some(req) = rx.next().await {
                let req_copy = req.clone();
                let job = Box::new(move || scan::run(&req_copy));
                let payload = exec.spawn_blocking(job).await;
                let outcome = tree.write().on_loaded(payload);

                // RFC 009: fan out prefetch requests as concurrent spawned
                // tasks. Prefetch completions do not cascade (S8.3 is
                // enforced by the core state machine via `prefetching_paths`).
                for prefetch_req in outcome.prefetch_requests {
                    let exec_clone = Arc::clone(&exec);
                    spawn(async move {
                        let r = prefetch_req.clone();
                        let job = Box::new(move || scan::run(&r));
                        let payload = exec_clone.spawn_blocking(job).await;
                        // Merge result. The core discards it if it's stale
                        // (e.g. the user expanded the path first, S8.7).
                        tree.write().on_loaded(payload);
                        // Prefetch completions never trigger further prefetch
                        // (S8.3 — handled by the core; no need to forward here).
                    });
                }
            }
        }
    })
}
