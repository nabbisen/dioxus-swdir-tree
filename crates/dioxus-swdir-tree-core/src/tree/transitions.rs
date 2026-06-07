//! State transitions of the lazy-loading state machine.
//!
//! Transitions mutate synchronously and return side effects as data;
//! none of them block on I/O or spawn tasks.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::cache::TreeCache;
use crate::config::DisplayFilter;
use crate::drag::{DragMsg, DragOutcome, DragState, is_valid_target};
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
    ///   If the path is in `prefetching_paths`, the user-initiated scan
    ///   supersedes the in-flight prefetch (S8.7).
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
            // A prefetch for this path (unlikely but possible) is
            // superseded by the explicit user action.
            self.prefetching_paths.remove(path);
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
        // User action supersedes any in-flight prefetch for this path (S8.7).
        self.prefetching_paths.remove(path);
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
    /// On acceptance the node is rebuilt and, if prefetch is enabled and
    /// this was a **user-initiated** scan (path not in `prefetching_paths`),
    /// up to `config.prefetch_per_parent` follow-up `ScanRequest`s are
    /// returned in `LoadedOutcome::prefetch_requests` (S8.2). Completions
    /// of prefetch scans never trigger further waves (S8.3).
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
                self.cache
                    .insert(payload.path.clone(), payload.generation, entries);
            }
            Err(issue) => {
                node.children.clear();
                node.error = Some(issue);
                node.is_loaded = true;
            }
        }

        // Step 5 — selection-flag sync.
        selection::sync_flags(&mut self.root, &self.selected_paths);
        // Step 6 — search recompute (S9.7: new children may match).
        crate::tree::recompute_search_if_active(self);

        // Step 7 — prefetch cascade check.
        if self.prefetching_paths.remove(&payload.path) {
            // This was a prefetch completion: no further cascade (S8.3).
            return LoadedOutcome::accepted();
        }

        // User-initiated scan: compute prefetch targets (S8.2).
        let prefetch_requests = compute_prefetch(self, &payload.path, payload.depth);
        LoadedOutcome {
            accepted: true,
            prefetch_requests,
        }
    }
}

/// Compute up to `config.prefetch_per_parent` prefetch scan requests for
/// folder-children of `parent_path` that are not yet loaded, not in the skip
/// list, and within `max_depth` (S8.2, S8.5, S8.6).
///
/// Each selected child path is inserted into `tree.prefetching_paths` and
/// assigned a fresh generation.
fn compute_prefetch(
    tree: &mut DirectoryTree,
    parent_path: &Path,
    parent_depth: u32,
) -> Vec<ScanRequest> {
    if tree.config.prefetch_per_parent == 0 {
        return Vec::new();
    }
    let max_prefetch = tree.config.prefetch_per_parent as usize;
    let max_depth = tree.config.max_depth;
    let child_depth = parent_depth + 1;

    // Collect candidate paths without holding a borrow on `tree.root`.
    let candidates: Vec<PathBuf> = tree
        .root
        .find(parent_path)
        .map(|node| {
            node.children
                .iter()
                .filter(|child| {
                    child.is_dir
                        && !child.is_loaded
                        && max_depth.is_none_or(|max| child_depth <= max)
                        && !is_prefetch_skip(child.file_name(), &tree.config.prefetch_skip)
                })
                .take(max_prefetch)
                .map(|child| child.path.clone())
                .collect()
        })
        .unwrap_or_default();

    if candidates.is_empty() {
        return Vec::new();
    }
    // Bump the generation ONCE for the entire wave so all N scans
    // carry the same generation tag and the staleness check admits
    // each result independently (S8.2, S8.3).
    tree.generation = tree.generation.wrapping_add(1);
    let wave_gen = tree.generation;

    let mut requests = Vec::with_capacity(candidates.len());
    for path in candidates {
        tree.prefetching_paths.insert(path.clone());
        requests.push(ScanRequest {
            path,
            generation: wave_gen,
            depth: child_depth,
        });
    }
    requests
}

/// `true` iff `name` matches an entry in `skip` (ASCII case-insensitive, S8.5).
fn is_prefetch_skip(name: &OsStr, skip: &[String]) -> bool {
    let lower = name.to_string_lossy().to_ascii_lowercase();
    skip.iter().any(|s| s.to_ascii_lowercase() == lower)
}

// ── Rebuild helpers (used by on_loaded and set_filter) ────────────────────────

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

// ── Drag transitions ──────────────────────────────────────────────────────────

impl DirectoryTree {
    /// React to a drag-and-drop gesture message.
    pub fn on_drag_msg(&mut self, msg: DragMsg) -> DragOutcome {
        match msg {
            DragMsg::Pressed { path, is_dir } => {
                let sources = if self.selected_paths.contains(&path) {
                    self.selected_paths.clone()
                } else {
                    vec![path.clone()]
                };
                self.drag = Some(DragState {
                    sources,
                    hovered_target: None,
                    started_at: path,
                    started_is_dir: is_dir,
                });
                DragOutcome::None
            }

            DragMsg::Entered(path) => {
                if self.drag.is_none() {
                    return DragOutcome::None;
                }
                let sources = self.drag.as_ref().unwrap().sources.clone();
                let is_dir = self.find(&path).map(|n| n.is_dir).unwrap_or(false);
                let valid = is_valid_target(&path, &sources, is_dir);
                self.drag.as_mut().unwrap().hovered_target = if valid { Some(path) } else { None };
                DragOutcome::None
            }

            DragMsg::Exited(path) => {
                if let Some(drag) = &mut self.drag
                    && drag.hovered_target.as_deref() == Some(path.as_path())
                {
                    drag.hovered_target = None;
                }
                DragOutcome::None
            }

            DragMsg::Released(path) => {
                let Some(drag) = self.drag.take() else {
                    return DragOutcome::None;
                };
                if path == drag.started_at {
                    DragOutcome::Clicked {
                        path,
                        is_dir: drag.started_is_dir,
                    }
                } else {
                    DragOutcome::Completed {
                        sources: drag.sources,
                        destination: path,
                    }
                }
            }

            DragMsg::Cancelled => {
                self.drag = None;
                DragOutcome::None
            }
        }
    }
}

// ── Selection transition ──────────────────────────────────────────────────────

impl DirectoryTree {
    /// React to a selection gesture on `path`.
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
            }
        }

        selection::sync_flags(&mut self.root, &self.selected_paths);
    }
}
