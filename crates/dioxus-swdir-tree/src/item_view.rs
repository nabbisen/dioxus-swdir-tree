//! The `ItemTreeView<T>` Dioxus component.

use std::fmt;

use dioxus::prelude::*;
use dioxus_swdir_tree_core::keyboard::{Modifiers as CoreMods, TreeKey};
use dioxus_swdir_tree_core::{ItemTree, ItemTreeEvent};

use crate::item_row::ItemTreeRow;
use crate::row::{ArcTheme, default_theme};
use crate::style as s;

/// A keyboard-navigable, expandable, searchable tree over caller-supplied
/// in-memory data.
///
/// No coroutine or scan driver required — the caller drives data updates
/// synchronously via [`ItemTree::set_tree`].
///
/// # Minimal wiring
///
/// ```no_run
/// # use std::sync::Arc;
/// # use dioxus::prelude::*;
/// # use dioxus_swdir_tree_core::{ItemTree, ItemNode, NodeId, ItemTreeEvent};
/// # use dioxus_swdir_tree_core::selection::SelectionMode;
/// # use dioxus_swdir_tree::ItemTreeView;
/// fn app() -> Element {
///     let mut tree = use_signal(|| {
///         ItemTree::new().with_display(|s: &String| s.clone())
///     });
///
///     // Build initial data
///     let root = ItemNode::branch(NodeId(0), "root".to_string(), vec![
///         ItemNode::leaf(NodeId(1), "alpha".to_string()),
///     ]);
///     use_effect(move || { tree.write().set_tree(root.clone()); });
///
///     let on_event = move |ev: ItemTreeEvent| match ev {
///         ItemTreeEvent::Toggled(id) => tree.write().on_toggled(id),
///         ItemTreeEvent::Selected(id, mode) => tree.write().on_selected(id, mode),
///     };
///
///     rsx! { ItemTreeView { tree, on_event } }
/// }
/// ```
#[derive(Props, Clone)]
pub struct ItemTreeViewProps<T: Clone + fmt::Debug + Send + Sync + 'static> {
    pub tree: Signal<ItemTree<T>>,
    pub on_event: EventHandler<ItemTreeEvent>,
    #[props(optional)]
    pub theme: Option<ArcTheme>,
}

impl<T: Clone + fmt::Debug + Send + Sync + 'static> PartialEq for ItemTreeViewProps<T> {
    fn eq(&self, other: &Self) -> bool {
        // Signal and EventHandler compare by ID, so this never requires T: PartialEq.
        self.tree == other.tree && self.on_event == other.on_event && self.theme == other.theme
    }
}

#[allow(non_snake_case)]
pub fn ItemTreeView<T: Clone + fmt::Debug + Send + Sync + 'static>(
    props: ItemTreeViewProps<T>,
) -> Element {
    let ItemTreeViewProps {
        tree,
        on_event,
        theme,
    } = props;
    let theme = theme.unwrap_or_else(default_theme);

    let t = tree.read();
    let rows = t.visible_rows();
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
            Key::Character(ref ch) if ch == " " => TreeKey::Space,
            _ => return,
        };
        let mods = CoreMods {
            shift: evt.modifiers().shift(),
            ctrl: evt.modifiers().ctrl(),
        };
        if let Some(ev) = tree.read().handle_key(tree_key, mods) {
            evt.prevent_default();
            on_event.call(ev);
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

            for row in rows {
                ItemTreeRow {
                    key: "{row.id.0}",
                    row,
                    on_event,
                    theme: theme.clone(),
                }
            }
        }
    }
}
