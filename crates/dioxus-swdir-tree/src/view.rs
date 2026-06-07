//! The flagship [`DirectoryTreeView`] Dioxus component.

use dioxus::prelude::*;
use dioxus_swdir_tree_core::keyboard::{self, Modifiers as CoreMods, TreeKey};
use dioxus_swdir_tree_core::{DirectoryTree, DragMsg};

use crate::event::DirectoryTreeEvent;
use crate::row::{ArcTheme, TreeRow, default_theme};
use crate::style as s;

/// A lazy-loading, filterable, keyboard-navigable, drag-and-drop,
/// search-aware directory-tree explorer widget for Dioxus.
///
/// # Props
///
/// | Prop | Type | Description |
/// |---|---|---|
/// | `tree` | `Signal<DirectoryTree>` | The tree state. |
/// | `on_event` | `EventHandler<DirectoryTreeEvent>` | Receives every interaction. |
/// | `theme` | `Option<ArcTheme>` | Icon theme; defaults to `UnicodeTheme` (or `LucideTheme` with the `icons` feature). |
///
/// See the crate README for a full wiring example.
#[component]
pub fn DirectoryTreeView(
    tree: Signal<DirectoryTree>,
    on_event: EventHandler<DirectoryTreeEvent>,
    #[props(optional)] theme: Option<ArcTheme>,
) -> Element {
    let theme = theme.unwrap_or_else(default_theme);

    let t = tree.read();
    let rows: Vec<(dioxus_swdir_tree_core::TreeNode, u32)> = t
        .visible_rows()
        .into_iter()
        .map(|(node, depth)| (node.clone(), depth))
        .collect();
    let drag = t.drag_state().cloned();
    let drag_active = drag.is_some();
    drop(t);

    #[cfg(feature = "default-style")]
    let default_style_css = Some(s::DEFAULT_CSS);
    #[cfg(not(feature = "default-style"))]
    let default_style_css: Option<&str> = None;

    let on_keydown = move |evt: KeyboardEvent| {
        let tree_key = match evt.key() {
            Key::ArrowUp => TreeKey::Up,
            Key::ArrowDown => TreeKey::Down,
            Key::Home => TreeKey::Home,
            Key::End => TreeKey::End,
            Key::Enter => TreeKey::Enter,
            Key::ArrowLeft => TreeKey::Left,
            Key::ArrowRight => TreeKey::Right,
            Key::Escape => TreeKey::Escape,
            Key::Character(ref s) if s == " " => TreeKey::Space,
            _ => return,
        };
        let mods = CoreMods {
            shift: evt.modifiers().shift(),
            ctrl: evt.modifiers().ctrl(),
        };
        if let Some(event) = keyboard::handle_key(&tree.read(), tree_key, mods) {
            evt.prevent_default();
            on_event.call(event);
        }
    };

    let on_container_mouseup = move |_evt: MouseEvent| {
        if drag_active {
            on_event.call(DirectoryTreeEvent::Drag(DragMsg::Cancelled));
        }
    };

    rsx! {
        if let Some(css) = default_style_css {
            style { "{css}" }
        }

        if let Some(ref d) = drag {
            div {
                style: "
                    position: fixed; bottom: 1rem; left: 50%;
                    transform: translateX(-50%);
                    background: rgba(0,0,0,0.75); color: #fff;
                    padding: 0.2rem 0.6rem; border-radius: 4px;
                    font-size: 0.75rem; pointer-events: none; z-index: 999;
                ",
                "Dragging {d.sources.len()} item(s)"
            }
        }

        div {
            class: s::CLASS_TREE,
            tabindex: "0",
            onkeydown: on_keydown,
            onmouseup: on_container_mouseup,

            for (node, depth) in rows {
                TreeRow {
                    key: "{node.path.display()}",
                    node,
                    depth,
                    on_event,
                    drag: drag.clone(),
                    theme: theme.clone(),
                }
            }
        }
    }
}
