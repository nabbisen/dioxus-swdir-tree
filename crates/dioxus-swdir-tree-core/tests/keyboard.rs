//! Feature 4 — Keyboard navigation (specification clauses S4.1–S4.10).
//!
//! All tests call `handle_key` on an immutable snapshot of the tree
//! produced by the shared on-disk fixture. `handle_key` never mutates;
//! the caller dispatches the returned event through `on_toggled` /
//! `on_selected` to advance state.

mod common;

use dioxus_swdir_tree_core::event::DirectoryTreeEvent;
use dioxus_swdir_tree_core::keyboard::{Modifiers, TreeKey, handle_key};
use dioxus_swdir_tree_core::{DirectoryTree, SelectionMode};

use common::fixture;

// ── Fixture helpers ───────────────────────────────────────────────────────────

fn no_mod() -> Modifiers {
    Modifiers::default()
}

fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ctrl: false,
    }
}

fn ctrl() -> Modifiers {
    Modifiers {
        shift: false,
        ctrl: true,
    }
}

/// Build the standard expanded test tree.
///
/// Visible rows (default FilesAndFolders filter):
/// ```text
/// 0  <root>
/// 1    alpha/    (expanded)
/// 2      inner/
/// 3      notes.txt
/// 4    beta/
/// 5    zeta.txt
/// ```
fn expanded_tree(root: &std::path::Path) -> DirectoryTree {
    let mut tree = DirectoryTree::new(root);
    tree.expand_blocking(root).expect("root scan");
    tree.expand_blocking(&root.join("alpha"))
        .expect("alpha scan");
    tree
}

// ── S4.1 — Up / Down move one row ────────────────────────────────────────────

/// S4.1 — Down moves to the next row; Up moves to the previous row.
#[test]
fn s4_1_up_down_move_one_row() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("alpha"), true, SelectionMode::Replace);

    let ev = handle_key(&tree, TreeKey::Down, no_mod()).expect("Down from alpha");
    assert!(
        matches!(&ev, DirectoryTreeEvent::Selected { path, mode: SelectionMode::Replace, .. } if path == &fx.path("alpha/inner"))
    );

    tree.on_selected(&fx.path("alpha/inner"), true, SelectionMode::Replace);
    let ev = handle_key(&tree, TreeKey::Up, no_mod()).expect("Up from inner");
    assert!(
        matches!(&ev, DirectoryTreeEvent::Selected { path, mode: SelectionMode::Replace, .. } if path == &fx.path("alpha"))
    );
}

/// S4.1 — No-wrap at the top boundary.
#[test]
fn s4_1_no_wrap_at_top() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.root, true, SelectionMode::Replace);
    assert!(handle_key(&tree, TreeKey::Up, no_mod()).is_none());
}

/// S4.1 — No-wrap at the bottom boundary.
#[test]
fn s4_1_no_wrap_at_bottom() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("zeta.txt"), false, SelectionMode::Replace);
    assert!(handle_key(&tree, TreeKey::Down, no_mod()).is_none());
}

// ── S4.2 — Shift+Up / Shift+Down extend the range ────────────────────────────

/// S4.2 — Shift+Down produces ExtendRange.
#[test]
fn s4_2_shift_down_extends_range() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("alpha"), true, SelectionMode::Replace);

    let ev = handle_key(&tree, TreeKey::Down, shift()).expect("Shift+Down");
    assert!(
        matches!(&ev, DirectoryTreeEvent::Selected { path, mode: SelectionMode::ExtendRange, .. } if path == &fx.path("alpha/inner"))
    );
}

/// S4.2 — Shift+Up produces ExtendRange.
#[test]
fn s4_2_shift_up_extends_range() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("alpha/inner"), true, SelectionMode::Replace);

    let ev = handle_key(&tree, TreeKey::Up, shift()).expect("Shift+Up");
    assert!(
        matches!(&ev, DirectoryTreeEvent::Selected { path, mode: SelectionMode::ExtendRange, .. } if path == &fx.path("alpha"))
    );
}

/// S4.2 — Shift+Down no-wraps at the bottom.
#[test]
fn s4_2_shift_down_no_wrap_at_bottom() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("zeta.txt"), false, SelectionMode::Replace);
    assert!(handle_key(&tree, TreeKey::Down, shift()).is_none());
}

// ── S4.3 — Home / End jump to first / last row ───────────────────────────────

/// S4.3 — Home jumps to the first row.
#[test]
fn s4_3_home_jumps_to_first_row() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("beta"), true, SelectionMode::Replace);

    let ev = handle_key(&tree, TreeKey::Home, no_mod()).expect("Home");
    assert!(
        matches!(&ev, DirectoryTreeEvent::Selected { path, mode: SelectionMode::Replace, .. } if path == &fx.root)
    );
}

/// S4.3 — End jumps to the last row.
#[test]
fn s4_3_end_jumps_to_last_row() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("alpha"), true, SelectionMode::Replace);

    let ev = handle_key(&tree, TreeKey::End, no_mod()).expect("End");
    assert!(
        matches!(&ev, DirectoryTreeEvent::Selected { path, mode: SelectionMode::Replace, .. } if path == &fx.path("zeta.txt"))
    );
}

/// S4.3 — Home works even when active_path is not visible.
#[test]
fn s4_3_home_works_without_active_path() {
    let fx = fixture();
    let tree = expanded_tree(&fx.root); // no selection yet
    let ev = handle_key(&tree, TreeKey::Home, no_mod()).expect("Home without active");
    assert!(matches!(&ev, DirectoryTreeEvent::Selected { path, .. } if path == &fx.root));
}

// ── S4.4 — Shift+Home / Shift+End extend to first / last ─────────────────────

/// S4.4 — Shift+Home extends range to the first row.
#[test]
fn s4_4_shift_home_extends_to_first() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("beta"), true, SelectionMode::Replace);

    let ev = handle_key(&tree, TreeKey::Home, shift()).expect("Shift+Home");
    assert!(
        matches!(&ev, DirectoryTreeEvent::Selected { path, mode: SelectionMode::ExtendRange, .. } if path == &fx.root)
    );
}

/// S4.4 — Shift+End extends range to the last row.
#[test]
fn s4_4_shift_end_extends_to_last() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("alpha"), true, SelectionMode::Replace);

    let ev = handle_key(&tree, TreeKey::End, shift()).expect("Shift+End");
    assert!(
        matches!(&ev, DirectoryTreeEvent::Selected { path, mode: SelectionMode::ExtendRange, .. } if path == &fx.path("zeta.txt"))
    );
}

// ── S4.5 — Enter toggles the active directory ────────────────────────────────

/// S4.5 — Enter on an expanded directory produces Toggled (collapse).
#[test]
fn s4_5_enter_on_expanded_dir_toggles() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("alpha"), true, SelectionMode::Replace);

    let ev = handle_key(&tree, TreeKey::Enter, no_mod()).expect("Enter on expanded dir");
    assert!(matches!(&ev, DirectoryTreeEvent::Toggled(p) if p == &fx.path("alpha")));
}

/// S4.5 — Enter on a file is a no-op.
#[test]
fn s4_5_enter_on_file_is_noop() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("zeta.txt"), false, SelectionMode::Replace);
    assert!(handle_key(&tree, TreeKey::Enter, no_mod()).is_none());
}

/// S4.5 — Enter on a collapsed directory produces Toggled (expand).
#[test]
fn s4_5_enter_on_collapsed_dir_toggles() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    tree.on_selected(&fx.path("alpha"), true, SelectionMode::Replace);
    // alpha is collapsed here (not expanded)
    assert!(!tree.find(&fx.path("alpha")).unwrap().is_expanded);

    let ev = handle_key(&tree, TreeKey::Enter, no_mod()).expect("Enter on collapsed dir");
    assert!(matches!(&ev, DirectoryTreeEvent::Toggled(p) if p == &fx.path("alpha")));
}

// ── S4.6 / S4.7 — Space and Ctrl+Space toggle-select ────────────────────────

/// S4.6 — Space toggle-selects the active row.
#[test]
fn s4_6_space_toggle_selects_active() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("beta"), true, SelectionMode::Replace);

    let ev = handle_key(&tree, TreeKey::Space, no_mod()).expect("Space");
    assert!(
        matches!(&ev, DirectoryTreeEvent::Selected { path, mode: SelectionMode::Toggle, .. } if path == &fx.path("beta"))
    );
}

/// S4.7 — Ctrl+Space produces the same Toggle event as plain Space.
#[test]
fn s4_7_ctrl_space_same_as_space() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("beta"), true, SelectionMode::Replace);

    let plain = handle_key(&tree, TreeKey::Space, no_mod());
    let ctrl_space = handle_key(&tree, TreeKey::Space, ctrl());
    assert_eq!(plain, ctrl_space);
}

// ── S4.8 — Left: collapse or move to parent ──────────────────────────────────

/// S4.8 — Left on an expanded directory collapses it.
#[test]
fn s4_8_left_on_expanded_dir_collapses() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("alpha"), true, SelectionMode::Replace);

    let ev = handle_key(&tree, TreeKey::Left, no_mod()).expect("Left on expanded alpha");
    assert!(matches!(&ev, DirectoryTreeEvent::Toggled(p) if p == &fx.path("alpha")));
}

/// S4.8 — Left on a collapsed directory moves to its parent.
#[test]
fn s4_8_left_on_collapsed_dir_moves_to_parent() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    // alpha is NOT expanded — collapsed dir
    tree.on_selected(&fx.path("alpha"), true, SelectionMode::Replace);

    let ev = handle_key(&tree, TreeKey::Left, no_mod()).expect("Left on collapsed alpha");
    assert!(
        matches!(&ev, DirectoryTreeEvent::Selected { path, is_dir: true, mode: SelectionMode::Replace } if path == &fx.root)
    );
}

/// S4.8 — Left on a file moves to its parent directory.
#[test]
fn s4_8_left_on_file_moves_to_parent() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("alpha/notes.txt"), false, SelectionMode::Replace);

    let ev = handle_key(&tree, TreeKey::Left, no_mod()).expect("Left on notes.txt");
    assert!(
        matches!(&ev, DirectoryTreeEvent::Selected { path, is_dir: true, mode: SelectionMode::Replace } if path == &fx.path("alpha"))
    );
}

/// S4.8 — Left on the expanded root collapses it (same as any expanded dir).
/// Left on the collapsed root is a no-op (no parent within the tree).
#[test]
fn s4_8_left_at_root() {
    let fx = fixture();
    // Case: root is expanded — Left collapses it.
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.root, true, SelectionMode::Replace);
    let ev = handle_key(&tree, TreeKey::Left, no_mod()).expect("Left on expanded root");
    assert!(matches!(&ev, DirectoryTreeEvent::Toggled(p) if p == &fx.root));

    // Case: root is collapsed — Left is a no-op (no parent in tree).
    let mut tree2 = DirectoryTree::new(&fx.root);
    tree2.expand_blocking(&fx.root).expect("root scan");
    tree2.on_toggled(&fx.root); // collapse root
    tree2.on_selected(&fx.root, true, SelectionMode::Replace);
    assert!(!tree2.root().is_expanded, "root is collapsed");
    assert!(handle_key(&tree2, TreeKey::Left, no_mod()).is_none());
}

// ── S4.9 — Right: expand or move to first child ──────────────────────────────

/// S4.9 — Right on a collapsed directory expands it.
#[test]
fn s4_9_right_on_collapsed_dir_expands() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    tree.on_selected(&fx.path("alpha"), true, SelectionMode::Replace);

    let ev = handle_key(&tree, TreeKey::Right, no_mod()).expect("Right on collapsed alpha");
    assert!(matches!(&ev, DirectoryTreeEvent::Toggled(p) if p == &fx.path("alpha")));
}

/// S4.9 — Right on an expanded directory moves to its first visible child.
#[test]
fn s4_9_right_on_expanded_dir_moves_to_first_child() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("alpha"), true, SelectionMode::Replace);

    let ev = handle_key(&tree, TreeKey::Right, no_mod()).expect("Right on expanded alpha");
    assert!(
        matches!(&ev, DirectoryTreeEvent::Selected { path, is_dir: true, mode: SelectionMode::Replace } if path == &fx.path("alpha/inner")),
        "first child of alpha is inner/: got {ev:?}"
    );
}

/// S4.9 — Right on a file is a no-op.
#[test]
fn s4_9_right_on_file_is_noop() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("zeta.txt"), false, SelectionMode::Replace);
    assert!(handle_key(&tree, TreeKey::Right, no_mod()).is_none());
}

/// S4.9 — Right on an expanded but empty directory is a no-op.
#[test]
fn s4_9_right_on_expanded_empty_dir_is_noop() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    // Expand beta (empty directory) and select it.
    tree.expand_blocking(&fx.path("beta")).expect("beta scan");
    tree.on_selected(&fx.path("beta"), true, SelectionMode::Replace);
    assert!(tree.find(&fx.path("beta")).unwrap().is_expanded);
    assert!(tree.find(&fx.path("beta")).unwrap().children.is_empty());

    assert!(handle_key(&tree, TreeKey::Right, no_mod()).is_none());
}

// ── S4.10 — Escape is unbound without active drag ────────────────────────────

/// S4.10 — Escape returns None when no drag is in progress (RFC 008
/// will extend this file to test the drag-cancel case).
#[test]
fn s4_10_escape_is_unbound_without_drag() {
    let fx = fixture();
    let mut tree = expanded_tree(&fx.root);
    tree.on_selected(&fx.path("alpha"), true, SelectionMode::Replace);
    assert!(handle_key(&tree, TreeKey::Escape, no_mod()).is_none());
}

// ── No-active-path edge cases ─────────────────────────────────────────────────

/// Movement keys (Up/Down) no-op when active_path is not visible.
#[test]
fn no_active_path_movement_keys_noop() {
    let fx = fixture();
    let tree = expanded_tree(&fx.root); // no selection
    assert!(handle_key(&tree, TreeKey::Up, no_mod()).is_none());
    assert!(handle_key(&tree, TreeKey::Down, no_mod()).is_none());
    assert!(handle_key(&tree, TreeKey::Left, no_mod()).is_none());
    assert!(handle_key(&tree, TreeKey::Right, no_mod()).is_none());
    assert!(handle_key(&tree, TreeKey::Enter, no_mod()).is_none());
    assert!(handle_key(&tree, TreeKey::Space, no_mod()).is_none());
}
