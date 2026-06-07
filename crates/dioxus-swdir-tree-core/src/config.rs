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

/// Settings fixed at construction or mutated by the application at
/// runtime.
///
/// Prefetch settings (`prefetch_per_parent`, `prefetch_skip`) arrive
/// with RFC 009.
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
}

impl TreeConfig {
    /// Configuration with default filter and unbounded depth.
    pub fn new(root_path: impl Into<PathBuf>) -> Self {
        Self {
            root_path: root_path.into(),
            filter: DisplayFilter::default(),
            max_depth: None,
        }
    }
}
