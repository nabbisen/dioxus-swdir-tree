//! The widget root: all state, all accessors, and the entry points the
//! embedding layer calls. State transitions live in the private
//! `tree::transitions` submodule.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::search::{self, SearchState};

use crate::cache::TreeCache;
use crate::config::{DisplayFilter, TreeConfig};
use crate::drag::DragState;
use crate::node::TreeNode;
use crate::scan::{self, LoadedOutcome};
use crate::selection;

pub(crate) mod transitions;

/// The directory-tree widget state.
///
/// Owns UI state only — which folders are open, what has loaded, the
/// active filter, and the selection set. It never creates, deletes,
/// renames, moves, or writes anything on disk; filesystem operations
/// belong to the application.
#[derive(Debug, Clone, PartialEq)]
pub struct DirectoryTree {
    pub(crate) root: TreeNode,
    pub(crate) config: TreeConfig,
    pub(crate) cache: TreeCache,
    pub(crate) generation: u32,
    /// Insertion-ordered, duplicate-free authoritative selection set.
    pub(crate) selected_paths: Vec<PathBuf>,
    /// Most recently touched path (focus / active styling).
    pub(crate) active_path: Option<PathBuf>,
    /// Shift-range pivot. Set by Replace and Toggle; *not* moved by
    /// ExtendRange (S6.3).
    pub(crate) anchor_path: Option<PathBuf>,
    /// Active drag session, or `None` when no drag is in progress.
    pub(crate) drag: Option<DragState>,
    /// Paths for which a speculative prefetch scan is in flight.
    pub(crate) prefetching_paths: HashSet<PathBuf>,
    /// Active incremental search session, or `None` when search is inactive.
    pub(crate) search: Option<crate::search::SearchState>,
}

impl DirectoryTree {
    /// Mount a tree at `root_path`. The root node is created eagerly and
    /// is never removed or replaced; loading it still requires the first
    /// expansion gesture.
    pub fn new(root_path: impl Into<PathBuf>) -> Self {
        let config = TreeConfig::new(root_path);
        let root = TreeNode::new_root(config.root_path.clone());
        Self {
            root,
            config,
            cache: TreeCache::default(),
            generation: 0,
            selected_paths: Vec::new(),
            active_path: None,
            anchor_path: None,
            drag: None,
            prefetching_paths: HashSet::new(),
            search: None,
        }
    }

    /// Builder: set the initial display filter.
    pub fn with_filter(mut self, filter: DisplayFilter) -> Self {
        self.config.filter = filter;
        self
    }

    /// Builder: cap the load depth (components below the root; `0`
    /// means only the root's direct children are ever loaded).
    pub fn with_max_depth(mut self, max_depth: u32) -> Self {
        self.config.max_depth = Some(max_depth);
        self
    }

    /// The root node. Always present.
    pub fn root(&self) -> &TreeNode {
        &self.root
    }

    /// Current configuration.
    pub fn config(&self) -> &TreeConfig {
        &self.config
    }

    /// Current display filter.
    pub fn filter(&self) -> DisplayFilter {
        self.config.filter
    }

    /// Current generation counter (diagnostics and tests).
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Raw scan results accepted so far.
    pub fn cache(&self) -> &TreeCache {
        &self.cache
    }

    /// Find the node for `path`, if it is currently in the tree.
    pub fn find(&self, path: &Path) -> Option<&TreeNode> {
        self.root.find(path)
    }

    /// Depth of `path` below the root in components; `None` if `path`
    /// is not the root or under it. The root itself is depth `0`.
    pub fn depth_of(&self, path: &Path) -> Option<u32> {
        let rel = path.strip_prefix(&self.config.root_path).ok()?;
        Some(rel.components().count() as u32)
    }

    /// Switch the display filter, re-deriving every loaded node's child
    /// list from the cache. Instant; **issues no I/O** and does not bump
    /// the generation. Expansion and loaded state survive (children are
    /// path-matched against the previous node graph). Selection flags
    /// are re-synced so that paths hidden by the new filter remain
    /// selected but their nodes' `is_selected` reflects reality once
    /// visible again (S2.6 / S6.4).
    pub fn set_filter(&mut self, filter: DisplayFilter) {
        if filter == self.config.filter {
            return;
        }
        self.config.filter = filter;
        transitions::refresh_from_cache(&mut self.root, &self.cache, filter);
        selection::sync_flags(&mut self.root, &self.selected_paths);
        recompute_search_if_active(self); // S9.6 — filter first, then search
    }

    /// The ordered list of rows currently drawn: a depth-first pre-order
    /// walk visiting the root and, beneath every directory that is
    /// expanded **and** loaded, its (already filter-derived) children.
    ///
    /// The single source of draw order — the view, keyboard navigation,
    /// and range selection all consume this list, so they never diverge.
    pub fn visible_rows(&self) -> Vec<(&TreeNode, u32)> {
        let mut rows = Vec::new();
        if let Some(search) = &self.search {
            collect_rows_search(&self.root, 0, &mut rows, &search.visible_paths);
        } else {
            collect_rows(&self.root, 0, &mut rows);
        }
        rows
    }

    /// Synchronously expand `path`: run [`DirectoryTree::on_toggled`],
    /// execute any produced scan **on the current thread**, and merge.
    ///
    /// Returns `None` when no scan was needed (fast-path expand,
    /// collapse, no-op) and `Some(outcome)` when a scan ran. This is the
    /// port of upstream's `__test_expand_blocking`: it lets the
    /// specification suite — and quick scripts — bypass all async
    /// infrastructure. GUI code should use `on_toggled` with a worker
    /// instead.
    pub fn expand_blocking(&mut self, path: &Path) -> Option<LoadedOutcome> {
        let request = self.on_toggled(path)?;
        let payload = scan::run(&request);
        Some(self.on_loaded(payload))
    }

    // ── Selection accessors ────────────────────────────────────────────

    /// The full insertion-ordered selection set (S6.x).
    pub fn selected_paths(&self) -> &[PathBuf] {
        &self.selected_paths
    }

    /// The single-select view: the most recently touched path (S3.3).
    ///
    /// Returns `None` before any selection gesture. This is *not*
    /// the last element of `selected_paths`; it is `active_path`,
    /// which the component renders with the distinct "active" style.
    pub fn selected_path(&self) -> Option<&Path> {
        self.active_path.as_deref()
    }

    /// `true` iff `path` is in the selection set.
    ///
    /// This is the authoritative query; prefer it over reading
    /// `node.is_selected`, which is a derived view hint.
    pub fn is_selected(&self, path: &Path) -> bool {
        self.selected_paths.iter().any(|p| p == path)
    }

    /// Builder: enable prefetch — after each user-initiated scan, speculatively
    /// scan up to `n` direct folder-children (S8.2). `0` disables prefetch.
    pub fn with_prefetch_limit(mut self, n: u32) -> Self {
        self.config.prefetch_per_parent = n;
        self
    }

    /// Builder: replace the prefetch skip list (S8.5).
    ///
    /// Entries are compared ASCII case-insensitively against each candidate
    /// child's basename. Defaults to [`crate::config::DEFAULT_PREFETCH_SKIP`].
    pub fn with_prefetch_skip(mut self, skip: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.config.prefetch_skip = skip.into_iter().map(Into::into).collect();
        self
    }

    /// Paths for which a speculative prefetch scan is currently in flight.
    pub fn prefetching_paths(&self) -> &HashSet<PathBuf> {
        &self.prefetching_paths
    }

    // ── Search accessors and mutation ─────────────────────────────────────

    /// The active search query, or `None` when search is inactive.
    pub fn search_query(&self) -> Option<&str> {
        self.search.as_ref().map(|s| s.query.as_str())
    }

    /// The active search state (query, visible paths, match count), or
    /// `None` when search is inactive.
    pub fn search_state(&self) -> Option<&SearchState> {
        self.search.as_ref()
    }

    /// Number of **direct** basename matches (S9.8). Use this for "N
    /// results" displays; ancestor rows shown for context are excluded.
    pub fn search_match_count(&self) -> usize {
        self.search.as_ref().map_or(0, |s| s.match_count)
    }

    /// Activate or update the incremental search filter (S9.1–S9.9).
    ///
    /// An empty string clears the search (S9.4). Search never triggers
    /// I/O — it filters only the already-loaded node graph (S9.9).
    pub fn set_search_query(&mut self, query: &str) {
        if query.is_empty() {
            self.search = None;
            return;
        }
        let query_lower = query.to_ascii_lowercase();
        let mut visible = HashSet::new();
        let mut match_count = 0;
        search::walk_for_search(&self.root, &query_lower, &mut visible, &mut match_count);
        self.search = Some(SearchState {
            query: query.to_string(),
            query_lower,
            visible_paths: visible,
            match_count,
        });
    }

    /// Clear the active search and return to normal tree view.
    ///
    /// Equivalent to `set_search_query("")` (S9.4).
    pub fn clear_search(&mut self) {
        self.search = None;
    }

    /// The active drag session, or `None` when no drag is in progress.
    pub fn drag_state(&self) -> Option<&DragState> {
        self.drag.as_ref()
    }
}

fn collect_rows<'a>(node: &'a TreeNode, depth: u32, rows: &mut Vec<(&'a TreeNode, u32)>) {
    rows.push((node, depth));
    if node.is_dir && node.is_expanded && node.is_loaded {
        for child in &node.children {
            collect_rows(child, depth + 1, rows);
        }
    }
}

/// Search-mode row collection: gates on `visible_paths`, descends into all
/// loaded directories regardless of `is_expanded` (S9.3 — sees through collapse).
fn collect_rows_search<'a>(
    node: &'a TreeNode,
    depth: u32,
    rows: &mut Vec<(&'a TreeNode, u32)>,
    visible_paths: &std::collections::HashSet<std::path::PathBuf>,
) {
    if !visible_paths.contains(&node.path) {
        return;
    }
    rows.push((node, depth));
    if node.is_dir && node.is_loaded {
        for child in &node.children {
            collect_rows_search(child, depth + 1, rows, visible_paths);
        }
    }
}

/// Recompute search visibility when search is active and the node graph
/// has changed (S9.6, S9.7).
pub(crate) fn recompute_search_if_active(tree: &mut DirectoryTree) {
    let query_lower = match &tree.search {
        Some(s) => s.query_lower.clone(),
        None => return,
    };
    let mut visible = HashSet::new();
    let mut match_count = 0;
    search::walk_for_search(&tree.root, &query_lower, &mut visible, &mut match_count);
    if let Some(s) = &mut tree.search {
        s.visible_paths = visible;
        s.match_count = match_count;
    }
}
