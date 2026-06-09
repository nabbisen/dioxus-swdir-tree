//! Incremental search for [`crate::ItemTree`].
//!
//! Mirrors `crate::search` but uses [`super::NodeId`] keys and a caller-
//! supplied `display_fn` rather than `file_name()`.

use std::collections::{HashMap, HashSet};

use super::node::{InternalItem, NodeId};

/// Active search session held on [`crate::ItemTree`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSearchState {
    /// Original query (caller casing).
    pub query: String,
    /// `query.to_ascii_lowercase()` — used for comparisons.
    pub(crate) query_lower: String,
    /// Visible IDs: direct matches ∪ proper ancestors of matches (S11.6 →
    /// S9.2 analogue).
    pub visible_ids: HashSet<NodeId>,
    /// Direct matches only (S11.6 → S9.8 analogue).
    pub match_count: usize,
}

/// Walk the subtree rooted at `id`, populating `visible` and incrementing
/// `match_count` for each node whose display string contains `query_lower`
/// (ASCII case-insensitive). Returns `true` iff the subtree contains at
/// least one match (so the caller knows to add `id` to `visible`).
///
/// Descends into all nodes regardless of `is_expanded` (S11.6 → S9.3
/// analogue — sees through collapse).
pub(crate) fn walk_for_search_item<T>(
    id: NodeId,
    store: &HashMap<NodeId, InternalItem<T>>,
    display_fn: &dyn Fn(&T) -> String,
    query_lower: &str,
    visible: &mut HashSet<NodeId>,
    match_count: &mut usize,
) -> bool {
    let Some(item) = store.get(&id) else {
        return false;
    };

    let label_lower = display_fn(&item.data).to_ascii_lowercase();
    let self_matches = label_lower.contains(query_lower);
    if self_matches {
        *match_count += 1;
    }

    let mut descendant_matches = false;
    for &child_id in &item.children_ids {
        if walk_for_search_item(
            child_id,
            store,
            display_fn,
            query_lower,
            visible,
            match_count,
        ) {
            descendant_matches = true;
        }
    }

    let has_visible = self_matches || descendant_matches;
    if has_visible {
        visible.insert(id);
    }
    has_visible
}
