//! Incremental search over the already-loaded node graph.
//!
//! Search never triggers I/O (S9.9): it filters the **currently loaded**
//! tree. [`walk_for_search`] is called every time the query, the filter,
//! or the node graph changes.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::node::TreeNode;

/// Active search session held on [`crate::DirectoryTree`].
///
/// `None` when no search is active.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchState {
    /// Query as provided by the application (original casing).
    pub query: String,
    /// `query.to_ascii_lowercase()` — used for comparisons (S9.1).
    pub query_lower: String,
    /// Paths that are visible in search mode: direct matches ∪ ancestors
    /// of matches (S9.2). Used by [`crate::DirectoryTree::visible_rows`]
    /// to gate which rows are drawn.
    pub visible_paths: HashSet<PathBuf>,
    /// Count of **direct** matches only; ancestors shown for context are
    /// not included (S9.8). Applications should use this for "N results"
    /// displays.
    pub match_count: usize,
}

/// Walk `node` and its already-loaded descendants, populating
/// `visible` with every matching path and every ancestor of a match
/// (S9.2, S9.3). Returns `true` iff the subtree rooted at `node`
/// contains at least one match (so the caller knows to include `node`
/// in the visible set).
///
/// The walk ignores `is_expanded` — it descends into any loaded
/// directory regardless of its expansion state (S9.3).
pub fn walk_for_search(
    node: &TreeNode,
    query_lower: &str,
    visible: &mut HashSet<PathBuf>,
    match_count: &mut usize,
) -> bool {
    let basename_lower = node.file_name().to_string_lossy().to_ascii_lowercase();
    let self_matches = basename_lower.contains(query_lower);
    if self_matches {
        *match_count += 1;
    }

    // Recurse into loaded children.
    let mut descendant_matches = false;
    for child in &node.children {
        if walk_for_search(child, query_lower, visible, match_count) {
            descendant_matches = true;
        }
    }

    let has_visible = self_matches || descendant_matches;
    if has_visible {
        visible.insert(node.path.clone());
    }
    has_visible
}
