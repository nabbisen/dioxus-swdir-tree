//! Minimal desktop file-explorer example for `dioxus-swdir-tree`.
//!
//! # Run
//!
//! ```sh
//! cd examples/explorer
//! cargo run
//! ```
//!
//! The explorer opens your home directory. Click a folder caret to
//! expand it; click a row to select it. The selected path is shown
//! at the bottom of the window.

use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_swdir_tree::{
    DirectoryTree, DirectoryTreeEvent, DirectoryTreeView, SelectionMode, ThreadExecutor,
    use_scan_driver,
};

fn main() {
    dioxus::launch(App);
}

fn App() -> Element {
    let root = dirs_home().unwrap_or_else(|| std::path::PathBuf::from("/"));

    let mut tree = use_signal(|| DirectoryTree::new(&root));
    let scans = use_scan_driver(tree, Arc::new(ThreadExecutor));

    let selected_label = use_memo(move || {
        tree.read()
            .selected_path()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    });

    let on_event = move |ev: DirectoryTreeEvent| match ev {
        DirectoryTreeEvent::Toggled(path) => {
            if let Some(req) = tree.write().on_toggled(&path) {
                scans.send(req);
            }
        }
        DirectoryTreeEvent::Selected { path, is_dir, mode } => {
            tree.write().on_selected(&path, is_dir, mode);
        }
    };

    rsx! {
        div {
            style: "
                display: flex;
                flex-direction: column;
                height: 100vh;
                font-family: system-ui, sans-serif;
            ",

            // ── Toolbar ───────────────────────────────────────────────
            div {
                style: "padding: 0.5rem; border-bottom: 1px solid #ddd; font-size: 0.8rem; color: #888;",
                "dioxus-swdir-tree explorer — {root.display()}"
            }

            // ── Tree pane ─────────────────────────────────────────────
            div {
                style: "flex: 1; overflow: hidden; padding: 0.25rem;",
                DirectoryTreeView { tree, on_event }
            }

            // ── Status bar ────────────────────────────────────────────
            div {
                style: "padding: 0.25rem 0.5rem; border-top: 1px solid #ddd; font-size: 0.75rem; color: #555; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                if selected_label().is_empty() {
                    "No selection"
                } else {
                    "{selected_label}"
                }
            }
        }
    }
}

fn dirs_home() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from)
}
