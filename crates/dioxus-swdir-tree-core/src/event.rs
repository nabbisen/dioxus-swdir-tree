//! Events produced by state-machine inspectors (keyboard, drag) and
//! consumed by both the core test suite and the view component.
//!
//! Placing `DirectoryTreeEvent` in the core crate lets
//! [`crate::keyboard::handle_key`] return it without a circular dependency,
//! and lets application code depend only on `dioxus-swdir-tree-core` when
//! integrating the widget without a Dioxus renderer.

use std::path::PathBuf;

use crate::drag::DragMsg;
use crate::selection::SelectionMode;

/// An event emitted by the keyboard handler, the view component, or the drag
/// state machine.
///
/// The host handles each variant by calling the corresponding method on
/// `DirectoryTree`:
///
/// ```no_run
/// # use dioxus_swdir_tree_core::{DirectoryTreeEvent, DirectoryTree, SelectionMode};
/// # use dioxus_swdir_tree_core::drag::DragOutcome;
/// # use std::path::Path;
/// fn handle(tree: &mut DirectoryTree, ev: DirectoryTreeEvent) {
///     match ev {
///         DirectoryTreeEvent::Toggled(path) => {
///             let _req = tree.on_toggled(&path);
///         }
///         DirectoryTreeEvent::Selected { path, is_dir, mode } => {
///             tree.on_selected(&path, is_dir, mode);
///         }
///         DirectoryTreeEvent::Drag(msg) => {
///             match tree.on_drag_msg(msg) {
///                 DragOutcome::Clicked { path, is_dir } => {
///                     tree.on_selected(&path, is_dir, SelectionMode::Replace);
///                 }
///                 DragOutcome::Completed { sources, destination } => {
///                     // application performs the filesystem operation
///                     let _ = (sources, destination);
///                 }
///                 DragOutcome::None => {}
///             }
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

    /// A drag-and-drop gesture. The host calls
    /// [`crate::DirectoryTree::on_drag_msg`] and acts on the returned
    /// [`crate::drag::DragOutcome`].
    Drag(DragMsg),
}
