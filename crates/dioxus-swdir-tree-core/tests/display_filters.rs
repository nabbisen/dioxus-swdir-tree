//! Feature 2 — Display filters (specification clauses S2.1–S2.7).
//!
//! S2.6 ("filter changes preserve selection") is deferred to RFC 004:
//! the selection dimension does not exist yet in v0.1.0. The clause is
//! re-verified the moment `on_selected` lands.

mod common;

use dioxus_swdir_tree_core::{DirectoryTree, DisplayFilter};

use common::{child_names, fixture, names_at};

/// S2.1 — The default filter is FilesAndFolders.
#[test]
fn s2_1_default_filter() {
    let fx = fixture();
    let tree = DirectoryTree::new(&fx.root);
    assert_eq!(tree.filter(), DisplayFilter::FilesAndFolders);
}

/// S2.2 — FoldersOnly shows visible directories and nothing else;
/// hidden directories are excluded under this mode too.
#[test]
fn s2_2_folders_only() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root).with_filter(DisplayFilter::FoldersOnly);
    tree.expand_blocking(&fx.root).expect("root scan");
    assert_eq!(child_names(tree.root()), ["alpha", "beta"]);
}

/// S2.3 — FilesAndFolders shows visible files and directories,
/// directories first, hidden entries excluded.
#[test]
fn s2_3_files_and_folders() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    assert_eq!(child_names(tree.root()), ["alpha", "beta", "zeta.txt"]);
}

/// S2.4 — AllIncludingHidden shows everything.
#[test]
fn s2_4_all_including_hidden() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root).with_filter(DisplayFilter::AllIncludingHidden);
    tree.expand_blocking(&fx.root).expect("root scan");
    assert_eq!(
        child_names(tree.root()),
        [".hidden_dir", "alpha", "beta", ".hidden_file", "zeta.txt"],
        "dirs first then files, name-ascending within each group"
    );
}

/// S2.5 — `set_filter` performs no I/O: it rebuilds from the cache,
/// touching neither the generation counter nor the cache contents.
#[test]
fn s2_5_set_filter_is_io_free() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    let generation = tree.generation();
    let cache_len = tree.cache().len();

    tree.set_filter(DisplayFilter::AllIncludingHidden);
    assert_eq!(
        child_names(tree.root()),
        [".hidden_dir", "alpha", "beta", ".hidden_file", "zeta.txt"]
    );

    tree.set_filter(DisplayFilter::FoldersOnly);
    assert_eq!(child_names(tree.root()), ["alpha", "beta"]);

    assert_eq!(tree.generation(), generation, "no scans were issued");
    assert_eq!(tree.cache().len(), cache_len, "cache untouched");
}

/// S2.7 — Expansion state survives a filter round-trip: nodes that
/// remain visible keep their expanded/loaded flags and their subtrees.
#[test]
fn s2_7_filter_round_trip_preserves_expansion() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    tree.expand_blocking(&fx.path("alpha")).expect("alpha scan");
    tree.expand_blocking(&fx.path("alpha/inner"))
        .expect("inner scan");

    // Under the default filter alpha shows a file too.
    assert_eq!(names_at(&tree, &fx.path("alpha")), ["inner", "notes.txt"]);

    tree.set_filter(DisplayFilter::FoldersOnly);
    assert_eq!(
        names_at(&tree, &fx.path("alpha")),
        ["inner"],
        "notes.txt filtered out"
    );

    tree.set_filter(DisplayFilter::FilesAndFolders);
    assert_eq!(
        names_at(&tree, &fx.path("alpha")),
        ["inner", "notes.txt"],
        "notes.txt restored from cache"
    );

    // The deep expansion chain survived both flips.
    for dir in ["alpha", "alpha/inner"] {
        let node = tree.find(&fx.path(dir)).expect("dir present");
        assert!(node.is_expanded, "{dir} still expanded");
        assert!(node.is_loaded, "{dir} still loaded");
    }
    let deep = tree.find(&fx.path("alpha/inner/deep")).expect("deep");
    assert!(deep.is_dir);
    assert!(!deep.is_loaded, "deep was never expanded — stays lazy");

    // And the drawn rows reflect the full chain:
    // root, alpha, inner, deep, notes.txt, beta, zeta.txt.
    assert_eq!(tree.visible_rows().len(), 7);
}
