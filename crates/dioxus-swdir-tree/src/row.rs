//! The per-row component rendered by [`crate::DirectoryTreeView`].

use std::path::PathBuf;

use dioxus::prelude::*;
use dioxus_swdir_tree_core::{SelectionMode, TreeNode};

use crate::event::DirectoryTreeEvent;
use crate::style as s;

/// Props for a single visible tree row.
#[derive(Props, Clone, PartialEq)]
pub(crate) struct TreeRowProps {
    /// Owned snapshot of the node (cloned from the signal read for this
    /// render frame).
    pub node: TreeNode,
    /// Depth in the tree (0 = root).
    pub depth: u32,
    /// Event sink provided by the host.
    pub on_event: EventHandler<DirectoryTreeEvent>,
}

/// One visible row: indented caret + icon + label.
///
/// v0.3 click semantics:
/// - Caret click → `Toggled(path)` (expand / collapse the directory).
/// - Row click (non-caret) → `Selected { path, is_dir, mode: Replace }`.
///   Modifier-aware modes (`Toggle`, `ExtendRange`) arrive with RFC 007/008.
#[component]
pub(crate) fn TreeRow(props: TreeRowProps) -> Element {
    let TreeRowProps {
        node,
        depth,
        on_event,
    } = props;

    let path: PathBuf = node.path.clone();
    let is_dir = node.is_dir;
    let is_expanded = node.is_expanded;
    let is_loaded = node.is_loaded;
    let has_error = node.error.is_some();
    let is_selected = node.is_selected;

    // ── Indent ────────────────────────────────────────────────────────
    let indent_px = depth * 16;

    // ── CSS class list ────────────────────────────────────────────────
    let mut classes = s::CLASS_ROW.to_string();
    if is_selected {
        classes.push(' ');
        classes.push_str(s::CLASS_ROW_SELECTED);
    }
    if has_error {
        classes.push(' ');
        classes.push_str(s::CLASS_ROW_ERROR);
    }

    // ── Caret glyph (directories only) ───────────────────────────────
    let caret: &str = if !is_dir {
        " " // blank spacer to keep label alignment
    } else if is_expanded && !is_loaded {
        "…" // loading indicator
    } else if is_expanded {
        "▾"
    } else {
        "▸"
    };

    // ── Icon glyph ────────────────────────────────────────────────────
    let icon: &str = if has_error {
        "⚠"
    } else if !is_dir {
        "📄"
    } else if is_expanded {
        "📂"
    } else {
        "📁"
    };

    // ── Basename label ────────────────────────────────────────────────
    let label = node.file_name().to_string_lossy().into_owned();

    // ── Error annotation ──────────────────────────────────────────────
    let error_title: String = node
        .error
        .as_ref()
        .map(|e| e.message().to_string())
        .unwrap_or_default();

    // ── Event handlers ────────────────────────────────────────────────
    let caret_path = path.clone();
    let on_caret_click = move |evt: MouseEvent| {
        evt.stop_propagation();
        on_event.call(DirectoryTreeEvent::Toggled(caret_path.clone()));
    };

    let row_path = path.clone();
    let on_row_click = move |_evt: MouseEvent| {
        on_event.call(DirectoryTreeEvent::Selected {
            path: row_path.clone(),
            is_dir,
            mode: SelectionMode::Replace,
        });
    };

    rsx! {
        div {
            class: "{classes}",
            style: "padding-left: {indent_px}px;",
            title: "{error_title}",
            onclick: on_row_click,

            span {
                class: s::CLASS_CARET,
                onclick: on_caret_click,
                "{caret}"
            }
            span {
                class: s::CLASS_ICON,
                "{icon}"
            }
            span {
                class: s::CLASS_LABEL,
                "{label}"
            }
        }
    }
}
