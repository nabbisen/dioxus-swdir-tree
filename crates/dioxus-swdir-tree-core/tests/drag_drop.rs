//! Feature 7 — Drag-and-drop (specification clauses S7.1–S7.6).

mod common;

use dioxus_swdir_tree_core::{
    DirectoryTree, DisplayFilter, SelectionMode,
    drag::{DragMsg, DragOutcome},
    event::DirectoryTreeEvent,
    keyboard::{Modifiers, TreeKey, handle_key},
};

use common::fixture;

// ── S7.1 — Drag is activated by mouse-press ───────────────────────────────────

/// S7.1 — Pressing on a row not in the selection: sources = [path].
#[test]
fn s7_1_pressed_unselected_row_sources_single() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    let outcome = tree.on_drag_msg(DragMsg::Pressed {
        path: alpha.clone(),
        is_dir: true,
    });
    assert!(matches!(outcome, DragOutcome::None));
    let drag = tree.drag_state().expect("drag active");
    assert_eq!(drag.sources, vec![alpha.clone()]);
    assert_eq!(drag.started_at, alpha);
    assert!(drag.started_is_dir);
    assert!(drag.hovered_target.is_none());
}

/// S7.1 — Pressing on a row that is in the selection: sources = all selected.
#[test]
fn s7_1_pressed_selected_row_sources_all_selected() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    let beta = fx.path("beta");
    tree.on_selected(&alpha, true, SelectionMode::Toggle);
    tree.on_selected(&beta, true, SelectionMode::Toggle);

    // Pressing on one of the selected rows: sources = full selection.
    tree.on_drag_msg(DragMsg::Pressed {
        path: alpha.clone(),
        is_dir: true,
    });
    let drag = tree.drag_state().expect("drag active");
    assert_eq!(drag.sources, vec![alpha.clone(), beta.clone()]);
}

// ── S7.2 — Dropping on the press row is a click ───────────────────────────────

/// S7.2 — Released on the same row as Pressed: DragOutcome::Clicked.
#[test]
fn s7_2_released_same_row_is_click() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    tree.on_drag_msg(DragMsg::Pressed {
        path: alpha.clone(),
        is_dir: true,
    });
    let outcome = tree.on_drag_msg(DragMsg::Released(alpha.clone()));
    assert!(
        matches!(&outcome, DragOutcome::Clicked { path, is_dir: true } if path == &alpha),
        "expected Clicked, got {outcome:?}"
    );
    assert!(tree.drag_state().is_none(), "drag cleared after release");
}

/// S7.2 — Released on a different row: DragOutcome::Completed.
#[test]
fn s7_2_released_different_row_is_drop() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    let beta = fx.path("beta");
    tree.on_drag_msg(DragMsg::Pressed {
        path: alpha.clone(),
        is_dir: true,
    });
    let outcome = tree.on_drag_msg(DragMsg::Released(beta.clone()));
    assert!(
        matches!(&outcome, DragOutcome::Completed { sources, destination }
            if sources == std::slice::from_ref(&alpha) && destination == &beta),
        "expected Completed, got {outcome:?}"
    );
    assert!(tree.drag_state().is_none());
}

// ── S7.3 — Target validity ────────────────────────────────────────────────────

/// S7.3 — A file is never a valid drop target.
#[test]
fn s7_3_file_is_not_valid_target() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    tree.on_drag_msg(DragMsg::Pressed {
        path: alpha.clone(),
        is_dir: true,
    });
    tree.on_drag_msg(DragMsg::Entered(fx.path("zeta.txt")));
    assert!(tree.drag_state().unwrap().hovered_target.is_none());
}

/// S7.3 — A source path is not a valid drop target.
#[test]
fn s7_3_source_is_not_valid_target() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    tree.on_drag_msg(DragMsg::Pressed {
        path: alpha.clone(),
        is_dir: true,
    });
    tree.on_drag_msg(DragMsg::Entered(alpha.clone()));
    assert!(tree.drag_state().unwrap().hovered_target.is_none());
}

/// S7.3 — A descendant of a source is not a valid drop target
/// (component-wise prefix check, not string prefix).
#[test]
fn s7_3_descendant_of_source_is_not_valid_target() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");
    tree.expand_blocking(&fx.path("alpha")).expect("alpha scan");

    let alpha = fx.path("alpha");
    let inner = fx.path("alpha/inner");
    tree.on_drag_msg(DragMsg::Pressed {
        path: alpha.clone(),
        is_dir: true,
    });
    tree.on_drag_msg(DragMsg::Entered(inner.clone()));
    assert!(tree.drag_state().unwrap().hovered_target.is_none());
}

/// S7.3 — A valid directory that is neither a source nor a descendant
/// becomes the hovered target.
#[test]
fn s7_3_valid_directory_becomes_target() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    let beta = fx.path("beta");
    tree.on_drag_msg(DragMsg::Pressed {
        path: alpha.clone(),
        is_dir: true,
    });
    tree.on_drag_msg(DragMsg::Entered(beta.clone()));
    assert_eq!(tree.drag_state().unwrap().hovered_target, Some(beta));
}

/// S7.3 — Exited clears hovered_target only if it still equals the exited path.
#[test]
fn s7_3_exited_guards_against_out_of_order() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    let beta = fx.path("beta");
    tree.on_drag_msg(DragMsg::Pressed {
        path: alpha.clone(),
        is_dir: true,
    });
    tree.on_drag_msg(DragMsg::Entered(beta.clone()));
    assert_eq!(
        tree.drag_state().unwrap().hovered_target,
        Some(beta.clone())
    );

    // Exited from a path that is NOT the current target: no-op.
    tree.on_drag_msg(DragMsg::Exited(fx.root.clone()));
    assert_eq!(
        tree.drag_state().unwrap().hovered_target,
        Some(beta.clone()),
        "hovered_target unchanged when Exited for a different path"
    );

    // Exited from the actual target: clears it.
    tree.on_drag_msg(DragMsg::Exited(beta.clone()));
    assert!(tree.drag_state().unwrap().hovered_target.is_none());
}

// ── S7.4 — Escape cancels the drag ───────────────────────────────────────────

/// S7.4 — DragMsg::Cancelled clears the drag state.
#[test]
fn s7_4_cancelled_clears_drag() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    tree.on_drag_msg(DragMsg::Pressed {
        path: fx.path("alpha"),
        is_dir: true,
    });
    assert!(tree.drag_state().is_some());

    let outcome = tree.on_drag_msg(DragMsg::Cancelled);
    assert!(matches!(outcome, DragOutcome::None));
    assert!(tree.drag_state().is_none());
}

/// S7.4 — Escape key via handle_key produces DragMsg::Cancelled when drag active.
#[test]
fn s7_4_escape_key_cancels_drag_when_active() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    tree.on_drag_msg(DragMsg::Pressed {
        path: fx.path("alpha"),
        is_dir: true,
    });
    let ev = handle_key(&tree, TreeKey::Escape, Modifiers::default())
        .expect("Escape should produce event when drag active");
    assert!(
        matches!(&ev, DirectoryTreeEvent::Drag(DragMsg::Cancelled)),
        "expected Drag(Cancelled), got {ev:?}"
    );
}

/// S7.4 / S4.10 — Escape returns None when no drag is active.
#[test]
fn s7_4_escape_key_is_noop_without_drag() {
    let fx = fixture();
    let tree = DirectoryTree::new(&fx.root);
    assert!(handle_key(&tree, TreeKey::Escape, Modifiers::default()).is_none());
}

// ── S7.5 — DragCompleted carries sources and destination ─────────────────────

/// S7.5 — The outcome carries all sources and the destination; the tree
/// does NOT auto-refresh (the application calls on_toggled explicitly).
#[test]
fn s7_5_drag_completed_carries_sources_and_destination() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    let beta = fx.path("beta");
    let zeta = fx.path("zeta.txt");

    // Select alpha and zeta, then press on alpha.
    tree.on_selected(&alpha, true, SelectionMode::Toggle);
    tree.on_selected(&zeta, false, SelectionMode::Toggle);
    tree.on_drag_msg(DragMsg::Pressed {
        path: alpha.clone(),
        is_dir: true,
    });
    tree.on_drag_msg(DragMsg::Entered(beta.clone()));

    let outcome = tree.on_drag_msg(DragMsg::Released(beta.clone()));
    match outcome {
        DragOutcome::Completed {
            sources,
            destination,
        } => {
            assert_eq!(sources, vec![alpha.clone(), zeta.clone()]);
            assert_eq!(destination, beta);
        }
        other => panic!("expected Completed, got {other:?}"),
    }
    // Tree is NOT automatically refreshed: beta is still in same state.
    assert!(tree.find(&beta).is_some());
    assert!(tree.drag_state().is_none());
}

// ── S7.6 — Drag survives filter change ───────────────────────────────────────

/// S7.6 — set_filter does not clear the drag state.
#[test]
fn s7_6_set_filter_preserves_drag_state() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    tree.expand_blocking(&fx.root).expect("root scan");

    let alpha = fx.path("alpha");
    tree.on_drag_msg(DragMsg::Pressed {
        path: alpha.clone(),
        is_dir: true,
    });
    assert!(tree.drag_state().is_some());

    tree.set_filter(DisplayFilter::FoldersOnly);
    assert!(
        tree.drag_state().is_some(),
        "drag must survive filter change (S7.6)"
    );
    assert_eq!(tree.drag_state().unwrap().started_at, alpha);
}

// ── Cancelled with no active drag is a no-op ─────────────────────────────────

#[test]
fn cancelled_without_active_drag_is_noop() {
    let fx = fixture();
    let mut tree = DirectoryTree::new(&fx.root);
    let snapshot = tree.clone();
    let outcome = tree.on_drag_msg(DragMsg::Cancelled);
    assert!(matches!(outcome, DragOutcome::None));
    assert_eq!(tree, snapshot);
}
