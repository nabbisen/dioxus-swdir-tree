//! The async-boundary types: side effects **as data**, and the one
//! blocking function that executes them.
//!
//! Transitions on [`crate::DirectoryTree`] never spawn tasks. When disk
//! access is needed they return a [`ScanRequest`]; the embedding layer
//! (a Dioxus coroutine, a thread pool, or a test) runs [`run`] on a
//! worker and feeds the produced [`LoadPayload`] back through
//! [`crate::DirectoryTree::on_loaded`].

use std::path::PathBuf;

use swdir::{ScanOptions, scan_dir_with_options};

use crate::entry::LoadedEntry;
use crate::error::ScanIssue;

/// A scan the embedding layer must execute off the UI thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanRequest {
    /// Directory to list (one level, non-recursive).
    pub path: PathBuf,
    /// Generation the result must carry to be accepted.
    pub generation: u32,
    /// Depth of `path` below the root, in components.
    pub depth: u32,
}

/// A completed scan, ready to merge.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadPayload {
    /// Directory that was listed.
    pub path: PathBuf,
    /// Generation copied from the originating [`ScanRequest`].
    pub generation: u32,
    /// Depth copied from the originating [`ScanRequest`].
    pub depth: u32,
    /// The entries, or the failure.
    pub result: Result<Vec<LoadedEntry>, ScanIssue>,
}

/// What [`crate::DirectoryTree::on_loaded`] did with a payload.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoadedOutcome {
    /// `false` means the payload was stale (or its node vanished) and
    /// the tree state is bit-identical to before the call.
    pub accepted: bool,
    /// Follow-up scans to execute. Always empty until prefetch
    /// (RFC 009) lands; the field exists now so the signature never
    /// breaks.
    pub prefetch_requests: Vec<ScanRequest>,
}

impl LoadedOutcome {
    /// Outcome for a silently discarded payload.
    pub(crate) fn discarded() -> Self {
        Self::default()
    }

    /// Outcome for an accepted merge with no follow-up work.
    pub(crate) fn accepted() -> Self {
        Self {
            accepted: true,
            prefetch_requests: Vec::new(),
        }
    }
}

/// Execute a [`ScanRequest`]: list the directory (sorted
/// directories-first, name-ascending — the layout tree widgets expect)
/// and package the result.
///
/// **Blocking.** Call this on a worker thread; never on the UI thread.
pub fn run(request: &ScanRequest) -> LoadPayload {
    let result = scan_dir_with_options(&request.path, &ScanOptions::default())
        .map(|entries| entries.iter().map(LoadedEntry::from).collect())
        .map_err(ScanIssue::from);
    LoadPayload {
        path: request.path.clone(),
        generation: request.generation,
        depth: request.depth,
        result,
    }
}
