//! Drag-and-drop state and messages for [`crate::DirectoryTree`].
//!
//! The widget tracks mouse press → hover → release and emits
//! [`DragOutcome::Completed`] on a valid drop. **No filesystem operations
//! are ever performed** — moving, copying, or rejecting the drop is the
//! application's decision (S7.5).
//!
//! ## Flow
//!
//! ```text
//! onmousedown → DragMsg::Pressed   → drag becomes active
//! onmouseenter → DragMsg::Entered  → hovered_target set iff valid
//! onmouseleave → DragMsg::Exited   → hovered_target cleared
//! onmouseup   → DragMsg::Released  → click (S7.2) or DragCompleted
//! Escape key  → DragMsg::Cancelled → drag cleared
//! ```
//!
//! The view crate calls [`crate::DirectoryTree::on_drag_msg`] with each
//! message and handles the returned [`DragOutcome`].

use std::path::PathBuf;

/// Active drag session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragState {
    /// Paths being dragged. If `started_at ∈ selected_paths` at press
    /// time, this is a clone of the full selection; otherwise `[started_at]`.
    pub sources: Vec<PathBuf>,
    /// The current valid drop target, if any. Only a directory that is
    /// neither a source nor a descendant of a source qualifies (S7.3).
    pub hovered_target: Option<PathBuf>,
    /// The row where the mouse was pressed.
    pub started_at: PathBuf,
    /// Whether `started_at` is a directory. Used to emit the correct
    /// `Selected` event in the click case (S7.2).
    pub started_is_dir: bool,
}

/// A drag gesture event sent by the view to
/// [`crate::DirectoryTree::on_drag_msg`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragMsg {
    /// Mouse pressed on a row — starts a drag session.
    Pressed { path: PathBuf, is_dir: bool },
    /// Mouse entered a row during an active drag.
    Entered(PathBuf),
    /// Mouse left a row during an active drag.
    Exited(PathBuf),
    /// Mouse released over a row. If the path equals `started_at` this
    /// was a click (S7.2); otherwise it is a genuine drop.
    Released(PathBuf),
    /// Drag explicitly cancelled (Escape key or mouse-up with no target).
    Cancelled,
}

/// The side effect produced by [`crate::DirectoryTree::on_drag_msg`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragOutcome {
    /// Nothing to do.
    None,
    /// This was a click (press and release on the same row, S7.2).
    /// The host should call `tree.on_selected(&path, is_dir, Replace)`.
    Clicked { path: PathBuf, is_dir: bool },
    /// A genuine drop. The host performs the move/copy/upload.
    /// The widget does NOT refresh automatically afterwards (S7.5).
    Completed {
        sources: Vec<PathBuf>,
        destination: PathBuf,
    },
}

// ── Target validity ───────────────────────────────────────────────────────────

/// `true` iff `path` is a valid drop target given the current drag state
/// and the tree's node graph (S7.3).
///
/// A path is valid iff:
/// 1. It is a directory currently present in the tree.
/// 2. It is not a drag source.
/// 3. It is not a descendant of any drag source (component-wise prefix,
///    never a bare string prefix).
pub(crate) fn is_valid_target(path: &std::path::Path, sources: &[PathBuf], is_dir: bool) -> bool {
    if !is_dir {
        return false;
    }
    // `path.starts_with(s)` uses component-wise comparison — it returns
    // true for `s` itself AND all of its descendants, which handles both
    // conditions 2 and 3 in one check.
    !sources.iter().any(|s| path.starts_with(s))
}
