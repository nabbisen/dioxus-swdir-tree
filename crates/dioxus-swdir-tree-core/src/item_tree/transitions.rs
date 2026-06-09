//! State transitions for [`crate::ItemTree`].

use std::collections::{HashMap, HashSet};

use crate::item_event::ItemTreeEvent;
use crate::item_tree::drag::{ItemDragMsg, ItemDragOutcome, ItemDragState, is_valid_drop};
use crate::item_tree::node::{InternalItem, ItemNode, NodeId};
use crate::item_tree::search::{ItemSearchState, walk_for_search_item};
use crate::item_tree::{DisplayFn, ItemTree};
use crate::keyboard::{Modifiers, TreeKey};
use crate::selection::SelectionMode;

// ── set_tree ──────────────────────────────────────────────────────────────────

impl<T: Clone + std::fmt::Debug + Send + Sync + 'static> ItemTree<T> {
    /// Replace the entire tree content with a new root (S11.4).
    ///
    /// State is preserved for any [`NodeId`] that survives across the
    /// replacement, regardless of whether the node moved to a new position
    /// (S11.5). Disappeared keys are silently dropped; their selection and
    /// expansion state is discarded.
    ///
    /// After `set_tree`, if a `display_fn` is set and a search is active,
    /// search visibility is recomputed automatically.
    pub fn set_tree(&mut self, root: ItemNode<T>) {
        // Snapshot old state before clobbering the store.
        let old_state: HashMap<NodeId, (bool, bool)> = self
            .store
            .iter()
            .map(|(&id, item)| (id, (item.is_expanded, item.is_selected)))
            .collect();

        // Flatten the new tree, preserving old expansion / selection.
        let mut new_store: HashMap<NodeId, InternalItem<T>> = HashMap::new();
        let mut new_order: Vec<NodeId> = Vec::new();
        flatten(&root, None, 0, &old_state, &mut new_store, &mut new_order);

        let live_ids: HashSet<NodeId> = new_order.iter().copied().collect();

        self.store = new_store;
        self.root_id = Some(root.id);
        self.order = new_order;

        // Prune selection/focus to surviving keys.
        self.selected_ids.retain(|id| live_ids.contains(id));
        if self.active_id.is_some_and(|id| !live_ids.contains(&id)) {
            self.active_id = None;
        }
        if self.anchor_id.is_some_and(|id| !live_ids.contains(&id)) {
            self.anchor_id = None;
        }

        // Re-sync is_selected flags.
        let selected: HashSet<NodeId> = self.selected_ids.iter().copied().collect();
        for item in self.store.values_mut() {
            item.is_selected = selected.contains(&item.id);
        }

        // Recompute search if active.
        if let Some(search) = &self.search {
            let query_lower = search.query_lower.clone();
            self.recompute_search(&query_lower);
        }
    }

    // ── on_toggled ────────────────────────────────────────────────────────────

    /// Expand or collapse the node identified by `id`.
    ///
    /// Leaves (nodes with no children) are silently ignored (S11.2).
    pub fn on_toggled(&mut self, id: NodeId) {
        let Some(item) = self.store.get_mut(&id) else {
            return;
        };
        if item.children_ids.is_empty() {
            return;
        }
        item.is_expanded = !item.is_expanded;
    }

    // ── on_selected ───────────────────────────────────────────────────────────

    /// Apply a selection gesture (S11.7 → S6.x analogue).
    pub fn on_selected(&mut self, id: NodeId, mode: SelectionMode) {
        if !self.store.contains_key(&id) {
            return;
        }
        self.active_id = Some(id);

        match mode {
            SelectionMode::Replace => {
                self.selected_ids = vec![id];
                self.anchor_id = Some(id);
            }
            SelectionMode::Toggle => {
                if let Some(pos) = self.selected_ids.iter().position(|&s| s == id) {
                    self.selected_ids.remove(pos);
                } else {
                    self.selected_ids.push(id);
                }
                self.anchor_id = Some(id);
            }
            SelectionMode::ExtendRange => {
                let anchor = match self.anchor_id {
                    Some(a) => a,
                    None => {
                        self.selected_ids = vec![id];
                        self.anchor_id = Some(id);
                        sync_flags(&mut self.store, &self.selected_ids);
                        return;
                    }
                };
                let rows = self.visible_rows();
                let anchor_idx = rows.iter().position(|r| r.id == anchor);
                let target_idx = rows.iter().position(|r| r.id == id);
                if let (Some(a), Some(t)) = (anchor_idx, target_idx) {
                    let (lo, hi) = if a <= t { (a, t) } else { (t, a) };
                    self.selected_ids = rows[lo..=hi].iter().map(|r| r.id).collect();
                }
            }
        }

        sync_flags(&mut self.store, &self.selected_ids);
    }

    // ── handle_key ────────────────────────────────────────────────────────────

    /// Translate a key press into an [`ItemTreeEvent`], or `None` when the
    /// key is unbound.
    ///
    /// Read-only — does not mutate `self`. The caller dispatches the returned
    /// event back through [`Self::on_toggled`] / [`Self::on_selected`].
    pub fn handle_key(&self, key: TreeKey, mods: Modifiers) -> Option<ItemTreeEvent> {
        let rows = self.visible_rows();
        if rows.is_empty() {
            return None;
        }

        let active_idx = self
            .active_id
            .and_then(|id| rows.iter().position(|r| r.id == id));

        match (key, mods.shift) {
            // ── Up / Down ────────────────────────────────────────────────────
            (TreeKey::Up, false) => {
                let idx = active_idx.map(|i| i.saturating_sub(1)).unwrap_or(0);
                Some(selected(rows[idx].id, SelectionMode::Replace))
            }
            (TreeKey::Down, false) => {
                let last = rows.len() - 1;
                let idx = active_idx.map(|i| (i + 1).min(last)).unwrap_or(0);
                Some(selected(rows[idx].id, SelectionMode::Replace))
            }
            (TreeKey::Up, true) => {
                let idx = active_idx.map(|i| i.saturating_sub(1)).unwrap_or(0);
                Some(selected(rows[idx].id, SelectionMode::ExtendRange))
            }
            (TreeKey::Down, true) => {
                let last = rows.len() - 1;
                let idx = active_idx.map(|i| (i + 1).min(last)).unwrap_or(0);
                Some(selected(rows[idx].id, SelectionMode::ExtendRange))
            }

            // ── Home / End ───────────────────────────────────────────────────
            (TreeKey::Home, false) => Some(selected(rows[0].id, SelectionMode::Replace)),
            (TreeKey::End, false) => {
                Some(selected(rows[rows.len() - 1].id, SelectionMode::Replace))
            }

            // ── Enter / Space — fire Selected ────────────────────────────────
            (TreeKey::Enter, _) | (TreeKey::Space, _) => {
                let id = active_idx.map(|i| rows[i].id).or(self.active_id)?;
                Some(selected(id, SelectionMode::Replace))
            }

            // ── Left — collapse or move to parent ────────────────────────────
            (TreeKey::Left, _) => {
                let id = active_idx.map(|i| rows[i].id).or(self.active_id)?;
                let item = self.store.get(&id)?;
                if item.is_expanded {
                    Some(ItemTreeEvent::Toggled(id))
                } else {
                    item.parent_id.map(|p| selected(p, SelectionMode::Replace))
                }
            }

            // ── Right — expand or move to first child ────────────────────────
            (TreeKey::Right, _) => {
                let id = active_idx.map(|i| rows[i].id).or(self.active_id)?;
                let item = self.store.get(&id)?;
                if !item.is_expanded && item.has_children() {
                    Some(ItemTreeEvent::Toggled(id))
                } else if let Some(&first_child) = item.children_ids.first() {
                    if self.store.contains_key(&first_child) {
                        Some(selected(first_child, SelectionMode::Replace))
                    } else {
                        None
                    }
                } else {
                    None // leaf or expanded with no children
                }
            }

            // ── Escape — cancels drag if active; unbound otherwise (S11.13)
            (TreeKey::Escape, _) => {
                if self.drag.is_some() {
                    Some(ItemTreeEvent::Drag(ItemDragMsg::Cancelled))
                } else {
                    None
                }
            }

            _ => None,
        }
    }

    // ── search ────────────────────────────────────────────────────────────────

    /// Activate or update the incremental search filter (S11.6).
    ///
    /// Requires a `display_fn` to have been set at construction time.
    /// Empty string clears the search.
    pub fn set_search_query(&mut self, query: &str) {
        if query.is_empty() {
            self.search = None;
            return;
        }
        if self.display_fn.is_none() {
            return;
        }
        let query_lower = query.to_ascii_lowercase();
        self.search = Some(ItemSearchState {
            query: query.to_string(),
            query_lower: query_lower.clone(),
            visible_ids: std::collections::HashSet::new(),
            match_count: 0,
        });
        self.recompute_search(&query_lower);
    }

    /// Clear the active search.
    pub fn clear_search(&mut self) {
        self.search = None;
    }

    /// Recompute `visible_ids` and `match_count` for the active query.
    pub(crate) fn recompute_search(&mut self, query_lower: &str) {
        let display_fn: DisplayFn<T> = match &self.display_fn {
            Some(f) => std::sync::Arc::clone(f),
            None => return,
        };
        let root_id = match self.root_id {
            Some(id) => id,
            None => return,
        };
        let mut visible_ids = std::collections::HashSet::new();
        let mut match_count = 0;
        walk_for_search_item(
            root_id,
            &self.store,
            &*display_fn,
            query_lower,
            &mut visible_ids,
            &mut match_count,
        );
        if let Some(s) = &mut self.search {
            s.visible_ids = visible_ids;
            s.match_count = match_count;
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn flatten<T: Clone>(
    node: &ItemNode<T>,
    parent_id: Option<NodeId>,
    depth: u32,
    old_state: &HashMap<NodeId, (bool, bool)>,
    store: &mut HashMap<NodeId, InternalItem<T>>,
    order: &mut Vec<NodeId>,
) {
    let (is_expanded, is_selected) = old_state.get(&node.id).copied().unwrap_or((false, false));
    let children_ids: Vec<NodeId> = node.children.iter().map(|c| c.id).collect();
    store.insert(
        node.id,
        InternalItem {
            id: node.id,
            data: node.data.clone(),
            depth,
            children_ids,
            parent_id,
            is_expanded,
            is_selected,
        },
    );
    order.push(node.id);
    for child in &node.children {
        flatten(child, Some(node.id), depth + 1, old_state, store, order);
    }
}

fn sync_flags<T>(store: &mut HashMap<NodeId, InternalItem<T>>, selected_ids: &[NodeId]) {
    let set: HashSet<NodeId> = selected_ids.iter().copied().collect();
    for item in store.values_mut() {
        item.is_selected = set.contains(&item.id);
    }
}

fn selected(id: NodeId, mode: SelectionMode) -> ItemTreeEvent {
    ItemTreeEvent::Selected(id, mode)
}

// ── Drag-and-drop transitions (RFC 013) ───────────────────────────────────────

impl<T: Clone + std::fmt::Debug + Send + Sync + 'static> ItemTree<T> {
    /// Drive the drag state machine (S11.9–S11.16).
    ///
    /// Returns an [`ItemDragOutcome`] describing the side effect the host must
    /// handle: a deferred click selection, or a completed drop intent. The
    /// widget mutates **no** node structure — the host rebuilds its model from
    /// `Completed { sources, target, position }` and calls
    /// [`Self::set_tree`].
    ///
    /// All variants are no-ops when drag-and-drop is disabled or no drag is in
    /// progress, so stray or out-of-order messages are harmless.
    pub fn on_drag_msg(&mut self, msg: ItemDragMsg) -> ItemDragOutcome {
        if !self.dnd_enabled {
            return ItemDragOutcome::None;
        }
        match msg {
            ItemDragMsg::Pressed(id) => {
                let sources = self.sources_for_drag(id);
                self.drag = Some(ItemDragState {
                    sources,
                    primary: id,
                    hover: None,
                });
                ItemDragOutcome::None
            }

            ItemDragMsg::Entered(target, position) => {
                if self.drag.is_none() {
                    return ItemDragOutcome::None;
                }
                let sources = self.drag.as_ref().unwrap().sources.clone();
                let valid = is_valid_drop(&self.store, &sources, target, position);
                self.drag.as_mut().unwrap().hover = if valid {
                    Some((target, position))
                } else {
                    None
                };
                ItemDragOutcome::None
            }

            ItemDragMsg::Exited(target, position) => {
                if let Some(d) = self.drag.as_mut()
                    && d.hover == Some((target, position))
                {
                    d.hover = None;
                }
                ItemDragOutcome::None
            }

            ItemDragMsg::Released(id, _position) => {
                let Some(d) = self.drag.take() else {
                    return ItemDragOutcome::None;
                };
                // Released on the press row → a click (S11.11). Position is
                // ignored; the drop target is the stored hover.
                if id == d.primary {
                    return ItemDragOutcome::Clicked(d.primary);
                }
                if let Some((target, position)) = d.hover {
                    return ItemDragOutcome::Completed {
                        sources: d.sources,
                        target,
                        position,
                    };
                }
                ItemDragOutcome::None
            }

            ItemDragMsg::Cancelled => {
                self.drag = None;
                ItemDragOutcome::None
            }
        }
    }

    /// The source set for a drag beginning on `id`: the whole selection (in
    /// tree pre-order) if `id` is selected, otherwise just `id` (S11.10).
    fn sources_for_drag(&self, id: NodeId) -> Vec<NodeId> {
        let set: HashSet<NodeId> = if self.selected_ids.contains(&id) {
            self.selected_ids.iter().copied().collect()
        } else {
            std::iter::once(id).collect()
        };
        let sources: Vec<NodeId> = self
            .order
            .iter()
            .copied()
            .filter(|i| set.contains(i))
            .collect();
        if sources.is_empty() {
            vec![id]
        } else {
            sources
        }
    }
}
