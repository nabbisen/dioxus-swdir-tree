//! Feature 11 drag-and-drop — integration tests (S11.9–S11.16).
//!
//! Drive the public drag state machine via `on_drag_msg` and assert on the
//! `is_dragging` / `drag_sources` / `drop_target` accessors and the returned
//! `ItemDragOutcome`. Test names mirror the iced-swdir-tree reference suite.

use dioxus_swdir_tree_core::{
    DropPosition, ItemDragMsg, ItemDragOutcome, ItemNode, ItemTree, NodeId,
    keyboard::{Modifiers, TreeKey},
    selection::SelectionMode,
};

fn nid(n: u64) -> NodeId {
    NodeId(n)
}

/// Build a DnD-enabled tree:
///
/// ```text
/// 0
/// ├── 1
/// │   ├── 11
/// │   └── 12
/// └── 2
///     └── 21
/// ```
fn tree() -> ItemTree<String> {
    fn n(id: u64, children: Vec<ItemNode<String>>) -> ItemNode<String> {
        ItemNode {
            id: NodeId(id),
            data: format!("node {id}"),
            children,
        }
    }
    let mut t = ItemTree::new()
        .with_display(|s: &String| s.clone())
        .with_drag_and_drop(true);
    t.set_tree(n(
        0,
        vec![
            n(1, vec![n(11, vec![]), n(12, vec![])]),
            n(2, vec![n(21, vec![])]),
        ],
    ));
    t
}

fn drag(t: &mut ItemTree<String>, msg: ItemDragMsg) -> ItemDragOutcome {
    t.on_drag_msg(msg)
}

// ── S11.9 — opt-in ────────────────────────────────────────────────────────────

#[test]
fn drag_is_initially_inactive() {
    let t = tree();
    assert!(t.is_drag_and_drop_enabled());
    assert!(!t.is_dragging());
    assert_eq!(t.drop_target(), None);
    assert!(t.drag_sources().is_empty());
}

#[test]
fn drag_disabled_ignores_press() {
    let mut t = tree().with_drag_and_drop(false);
    drag(&mut t, ItemDragMsg::Pressed(nid(11)));
    assert!(
        !t.is_dragging(),
        "press must be a no-op when DnD is off (S11.9)"
    );
}

// ── S11.10 — press starts drag, source set ───────────────────────────────────

#[test]
fn press_starts_drag_with_single_source() {
    let mut t = tree();
    drag(&mut t, ItemDragMsg::Pressed(nid(11)));
    assert!(t.is_dragging());
    assert_eq!(t.drag_sources(), &[nid(11)]);
    assert_eq!(t.drop_target(), None);
}

#[test]
fn pressing_a_selected_node_drags_the_whole_selection_in_tree_order() {
    let mut t = tree();
    t.on_toggled(nid(0));
    t.on_toggled(nid(1));
    // Select 12 then 11 (click order 12, 11).
    t.on_selected(nid(12), SelectionMode::Replace);
    t.on_selected(nid(11), SelectionMode::Toggle);
    drag(&mut t, ItemDragMsg::Pressed(nid(11)));
    // Sources reported in tree pre-order: 11 before 12.
    assert_eq!(t.drag_sources(), &[nid(11), nid(12)]);
}

#[test]
fn pressing_an_unselected_node_drags_only_that_node() {
    let mut t = tree();
    t.on_toggled(nid(0));
    t.on_selected(nid(2), SelectionMode::Replace);
    drag(&mut t, ItemDragMsg::Pressed(nid(11)));
    assert_eq!(t.drag_sources(), &[nid(11)]);
}

// ── S11.12 — hover validity ───────────────────────────────────────────────────

#[test]
fn hover_over_valid_sibling_sets_drop_target() {
    let mut t = tree();
    drag(&mut t, ItemDragMsg::Pressed(nid(11)));
    drag(&mut t, ItemDragMsg::Entered(nid(12), DropPosition::Before));
    assert_eq!(t.drop_target(), Some((nid(12), DropPosition::Before)));
}

#[test]
fn hover_creating_cycle_leaves_target_unset() {
    let mut t = tree();
    drag(&mut t, ItemDragMsg::Pressed(nid(1)));
    // Nesting 1 into its own child 11 is a cycle.
    drag(&mut t, ItemDragMsg::Entered(nid(11), DropPosition::Into));
    assert_eq!(t.drop_target(), None);
}

#[test]
fn exiting_hovered_zone_clears_target() {
    let mut t = tree();
    drag(&mut t, ItemDragMsg::Pressed(nid(11)));
    drag(&mut t, ItemDragMsg::Entered(nid(12), DropPosition::After));
    assert_eq!(t.drop_target(), Some((nid(12), DropPosition::After)));
    drag(&mut t, ItemDragMsg::Exited(nid(12), DropPosition::After));
    assert_eq!(t.drop_target(), None);
}

#[test]
fn exit_of_a_different_zone_does_not_clear_target() {
    let mut t = tree();
    drag(&mut t, ItemDragMsg::Pressed(nid(11)));
    drag(&mut t, ItemDragMsg::Entered(nid(12), DropPosition::Before));
    // Exit a zone we are not hovering — no effect.
    drag(&mut t, ItemDragMsg::Exited(nid(12), DropPosition::After));
    assert_eq!(t.drop_target(), Some((nid(12), DropPosition::Before)));
}

// ── S11.13 — Escape ───────────────────────────────────────────────────────────

#[test]
fn escape_cancels_drag() {
    let mut t = tree();
    drag(&mut t, ItemDragMsg::Pressed(nid(11)));
    assert!(t.is_dragging());
    let ev = t.handle_key(TreeKey::Escape, Modifiers::default());
    assert_eq!(
        ev,
        Some(dioxus_swdir_tree_core::ItemTreeEvent::Drag(
            ItemDragMsg::Cancelled
        ))
    );
    drag(&mut t, ItemDragMsg::Cancelled);
    assert!(!t.is_dragging());
    assert_eq!(t.drop_target(), None);
}

#[test]
fn escape_without_drag_is_unbound() {
    let t = tree();
    assert_eq!(t.handle_key(TreeKey::Escape, Modifiers::default()), None);
}

// ── S11.11 / S11.14 — release semantics ──────────────────────────────────────

#[test]
fn release_over_valid_target_completes_and_clears_state() {
    let mut t = tree();
    drag(&mut t, ItemDragMsg::Pressed(nid(11)));
    drag(&mut t, ItemDragMsg::Entered(nid(12), DropPosition::Before));
    let outcome = drag(&mut t, ItemDragMsg::Released(nid(12), DropPosition::Before));
    assert_eq!(
        outcome,
        ItemDragOutcome::Completed {
            sources: vec![nid(11)],
            target: nid(12),
            position: DropPosition::Before,
        }
    );
    assert!(!t.is_dragging());
    assert_eq!(t.drop_target(), None);
}

#[test]
fn release_on_same_node_is_a_click_and_does_not_mutate_selection() {
    let mut t = tree();
    drag(&mut t, ItemDragMsg::Pressed(nid(11)));
    let outcome = drag(&mut t, ItemDragMsg::Released(nid(11), DropPosition::Into));
    assert_eq!(
        outcome,
        ItemDragOutcome::Clicked(nid(11)),
        "release on press row = click (S11.11)"
    );
    assert!(!t.is_dragging());
    // The widget did NOT mutate selection — the host applies the deferred Selected.
    assert!(t.selected_ids().is_empty());
}

#[test]
fn release_over_nothing_valid_is_a_noop() {
    let mut t = tree();
    drag(&mut t, ItemDragMsg::Pressed(nid(1)));
    // Hover an invalid target (cycle), so hover stays None.
    drag(&mut t, ItemDragMsg::Entered(nid(11), DropPosition::Into));
    let outcome = drag(&mut t, ItemDragMsg::Released(nid(11), DropPosition::Into));
    assert_eq!(outcome, ItemDragOutcome::None);
    assert!(!t.is_dragging());
}

// ── lifecycle robustness ─────────────────────────────────────────────────────

#[test]
fn cancelled_message_clears_state() {
    let mut t = tree();
    drag(&mut t, ItemDragMsg::Pressed(nid(11)));
    drag(&mut t, ItemDragMsg::Cancelled);
    assert!(!t.is_dragging());
}

#[test]
fn stray_entered_without_press_is_a_noop() {
    let mut t = tree();
    drag(&mut t, ItemDragMsg::Entered(nid(12), DropPosition::Before));
    assert!(!t.is_dragging());
    assert_eq!(t.drop_target(), None);
}

// ── S11.16 — drag survives search ─────────────────────────────────────────────

#[test]
fn drag_survives_set_search_query() {
    let mut t = tree();
    drag(&mut t, ItemDragMsg::Pressed(nid(11)));
    assert!(t.is_dragging());
    t.set_search_query("node");
    assert!(t.is_dragging(), "active drag must survive search (S11.16)");
    assert_eq!(t.drag_sources(), &[nid(11)]);
}
