//! The flagship [`DirectoryTreeView`] Dioxus component.

use dioxus::prelude::*;
use dioxus_swdir_tree_core::DirectoryTree;
use dioxus_swdir_tree_core::keyboard::{self, Modifiers as CoreMods, TreeKey};

use crate::event::DirectoryTreeEvent;
use crate::row::TreeRow;
use crate::style as s;

/// A lazy-loading, filterable directory-tree explorer widget for Dioxus.
///
/// # Props
///
/// | Prop | Type | Description |
/// |---|---|---|
/// | `tree` | `Signal<DirectoryTree>` | The tree state, typically created with `use_signal`. |
/// | `on_event` | `EventHandler<DirectoryTreeEvent>` | Receives every interaction. |
///
/// # Wiring
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use dioxus_swdir_tree::{DirectoryTreeView, DirectoryTreeEvent, use_scan_driver};
/// # use dioxus_swdir_tree_core::{DirectoryTree, ThreadExecutor};
/// # use std::sync::Arc;
/// fn app() -> Element {
///     let mut tree = use_signal(|| DirectoryTree::new("/home"));
///     let scans = use_scan_driver(tree, Arc::new(ThreadExecutor));
///
///     let on_event = move |ev: DirectoryTreeEvent| match ev {
///         DirectoryTreeEvent::Toggled(path) => {
///             if let Some(req) = tree.write().on_toggled(&path) {
///                 scans.send(req);
///             }
///         }
///         DirectoryTreeEvent::Selected { path, is_dir, mode } => {
///             tree.write().on_selected(&path, is_dir, mode);
///         }
///     };
///
///     rsx! {
///         div { style: "width: 280px; height: 100vh;",
///             DirectoryTreeView { tree, on_event }
///         }
///     }
/// }
/// ```
///
/// # Styling
///
/// The component applies `dx-swdir-*` CSS classes (see [`crate::style`]).
/// A minimal baseline stylesheet is injected when the `default-style`
/// feature is enabled (default). Disable it to theme from scratch.
#[component]
pub fn DirectoryTreeView(
    tree: Signal<DirectoryTree>,
    on_event: EventHandler<DirectoryTreeEvent>,
) -> Element {
    // Collect owned rows so the `tree.read()` guard is dropped before rsx!.
    let rows: Vec<(dioxus_swdir_tree_core::TreeNode, u32)> = tree
        .read()
        .visible_rows()
        .into_iter()
        .map(|(node, depth)| (node.clone(), depth))
        .collect();

    // Prepare the optional default stylesheet outside rsx! (cfg inside
    // the macro is not supported).
    #[cfg(feature = "default-style")]
    let default_style_css = Some(s::DEFAULT_CSS);
    #[cfg(not(feature = "default-style"))]
    let default_style_css: Option<&str> = None;

    // Keyboard handler: map Dioxus KeyboardEvent → TreeKey + Modifiers,
    // call handle_key, call prevent_default only when a key was consumed.
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
            _ => return, // unbound key — let the browser handle it
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

    rsx! {
        if let Some(css) = default_style_css {
            style { "{css}" }
        }

        div {
            class: s::CLASS_TREE,
            tabindex: "0",
            onkeydown: on_keydown,

            for (node, depth) in rows {
                TreeRow {
                    key: "{node.path.display()}",
                    node,
                    depth,
                    on_event,
                }
            }
        }
    }
}
