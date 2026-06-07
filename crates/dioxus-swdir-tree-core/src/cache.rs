//! Cache of raw, **unfiltered** scan results, keyed by path.
//!
//! The cache exists so [`crate::DirectoryTree::set_filter`] can re-derive
//! every loaded node's child list in memory with zero I/O. It is never
//! explicitly invalidated in normal use; a newer accepted scan for the
//! same path simply overwrites the entry.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::entry::LoadedEntry;

/// One completed scan: the generation it was accepted under and the
/// complete (unfiltered) entry list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedScan {
    /// Generation the result carried when it was accepted.
    pub generation: u32,
    /// Raw entries, exactly as returned by the scan.
    pub entries: Vec<LoadedEntry>,
}

/// Flat map from scanned path to its latest accepted result.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeCache {
    map: HashMap<PathBuf, CachedScan>,
}

impl TreeCache {
    /// Store (or overwrite) the result for `path`.
    pub(crate) fn insert(&mut self, path: PathBuf, generation: u32, entries: Vec<LoadedEntry>) {
        self.map.insert(
            path,
            CachedScan {
                generation,
                entries,
            },
        );
    }

    /// Latest accepted result for `path`, if any.
    pub fn get(&self, path: &Path) -> Option<&CachedScan> {
        self.map.get(path)
    }

    /// `true` if a result for `path` is cached.
    pub fn contains(&self, path: &Path) -> bool {
        self.map.contains_key(path)
    }

    /// Number of cached scans.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// `true` if nothing has been cached yet.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
