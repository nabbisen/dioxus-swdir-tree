//! Node types for [`crate::ItemTree`].
//!
//! [`ItemNode`] is the **caller-facing input** for [`crate::ItemTree::set_tree`].
//! `InternalItem` (private) is the **stored representation** — flattened and enriched
//! with tree-position and UI state.
//! [`VisibleItem`] is the **view-facing output** of
//! [`crate::ItemTree::visible_rows`] — pre-computed and owned so the Dioxus
//! component holds no borrows into the tree.

use std::fmt;

/// Opaque node identity. The caller assigns IDs; the tree treats them as
/// black-box `u64` values (S11.4 key-based diffing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Caller-facing input to [`crate::ItemTree::set_tree`]: a recursive tree
/// of items.
///
/// Build the tree from your own data model and hand the root to `set_tree`.
/// `ItemTree` will flatten it internally and diff it against the previous
/// state, preserving expansion and selection for surviving [`NodeId`]s.
#[derive(Debug, Clone)]
pub struct ItemNode<T> {
    /// Stable, caller-assigned identity.
    pub id: NodeId,
    /// Application data displayed and searched.
    pub data: T,
    /// Ordered children, or empty for leaf nodes.
    pub children: Vec<ItemNode<T>>,
}

impl<T> ItemNode<T> {
    /// Convenience constructor for leaf nodes.
    pub fn leaf(id: NodeId, data: T) -> Self {
        Self {
            id,
            data,
            children: Vec::new(),
        }
    }

    /// Convenience constructor for branch nodes.
    pub fn branch(id: NodeId, data: T, children: Vec<ItemNode<T>>) -> Self {
        Self { id, data, children }
    }
}

// ── Internal representation ───────────────────────────────────────────────────

/// Flattened, key-indexed node stored inside [`crate::ItemTree`].
#[derive(Debug, Clone)]
pub(crate) struct InternalItem<T> {
    pub(crate) id: NodeId,
    pub(crate) data: T,
    /// Pre-order depth: root = 0, root's children = 1, etc.
    pub(crate) depth: u32,
    /// Ordered child IDs (empty for leaves).
    pub(crate) children_ids: Vec<NodeId>,
    /// Parent ID, or `None` for the root.
    pub(crate) parent_id: Option<NodeId>,
    pub(crate) is_expanded: bool,
    pub(crate) is_selected: bool,
}

impl<T> InternalItem<T> {
    pub(crate) fn has_children(&self) -> bool {
        !self.children_ids.is_empty()
    }
}

// ── View-facing output ────────────────────────────────────────────────────────

/// Pre-computed row data returned by [`crate::ItemTree::visible_rows`].
///
/// Owned, cloneable, and free of references into the tree — suitable for
/// passing directly as Dioxus component props.
#[derive(Debug, Clone, PartialEq)]
pub struct VisibleItem {
    /// Node identity (used for event firing).
    pub id: NodeId,
    /// Display string produced by the tree's `display_fn`.
    /// Empty if no `display_fn` was provided at construction.
    pub label: String,
    /// Indentation level: root = 0.
    pub depth: u32,
    /// Whether the node is currently expanded.
    pub is_expanded: bool,
    /// Whether the node carries at least one child.
    pub has_children: bool,
    /// Whether this node is in the selection set.
    pub is_selected: bool,
    /// Whether this is the keyboard-focus / active node.
    pub is_active: bool,
}
