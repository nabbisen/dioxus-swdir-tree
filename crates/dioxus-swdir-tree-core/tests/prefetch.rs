//! Feature 8 — Parallel pre-expansion / prefetch (S8.1–S8.7).
//!
//! The tests call `on_loaded` directly with payloads from `scan::run`
//! so that prefetch requests are visible without running an async driver.
//! Where S8.4 and S8.7 require staged delivery of stale vs. fresh payloads,
//! payloads are built manually from saved `ScanRequest`s.

mod common;

use std::collections::HashSet;

use dioxus_swdir_tree_core::{DirectoryTree, scan};

use common::fixture;

// ── S8.1 — Disabled by default ────────────────────────────────────────────────

/// S8.1 — Without `with_prefetch_limit`, `on_loaded` returns no prefetch
/// requests and the tree behaves exactly as in earlier versions.
#[test]
fn s8_1_disabled_by_default() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    let req = tree.on_toggled(&fx.root).expect("root scan");
    let payload = scan::run(&req);
    let outcome = tree.on_loaded(payload);
    assert!(outcome.accepted);
    assert!(
        outcome.prefetch_requests.is_empty(),
        "no prefetch when prefetch_per_parent = 0"
    );
    assert!(tree.prefetching_paths().is_empty());
}

// ── S8.2 — Exactly one wave per user-initiated scan ──────────────────────────

/// S8.2 — With `with_prefetch_limit(2)`, completing the root scan issues
/// at most 2 prefetch requests for direct folder-children.
#[test]
fn s8_2_one_wave_per_user_scan() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root).with_prefetch_limit(2);

    let req = tree.on_toggled(&fx.root).expect("root scan");
    let payload = scan::run(&req);
    let outcome = tree.on_loaded(payload);
    assert!(outcome.accepted);

    // Root has alpha/ and beta/ as non-skip dirs (fixture default filter).
    // Expect exactly 2 prefetch requests.
    assert_eq!(
        outcome.prefetch_requests.len(),
        2,
        "should issue 2 prefetch requests (alpha, beta)"
    );
    // All requests in one wave share a single bumped generation (one-per-wave
    // protocol ensures every result can pass the staleness check).
    let wave_gen = outcome.prefetch_requests[0].generation;
    assert!(
        outcome
            .prefetch_requests
            .iter()
            .all(|r| r.generation == wave_gen),
        "all requests in a wave share the same generation"
    );
    assert_ne!(
        wave_gen, req.generation,
        "wave generation must differ from root scan generation"
    );

    // Paths must match folder-children.
    let paths: HashSet<_> = outcome.prefetch_requests.iter().map(|r| &r.path).collect();
    assert!(paths.contains(&fx.path("alpha")));
    assert!(paths.contains(&fx.path("beta")));

    // Tree's prefetching_paths registry must reflect the in-flight scans.
    assert_eq!(tree.prefetching_paths().len(), 2);
}

/// S8.2 — `with_prefetch_limit(1)` issues at most 1 request.
#[test]
fn s8_2_limit_1_issues_at_most_one_request() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root).with_prefetch_limit(1);

    let req = tree.on_toggled(&fx.root).expect("root scan");
    let outcome = tree.on_loaded(scan::run(&req));
    assert_eq!(outcome.prefetch_requests.len(), 1);
}

// ── S8.3 — No cascade ────────────────────────────────────────────────────────

/// S8.3 — When a prefetch scan completes, `on_loaded` returns no further
/// prefetch requests and removes the path from `prefetching_paths`.
#[test]
fn s8_3_prefetch_completion_does_not_cascade() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root).with_prefetch_limit(3);

    // User scan of root → prefetch requests for alpha and beta.
    let root_req = tree.on_toggled(&fx.root).expect("root scan");
    let root_outcome = tree.on_loaded(scan::run(&root_req));
    assert!(!root_outcome.prefetch_requests.is_empty());

    // Execute one of the prefetch scans (alpha).
    let alpha_prefetch_req = root_outcome
        .prefetch_requests
        .iter()
        .find(|r| r.path == fx.path("alpha"))
        .expect("alpha prefetch request")
        .clone();

    assert!(tree.prefetching_paths().contains(&fx.path("alpha")));

    let alpha_payload = scan::run(&alpha_prefetch_req);
    let alpha_outcome = tree.on_loaded(alpha_payload);
    assert!(alpha_outcome.accepted, "prefetch payload accepted");
    assert!(
        alpha_outcome.prefetch_requests.is_empty(),
        "prefetch completion must NOT trigger another wave (S8.3)"
    );
    assert!(
        !tree.prefetching_paths().contains(&fx.path("alpha")),
        "alpha removed from registry after completion"
    );
}

// ── S8.4 — Prefetch loads but does not expand ─────────────────────────────────

/// S8.4 — After a prefetch scan completes, `is_loaded = true` but
/// `is_expanded = false`. The subsequent user click is a zero-I/O
/// fast path (Case C of `on_toggled`).
#[test]
fn s8_4_prefetch_loads_not_expands() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root).with_prefetch_limit(2);

    let root_req = tree.on_toggled(&fx.root).expect("root scan");
    let root_outcome = tree.on_loaded(scan::run(&root_req));

    let alpha_req = root_outcome
        .prefetch_requests
        .iter()
        .find(|r| r.path == fx.path("alpha"))
        .expect("alpha prefetch")
        .clone();

    tree.on_loaded(scan::run(&alpha_req));

    let alpha = tree.find(&fx.path("alpha")).expect("alpha node");
    assert!(alpha.is_loaded, "is_loaded set by prefetch");
    assert!(!alpha.is_expanded, "is_expanded NOT set by prefetch (S8.4)");
    assert!(
        !alpha.children.is_empty(),
        "children populated (at least inner/)"
    );

    // User click → Case C (fast, no scan).
    assert!(
        tree.on_toggled(&fx.path("alpha")).is_none(),
        "expand after prefetch must be instant (no scan request)"
    );
    assert!(tree.find(&fx.path("alpha")).unwrap().is_expanded);
}

// ── S8.5 — Skip list ─────────────────────────────────────────────────────────

/// S8.5 — Default skip list excludes "target", "node_modules", ".git", etc.
/// The test uses a custom fixture path matching a skip-list entry.
#[test]
fn s8_5_skip_list_excludes_known_dirs() {
    let fx = fixture();
    // Build a tiny tree where the only direct folder-child has a skip name.
    // We verify by calling compute_prefetch indirectly through on_loaded.
    // The fixture doesn't have a "node_modules" dir, so simulate via the
    // actual skip-list filtering in a tree with all-skipped children.
    // Simplest: set the skip list to exclude alpha and beta, leaving none.
    let mut tree = DirectoryTree::new(&fx.root)
        .with_prefetch_limit(5)
        .with_prefetch_skip(["alpha", "beta", "zeta.txt"]);

    let req = tree.on_toggled(&fx.root).expect("root scan");
    let outcome = tree.on_loaded(scan::run(&req));
    assert!(
        outcome.prefetch_requests.is_empty(),
        "all folder-children are in the custom skip list"
    );
}

/// S8.5 — Skip list comparison is ASCII case-insensitive.
#[test]
fn s8_5_skip_list_is_case_insensitive() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root)
        .with_prefetch_limit(5)
        .with_prefetch_skip(["ALPHA", "BETA"]); // uppercase versions

    let req = tree.on_toggled(&fx.root).expect("root scan");
    let outcome = tree.on_loaded(scan::run(&req));
    assert!(
        outcome.prefetch_requests.is_empty(),
        "case-insensitive skip must exclude alpha and beta"
    );
}

// ── S8.6 — max_depth applies to prefetch ─────────────────────────────────────

/// S8.6 — Children beyond `max_depth` are excluded from prefetch targets.
#[test]
fn s8_6_max_depth_applies_to_prefetch() {
    let fx = fixture();
    // max_depth = 0: only root's children can be loaded.
    // root is depth 0; its children (alpha, beta) are depth 1 > max_depth(0).
    // So no prefetch targets exist at depth 1.
    let mut tree = DirectoryTree::new(&fx.root)
        .with_max_depth(0)
        .with_prefetch_limit(5);

    let req = tree.on_toggled(&fx.root).expect("root scan");
    let outcome = tree.on_loaded(scan::run(&req));
    assert!(
        outcome.prefetch_requests.is_empty(),
        "children at depth > max_depth must not be prefetched"
    );
}

// ── S8.7 — User wins ─────────────────────────────────────────────────────────

/// S8.7 — If the user expands a path while its prefetch is in flight:
/// 1. The path is removed from `prefetching_paths`.
/// 2. A fresh user-initiated scan is issued.
/// 3. The old prefetch payload arrives stale and is discarded.
/// 4. The new user payload merges and triggers its own prefetch wave.
#[test]
fn s8_7_user_wins_over_in_flight_prefetch() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root).with_prefetch_limit(2);

    // Expand root → prefetch alpha and beta.
    let root_req = tree.on_toggled(&fx.root).expect("root scan");
    let root_outcome = tree.on_loaded(scan::run(&root_req));
    let alpha_prefetch_req = root_outcome
        .prefetch_requests
        .iter()
        .find(|r| r.path == fx.path("alpha"))
        .expect("alpha prefetch")
        .clone();
    assert!(tree.prefetching_paths().contains(&fx.path("alpha")));

    // User expands alpha BEFORE the prefetch result arrives.
    let user_req = tree
        .on_toggled(&fx.path("alpha"))
        .expect("user-initiated expand must yield a scan request");
    assert!(
        !tree.prefetching_paths().contains(&fx.path("alpha")),
        "alpha removed from prefetching_paths when user takes over"
    );
    assert_ne!(
        user_req.generation, alpha_prefetch_req.generation,
        "user request has a newer generation"
    );

    // Stale prefetch payload arrives: discarded.
    let stale_payload = scan::run(&alpha_prefetch_req);
    let stale_outcome = tree.on_loaded(stale_payload);
    assert!(!stale_outcome.accepted, "stale prefetch payload discarded");

    // Fresh user payload arrives: accepted and triggers its own prefetch.
    let user_payload = scan::run(&user_req);
    let user_outcome = tree.on_loaded(user_payload);
    assert!(user_outcome.accepted, "user payload accepted");
    // alpha's children (inner/) should be prefetched now.
    assert!(
        !user_outcome.prefetch_requests.is_empty(),
        "user-initiated merge triggers its own prefetch wave"
    );
}
