//! Framework-neutral keyboard navigation for [`crate::DirectoryTree`].
//!
//! [`handle_key`] is **read-only**: it inspects tree state and returns an
//! optional [`crate::DirectoryTreeEvent`] without mutating anything.
//! The host dispatches the event back through `on_toggled` / `on_selected`,
//! keeping a single mutation funnel and making the function trivially
//! testable without any async infrastructure.
//!
//! The view crate maps Dioxus `KeyboardEvent` values onto [`TreeKey`] and
//! [`Modifiers`]; other embedding layers can supply their own mapping.

use std::path::PathBuf;

use crate::DirectoryTree;
use crate::event::DirectoryTreeEvent;
use crate::selection::SelectionMode;

// ── Public types ──────────────────────────────────────────────────────────────

/// A bound key — the framework-neutral representation of a key press.
///
/// The view crate maps each `dioxus::prelude::Key` variant to one of
/// these; other embedding layers supply their own mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeKey {
    Up,
    Down,
    Home,
    End,
    Enter,
    /// Space and Ctrl+Space are both mapped here (S4.6, S4.7).
    Space,
    Left,
    Right,
    /// Escape — only produces an event when drag is active (RFC 008).
    /// Deliberately `None` when no drag is in progress (S4.10).
    Escape,
}

/// Active modifier keys at the time of the key press.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
}

// ── handle_key ────────────────────────────────────────────────────────────────

/// Translate a key press into a [`DirectoryTreeEvent`], or `None` when
/// the key is unbound (hosts may handle it themselves).
///
/// All movement is computed over [`DirectoryTree::visible_rows`] relative
/// to `active_path`. If `active_path` is not currently visible, movement
/// keys and all keys that need an active node no-op; `Home` and `End`
/// still work.
///
/// # Escape
///
/// Escape is unbound in v0.4 because drag is not yet active (RFC 008).
/// It always returns `None` here; the host can safely bind it for other
/// purposes.
pub fn handle_key(
    tree: &DirectoryTree,
    key: TreeKey,
    modifiers: Modifiers,
) -> Option<DirectoryTreeEvent> {
    let rows = tree.visible_rows();

    // Index of the currently active row (None if active_path is invisible).
    let active_idx: Option<usize> = tree
        .selected_path()
        .and_then(|active| rows.iter().position(|(node, _)| node.path == active));

    match (key, modifiers.shift) {
        // ── Arrow keys ────────────────────────────────────────────────
        (TreeKey::Up, false) => {
            let idx = active_idx?.checked_sub(1)?;
            let (node, _) = &rows[idx];
            Some(selected(
                node.path.clone(),
                node.is_dir,
                SelectionMode::Replace,
            ))
        }

        (TreeKey::Up, true) => {
            // Shift+Up
            let idx = active_idx?.checked_sub(1)?;
            let (node, _) = &rows[idx];
            Some(selected(
                node.path.clone(),
                node.is_dir,
                SelectionMode::ExtendRange,
            ))
        }

        (TreeKey::Down, false) => {
            let idx = active_idx? + 1;
            let (node, _) = rows.get(idx)?;
            Some(selected(
                node.path.clone(),
                node.is_dir,
                SelectionMode::Replace,
            ))
        }

        (TreeKey::Down, true) => {
            // Shift+Down
            let idx = active_idx? + 1;
            let (node, _) = rows.get(idx)?;
            Some(selected(
                node.path.clone(),
                node.is_dir,
                SelectionMode::ExtendRange,
            ))
        }

        // ── Home / End ────────────────────────────────────────────────
        (TreeKey::Home, false) => {
            let (node, _) = rows.first()?;
            Some(selected(
                node.path.clone(),
                node.is_dir,
                SelectionMode::Replace,
            ))
        }

        (TreeKey::Home, true) => {
            let (node, _) = rows.first()?;
            Some(selected(
                node.path.clone(),
                node.is_dir,
                SelectionMode::ExtendRange,
            ))
        }

        (TreeKey::End, false) => {
            let (node, _) = rows.last()?;
            Some(selected(
                node.path.clone(),
                node.is_dir,
                SelectionMode::Replace,
            ))
        }

        (TreeKey::End, true) => {
            let (node, _) = rows.last()?;
            Some(selected(
                node.path.clone(),
                node.is_dir,
                SelectionMode::ExtendRange,
            ))
        }

        // ── Enter — toggle active directory ──────────────────────────
        (TreeKey::Enter, _) => {
            let idx = active_idx?;
            let (node, _) = &rows[idx];
            if !node.is_dir {
                return None;
            }
            Some(DirectoryTreeEvent::Toggled(node.path.clone()))
        }

        // ── Space / Ctrl+Space — toggle-select active row ─────────────
        (TreeKey::Space, _) => {
            let idx = active_idx?;
            let (node, _) = &rows[idx];
            Some(selected(
                node.path.clone(),
                node.is_dir,
                SelectionMode::Toggle,
            ))
        }

        // ── Left — collapse or move to parent ─────────────────────────
        (TreeKey::Left, _) => {
            let idx = active_idx?;
            let (node, _) = &rows[idx];
            if node.is_dir && node.is_expanded {
                // Collapse the expanded directory.
                Some(DirectoryTreeEvent::Toggled(node.path.clone()))
            } else {
                // Move to parent — no-op at the tree root.
                if node.path == tree.config().root_path {
                    return None;
                }
                let parent: PathBuf = node.path.parent()?.to_path_buf();
                Some(selected(parent, true, SelectionMode::Replace))
            }
        }

        // ── Right — expand or move to first child ─────────────────────
        (TreeKey::Right, _) => {
            let idx = active_idx?;
            let (node, _) = &rows[idx];
            if !node.is_dir {
                return None;
            }
            if !node.is_expanded {
                // Expand the collapsed directory.
                Some(DirectoryTreeEvent::Toggled(node.path.clone()))
            } else {
                // Move to the first visible child (the next row that is a
                // direct child of this node).
                let next_idx = idx + 1;
                let (next_node, _) = rows.get(next_idx)?;
                if next_node.path.parent() != Some(node.path.as_path()) {
                    return None; // directory is expanded but has no visible children
                }
                Some(selected(
                    next_node.path.clone(),
                    next_node.is_dir,
                    SelectionMode::Replace,
                ))
            }
        }

        // ── Escape — unbound until drag (RFC 008) ────────────────────
        (TreeKey::Escape, _) => None,
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

#[inline]
fn selected(path: PathBuf, is_dir: bool, mode: SelectionMode) -> DirectoryTreeEvent {
    DirectoryTreeEvent::Selected { path, is_dir, mode }
}
