//! Feature 11 — Generic item tree (specification clauses S11.x).
//!
//! Test names mirror the iced-swdir-tree reference implementation for
//! cross-framework spec alignment.

use dioxus_swdir_tree_core::{
    ItemNode, ItemTree, NodeId,
    item_event::ItemTreeEvent,
    keyboard::{Modifiers, TreeKey},
    selection::SelectionMode,
};

// ── Fixture helpers ───────────────────────────────────────────────────────────

fn id(n: u64) -> NodeId {
    NodeId(n)
}

/// Build a small reference tree:
///
/// ```text
/// root(0)
///   alpha(1)
///     inner(3)
///   beta(2)
/// ```
fn fixture() -> ItemNode<String> {
    ItemNode::branch(
        id(0),
        "root".into(),
        vec![
            ItemNode::branch(
                id(1),
                "alpha".into(),
                vec![ItemNode::leaf(id(3), "inner".into())],
            ),
            ItemNode::leaf(id(2), "beta".into()),
        ],
    )
}

fn tree_with_fixture() -> ItemTree<String> {
    let mut t = ItemTree::new().with_display(|s: &String| s.clone());
    t.set_tree(fixture());
    t
}

fn visible_ids(tree: &ItemTree<String>) -> Vec<NodeId> {
    tree.visible_rows().iter().map(|r| r.id).collect()
}

// ── Construction ──────────────────────────────────────────────────────────────

#[test]
fn new_tree_is_empty() {
    let t: ItemTree<String> = ItemTree::new();
    assert!(t.visible_rows().is_empty());
    assert_eq!(t.node_count(), 0);
}

// ── S11.2 — Expand / collapse / leaf behaviour ────────────────────────────────

#[test]
fn set_tree_populates_visible_root() {
    let t = tree_with_fixture();
    // Root is visible; children are collapsed.
    let ids = visible_ids(&t);
    assert_eq!(ids, vec![id(0)], "only root visible after set_tree");
}

#[test]
fn toggled_expands_branch_node() {
    let mut t = tree_with_fixture();
    t.on_toggled(id(0));
    let ids = visible_ids(&t);
    assert!(ids.contains(&id(1)));
    assert!(ids.contains(&id(2)));
}

#[test]
fn toggled_collapses_expanded_branch() {
    let mut t = tree_with_fixture();
    t.on_toggled(id(0));
    t.on_toggled(id(0));
    assert_eq!(visible_ids(&t), vec![id(0)]);
}

#[test]
fn toggled_on_leaf_is_noop() {
    let mut t = tree_with_fixture();
    t.on_toggled(id(0)); // expand root
    let before = visible_ids(&t);
    t.on_toggled(id(2)); // beta is a leaf
    assert_eq!(visible_ids(&t), before);
}

// ── S11.7 → S6.1 analogue — Replace and Toggle selection ─────────────────────

#[test]
fn replace_sets_exactly_one_id() {
    let mut t = tree_with_fixture();
    t.on_toggled(id(0));
    t.on_selected(id(1), SelectionMode::Replace);
    assert_eq!(t.selected_ids(), &[id(1)]);
    assert!(t.is_selected(id(1)));
    assert!(!t.is_selected(id(2)));
}

#[test]
fn toggle_adds_then_removes() {
    let mut t = tree_with_fixture();
    t.on_toggled(id(0));
    t.on_selected(id(1), SelectionMode::Toggle);
    t.on_selected(id(2), SelectionMode::Toggle);
    assert_eq!(t.selected_ids().len(), 2);
    t.on_selected(id(1), SelectionMode::Toggle);
    assert_eq!(t.selected_ids(), &[id(2)]);
}

// ── S11.8 — ExtendRange ───────────────────────────────────────────────────────

#[test]
fn extend_range_covers_visible_rows() {
    let mut t = tree_with_fixture();
    t.on_toggled(id(0)); // expand root so alpha and beta are visible
    t.on_selected(id(1), SelectionMode::Replace); // anchor = alpha
    t.on_selected(id(2), SelectionMode::ExtendRange); // extend to beta
    // visible rows: root(0), alpha(1), beta(2) — range [1..2] from anchor
    assert!(t.is_selected(id(1)));
    assert!(t.is_selected(id(2)));
}

// ── S11.4 — set_tree preserves state for surviving keys ──────────────────────

#[test]
fn set_tree_preserves_expansion_for_surviving_keys() {
    let mut t = tree_with_fixture();
    t.on_toggled(id(0)); // expand root
    assert!(t.is_expanded(id(0)).unwrap());

    // Replace with an updated tree that keeps the same IDs.
    t.set_tree(fixture());
    assert!(
        t.is_expanded(id(0)).unwrap(),
        "expansion must survive set_tree for surviving key (S11.4)"
    );
}

#[test]
fn set_tree_preserves_selection_for_surviving_keys() {
    let mut t = tree_with_fixture();
    t.on_toggled(id(0));
    t.on_selected(id(1), SelectionMode::Replace);
    t.set_tree(fixture());
    assert!(t.is_selected(id(1)), "selection survives set_tree (S11.4)");
}

#[test]
fn set_tree_drops_selection_for_removed_keys() {
    let mut t = tree_with_fixture();
    t.on_toggled(id(0));
    t.on_selected(id(2), SelectionMode::Replace);

    // New tree omits beta (id 2).
    let new_root = ItemNode::branch(
        id(0),
        "root".into(),
        vec![ItemNode::leaf(id(1), "alpha".into())],
    );
    t.set_tree(new_root);
    assert!(
        !t.is_selected(id(2)),
        "removed key must drop from selection (S11.4)"
    );
    assert!(t.selected_ids().is_empty());
}

#[test]
fn set_tree_resets_active_id_on_removal() {
    let mut t = tree_with_fixture();
    t.on_selected(id(0), SelectionMode::Replace);
    assert_eq!(t.active_id(), Some(id(0)));

    let new_root = ItemNode::leaf(id(99), "new".into());
    t.set_tree(new_root);
    assert_eq!(
        t.active_id(),
        None,
        "active_id dropped when key disappears (S11.4)"
    );
}

// ── S11.5 — Position changes preserve state ───────────────────────────────────

#[test]
fn set_tree_preserves_expansion_regardless_of_position_change() {
    let mut t = tree_with_fixture();
    t.on_toggled(id(0)); // expand root
    t.on_toggled(id(1)); // expand alpha

    assert!(t.is_expanded(id(1)).unwrap());

    // Swap alpha and beta in the new tree (position change: alpha becomes 2nd child).
    let new_root = ItemNode::branch(
        id(0),
        "root".into(),
        vec![
            ItemNode::leaf(id(2), "beta".into()),
            ItemNode::branch(
                id(1),
                "alpha".into(),
                vec![ItemNode::leaf(id(3), "inner".into())],
            ),
        ],
    );
    t.set_tree(new_root);
    assert!(
        t.is_expanded(id(1)).unwrap(),
        "expansion must survive position change (S11.5)"
    );
}

// ── S11.6 — Search ────────────────────────────────────────────────────────────

#[test]
fn search_filters_to_matches_and_ancestors() {
    let mut t = tree_with_fixture();
    t.on_toggled(id(0));
    t.on_toggled(id(1)); // expand alpha so inner is loaded
    t.on_toggled(id(1)); // collapse alpha back — search should still find inner

    t.set_search_query("inner");
    let rows = t.visible_rows();
    let ids: Vec<NodeId> = rows.iter().map(|r| r.id).collect();
    assert!(ids.contains(&id(3)), "inner matches");
    assert!(ids.contains(&id(1)), "alpha is ancestor of inner");
    assert!(ids.contains(&id(0)), "root is ancestor of inner");
    assert!(!ids.contains(&id(2)), "beta is unrelated");
}

#[test]
fn search_case_insensitive() {
    let mut t = tree_with_fixture();
    t.set_search_query("ALPHA");
    let ids: Vec<NodeId> = t.visible_rows().iter().map(|r| r.id).collect();
    assert!(ids.contains(&id(1)));
}

#[test]
fn empty_query_clears_search() {
    let mut t = tree_with_fixture();
    t.set_search_query("alpha");
    assert!(t.search_state().is_some());
    t.set_search_query("");
    assert!(t.search_state().is_none());
}

#[test]
fn search_selection_survives() {
    let mut t = tree_with_fixture();
    t.on_toggled(id(0));
    t.on_selected(id(1), SelectionMode::Toggle);
    t.on_selected(id(2), SelectionMode::Toggle);
    t.set_search_query("alpha");
    assert_eq!(
        t.selected_ids().len(),
        2,
        "search must not clear selection (S11.7→S9.5)"
    );
}

#[test]
fn clear_search_restores_normal_view() {
    let mut t = tree_with_fixture();
    t.set_search_query("alpha");
    t.clear_search();
    assert!(t.search_state().is_none());
    // Back to normal view: only root visible (root is collapsed).
    assert_eq!(visible_ids(&t), vec![id(0)]);
}

// ── S11.7 — Keyboard (S4.x analogues) ────────────────────────────────────────

#[test]
fn arrow_down_moves_selection() {
    let mut t = tree_with_fixture();
    t.on_toggled(id(0));
    t.on_selected(id(0), SelectionMode::Replace); // active = root

    let ev = t.handle_key(TreeKey::Down, Modifiers::default());
    assert!(
        matches!(ev, Some(ItemTreeEvent::Selected(nid, SelectionMode::Replace)) if nid == id(1)),
        "Down from root should move to alpha, got {ev:?}"
    );
}

#[test]
fn escape_is_unbound_when_no_drag() {
    let t = tree_with_fixture();
    assert_eq!(
        t.handle_key(TreeKey::Escape, Modifiers::default()),
        None,
        "Escape is unbound in ItemTree (no drag)"
    );
}

// ── S11.7 → S10.3 analogue — Icon theme ──────────────────────────────────────

#[test]
fn with_icon_theme_accepts_arc_dyn() {
    use dioxus_swdir_tree_core::icon::{IconRole, IconSpec, IconTheme, UnicodeTheme};
    use std::sync::Arc;

    // Just verify Arc<dyn IconTheme> is object-safe and usable — the actual
    // theme injection lives in the view crate.
    let theme: Arc<dyn IconTheme> = Arc::new(UnicodeTheme);
    let spec: IconSpec = theme.glyph(IconRole::FolderClosed);
    assert!(!spec.glyph.is_empty());
}
