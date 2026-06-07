//! Widget configuration: display filter modes and the per-tree settings.

use std::path::PathBuf;

use crate::entry::LoadedEntry;

/// Which scanned entries become visible tree nodes.
///
/// Switching the mode at runtime via
/// [`crate::DirectoryTree::set_filter`] is instant and issues **zero
/// I/O**: child lists are re-derived from the raw entries kept in the
/// [`crate::TreeCache`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayFilter {
    /// Non-hidden directories only. Note that a *hidden directory* is
    /// hidden under this mode too — the mode is "folders only", not
    /// "everything that is a folder".
    FoldersOnly,
    /// Non-hidden files and directories. The default.
    #[default]
    FilesAndFolders,
    /// Everything the scan returned, hidden entries included.
    AllIncludingHidden,
}

impl DisplayFilter {
    /// `true` if `entry` survives this filter mode.
    ///
    /// The filter applies to children only; the root node is always
    /// visible regardless of mode.
    pub fn admits(self, entry: &LoadedEntry) -> bool {
        match self {
            Self::FoldersOnly => entry.is_dir && !entry.is_hidden,
            Self::FilesAndFolders => !entry.is_hidden,
            Self::AllIncludingHidden => true,
        }
    }
}

/// Basenames skipped by the prefetch heuristic (S8.5) — directories
/// whose names match any entry here (ASCII case-insensitive) are never
/// speculatively scanned.
///
/// This list only affects **prefetch**; user-initiated scans always
/// proceed regardless of the skip list.
pub const DEFAULT_PREFETCH_SKIP: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    "__pycache__",
    ".venv",
    "venv",
    "target",
    "build",
    "dist",
];

/// Settings fixed at construction or mutated by the application at
/// runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeConfig {
    /// The mounted root. The tree never navigates above it.
    pub root_path: PathBuf,
    /// Active display filter.
    pub filter: DisplayFilter,
    /// Maximum load depth measured in components below the root;
    /// `None` is unbounded. `Some(0)` means only the root's direct
    /// children are ever loaded.
    pub max_depth: Option<u32>,
    /// How many direct folder-children to prefetch after each
    /// user-initiated scan. `0` (the default) disables prefetch
    /// entirely (S8.1).
    pub prefetch_per_parent: u32,
    /// Basenames to skip during prefetch target selection (S8.5).
    /// Defaults to [`DEFAULT_PREFETCH_SKIP`].
    pub prefetch_skip: Vec<String>,
}

impl TreeConfig {
    /// Configuration with default filter, unbounded depth, and
    /// prefetch disabled.
    pub fn new(root_path: impl Into<PathBuf>) -> Self {
        Self {
            root_path: root_path.into(),
            filter: DisplayFilter::default(),
            max_depth: None,
            prefetch_per_parent: 0,
            prefetch_skip: DEFAULT_PREFETCH_SKIP
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}
