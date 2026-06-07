//! Events produced by state-machine inspectors (keyboard, future drag)
//! and consumed by both the core test suite and the view component.
//!
//! Placing `DirectoryTreeEvent` in the core crate lets [`crate::keyboard::handle_key`]
//! return it without a circular dependency, and lets application code
//! depend only on `dioxus-swdir-tree-core` when integrating the widget
//! without a Dioxus renderer.

use std::path::PathBuf;

use crate::selection::SelectionMode;

/// An event emitted by the tree's keyboard handler or the view component.
///
/// The host handles each variant by calling the corresponding method on
/// `DirectoryTree`:
///
/// ```no_run
/// # use dioxus_swdir_tree_core::{DirectoryTreeEvent, DirectoryTree};
/// # use std::path::Path;
/// fn handle(tree: &mut DirectoryTree, ev: DirectoryTreeEvent) {
///     match ev {
///         DirectoryTreeEvent::Toggled(path) => {
///             // Forward any ScanRequest to the scan driver.
///             let _req = tree.on_toggled(&path);
///         }
///         DirectoryTreeEvent::Selected { path, is_dir, mode } => {
///             tree.on_selected(&path, is_dir, mode);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum DirectoryTreeEvent {
    /// Expand or collapse the directory at `path`.
    Toggled(PathBuf),

    /// Change the selection to include `path`.
    Selected {
        /// Target path.
        path: PathBuf,
        /// Whether the target is a directory (used by the view layer for
        /// icon/styling; the core's `on_selected` ignores it).
        is_dir: bool,
        /// How the selection should change.
        mode: SelectionMode,
    },
}
