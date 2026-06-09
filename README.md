# dioxus-swdir-tree

[![crates.io](https://img.shields.io/crates/v/dioxus-swdir-tree?label=rust)](https://crates.io/crates/dioxus-swdir-tree)
[![License](https://img.shields.io/github/license/nabbisen/dioxus-swdir-tree)](https://github.com/nabbisen/dioxus-swdir-tree/blob/main/LICENSE)
[![Documentation](https://docs.rs/dioxus-swdir-tree/badge.svg?version=latest)](https://docs.rs/dioxus-swdir-tree)
[![Dependency Status](https://deps.rs/crate/dioxus-swdir-tree/latest/status.svg)](https://deps.rs/crate/dioxus-swdir-tree)

A directory-tree explorer widget for [Dioxus](https://dioxuslabs.com) GUI apps —
lazy loading, display filters, multi-selection, keyboard navigation, drag & drop,
prefetch, incremental search, and icon themes —
built on the [`swdir`](https://crates.io/crates/swdir) directory scanner and ported
from the design of [`iced-swdir-tree`](https://crates.io/crates/iced-swdir-tree).

```text
▾ 📂 projects
  ▾ 📂 dioxus-swdir-tree
    ▸ 📁 crates
    ▸ 📁 rfcs
      📄 Cargo.toml
      📄 README.md
  ▸ 📁 sandbox
    📄 notes.txt
```

## Status

**v0.7 — full feature parity with `iced-swdir-tree` 0.7.** All ten features
are implemented and tested against the upstream specification. The `DirectoryTreeView`
Dioxus component is available now. See [ROADMAP.md](ROADMAP.md).

The crate never reaches **v1.0.0 without explicit confirmation by the project
owner** — this is recorded policy (RFC 001), not just convention.

## Why this design

The widget is a **viewer with gestures, not a file manager**:

- **Lazy by contract.** One `swdir::scan_dir` per expansion gesture, one level
  deep, never recursive. A million-file home directory costs only what you
  actually open.
- **Side effects as data.** State transitions never spawn tasks. Expanding an
  unloaded directory returns a `ScanRequest`; you run it through the
  `use_scan_driver` Dioxus hook (or any worker) and feed the `LoadPayload` back
  through `on_loaded`.
- **Stale results can't corrupt the tree.** Every scan carries a generation tag;
  payloads are accepted only on strict generation equality. Collapse during an
  in-flight scan, re-expand, and the late result is silently discarded —
  the tree is bit-identical afterwards (the test suite proves it with `PartialEq`).
- **Filters are free.** Raw, unfiltered scan results are cached per directory,
  so switching between *FoldersOnly* / *FilesAndFolders* / *AllIncludingHidden*
  rebuilds the tree with zero I/O and preserves expansion state.
- **Search sees through collapse.** Incremental search walks the entire loaded
  graph regardless of which directories are expanded, surfacing deep matches
  with all their ancestors — no I/O triggered.

## Quickstart

Add the flagship crate:

```toml
[dependencies]
dioxus-swdir-tree = "0.7"
```

Wire it up in a Dioxus app:

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
        }
    };

    rsx! { DirectoryTreeView { tree, on_event } }
}
```

For scripts and tests, `expand_blocking` runs the full scan→merge cycle
synchronously:

```rust
use dioxus_swdir_tree_core::DirectoryTree;

let mut tree = DirectoryTree::new("/home/me/projects");
tree.expand_blocking(std::path::Path::new("/home/me/projects"));
for (node, depth) in tree.visible_rows() {
    println!("{}{}", "  ".repeat(depth as usize), node.file_name().display());
}
```

## Features

| Feature | API entry point |
|---|---|
| Lazy one-level loading | `on_toggled` / `on_loaded` / `use_scan_driver` |
| Display filters | `with_filter` / `set_filter` |
| Single & multi-selection | `on_selected` / `SelectionMode` |
| Keyboard navigation | `handle_key` / `TreeKey` wired in `DirectoryTreeView` |
| Pluggable async scanning | `ScanExecutor` / `ThreadExecutor` |
| Drag & drop | `on_drag_msg` / `DragMsg` / `DragOutcome` |
| Speculative prefetch | `with_prefetch_limit` / `DEFAULT_PREFETCH_SKIP` |
| Incremental search | `set_search_query` / `search_match_count` |
| Icon themes | `IconTheme` / `UnicodeTheme` / `LucideTheme` (feature `icons`) |

## Workspace layout

| Crate | Contents | Dependencies |
|---|---|---|
| [`dioxus-swdir-tree-core`](crates/dioxus-swdir-tree-core) | Framework-free state machine: all transitions, generations, cache, selection, drag, prefetch, search, icon trait | `swdir` only |
| [`dioxus-swdir-tree`](crates/dioxus-swdir-tree) | Flagship crate: `DirectoryTreeView`, `use_scan_driver`, icon wiring | core + `dioxus 0.7` |

## What this widget is *not*

It never moves, renames, deletes, or watches files. Drag & drop ends in a
`DragCompleted { sources, destination }` event — performing the actual move and
refreshing affected folders is the application's job. Incremental search matches
only what is already loaded; it never triggers I/O.

## Design record

Every feature is specified by an RFC before it is built — see
[rfcs/README.md](rfcs/README.md) for the full index (all 12 RFCs in `done/`) and
[rfcs/done/000-rfc-lifecycle-policy.md](rfcs/done/000-rfc-lifecycle-policy.md)
for the process. Longer-form guides live in [docs/](docs/src/SUMMARY.md).

## License

Apache-2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
