//! Unit tests for [`super::is_valid_drop`] (RFC 013, S11.12).
//!
//! Built against a synthetic arena store, exercising the validity rules
//! without a live `ItemTree`. The arena's native `parent_id` links make
//! these direct — no parent-map snapshot.

use std::collections::HashMap;

use super::DropPosition::{After, Before, Into};
use super::{DropPosition, is_valid_drop};
use crate::item_tree::node::{InternalItem, NodeId};

/// Fixed test tree:
///
/// ```text
/// 0 (root)
/// ├── 1
/// │   ├── 11
/// │   └── 12
/// └── 2
///     ├── 21
///     │   └── 211
///     └── 22
/// ```
fn store() -> HashMap<NodeId, InternalItem<()>> {
    let edges: &[(u64, Option<u64>)] = &[
        (0, None),
        (1, Some(0)),
        (2, Some(0)),
        (11, Some(1)),
        (12, Some(1)),
        (21, Some(2)),
        (22, Some(2)),
        (211, Some(21)),
    ];
    edges
        .iter()
        .map(|&(c, p)| {
            (
                NodeId(c),
                InternalItem {
                    id: NodeId(c),
                    data: (),
                    depth: 0,
                    children_ids: Vec::new(),
                    parent_id: p.map(NodeId),
                    is_expanded: false,
                    is_selected: false,
                },
            )
        })
        .collect()
}

fn check(sources: &[u64], target: u64, pos: DropPosition) -> bool {
    let s: Vec<NodeId> = sources.iter().map(|&x| NodeId(x)).collect();
    is_valid_drop(&store(), &s, NodeId(target), pos)
}

#[test]
fn self_drop_rejected_all_positions() {
    assert!(!check(&[1], 1, Into));
    assert!(!check(&[1], 1, Before));
    assert!(!check(&[1], 1, After));
}

#[test]
fn nest_into_own_descendant_is_a_cycle() {
    assert!(!check(&[1], 11, Into));
    assert!(!check(&[1], 11, Before));
    assert!(!check(&[1], 11, After));
}

#[test]
fn deep_cycle_rejected() {
    // Dragging 2 into 211 (its grand-descendant) is a cycle.
    assert!(!check(&[2], 211, Into));
}

#[test]
fn sibling_reorder_accepted() {
    // Move 11 before/after its sibling 12.
    assert!(check(&[11], 12, Before));
    assert!(check(&[11], 12, After));
}

#[test]
fn nest_into_unrelated_node_accepted() {
    assert!(check(&[11], 2, Into));
    assert!(check(&[11], 21, Into));
}

#[test]
fn root_has_no_sibling_slot() {
    assert!(!check(&[1], 0, Before));
    assert!(!check(&[1], 0, After));
}

#[test]
fn nest_into_root_accepted() {
    assert!(check(&[11], 0, Into));
}

#[test]
fn drop_into_current_parent_is_allowed() {
    // 211 is already a child of 21; nesting it into 21 again is allowed
    // (the app may treat it as a no-op, but it is not invalid).
    assert!(check(&[211], 21, Into));
}

#[test]
fn nonexistent_target_rejected() {
    assert!(!check(&[1], 999, Into));
    assert!(!check(&[1], 999, Before));
}

#[test]
fn multi_source_all_valid_accepted() {
    assert!(check(&[11, 12], 2, Into));
    assert!(check(&[11, 12], 22, After));
}

#[test]
fn multi_source_one_cycle_rejects_whole_drop() {
    // 21 is a descendant of 2; if 2 is among the sources, nesting into 21
    // is a cycle and the whole drop is rejected.
    assert!(!check(&[1, 2], 21, Into));
}

#[test]
fn target_in_multi_source_rejected() {
    assert!(!check(&[1, 2], 2, After));
    assert!(!check(&[1, 2], 1, Before));
}
