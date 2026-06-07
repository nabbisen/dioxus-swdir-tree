//! Cloneable, comparable error attached to a [`crate::TreeNode`] whose
//! scan failed.
//!
//! [`swdir::ScanError`] wraps a [`std::io::Error`], which is neither
//! `Clone` nor `PartialEq`; node graphs must be both (tests compare whole
//! trees, views clone rows). `ScanIssue` preserves the failing path, the
//! [`std::io::ErrorKind`], and the rendered message — everything a GUI
//! needs to display "Permission denied" on a greyed-out row.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

/// Owned record of a failed directory scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanIssue {
    path: PathBuf,
    kind: io::ErrorKind,
    message: String,
}

impl ScanIssue {
    /// Construct an issue from parts. Primarily useful in tests that
    /// inject synthetic failures through [`crate::DirectoryTree::on_loaded`].
    pub fn new(path: impl Into<PathBuf>, kind: io::ErrorKind, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            kind,
            message: message.into(),
        }
    }

    /// Path the failure refers to.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Kind of the underlying I/O error.
    pub fn kind(&self) -> io::ErrorKind {
        self.kind
    }

    /// Human-readable message, suitable for a tooltip or status row.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ScanIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "scan failed at {}: {}",
            self.path.display(),
            self.message
        )
    }
}

impl std::error::Error for ScanIssue {}

impl From<swdir::ScanError> for ScanIssue {
    fn from(err: swdir::ScanError) -> Self {
        let path = err.path().to_path_buf();
        let kind = err.io_kind();
        let message = err.to_string();
        Self {
            path,
            kind,
            message,
        }
    }
}
