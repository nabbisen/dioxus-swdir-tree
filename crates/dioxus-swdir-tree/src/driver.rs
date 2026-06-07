//! The async scan driver: a `use_coroutine`-backed hook that runs
//! [`ScanRequest`]s off the UI thread and merges results back through the
//! [`Signal<DirectoryTree>`].

use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_swdir_tree_core::{DirectoryTree, ScanExecutor, ScanRequest, scan};
use futures_util::StreamExt;

/// Set up the scan coroutine for a [`Signal<DirectoryTree>`].
///
/// Returns a [`Coroutine<ScanRequest>`] handle. Call `.send(req)` on it
/// from any event handler that receives a `Some(req)` from
/// [`DirectoryTree::on_toggled`].
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
    // `use_coroutine` takes an `FnMut` init closure, so we must not move
    // `executor` out of the closure body. Instead, the closure owns the Arc
    // and clones it into each generated future.
    use_coroutine(move |mut rx: UnboundedReceiver<ScanRequest>| {
        // Clone the Arc for this future — the outer FnMut retains ownership.
        let exec = Arc::clone(&executor);
        async move {
            while let Some(req) = rx.next().await {
                let req_copy = req.clone();
                let job = Box::new(move || scan::run(&req_copy));
                let payload = exec.spawn_blocking(job).await;
                let outcome = tree.write().on_loaded(payload);
                // Prefetch follow-up requests (RFC 009 — always empty v0.3).
                let _ = outcome;
            }
        }
    })
}
