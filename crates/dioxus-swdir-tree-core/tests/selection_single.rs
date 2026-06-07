//! Feature 3 — Single-path selection (specification clauses S3.1–S3.4).
//!
//! `selected_path()` returns `active_path`, not the last element of
//! `selected_paths` (S3.3). Replace on an already-selected row keeps
//! exactly that row selected — no implicit deselect (S3.4).

mod common;

use dioxus_swdir_tree_core::{DirectoryTree, SelectionMode};

use common::fixture;

/// S3.1 — Before any selection gesture the selection is empty.
#[test]
fn s3_1_initial_selection_is_empty() {
    let fx = fixture();
    let tree = DirectoryTree::new(&fx.root);
    assert!(tree.selected_paths().is_empty());
    assert!(tree.selected_path().is_none());
}

/// S3.2 — Replace sets exactly one path as selected and active.
#[test]
fn s3_2_replace_selects_one_path() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    tree.on_selected(&alpha, true, SelectionMode::Replace);

    assert_eq!(tree.selected_paths(), std::slice::from_ref(&alpha));
    assert_eq!(tree.selected_path(), Some(alpha.as_path()));
    assert!(tree.is_selected(&alpha));

    let zeta = fx.path("zeta.txt");
    assert!(!tree.is_selected(&zeta));

    // Node flag must match.
    let node = tree.find(&alpha).unwrap();
    assert!(node.is_selected);
    let node_zeta = tree.find(&zeta).unwrap();
    assert!(!node_zeta.is_selected);
}

/// S3.3 — `selected_path()` is a view onto `active_path`, NOT the last
/// element of `selected_paths`. When the most recently touched row is
/// different from the last pushed path, `selected_path` tracks the
/// former.
#[test]
fn s3_3_selected_path_tracks_active_not_last_pushed() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    let beta = fx.path("beta");

    // Add beta, then alpha with Toggle so selected_paths = [beta, alpha].
    tree.on_selected(&beta, true, SelectionMode::Toggle);
    tree.on_selected(&alpha, true, SelectionMode::Toggle);

    assert_eq!(tree.selected_paths(), [beta.clone(), alpha.clone()]);
    // active_path is the most recently touched: alpha.
    assert_eq!(tree.selected_path(), Some(alpha.as_path()));

    // Now touch beta again (Toggle removes it), but active becomes beta.
    tree.on_selected(&beta, true, SelectionMode::Toggle);
    assert_eq!(tree.selected_path(), Some(beta.as_path()));
    assert!(!tree.is_selected(&beta));
}

/// S3.4 — Replace on an already-selected row still yields exactly
/// that one row; no implicit deselect / re-select cycle.
#[test]
fn s3_4_replace_on_already_selected_row_is_idempotent() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    tree.on_selected(&alpha, true, SelectionMode::Replace);
    tree.on_selected(&alpha, true, SelectionMode::Replace);

    assert_eq!(tree.selected_paths(), std::slice::from_ref(&alpha));
    assert_eq!(tree.selected_path(), Some(alpha.as_path()));
}

/// Selecting a path that has no node in the tree (unloaded deep path)
/// is allowed — the path sits in `selected_paths` authoritatively; the
/// node flag syncs once the path becomes visible.
#[test]
fn selecting_unloaded_path_survives_until_visible() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let inner = fx.path("alpha/inner");
    // `inner` is in the tree but its parent alpha is not yet expanded;
    // the node for `inner` doesn't exist yet.
    assert!(tree.find(&inner).is_none());

    // Select it anyway — authoritative.
    tree.on_selected(&inner, true, SelectionMode::Replace);
    assert!(tree.is_selected(&inner));

    // Now expand alpha: the inner node appears and its flag is synced.
    tree.expand_blocking(&fx.path("alpha")).expect("alpha scan");
    let node = tree.find(&inner).expect("inner now visible");
    assert!(node.is_selected);
}
