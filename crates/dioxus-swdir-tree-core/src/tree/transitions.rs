//! State transitions of the lazy-loading state machine.
//!
//! Transitions mutate synchronously and return side effects as data;
//! none of them block on I/O or spawn tasks.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::cache::TreeCache;
use crate::config::DisplayFilter;
use crate::entry::LoadedEntry;
use crate::node::TreeNode;
use crate::scan::{LoadPayload, LoadedOutcome, ScanRequest};
use crate::selection::{self, SelectionMode};
use crate::tree::DirectoryTree;

impl DirectoryTree {
    /// React to an expand/collapse gesture on `path`.
    ///
    /// Returns `Some(request)` exactly when a scan must be executed —
    /// the slow-path expansion of an unloaded directory. All other
    /// cases mutate (or ignore) in memory and return `None`:
    ///
    /// - **A** — `path` is a file or not in the tree: no-op.
    /// - **B** — expanded directory: collapse. The generation is *not*
    ///   bumped, so an in-flight scan for this subtree stays valid; its
    ///   result will merge into the collapsed (cached) node silently.
    /// - **C** — collapsed and already loaded: fast-path expand, no I/O.
    /// - **D** — collapsed and unloaded: bump the generation and request
    ///   a scan; or, beyond `max_depth`, mark loaded-empty instead.
    pub fn on_toggled(&mut self, path: &Path) -> Option<ScanRequest> {
        let depth = self.depth_of(path)?;
        let max_depth = self.config.max_depth;
        let node = self.root.find_mut(path)?;

        // Case A — only directories toggle.
        if !node.is_dir {
            return None;
        }

        // Case B — collapse.
        if node.is_expanded {
            node.is_expanded = false;
            return None;
        }

        // Case C — fast-path expand from cache.
        if node.is_loaded {
            node.is_expanded = true;
            return None;
        }

        // Case D — slow-path expand.
        if let Some(max) = max_depth
            && depth > max
        {
            // Beyond the cap: treat as already-loaded and empty.
            node.is_loaded = true;
            node.is_expanded = true;
            node.children.clear();
            return None;
        }
        node.is_expanded = true;
        self.generation = self.generation.wrapping_add(1);
        Some(ScanRequest {
            path: path.to_path_buf(),
            generation: self.generation,
            depth,
        })
    }

    /// Merge a completed scan.
    ///
    /// A payload is accepted iff its generation **strictly equals** the
    /// tree's current counter (the counter wraps, so ordering
    /// comparisons would be meaningless). Stale payloads — and payloads
    /// for paths no longer in the tree — are discarded silently, leaving
    /// the state bit-identical.
    ///
    /// On acceptance: the node's children are rebuilt from the filtered
    /// entry list (path-matching the previous children so existing
    /// subtrees, expansion, and loaded flags survive a re-merge), the
    /// raw unfiltered entries are cached for zero-I/O filter switching,
    /// and the node is marked loaded. A failed scan stores the
    /// [`crate::ScanIssue`] on the node with an empty child list.
    /// Selection flags are re-synced after every accepted merge (S6.4).
    pub fn on_loaded(&mut self, payload: LoadPayload) -> LoadedOutcome {
        // Step 1 — staleness check.
        if payload.generation != self.generation {
            return LoadedOutcome::discarded();
        }

        // Step 2 — find the node.
        let filter = self.config.filter;
        let Some(node) = self.root.find_mut(&payload.path) else {
            return LoadedOutcome::discarded();
        };

        // Steps 3–4 — merge; cache raw entries.
        match payload.result {
            Ok(entries) => {
                rebuild_children(node, &entries, filter);
                node.error = None;
                node.is_loaded = true;
                self.cache.insert(payload.path, payload.generation, entries);
            }
            Err(issue) => {
                node.children.clear();
                node.error = Some(issue);
                node.is_loaded = true;
            }
        }

        // Step 5 — selection-flag sync (RFC 004).
        selection::sync_flags(&mut self.root, &self.selected_paths);
        // Search recompute (RFC 010) and prefetch cascade (RFC 009)
        // hook in here when those features land.
        LoadedOutcome::accepted()
    }

    /// React to a selection gesture on `path`.
    ///
    /// All modes set `active_path` and end with a flag sync. No I/O is
    /// performed; no `ScanRequest` is returned. The caller must issue
    /// a separate `on_toggled` to expand the directory if needed.
    ///
    /// `is_dir` is required by future RFC 006 to distinguish file and
    /// folder icons; the core does not gate behaviour on it here.
    pub fn on_selected(&mut self, path: &Path, _is_dir: bool, mode: SelectionMode) {
        let path = path.to_path_buf();
        self.active_path = Some(path.clone());

        match mode {
            SelectionMode::Replace => {
                self.selected_paths = vec![path.clone()];
                self.anchor_path = Some(path);
            }

            SelectionMode::Toggle => {
                if let Some(pos) = self.selected_paths.iter().position(|p| p == &path) {
                    self.selected_paths.remove(pos);
                } else {
                    self.selected_paths.push(path.clone());
                }
                self.anchor_path = Some(path);
            }

            SelectionMode::ExtendRange => {
                let Some(anchor) = self.anchor_path.clone() else {
                    // No anchor: behave as Replace.
                    self.selected_paths = vec![path.clone()];
                    self.anchor_path = Some(path);
                    selection::sync_flags(&mut self.root, &self.selected_paths);
                    return;
                };
                let rows = self.visible_rows();
                let anchor_idx = rows.iter().position(|(n, _)| n.path == anchor);
                let target_idx = rows.iter().position(|(n, _)| n.path == path);
                if let (Some(a), Some(t)) = (anchor_idx, target_idx) {
                    let (lo, hi) = if a <= t { (a, t) } else { (t, a) };
                    self.selected_paths =
                        rows[lo..=hi].iter().map(|(n, _)| n.path.clone()).collect();
                }
                // anchor_path intentionally unchanged (S6.3).
            }
        }

        selection::sync_flags(&mut self.root, &self.selected_paths);
    }
}

/// Rebuild `node.children` from a raw entry list under `filter`,
/// preserving any existing child subtree whose path still appears:
/// expansion, loaded flags, grandchildren, and errors all survive.
pub(crate) fn rebuild_children(
    node: &mut TreeNode,
    entries: &[LoadedEntry],
    filter: DisplayFilter,
) {
    let mut previous: HashMap<PathBuf, TreeNode> = std::mem::take(&mut node.children)
        .into_iter()
        .map(|child| (child.path.clone(), child))
        .collect();
    node.children = entries
        .iter()
        .filter(|entry| filter.admits(entry))
        .map(|entry| {
            previous
                .remove(&entry.path)
                .unwrap_or_else(|| TreeNode::from_entry(entry))
        })
        .collect();
}

/// Re-derive the child list of every loaded directory in `node`'s
/// subtree from the cache, under `filter`. Used by `set_filter`; pure
/// in-memory work.
pub(crate) fn refresh_from_cache(node: &mut TreeNode, cache: &TreeCache, filter: DisplayFilter) {
    if node.is_dir
        && node.is_loaded
        && node.error.is_none()
        && let Some(cached) = cache.get(&node.path)
    {
        rebuild_children(node, &cached.entries, filter);
    }
    for child in &mut node.children {
        refresh_from_cache(child, cache, filter);
    }
}
