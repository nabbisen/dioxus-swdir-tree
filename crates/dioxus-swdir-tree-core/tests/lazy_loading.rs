//! Feature 1 — Lazy loading (specification clauses S1.1–S1.6).
//!
//! Each test names the clause(s) of the upstream feature spec it
//! verifies. The fixture layout is documented in `common/mod.rs`.

mod common;

use std::fs;

use dioxus_swdir_tree_core::{DirectoryTree, scan};

use common::{child_names, fixture};

/// S1.1 — Construction performs no I/O: the tree starts as a single
/// unloaded, unexpanded root row and the generation counter at zero.
#[test]
fn s1_1_construction_is_io_free() {
    let fx = fixture();
    let tree = DirectoryTree::new(&fx.root);

    let root = tree.root();
    assert!(root.is_dir);
    assert!(!root.is_loaded);
    assert!(!root.is_expanded);
    assert!(root.children.is_empty());
    assert_eq!(root.error, None);
    assert_eq!(tree.generation(), 0);
    assert!(tree.cache().is_empty());

    let rows = tree.visible_rows();
    assert_eq!(rows.len(), 1, "only the root row is drawn before expand");
    assert_eq!(rows[0].0.path, fx.root);
    assert_eq!(rows[0].1, 0);
}

/// S1.2 — Expansion scans exactly one level; the generation bumps
/// before the scan is issued; collapse never bumps; re-expansion of an
/// unloaded directory issues a fresh-generation scan and the older
/// in-flight payload is discarded while the fresh one is accepted.
#[test]
fn s1_2_generation_protocol_over_expand_collapse_expand() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);

    // First expansion: Case D — bump generation, issue scan.
    let first = tree
        .on_toggled(&fx.root)
        .expect("unloaded expand must produce a scan request");
    assert_eq!(first.generation, 1);
    assert_eq!(tree.generation(), 1, "bumped before the scan runs");
    assert!(tree.root().is_expanded, "expands optimistically");
    assert!(!tree.root().is_loaded, "not loaded until the payload lands");

    // Collapse while the scan is notionally in flight: no bump.
    assert!(tree.on_toggled(&fx.root).is_none());
    assert_eq!(tree.generation(), 1, "collapse never bumps the generation");
    assert!(!tree.root().is_expanded);

    // Re-expand: the node is still unloaded, so Case D again.
    let second = tree
        .on_toggled(&fx.root)
        .expect("still-unloaded expand must produce a scan request");
    assert_eq!(second.generation, 2);

    // The first (stale) payload arrives late: silently discarded.
    let stale = scan::run(&first);
    let outcome = tree.on_loaded(stale);
    assert!(!outcome.accepted);
    assert!(!tree.root().is_loaded, "stale payload must not merge");

    // The current-generation payload merges.
    let fresh = scan::run(&second);
    let outcome = tree.on_loaded(fresh);
    assert!(outcome.accepted);
    let root = tree.root();
    assert!(root.is_loaded);
    assert!(root.is_expanded);
    // Default filter: FilesAndFolders — dotfiles excluded, dirs first.
    assert_eq!(child_names(root), ["alpha", "beta", "zeta.txt"]);
    // One level only: alpha's own children were not scanned.
    let alpha = tree.find(&fx.path("alpha")).expect("alpha listed");
    assert!(!alpha.is_loaded);
    assert!(alpha.children.is_empty());
}

/// S1.3 — A stale payload leaves the tree bit-identical: not just
/// "looks the same", but `PartialEq`-equal across every dimension
/// (nodes, cache, generation, config).
#[test]
fn s1_3_stale_payload_leaves_tree_identical() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);

    let request = tree.on_toggled(&fx.root).expect("scan request");
    let payload = scan::run(&request);

    // Bump the generation past the payload by starting another scan
    // elsewhere conceptually — here, collapse + re-expand the root.
    tree.on_toggled(&fx.root); // collapse, no bump
    let _fresh = tree.on_toggled(&fx.root).expect("re-expand"); // bump

    let snapshot = tree.clone();
    let outcome = tree.on_loaded(payload);
    assert!(!outcome.accepted);
    assert!(outcome.prefetch_requests.is_empty());
    assert_eq!(tree, snapshot, "discard must be a perfect no-op");
}

/// S1.4 — Children stay in memory across collapse: re-expanding a
/// loaded directory is the zero-I/O fast path (Case C) and never
/// produces a scan request or a generation bump.
#[test]
fn s1_4_collapse_keeps_children_reexpand_is_io_free() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("initial scan");
    let gen_after_load = tree.generation();

    // Collapse: children retained, only the drawn flag flips.
    assert!(tree.on_toggled(&fx.root).is_none());
    let root = tree.root();
    assert!(!root.is_expanded);
    assert!(root.is_loaded);
    assert_eq!(root.children.len(), 3, "children survive collapse");
    assert_eq!(tree.visible_rows().len(), 1, "but are not drawn");

    // Re-expand: Case C — no scan, no bump, rows reappear instantly.
    assert!(tree.on_toggled(&fx.root).is_none());
    assert_eq!(tree.generation(), gen_after_load);
    assert!(tree.root().is_expanded);
    assert_eq!(tree.visible_rows().len(), 4);
}

/// S1.5 — Depth cap: a directory deeper than `max_depth` expands to a
/// permanently-empty loaded state without any scan being issued.
#[test]
fn s1_5_max_depth_caps_scanning() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root).with_max_depth(0);

    // The root sits at depth 0 == max_depth: scans normally.
    tree.expand_blocking(&fx.root).expect("root scan allowed");
    let gen_after_root = tree.generation();

    // alpha is at depth 1 > max_depth: loaded-empty, no scan, no bump.
    let alpha = fx.path("alpha");
    assert!(tree.on_toggled(&alpha).is_none());
    assert_eq!(tree.generation(), gen_after_root);
    let node = tree.find(&alpha).expect("alpha node");
    assert!(node.is_loaded);
    assert!(node.is_expanded);
    assert!(node.children.is_empty());
    assert_eq!(node.error, None);
}

/// S1.6 — Scan failure: the node becomes a loaded error leaf; later
/// toggles collapse/expand it without retrying the scan.
#[test]
fn s1_6_error_nodes_are_loaded_and_not_auto_retried() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    // Make alpha unscannable, then try to expand it.
    let alpha = fx.path("alpha");
    fs::remove_dir_all(&alpha).expect("remove alpha from disk");
    let request = tree.on_toggled(&alpha).expect("expand issues a scan");
    let payload = scan::run(&request);
    assert!(payload.result.is_err(), "scan of a removed dir must fail");

    let outcome = tree.on_loaded(payload);
    assert!(outcome.accepted, "an error result is still current");
    let node = tree.find(&alpha).expect("node remains in the tree");
    assert!(node.is_loaded, "error nodes count as loaded");
    assert!(node.children.is_empty());
    let issue = node.error.as_ref().expect("error recorded on the node");
    assert_eq!(issue.path(), alpha.as_path());

    // Toggling now: collapse (no scan), then Case C re-expand (no scan).
    let generation = tree.generation();
    assert!(tree.on_toggled(&alpha).is_none(), "collapse");
    assert!(
        tree.on_toggled(&alpha).is_none(),
        "fast re-expand, no retry"
    );
    assert_eq!(tree.generation(), generation);
    let node = tree.find(&alpha).expect("alpha node");
    assert!(node.error.is_some(), "error sticks until a real refresh");
}

/// Files are never expandable (toggle Case A).
#[test]
fn toggling_a_file_is_a_no_op() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let zeta = fx.path("zeta.txt");
    let snapshot = tree.clone();
    assert!(tree.on_toggled(&zeta).is_none());
    assert_eq!(tree, snapshot, "file toggle must not change anything");
}
