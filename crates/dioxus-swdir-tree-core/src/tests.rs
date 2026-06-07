//! Unit tests for crate internals. Specification-level tests (the
//! `feature-specs.md` oracle) live as integration tests under `tests/`.

use std::ffi::OsStr;
use std::io;
use std::path::PathBuf;

use crate::config::DisplayFilter;
use crate::entry::{LoadedEntry, is_dotfile};
use crate::error::ScanIssue;
use crate::node::TreeNode;
use crate::tree::transitions::rebuild_children;

fn entry(path: &str, is_dir: bool, is_hidden: bool) -> LoadedEntry {
    LoadedEntry {
        path: PathBuf::from(path),
        is_dir,
        is_hidden,
    }
}

#[test]
fn dotfile_detection() {
    assert!(is_dotfile(OsStr::new(".git")));
    assert!(!is_dotfile(OsStr::new("src")));
    assert!(!is_dotfile(OsStr::new("")));
}

#[test]
fn filter_predicates() {
    let dir = entry("/r/src", true, false);
    let hidden_dir = entry("/r/.git", true, true);
    let file = entry("/r/main.rs", false, false);
    let hidden_file = entry("/r/.env", false, true);

    let folders = DisplayFilter::FoldersOnly;
    assert!(folders.admits(&dir));
    assert!(!folders.admits(&hidden_dir)); // hidden dirs are hidden here too
    assert!(!folders.admits(&file));
    assert!(!folders.admits(&hidden_file));

    let both = DisplayFilter::FilesAndFolders;
    assert!(both.admits(&dir));
    assert!(!both.admits(&hidden_dir));
    assert!(both.admits(&file));
    assert!(!both.admits(&hidden_file));

    let all = DisplayFilter::AllIncludingHidden;
    for e in [&dir, &hidden_dir, &file, &hidden_file] {
        assert!(all.admits(e));
    }
}

#[test]
fn find_descends_componentwise_not_by_string_prefix() {
    let mut root = TreeNode::new_root(PathBuf::from("/r"));
    rebuild_children(
        &mut root,
        &[
            entry("/r/bar", true, false),
            entry("/r/barbaz", true, false),
        ],
        DisplayFilter::FilesAndFolders,
    );
    // "/r/barbaz" must not be reached through "/r/bar".
    assert_eq!(
        root.find(&PathBuf::from("/r/barbaz"))
            .map(|n| n.path.clone()),
        Some(PathBuf::from("/r/barbaz"))
    );
    assert!(root.find(&PathBuf::from("/r/elsewhere")).is_none());
}

#[test]
fn rebuild_preserves_matched_subtrees_and_drops_vanished_ones() {
    let mut root = TreeNode::new_root(PathBuf::from("/r"));
    rebuild_children(
        &mut root,
        &[entry("/r/keep", true, false), entry("/r/gone", true, false)],
        DisplayFilter::FilesAndFolders,
    );
    // Simulate state on the child that must survive a rebuild.
    {
        let keep = root.find_mut(&PathBuf::from("/r/keep")).unwrap();
        keep.is_expanded = true;
        keep.is_loaded = true;
        keep.children
            .push(TreeNode::from_entry(&entry("/r/keep/deep", false, false)));
    }
    rebuild_children(
        &mut root,
        &[entry("/r/keep", true, false), entry("/r/new", false, false)],
        DisplayFilter::FilesAndFolders,
    );
    let keep = root.find(&PathBuf::from("/r/keep")).unwrap();
    assert!(keep.is_expanded && keep.is_loaded);
    assert_eq!(keep.children.len(), 1);
    assert!(root.find(&PathBuf::from("/r/gone")).is_none());
    assert!(root.find(&PathBuf::from("/r/new")).is_some());
}

#[test]
fn scan_issue_is_cloneable_and_comparable() {
    let a = ScanIssue::new("/r/x", io::ErrorKind::PermissionDenied, "denied");
    let b = a.clone();
    assert_eq!(a, b);
    assert_eq!(a.kind(), io::ErrorKind::PermissionDenied);
    assert!(a.to_string().contains("/r/x"));
}
