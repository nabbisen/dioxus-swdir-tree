//! Generic in-memory item tree — framework-free sibling of [`crate::DirectoryTree`].
//!
//! `ItemTree<T>` provides the same keyboard navigation, multi-select,
//! expand/collapse, and incremental search as `DirectoryTree`, but over
//! caller-supplied, fully-preloaded data keyed by [`NodeId`] rather than
//! `PathBuf`. There is no lazy loading, no generation counter, and no
//! filesystem dependency.
//!
//! # Lifecycle
//!
//! ```no_run
//! # use dioxus_swdir_tree_core::item_tree::{ItemTree, ItemNode, NodeId};
//! # use dioxus_swdir_tree_core::item_event::ItemTreeEvent;
//! # use dioxus_swdir_tree_core::selection::SelectionMode;
//! let root = ItemNode::branch(
//!     NodeId(0),
//!     "root",
//!     vec![
//!         ItemNode::leaf(NodeId(1), "alpha"),
//!         ItemNode::leaf(NodeId(2), "beta"),
//!     ],
//! );
//!
//! let mut tree = ItemTree::new()
//!     .with_display(|s: &&str| s.to_string());
//! tree.set_tree(root);
//!
//! // Drive the tree from events returned by handle_key / UI:
//! // let ev = tree.handle_key(key, mods);
//! // match ev { Some(ItemTreeEvent::Toggled(id)) => tree.on_toggled(id), ... }
//! ```

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

pub mod drag;
pub mod node;
pub(crate) mod search;
pub(crate) mod transitions;

pub use drag::{DropPosition, ItemDragMsg, ItemDragOutcome};
pub use node::{ItemNode, NodeId, VisibleItem};
pub use search::ItemSearchState;

use drag::ItemDragState;
use node::InternalItem;

/// Type alias for the display function stored on [`ItemTree`].
pub type DisplayFn<T> = Arc<dyn Fn(&T) -> String + Send + Sync>;

// ── ItemTree ──────────────────────────────────────────────────────────────────

/// Generic in-memory item tree (S11).
///
/// `T: Clone + Debug + Send + Sync + 'static` mirrors the iced reference
/// implementation's bounds. No additional bounds are imposed by `ItemTree`
/// itself; the optional `display_fn` (needed for search and label rendering)
/// is set at construction time via [`Self::with_display`].
pub struct ItemTree<T: Clone + fmt::Debug + Send + Sync + 'static> {
    // ── Node graph ─────────────────────────────────────────────────────────
    pub(crate) store: HashMap<NodeId, InternalItem<T>>,
    pub(crate) root_id: Option<NodeId>,
    /// Pre-order traversal order — used for iteration and diffing.
    pub(crate) order: Vec<NodeId>,

    // ── Selection ──────────────────────────────────────────────────────────
    /// Insertion-ordered selected IDs.
    pub(crate) selected_ids: Vec<NodeId>,
    /// Most recently touched node (keyboard focus / active styling).
    pub(crate) active_id: Option<NodeId>,
    /// Shift-range pivot — not moved by ExtendRange (S11.7 → S6.3 analogue).
    pub(crate) anchor_id: Option<NodeId>,

    // ── Search ─────────────────────────────────────────────────────────────
    pub(crate) search: Option<ItemSearchState>,

    // ── Display function ───────────────────────────────────────────────────
    /// Converts `T` to a display string for rendering and search.
    /// If `None`, search is disabled and `VisibleItem::label` is empty.
    pub(crate) display_fn: Option<DisplayFn<T>>,

    // ── Drag-and-drop (RFC 013) ────────────────────────────────────────────
    /// Whether drag-and-drop is enabled (opt-in, S11.9).
    pub(crate) dnd_enabled: bool,
    /// Active drag session, or `None` when no drag is in progress.
    pub(crate) drag: Option<ItemDragState>,
}

// ── Manual trait impls (can't derive because of Arc<dyn Fn>) ─────────────────

impl<T: Clone + fmt::Debug + Send + Sync + 'static> Clone for ItemTree<T> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            root_id: self.root_id,
            order: self.order.clone(),
            selected_ids: self.selected_ids.clone(),
            active_id: self.active_id,
            anchor_id: self.anchor_id,
            search: self.search.clone(),
            display_fn: self.display_fn.clone(),
            dnd_enabled: self.dnd_enabled,
            drag: self.drag.clone(),
        }
    }
}

impl<T: Clone + fmt::Debug + Send + Sync + 'static> fmt::Debug for ItemTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ItemTree")
            .field("root_id", &self.root_id)
            .field("nodes", &self.order.len())
            .field("selected_ids", &self.selected_ids)
            .field("active_id", &self.active_id)
            .field("search", &self.search.as_ref().map(|s| &s.query))
            .field("dnd_enabled", &self.dnd_enabled)
            .field("dragging", &self.drag.is_some())
            .finish_non_exhaustive()
    }
}

// ── Construction ──────────────────────────────────────────────────────────────

impl<T: Clone + fmt::Debug + Send + Sync + 'static> ItemTree<T> {
    /// Create an empty tree with no display function.
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            root_id: None,
            order: Vec::new(),
            selected_ids: Vec::new(),
            active_id: None,
            anchor_id: None,
            search: None,
            display_fn: None,
            dnd_enabled: false,
            drag: None,
        }
    }

    /// Builder: attach a display function used for search and label rendering.
    ///
    /// The function maps `&T → String`. Without it, [`Self::set_search_query`]
    /// is a no-op and all [`VisibleItem::label`]s are empty.
    pub fn with_display(mut self, f: impl Fn(&T) -> String + Send + Sync + 'static) -> Self {
        self.display_fn = Some(Arc::new(f));
        self
    }

    /// Builder: enable drag-and-drop reorder/nest (opt-in, S11.9).
    /// Off by default; when disabled, mouse press selects directly.
    pub fn with_drag_and_drop(mut self, enabled: bool) -> Self {
        self.dnd_enabled = enabled;
        self
    }
}

impl<T: Clone + fmt::Debug + Send + Sync + 'static> Default for ItemTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ── Accessors ─────────────────────────────────────────────────────────────────

impl<T: Clone + fmt::Debug + Send + Sync + 'static> ItemTree<T> {
    /// Ordered list of currently visible rows.
    ///
    /// In normal mode: the root and all descendants in expanded branches.
    /// In search mode: matches and all their ancestors, regardless of
    /// expansion state (S11.6 → S9.3 analogue).
    pub fn visible_rows(&self) -> Vec<VisibleItem> {
        let mut rows = Vec::new();
        let Some(root_id) = self.root_id else {
            return rows;
        };
        if let Some(search) = &self.search {
            self.collect_search(root_id, &mut rows, &search.visible_ids);
        } else {
            self.collect_normal(root_id, &mut rows);
        }
        rows
    }

    /// `true` iff `id` is in the current selection set.
    pub fn is_selected(&self, id: NodeId) -> bool {
        self.selected_ids.contains(&id)
    }

    /// Ordered selected IDs.
    pub fn selected_ids(&self) -> &[NodeId] {
        &self.selected_ids
    }

    /// The keyboard-focus / active node.
    pub fn active_id(&self) -> Option<NodeId> {
        self.active_id
    }

    /// Active search session, or `None`.
    pub fn search_state(&self) -> Option<&ItemSearchState> {
        self.search.as_ref()
    }

    /// Current search query, or `None`.
    pub fn search_query(&self) -> Option<&str> {
        self.search.as_ref().map(|s| s.query.as_str())
    }

    /// Direct match count (S11.6 → S9.8 analogue).
    pub fn search_match_count(&self) -> usize {
        self.search.as_ref().map_or(0, |s| s.match_count)
    }

    /// Number of nodes currently in the tree.
    pub fn node_count(&self) -> usize {
        self.store.len()
    }

    /// Whether drag-and-drop is enabled (S11.9).
    pub fn is_drag_and_drop_enabled(&self) -> bool {
        self.dnd_enabled
    }

    /// Whether a drag is currently in progress.
    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// The nodes being dragged (pre-order), or an empty slice when idle.
    pub fn drag_sources(&self) -> &[NodeId] {
        self.drag.as_ref().map_or(&[], |d| d.sources.as_slice())
    }

    /// The current valid drop target and position, or `None`.
    pub fn drop_target(&self) -> Option<(NodeId, DropPosition)> {
        self.drag.as_ref().and_then(|d| d.hover)
    }

    /// Return whether the node identified by `id` is currently expanded.
    /// Returns `None` if the id is not in the tree.
    pub fn is_expanded(&self, id: NodeId) -> Option<bool> {
        self.store.get(&id).map(|item| item.is_expanded)
    }

    // ── Internal traversal ────────────────────────────────────────────────

    fn collect_normal(&self, id: NodeId, rows: &mut Vec<VisibleItem>) {
        let Some(item) = self.store.get(&id) else {
            return;
        };
        rows.push(self.to_visible(item));
        if item.is_expanded {
            for &child_id in &item.children_ids {
                self.collect_normal(child_id, rows);
            }
        }
    }

    fn collect_search(
        &self,
        id: NodeId,
        rows: &mut Vec<VisibleItem>,
        visible_ids: &std::collections::HashSet<NodeId>,
    ) {
        if !visible_ids.contains(&id) {
            return;
        }
        let Some(item) = self.store.get(&id) else {
            return;
        };
        rows.push(self.to_visible(item));
        // Descend regardless of is_expanded — sees through collapse.
        for &child_id in &item.children_ids {
            self.collect_search(child_id, rows, visible_ids);
        }
    }

    fn to_visible(&self, item: &InternalItem<T>) -> VisibleItem {
        let label = self
            .display_fn
            .as_ref()
            .map(|f| f(&item.data))
            .unwrap_or_default();
        VisibleItem {
            id: item.id,
            label,
            depth: item.depth,
            is_expanded: item.is_expanded,
            has_children: item.has_children(),
            is_selected: item.is_selected,
            is_active: self.active_id == Some(item.id),
        }
    }
}
