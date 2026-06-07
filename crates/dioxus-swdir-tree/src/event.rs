//! Events emitted by [`crate::DirectoryTreeView`] to the host application.
//!
//! The host handles each event by calling the corresponding method on the
//! `Signal<DirectoryTree>` and, for `Toggled`, forwarding any returned
//! `ScanRequest` to the [`crate::use_scan_driver`] coroutine.

use std::path::PathBuf;

use dioxus_swdir_tree_core::SelectionMode;

/// An event emitted by the tree view.
///
/// Pass an `EventHandler<DirectoryTreeEvent>` to [`crate::DirectoryTreeView`]
/// and dispatch each variant in the handler:
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use dioxus_swdir_tree::{DirectoryTreeEvent, DirectoryTreeView, use_scan_driver};
/// # use dioxus_swdir_tree_core::{DirectoryTree, ThreadExecutor};
/// # use std::sync::Arc;
/// # fn app() -> Element {
/// let mut tree = use_signal(|| DirectoryTree::new("/home"));
/// let scans = use_scan_driver(tree, Arc::new(ThreadExecutor));
///
/// let on_event = move |ev: DirectoryTreeEvent| match ev {
///     DirectoryTreeEvent::Toggled(path) => {
///         if let Some(req) = tree.write().on_toggled(&path) {
///             scans.send(req);
///         }
///     }
///     DirectoryTreeEvent::Selected { path, is_dir, mode } => {
///         tree.write().on_selected(&path, is_dir, mode);
///     }
/// };
///
/// rsx! { DirectoryTreeView { tree, on_event } }
/// # }
/// ```
#[derive(Debug, Clone)]
pub enum DirectoryTreeEvent {
    /// The user clicked a caret or activated a row: the host should call
    /// `tree.write().on_toggled(&path)` and forward any returned
    /// `ScanRequest` to the scan driver.
    Toggled(PathBuf),

    /// The user clicked or keyboard-selected a row (v0.3: plain click →
    /// `Replace`; modifier-based modes arrive with RFC 007/008). The host
    /// should call `tree.write().on_selected(&path, is_dir, mode)`.
    Selected {
        path: PathBuf,
        is_dir: bool,
        mode: SelectionMode,
    },
}
