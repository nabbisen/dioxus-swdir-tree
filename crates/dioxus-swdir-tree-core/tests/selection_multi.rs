//! Feature 6 — Multi-select (specification clauses S6.1–S6.5).
//!
//! S2.6 (filter change preserves selection, previously deferred) is now
//! covered as part of S6.4/S6.5.

mod common;

use dioxus_swdir_tree_core::{DirectoryTree, DisplayFilter, SelectionMode};

use common::fixture;

/// S6.1 — `selected_paths` is insertion-ordered, duplicate-free.
#[test]
fn s6_1_insertion_order_and_no_duplicates() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    let beta = fx.path("beta");
    let zeta = fx.path("zeta.txt");

    tree.on_selected(&alpha, true, SelectionMode::Toggle);
    tree.on_selected(&beta, true, SelectionMode::Toggle);
    tree.on_selected(&zeta, false, SelectionMode::Toggle);

    assert_eq!(
        tree.selected_paths(),
        [alpha.clone(), beta.clone(), zeta.clone()]
    );

    // Re-adding alpha after removing it — order reflects re-insertion.
    tree.on_selected(&alpha, true, SelectionMode::Toggle); // remove
    tree.on_selected(&alpha, true, SelectionMode::Toggle); // re-add at end
    assert_eq!(
        tree.selected_paths(),
        [beta.clone(), zeta.clone(), alpha.clone()]
    );
}

/// S6.2 — Toggle adds absent paths, removes present paths.
#[test]
fn s6_2_toggle_add_and_remove() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    assert!(!tree.is_selected(&alpha));

    tree.on_selected(&alpha, true, SelectionMode::Toggle);
    assert!(tree.is_selected(&alpha));
    assert!(tree.find(&alpha).unwrap().is_selected);

    tree.on_selected(&alpha, true, SelectionMode::Toggle);
    assert!(!tree.is_selected(&alpha));
    assert!(!tree.find(&alpha).unwrap().is_selected);
}

/// S6.3 — ExtendRange: the anchor does NOT move across repeated
/// Shift-clicks; ranges grow or shrink from the same pivot.
#[test]
fn s6_3_extend_range_anchor_does_not_move() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    // Rows (depth 1): root, alpha, beta, zeta.txt  — indices 0,1,2,3.
    let root = fx.root.clone();
    let alpha = fx.path("alpha");
    let beta = fx.path("beta");
    let zeta = fx.path("zeta.txt");

    // Set anchor at alpha.
    tree.on_selected(&alpha, true, SelectionMode::Replace);
    assert_eq!(tree.selected_paths(), std::slice::from_ref(&alpha));

    // Extend to beta → range = [alpha, beta].
    tree.on_selected(&beta, true, SelectionMode::ExtendRange);
    assert_eq!(tree.selected_paths(), [alpha.clone(), beta.clone()]);

    // Extend further to zeta → range = [alpha, beta, zeta].
    tree.on_selected(&zeta, false, SelectionMode::ExtendRange);
    assert_eq!(
        tree.selected_paths(),
        [alpha.clone(), beta.clone(), zeta.clone()]
    );

    // Shrink back to root → range = [root, alpha] (anchor still alpha,
    // so from root..=alpha upward).
    tree.on_selected(&root, true, SelectionMode::ExtendRange);
    assert_eq!(tree.selected_paths(), [root.clone(), alpha.clone()]);

    // Anchor is still alpha, not zeta or root.
    // (We verify by extending to beta again and checking the range.)
    tree.on_selected(&beta, true, SelectionMode::ExtendRange);
    assert_eq!(tree.selected_paths(), [alpha.clone(), beta.clone()]);
}

/// S6.3 supplemental — ExtendRange with no anchor behaves as Replace.
#[test]
fn s6_3_extend_range_without_anchor_is_replace() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let beta = fx.path("beta");
    tree.on_selected(&beta, true, SelectionMode::ExtendRange);
    assert_eq!(tree.selected_paths(), std::slice::from_ref(&beta));
}

/// S6.4 — Selection set is not changed by `set_filter`; only the
/// per-node flags are re-derived. A path hidden by the new filter stays
/// selected (the authoritative vec is unchanged) and its node's flag
/// is restored when the path becomes visible again (S2.6).
#[test]
fn s6_4_selection_survives_filter_change() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root).with_filter(DisplayFilter::AllIncludingHidden);
    tree.expand_blocking(&fx.root).expect("root scan");

    let hidden_dir = fx.path(".hidden_dir");
    tree.on_selected(&hidden_dir, true, SelectionMode::Replace);
    assert!(tree.is_selected(&hidden_dir));
    assert!(tree.find(&hidden_dir).unwrap().is_selected);

    // Switch to FoldersOnly — hidden_dir disappears (it IS a folder but
    // it's hidden; FoldersOnly hides it too, per feature spec S2.2).
    tree.set_filter(DisplayFilter::FilesAndFolders);
    assert!(tree.find(&hidden_dir).is_none(), "node hidden by filter");
    assert!(
        tree.is_selected(&hidden_dir),
        "authoritative path survives filter change"
    );

    // Restore AllIncludingHidden — node reappears and flag is set.
    tree.set_filter(DisplayFilter::AllIncludingHidden);
    let node = tree.find(&hidden_dir).expect("node visible again");
    assert!(
        node.is_selected,
        "flag re-synced when node becomes visible again"
    );
}

/// S6.5 — Selection survives `on_loaded` merges. When a scan for a
/// subdirectory completes, the flags for other nodes already visible
/// are re-synced correctly (not cleared), and any selected path that
/// was invisible (unloaded parent) gains its flag once it becomes a
/// loaded node.
#[test]
fn s6_5_selection_survives_on_loaded_merge() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("initial root scan");

    let alpha = fx.path("alpha");
    let beta = fx.path("beta");
    let inner = fx.path("alpha/inner");

    // Select alpha, beta, and inner (inner is not yet in the tree).
    tree.on_selected(&alpha, true, SelectionMode::Toggle);
    tree.on_selected(&beta, true, SelectionMode::Toggle);
    tree.on_selected(&inner, true, SelectionMode::Toggle);
    assert_eq!(
        tree.selected_paths(),
        [alpha.clone(), beta.clone(), inner.clone()]
    );
    assert!(tree.find(&inner).is_none(), "inner not yet loaded");

    // Expand alpha: the `on_loaded` merge for alpha must
    //   (a) not disturb the alpha / beta selection, and
    //   (b) sync inner's flag now that the node exists.
    let request = tree
        .on_toggled(&alpha)
        .expect("alpha is unloaded; scan required");
    let payload = crate::common::fixture_scan_run(&tree, &request);
    let outcome = tree.on_loaded(payload);
    assert!(outcome.accepted);

    // (a) existing selections intact.
    assert_eq!(
        tree.selected_paths(),
        [alpha.clone(), beta.clone(), inner.clone()]
    );
    assert!(tree.find(&alpha).unwrap().is_selected);
    assert!(tree.find(&beta).unwrap().is_selected);
    // (b) newly visible inner node has its flag set.
    let inner_node = tree.find(&inner).expect("inner now visible");
    assert!(inner_node.is_selected);
}

/// Range selection works correctly over rows from collapsed subtrees:
/// the range uses `visible_rows()` order, so collapsed children are
/// not between their parent and the next sibling.
#[test]
fn extend_range_respects_visible_rows_skipping_collapsed_children() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    tree.expand_blocking(&fx.path("alpha")).expect("alpha scan");

    // Rows: root(0), alpha(1), inner(2), notes.txt(3), beta(4), zeta.txt(5)
    let root = fx.root.clone();
    let beta = fx.path("beta");

    // Collapse alpha back so its children are hidden.
    tree.on_toggled(&fx.path("alpha"));
    // Rows now: root(0), alpha(1), beta(2), zeta.txt(3)

    tree.on_selected(&root, true, SelectionMode::Replace);
    tree.on_selected(&beta, true, SelectionMode::ExtendRange);

    // Range is root, alpha, beta — collapsed alpha is still a visible row.
    assert_eq!(
        tree.selected_paths(),
        [root.clone(), fx.path("alpha"), beta.clone()]
    );
    // inner and notes.txt are NOT in the range (they're hidden).
    assert!(!tree.is_selected(&fx.path("alpha/inner")));
    assert!(!tree.is_selected(&fx.path("alpha/notes.txt")));
}
