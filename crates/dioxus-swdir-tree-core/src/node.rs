//! The recursive node value making up the in-memory tree.
//!
//! Invariants (upheld by [`crate::DirectoryTree`] transitions, asserted
//! by the test suite):
//!
//! 1. `is_expanded` ⟹ `is_dir` — files are never expanded.
//! 2. `is_expanded` and `is_loaded` are independent: a node may be
//!    loaded-but-collapsed, or briefly expanded-but-not-loaded while a
//!    scan is in flight.
//! 3. `children` is empty iff `!is_loaded`, except for genuinely empty
//!    (or fully filtered) directories, which are loaded with no children.
//! 4. `error.is_some()` ⟹ `is_loaded` ∧ `children` empty.
//!
//! Nodes are **ephemeral**: rebuilt when filters change or fresh scan
//! data merges. Anything that must survive rebuilds (the selection set,
//! from RFC 004 on) lives on the tree root keyed by path, never on nodes.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::entry::LoadedEntry;
use crate::error::ScanIssue;

/// One directory entry in the tree.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeNode {
    /// Absolute path.
    pub path: PathBuf,
    /// `true` for directories.
    pub is_dir: bool,
    /// `true` if the node's children are drawn on screen.
    pub is_expanded: bool,
    /// `true` once a scan has populated (or errored) this node.
    pub is_loaded: bool,
    /// Derived view hint: `true` iff this path is in the tree's
    /// [`crate::DirectoryTree::selected_paths`] set.
    ///
    /// Re-synced after every selection mutation; never authoritative —
    /// paths remain selected while their node is unloaded, filtered out,
    /// or temporarily absent.
    pub is_selected: bool,
    /// Filtered children; empty until loaded.
    pub children: Vec<TreeNode>,
    /// Set when the scan for this directory failed.
    pub error: Option<ScanIssue>,
}

impl TreeNode {
    /// The eagerly-created root node. Always a directory, initially
    /// neither loaded nor expanded.
    pub(crate) fn new_root(path: PathBuf) -> Self {
        Self {
            path,
            is_dir: true,
            is_expanded: false,
            is_loaded: false,
            is_selected: false,
            children: Vec::new(),
            error: None,
        }
    }

    /// A fresh, unloaded node for a scanned entry.
    pub(crate) fn from_entry(entry: &LoadedEntry) -> Self {
        Self {
            path: entry.path.clone(),
            is_dir: entry.is_dir,
            is_expanded: false,
            is_loaded: false,
            is_selected: false,
            children: Vec::new(),
            error: None,
        }
    }

    /// Final path component, for use as a row label.
    pub fn file_name(&self) -> &OsStr {
        self.path.file_name().unwrap_or(OsStr::new(""))
    }

    /// Find the node for `path` in this subtree.
    ///
    /// Descends only into the child whose path is a component-wise
    /// prefix of `path`, so lookup cost is proportional to depth, not
    /// to tree size.
    pub fn find(&self, path: &Path) -> Option<&TreeNode> {
        if self.path == path {
            return Some(self);
        }
        self.children
            .iter()
            .find(|c| path.starts_with(&c.path))
            .and_then(|c| c.find(path))
    }

    /// Mutable variant of [`TreeNode::find`].
    pub(crate) fn find_mut(&mut self, path: &Path) -> Option<&mut TreeNode> {
        if self.path == path {
            return Some(self);
        }
        self.children
            .iter_mut()
            .find(|c| path.starts_with(&c.path))
            .and_then(|c| c.find_mut(path))
    }
}
