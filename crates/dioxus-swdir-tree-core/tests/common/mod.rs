//! Shared on-disk fixture for the specification suite.
//!
//! Layout (hidden entries are dotfiles, which is the hiddenness rule on
//! every CI platform this suite runs on):
//!
//! ```text
//! <root>/
//!   alpha/
//!     inner/
//!       deep/
//!     notes.txt
//!   beta/
//!   .hidden_dir/
//!   .hidden_file
//!   zeta.txt
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

pub struct Fixture {
    // Held for its Drop: removes the directory when the test ends.
    _dir: TempDir,
    pub root: PathBuf,
}

impl Fixture {
    pub fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }
}

pub fn fixture() -> Fixture {
    let dir = TempDir::new().expect("create temp dir");
    let root = dir.path().to_path_buf();
    for d in ["alpha/inner/deep", "beta", ".hidden_dir"] {
        fs::create_dir_all(root.join(d)).expect("create fixture dirs");
    }
    for f in ["alpha/notes.txt", ".hidden_file", "zeta.txt"] {
        fs::write(root.join(f), b"fixture").expect("create fixture files");
    }
    Fixture { _dir: dir, root }
}

/// Basenames of `node.children`, in order, as UTF-8.
pub fn child_names(node: &dioxus_swdir_tree_core::TreeNode) -> Vec<String> {
    node.children
        .iter()
        .map(|c| c.file_name().to_string_lossy().into_owned())
        .collect()
}

#[allow(dead_code)]
pub fn names_at(tree: &dioxus_swdir_tree_core::DirectoryTree, path: &Path) -> Vec<String> {
    tree.find(path).map(child_names).unwrap_or_default()
}
