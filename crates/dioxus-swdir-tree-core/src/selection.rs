//! Selection state and mode types for [`crate::DirectoryTree`].
//!
//! The authoritative selection is held **by path** on the tree root —
//! never by node reference — so it survives node rebuilds, filter
//! changes, and re-merges transparently.
//!
//! Per-node [`crate::TreeNode::is_selected`] flags are derived view
//! hints re-synced after every mutation; see [`sync_flags`].

use std::collections::HashSet;
use std::path::PathBuf;

use crate::node::TreeNode;

/// How a click or keyboard gesture modifies the selection set.
///
/// Modifier-key mapping (Ctrl → `Toggle`, Shift → `ExtendRange`) is
/// handled by the component layer (RFC 006) because keyboard modifiers
/// arrive on the event object, not in widget state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// Clear the selection and select only this path.
    ///
    /// Clicking an already-selected row still results in exactly that
    /// one row selected (S3.4 — no implicit deselect-on-re-click).
    Replace,

    /// Add if absent; remove if already selected. Updates the anchor.
    Toggle,

    /// Select the contiguous range from the current anchor to this path
    /// in [`crate::DirectoryTree::visible_rows`] order.
    ///
    /// If no anchor is set, behaves as [`SelectionMode::Replace`].
    /// **The anchor does not move** (S6.3): repeated Shift-clicks grow
    /// or shrink the range from the same pivot.
    ExtendRange,
}

/// Re-derive the per-node `is_selected` flag for every loaded node.
///
/// Builds a `HashSet` over the path references so the walk is
/// O(N_loaded) in node visits — not O(N_loaded × M_selected).
pub(crate) fn sync_flags(root: &mut TreeNode, selected_paths: &[PathBuf]) {
    let set: HashSet<&PathBuf> = selected_paths.iter().collect();
    sync_node(root, &set);
}

fn sync_node(node: &mut TreeNode, selected: &HashSet<&PathBuf>) {
    node.is_selected = selected.contains(&node.path);
    for child in &mut node.children {
        sync_node(child, selected);
    }
}
