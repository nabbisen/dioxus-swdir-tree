//! The per-row component rendered by [`crate::DirectoryTreeView`].

use std::path::PathBuf;

use dioxus::prelude::*;
use dioxus_swdir_tree_core::{DragMsg, DragState, TreeNode};

use crate::event::DirectoryTreeEvent;
use crate::style as s;

/// Props for a single visible tree row.
#[derive(Props, Clone, PartialEq)]
pub(crate) struct TreeRowProps {
    pub node: TreeNode,
    pub depth: u32,
    pub on_event: EventHandler<DirectoryTreeEvent>,
    /// Active drag session (if any), for drop-target highlighting.
    pub drag: Option<DragState>,
}

/// One visible row: indented caret + icon + label.
///
/// Mouse event wiring (RFC 008):
/// - `onmousedown` → `DragMsg::Pressed` (starts drag or registers click).
/// - `onmouseenter` → `DragMsg::Entered` (updates hover target during drag).
/// - `onmouseleave` → `DragMsg::Exited` (clears hover target during drag).
/// - `onmouseup` → `DragMsg::Released` during drag; stop propagation so
///   the container fallback does not also fire `Cancelled`.
/// - `caret click` → `Toggled` (intercepted before row-level handling).
#[component]
pub(crate) fn TreeRow(props: TreeRowProps) -> Element {
    let TreeRowProps {
        node,
        depth,
        on_event,
        drag,
    } = props;

    let path: PathBuf = node.path.clone();
    let is_dir = node.is_dir;
    let is_expanded = node.is_expanded;
    let is_loaded = node.is_loaded;
    let has_error = node.error.is_some();
    let is_selected = node.is_selected;
    let is_drag_active = drag.is_some();

    // Is this row the current valid drop target?
    let is_drop_target = drag
        .as_ref()
        .and_then(|d| d.hovered_target.as_ref())
        .map(|t| *t == path)
        .unwrap_or(false);

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
    if is_drop_target {
        classes.push(' ');
        classes.push_str(s::CLASS_ROW_DROP_TARGET);
    }

    // ── Glyphs ────────────────────────────────────────────────────────
    let caret: &str = if !is_dir {
        " "
    } else if is_expanded && !is_loaded {
        "…"
    } else if is_expanded {
        "▾"
    } else {
        "▸"
    };

    let icon: &str = if has_error {
        "⚠"
    } else if !is_dir {
        "📄"
    } else if is_expanded {
        "📂"
    } else {
        "📁"
    };

    let label = node.file_name().to_string_lossy().into_owned();
    let error_title: String = node
        .error
        .as_ref()
        .map(|e| e.message().to_string())
        .unwrap_or_default();

    // ── Event handlers ────────────────────────────────────────────────

    // Caret click: always Toggled (expand/collapse), never a select.
    let caret_path = path.clone();
    let on_caret_click = move |evt: MouseEvent| {
        evt.stop_propagation();
        on_event.call(DirectoryTreeEvent::Toggled(caret_path.clone()));
    };

    // Mouse down: start a drag (or click) session.
    let press_path = path.clone();
    let on_mousedown = move |_evt: MouseEvent| {
        on_event.call(DirectoryTreeEvent::Drag(DragMsg::Pressed {
            path: press_path.clone(),
            is_dir,
        }));
    };

    // Mouse enter/leave: update hover target during drag.
    let enter_path = path.clone();
    let on_mouseenter = move |_evt: MouseEvent| {
        on_event.call(DirectoryTreeEvent::Drag(DragMsg::Entered(
            enter_path.clone(),
        )));
    };

    let exit_path = path.clone();
    let on_mouseleave = move |_evt: MouseEvent| {
        on_event.call(DirectoryTreeEvent::Drag(DragMsg::Exited(exit_path.clone())));
    };

    // Mouse up: resolve click vs drop during drag (stop propagation so
    // the container's Cancelled fallback doesn't also fire).
    let release_path = path.clone();
    let on_mouseup = move |evt: MouseEvent| {
        if is_drag_active {
            evt.stop_propagation();
            on_event.call(DirectoryTreeEvent::Drag(DragMsg::Released(
                release_path.clone(),
            )));
        }
    };

    rsx! {
        div {
            class: "{classes}",
            style: "padding-left: {indent_px}px;",
            title: "{error_title}",
            onmousedown: on_mousedown,
            onmouseenter: on_mouseenter,
            onmouseleave: on_mouseleave,
            onmouseup: on_mouseup,

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
