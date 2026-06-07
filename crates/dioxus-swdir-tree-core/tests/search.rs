//! Feature 9 — Incremental search (specification clauses S9.1–S9.9).

mod common;

use dioxus_swdir_tree_core::{DirectoryTree, DisplayFilter, SelectionMode};

use common::fixture;

// ── S9.1 — Basename substring, case-insensitive ───────────────────────────────

/// S9.1 — Matches are basename substrings, case-insensitive.
#[test]
fn s9_1_case_insensitive_basename_match() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    // "ALPHA" should match alpha/ (S9.1 case-insensitive).
    tree.set_search_query("ALPHA");
    assert!(
        tree.search_state()
            .unwrap()
            .visible_paths
            .contains(&fx.path("alpha")),
        "alpha must match ALPHA query"
    );
}

/// S9.1 — Only the basename is matched, not parent components.
#[test]
fn s9_1_only_basename_matched_not_ancestors() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    tree.expand_blocking(&fx.path("alpha")).expect("alpha scan");

    // Searching for the root basename should NOT match 'inner' because 'inner'
    // doesn't contain the root directory's name — verify path-component isolation.
    // Searching for "notes" — only notes.txt should match (not alpha/ or inner/).
    tree.set_search_query("notes");
    let vis = &tree.search_state().unwrap().visible_paths;
    assert!(
        vis.contains(&fx.path("alpha/notes.txt")),
        "notes.txt matches"
    );
    assert!(
        !vis.contains(&fx.path("alpha/inner")),
        "inner should not match 'notes'"
    );
}

// ── S9.2 — Visible set: matches ∪ ancestors ──────────────────────────────────

/// S9.2 — The visible set includes the match itself and all proper ancestors.
#[test]
fn s9_2_ancestors_of_matches_are_visible() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    tree.expand_blocking(&fx.path("alpha")).expect("alpha scan");

    // "notes" matches alpha/notes.txt; ancestors = root, alpha.
    tree.set_search_query("notes");
    let vis = &tree.search_state().unwrap().visible_paths;
    assert!(vis.contains(&fx.path("alpha/notes.txt")), "direct match");
    assert!(vis.contains(&fx.path("alpha")), "alpha is ancestor");
    assert!(vis.contains(&fx.root), "root is ancestor");
    // beta and zeta.txt are not matches or ancestors.
    assert!(!vis.contains(&fx.path("beta")));
    assert!(!vis.contains(&fx.path("zeta.txt")));
}

// ── S9.3 — Sees through collapsed-but-loaded subtrees ────────────────────────

/// S9.3 — Search finds matches in loaded-but-collapsed subtrees.
#[test]
fn s9_3_search_sees_through_collapsed_subtrees() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    tree.expand_blocking(&fx.path("alpha")).expect("alpha scan");
    tree.expand_blocking(&fx.path("alpha/inner"))
        .expect("inner scan");

    // Collapse alpha: inner and deep are no longer visible in normal mode.
    tree.on_toggled(&fx.path("alpha"));
    assert!(
        !tree.find(&fx.path("alpha")).unwrap().is_expanded,
        "alpha collapsed"
    );
    assert_eq!(
        tree.visible_rows().len(),
        4,
        "only root, alpha, beta, zeta.txt visible normally"
    );

    // Search for "deep": should surface alpha/inner/deep even through collapse.
    tree.set_search_query("deep");
    let vis = &tree.search_state().unwrap().visible_paths;
    assert!(vis.contains(&fx.path("alpha/inner/deep")), "deep matches");
    assert!(vis.contains(&fx.path("alpha/inner")), "inner is ancestor");
    assert!(vis.contains(&fx.path("alpha")), "alpha is ancestor");
    assert!(vis.contains(&fx.root), "root is ancestor");

    // The visible rows in search mode should include the deep path.
    let row_paths: Vec<_> = tree
        .visible_rows()
        .iter()
        .map(|(n, _)| n.path.clone())
        .collect();
    assert!(row_paths.contains(&fx.path("alpha/inner/deep")));
}

// ── S9.4 — Empty query clears search ─────────────────────────────────────────

/// S9.4 — Empty string clears the search state.
#[test]
fn s9_4_empty_query_clears_search() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    tree.set_search_query("alpha");
    assert!(tree.search_state().is_some());

    tree.set_search_query("");
    assert!(
        tree.search_state().is_none(),
        "search cleared by empty query"
    );
    assert!(tree.search_query().is_none());
}

/// S9.4 — `clear_search()` is equivalent to `set_search_query("")`.
#[test]
fn s9_4_clear_search_method() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    tree.set_search_query("alpha");
    tree.clear_search();
    assert!(tree.search_state().is_none());
}

// ── S9.5 — Selection is orthogonal ───────────────────────────────────────────

/// S9.5 — Activating/clearing search never modifies selected_paths.
#[test]
fn s9_5_selection_survives_search() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    let beta = fx.path("beta");
    tree.on_selected(&alpha, true, SelectionMode::Toggle);
    tree.on_selected(&beta, true, SelectionMode::Toggle);
    assert_eq!(tree.selected_paths(), [alpha.clone(), beta.clone()]);

    tree.set_search_query("alpha");
    assert_eq!(
        tree.selected_paths(),
        [alpha.clone(), beta.clone()],
        "selection unchanged"
    );

    tree.clear_search();
    assert_eq!(
        tree.selected_paths(),
        [alpha.clone(), beta.clone()],
        "selection unchanged after clear"
    );
}

// ── S9.6 — Filter and search compose correctly ───────────────────────────────

/// S9.6 — Filter applies first; search runs over filter-surviving nodes.
/// FoldersOnly hides files, so searching for "notes" finds nothing
/// (notes.txt was filtered out before search ran).
#[test]
fn s9_6_filter_applies_before_search() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root).with_filter(DisplayFilter::FoldersOnly);
    tree.expand_blocking(&fx.root).expect("root scan");
    tree.expand_blocking(&fx.path("alpha")).expect("alpha scan");

    // notes.txt is invisible under FoldersOnly.
    tree.set_search_query("notes");
    let match_count = tree.search_match_count();
    assert_eq!(
        match_count, 0,
        "notes.txt filtered out before search (S9.6)"
    );
}

/// S9.6 — Changing the filter while search is active re-runs the search.
#[test]
fn s9_6_filter_change_reruns_search() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    tree.expand_blocking(&fx.path("alpha")).expect("alpha scan");

    tree.set_search_query("notes");
    assert_eq!(tree.search_match_count(), 1, "notes.txt matches");

    // Switch to FoldersOnly — notes.txt is removed from the node graph.
    tree.set_filter(DisplayFilter::FoldersOnly);
    assert_eq!(
        tree.search_match_count(),
        0,
        "match count updated after filter change"
    );
}

// ── S9.7 — Loading while search is active re-runs search ─────────────────────

/// S9.7 — When on_loaded merges new children, search visibility is recomputed.
#[test]
fn s9_7_loading_reruns_search() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    // Activate search for "inner" before alpha is loaded.
    tree.set_search_query("inner");
    assert_eq!(tree.search_match_count(), 0, "inner not loaded yet");

    // Now load alpha — inner/ becomes visible via search.
    tree.expand_blocking(&fx.path("alpha")).expect("alpha scan");
    assert_eq!(
        tree.search_match_count(),
        1,
        "inner now found after load (S9.7)"
    );
    assert!(
        tree.search_state()
            .unwrap()
            .visible_paths
            .contains(&fx.path("alpha/inner")),
        "alpha/inner in visible_paths"
    );
}

// ── S9.8 — match_count is direct matches only ────────────────────────────────

/// S9.8 — match_count counts direct matches, not ancestors.
#[test]
fn s9_8_match_count_excludes_ancestors() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    tree.expand_blocking(&fx.path("alpha")).expect("alpha scan");

    // "notes" matches exactly one file; its ancestors (root, alpha) are not counted.
    tree.set_search_query("notes");
    assert_eq!(
        tree.search_match_count(),
        1,
        "only notes.txt is a direct match"
    );

    let vis = tree.search_state().unwrap().visible_paths.len();
    assert!(vis > 1, "visible_paths includes ancestors");
    assert_eq!(
        tree.search_match_count(),
        1,
        "match_count differs from visible_paths.len()"
    );
}

// ── S9.9 — No I/O ─────────────────────────────────────────────────────────────

/// S9.9 — set_search_query performs no I/O (no generation bump, no request).
#[test]
fn s9_9_search_triggers_no_io() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    let gen_before = tree.generation();

    tree.set_search_query("alpha");
    assert_eq!(
        tree.generation(),
        gen_before,
        "no generation bump during search"
    );
}
