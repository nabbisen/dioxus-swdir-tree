//! Drag-and-drop state and validity for [`crate::ItemTree`] (RFC 013).
//!
//! Mirrors `iced-swdir-tree` RFC 002 / spec S11.9–S11.16. The widget
//! mutates nothing on a completed drop — it emits the drop *intent*
//! ([`ItemDragOutcome::Completed`]) and the host rebuilds its model and
//! calls [`crate::ItemTree::set_tree`].
//!
//! # Drop model
//!
//! Unlike [`crate::DirectoryTree`] (drop is always *into* a folder), an
//! item-tree drop lands relative to a target node at one of three
//! [`DropPosition`]s, expressing both reorder (sibling) and nest (child).
//!
//! # Validity (S11.12)
//!
//! A drop of sources `S` at `(target, position)` is valid iff:
//! 1. `target` is a live node.
//! 2. `target ∉ S`.
//! 3. For `Before`/`After`, `target` is not the root (no sibling slot).
//! 4. No cycle: the effective new parent (`target` for `Into`, else
//!    `target`'s parent) is neither a source nor a descendant of any source.
//!
//! Our arena stores `parent_id` natively, so rule 4 is a direct ancestor
//! walk over the live store — no parent-map snapshot.

use std::collections::HashMap;

use crate::item_tree::node::{InternalItem, NodeId};

/// Where a dragged node lands relative to the drop target (S11.15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropPosition {
    /// Insert as a sibling immediately before `target`.
    Before,
    /// Append as the last child of `target` (nest).
    Into,
    /// Insert as a sibling immediately after `target`.
    After,
}

/// Opaque drag-gesture message produced by the view, routed back to
/// [`crate::ItemTree::on_drag_msg`] unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemDragMsg {
    /// Mouse pressed on a row body — begins a drag.
    Pressed(NodeId),
    /// Cursor entered a drop zone of `target` at the given position.
    Entered(NodeId, DropPosition),
    /// Cursor left a drop zone of `target` at the given position.
    Exited(NodeId, DropPosition),
    /// Mouse released over a drop zone (position is informational — the
    /// drop target is the stored hover).
    Released(NodeId, DropPosition),
    /// Drag cancelled (Escape, or release over nothing valid). Idempotent.
    Cancelled,
}

/// The side effect produced by [`crate::ItemTree::on_drag_msg`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemDragOutcome {
    /// Nothing to do.
    None,
    /// Release on the press row — a click. Host should call
    /// `on_selected(id, SelectionMode::Replace)` (S11.11).
    Clicked(NodeId),
    /// A genuine drop. Host moves the nodes in its own model and calls
    /// `set_tree` (S11.14).
    Completed {
        sources: Vec<NodeId>,
        target: NodeId,
        position: DropPosition,
    },
}

/// In-flight drag state, held on [`crate::ItemTree`].
#[derive(Debug, Clone)]
pub(crate) struct ItemDragState {
    /// Dragged node ids in tree (pre-order) order.
    pub(crate) sources: Vec<NodeId>,
    /// The node actually pressed — distinguishes click from drag.
    pub(crate) primary: NodeId,
    /// Current valid hover target, or `None` over an invalid zone.
    pub(crate) hover: Option<(NodeId, DropPosition)>,
}

/// `true` iff a drop of `sources` at `(target, position)` is valid (S11.12),
/// checked against the live arena `store`.
pub(crate) fn is_valid_drop<T>(
    store: &HashMap<NodeId, InternalItem<T>>,
    sources: &[NodeId],
    target: NodeId,
    position: DropPosition,
) -> bool {
    // Rule 1.
    let Some(target_item) = store.get(&target) else {
        return false;
    };
    // Rule 2.
    if sources.contains(&target) {
        return false;
    }
    // Effective new parent.
    let effective_parent = match position {
        DropPosition::Into => target,
        DropPosition::Before | DropPosition::After => match target_item.parent_id {
            Some(p) => p,
            None => return false, // Rule 3: root has no sibling slot.
        },
    };
    // Rule 4: no cycle.
    for &s in sources {
        if is_ancestor_or_self(store, s, effective_parent) {
            return false;
        }
    }
    true
}

/// Walk `node`'s ancestor chain (inclusive); `true` if `maybe_ancestor`
/// is `node` or an ancestor of it. O(depth).
fn is_ancestor_or_self<T>(
    store: &HashMap<NodeId, InternalItem<T>>,
    maybe_ancestor: NodeId,
    node: NodeId,
) -> bool {
    let mut cur = Some(node);
    while let Some(c) = cur {
        if c == maybe_ancestor {
            return true;
        }
        cur = store.get(&c).and_then(|item| item.parent_id);
    }
    false
}

#[cfg(test)]
mod tests;
