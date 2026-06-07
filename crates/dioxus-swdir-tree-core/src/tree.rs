//! The widget root: all state, all accessors, and the entry points the
//! embedding layer calls. State transitions live in the private
//! `tree::transitions` submodule.

use std::path::{Path, PathBuf};

use crate::cache::TreeCache;
use crate::config::{DisplayFilter, TreeConfig};
use crate::node::TreeNode;
use crate::scan::{self, LoadedOutcome};

pub(crate) mod transitions;

/// The directory-tree widget state.
///
/// Owns UI state only — which folders are open, what has loaded, the
/// active filter. It never creates, deletes, renames, moves, or writes
/// anything on disk; filesystem operations belong to the application.
#[derive(Debug, Clone, PartialEq)]
pub struct DirectoryTree {
    pub(crate) root: TreeNode,
    pub(crate) config: TreeConfig,
    pub(crate) cache: TreeCache,
    pub(crate) generation: u32,
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
    /// path-matched against the previous node graph).
    pub fn set_filter(&mut self, filter: DisplayFilter) {
        if filter == self.config.filter {
            return;
        }
        self.config.filter = filter;
        transitions::refresh_from_cache(&mut self.root, &self.cache, filter);
        // Selection-flag sync (RFC 004) and search recompute (RFC 010)
        // hook in here when those features land.
    }

    /// The ordered list of rows currently drawn: a depth-first pre-order
    /// walk visiting the root and, beneath every directory that is
    /// expanded **and** loaded, its (already filter-derived) children.
    ///
    /// The single source of draw order — the view, keyboard navigation,
    /// and range selection all consume this list, so they never diverge.
    pub fn visible_rows(&self) -> Vec<(&TreeNode, u32)> {
        let mut rows = Vec::new();
        collect_rows(&self.root, 0, &mut rows);
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
}

fn collect_rows<'a>(node: &'a TreeNode, depth: u32, rows: &mut Vec<(&'a TreeNode, u32)>) {
    rows.push((node, depth));
    if node.is_dir && node.is_expanded && node.is_loaded {
        for child in &node.children {
            collect_rows(child, depth + 1, rows);
        }
    }
}
