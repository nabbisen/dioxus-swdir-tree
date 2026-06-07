//! The per-row component rendered by [`crate::DirectoryTreeView`].

use std::path::PathBuf;
use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_swdir_tree_core::{DragMsg, DragState, IconRole, IconTheme, TreeNode};

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
    /// Icon theme wrapper (pointer-equality checked for PartialEq).
    pub theme: ArcTheme,
}

// ── ArcTheme wrapper ──────────────────────────────────────────────────────────

/// Thin `Arc<dyn IconTheme>` wrapper that implements `PartialEq` via
/// pointer equality so it can be used as a Dioxus prop.
#[derive(Clone)]
pub struct ArcTheme(pub Arc<dyn IconTheme>);

impl PartialEq for ArcTheme {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Default for ArcTheme {
    fn default() -> Self {
        default_theme()
    }
}

/// Return the default theme for the active feature set.
pub fn default_theme() -> ArcTheme {
    #[cfg(feature = "icons")]
    {
        ArcTheme(Arc::new(dioxus_swdir_tree_core::LucideTheme))
    }
    #[cfg(not(feature = "icons"))]
    {
        ArcTheme(Arc::new(dioxus_swdir_tree_core::UnicodeTheme))
    }
}

// ── TreeRow component ─────────────────────────────────────────────────────────

/// One visible row: indented caret + icon + label.
#[component]
pub(crate) fn TreeRow(props: TreeRowProps) -> Element {
    let TreeRowProps {
        node,
        depth,
        on_event,
        drag,
        theme,
    } = props;

    let path: PathBuf = node.path.clone();
    let is_dir = node.is_dir;
    let is_expanded = node.is_expanded;
    let is_loaded = node.is_loaded;
    let has_error = node.error.is_some();
    let is_selected = node.is_selected;
    let is_drag_active = drag.is_some();

    let is_drop_target = drag
        .as_ref()
        .and_then(|d| d.hovered_target.as_ref())
        .map(|t| *t == path)
        .unwrap_or(false);

    let indent_px = depth * 16;

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

    // ── Icon glyphs from theme ────────────────────────────────────────
    let caret_spec = if !is_dir {
        None
    } else if is_expanded && !is_loaded {
        // Loading indicator — no theme role for this; use raw glyph.
        Some(("…", None, None))
    } else {
        let role = if is_expanded {
            IconRole::CaretDown
        } else {
            IconRole::CaretRight
        };
        let spec = theme.0.glyph(role);
        Some((
            // Safety: lifetime of glyph is 'static since IconSpec uses Cow<'static>
            Box::leak(spec.glyph.into_owned().into_boxed_str()) as &str,
            spec.font,
            spec.size,
        ))
    };

    let icon_role = if has_error {
        IconRole::Error
    } else if !is_dir {
        IconRole::File
    } else if is_expanded {
        IconRole::FolderOpen
    } else {
        IconRole::FolderClosed
    };
    let icon_spec = theme.0.glyph(icon_role);

    let caret_str = caret_spec.map(|(g, _, _)| g).unwrap_or(" ");
    let caret_font = caret_spec.and_then(|(_, f, _)| f).unwrap_or("");
    let caret_size = caret_spec.and_then(|(_, _, s)| s);
    let caret_style = if caret_font.is_empty() {
        String::new()
    } else {
        format!("font-family: {caret_font};")
    };
    let caret_size_style = caret_size
        .map(|s| format!("{} font-size: {s}px;", caret_style))
        .unwrap_or(caret_style);

    let icon_str = Box::leak(icon_spec.glyph.into_owned().into_boxed_str()) as &str;
    let icon_font = icon_spec.font.unwrap_or("");
    let icon_size = icon_spec.size;
    let icon_style = if icon_font.is_empty() {
        String::new()
    } else {
        format!("font-family: {icon_font};")
    };
    let icon_size_style = icon_size
        .map(|s| format!("{} font-size: {s}px;", icon_style))
        .unwrap_or(icon_style);

    let label = node.file_name().to_string_lossy().into_owned();
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

    let press_path = path.clone();
    let on_mousedown = move |_evt: MouseEvent| {
        on_event.call(DirectoryTreeEvent::Drag(DragMsg::Pressed {
            path: press_path.clone(),
            is_dir,
        }));
    };

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
                style: "{caret_size_style}",
                onclick: on_caret_click,
                "{caret_str}"
            }
            span {
                class: s::CLASS_ICON,
                style: "{icon_size_style}",
                "{icon_str}"
            }
            span {
                class: s::CLASS_LABEL,
                "{label}"
            }
        }
    }
}
