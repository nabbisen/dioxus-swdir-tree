# Getting started

Add the flagship crate:

```toml
[dependencies]
dioxus-swdir-tree = "0.7"
```

## Minimal Dioxus app

```rust
use std::sync::Arc;
use dioxus::prelude::*;
use dioxus_swdir_tree::{DirectoryTreeView, DirectoryTreeEvent, use_scan_driver};
use dioxus_swdir_tree_core::{DirectoryTree, SelectionMode, ThreadExecutor};
use dioxus_swdir_tree_core::drag::DragOutcome;

fn app() -> Element {
    let mut tree = use_signal(|| DirectoryTree::new("/home"));
    let scans = use_scan_driver(tree, Arc::new(ThreadExecutor));

    let on_event = move |ev: DirectoryTreeEvent| match ev {
        DirectoryTreeEvent::Toggled(path) => {
            if let Some(req) = tree.write().on_toggled(&path) {
                scans.send(req);
            }
        }
        DirectoryTreeEvent::Selected { path, is_dir, mode } => {
            tree.write().on_selected(&path, is_dir, mode);
        }
        DirectoryTreeEvent::Drag(msg) => {
            let outcome = tree.write().on_drag_msg(msg);
            if let DragOutcome::Clicked { path, is_dir } = outcome {
                tree.write().on_selected(&path, is_dir, SelectionMode::Replace);
            }
            // DragOutcome::Completed { sources, destination } → move files here
        }
    };

    rsx! {
        div { style: "width: 280px; height: 100vh;",
            DirectoryTreeView { tree, on_event }
        }
    }
}

fn main() {
    dioxus::launch(app);
}
```

The component handles keyboard events (arrow keys, Home/End, Enter, Space,
Escape) automatically; your `on_event` handler only needs to process the
resulting `Toggled` and `Selected` events that arrive from them.

## Scripts and tests (blocking helper)

```rust
use dioxus_swdir_tree_core::DirectoryTree;

let mut tree = DirectoryTree::new("/home/me/projects");
tree.expand_blocking(std::path::Path::new("/home/me/projects"));

for (node, depth) in tree.visible_rows() {
    println!("{}{}", "  ".repeat(depth as usize), node.file_name().display());
}
```

`expand_blocking` is the port of the upstream test helper: it runs the full
`on_toggled` → `scan::run` → `on_loaded` cycle on the calling thread.
