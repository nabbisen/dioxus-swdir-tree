//! Owned, framework-neutral directory entry produced by a scan.
//!
//! [`LoadedEntry`] is the shape stored in the [`crate::TreeCache`] and
//! carried by [`crate::LoadPayload`]: just the three facts the tree needs
//! (`path`, `is_dir`, `is_hidden`), fully owned so values are
//! `Send + 'static` and can cross worker-thread boundaries.
//!
//! Hiddenness is decided **once, at scan time**, using the OS-native rule:
//! dotfile on Unix; the `FILE_ATTRIBUTE_HIDDEN` bit (with the dotfile rule
//! as fallback) on Windows; dotfile elsewhere. `swdir::DirEntry` caches
//! `FileType` and `Metadata`, so no extra syscalls are made here.

use std::ffi::OsStr;
use std::path::PathBuf;

/// One scanned directory entry, reduced to what the tree model needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedEntry {
    /// Absolute path of the entry.
    pub path: PathBuf,
    /// `true` for directories (symlinks are not followed).
    pub is_dir: bool,
    /// `true` if the entry is hidden by the host OS's convention.
    pub is_hidden: bool,
}

impl LoadedEntry {
    /// Final path component, for use as a row label. Falls back to an
    /// empty string if the path somehow lacks a file name.
    pub fn file_name(&self) -> &OsStr {
        self.path.file_name().unwrap_or(OsStr::new(""))
    }
}

impl From<&swdir::DirEntry> for LoadedEntry {
    fn from(entry: &swdir::DirEntry) -> Self {
        Self {
            path: entry.path().to_path_buf(),
            is_dir: entry.is_dir(),
            is_hidden: detect_hidden(entry),
        }
    }
}

/// OS-native hiddenness, computed from data `swdir` already cached.
fn detect_hidden(entry: &swdir::DirEntry) -> bool {
    let dotfile = is_dotfile(entry.display_name());

    #[cfg(windows)]
    {
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        use std::os::windows::fs::MetadataExt;
        let attr_hidden = entry
            .metadata()
            .map(|m| m.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0)
            .unwrap_or(false);
        attr_hidden || dotfile
    }

    #[cfg(not(windows))]
    {
        dotfile
    }
}

/// `true` if the basename starts with an ASCII dot.
pub(crate) fn is_dotfile(name: &OsStr) -> bool {
    name.as_encoded_bytes().first() == Some(&b'.')
}
