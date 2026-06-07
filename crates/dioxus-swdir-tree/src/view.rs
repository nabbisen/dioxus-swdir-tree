//! The flagship [`DirectoryTreeView`] Dioxus component.

use dioxus::prelude::*;
use dioxus_swdir_tree_core::DirectoryTree;

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
    // Collect owned (cloned) rows so the `tree.read()` guard is dropped
    // before entering `rsx!`.
    let rows: Vec<(dioxus_swdir_tree_core::TreeNode, u32)> = tree
        .read()
        .visible_rows()
        .into_iter()
        .map(|(node, depth)| (node.clone(), depth))
        .collect();

    // Prepare the optional default stylesheet. We build it outside rsx!
    // because #[cfg] attributes are not valid inside the macro.
    #[cfg(feature = "default-style")]
    let default_style_css = Some(s::DEFAULT_CSS);
    #[cfg(not(feature = "default-style"))]
    let default_style_css: Option<&str> = None;

    rsx! {
        if let Some(css) = default_style_css {
            style { "{css}" }
        }

        div {
            class: s::CLASS_TREE,
            // tabindex="0" makes the container focusable so RFC 007
            // can attach onkeydown.
            tabindex: "0",

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
