# dioxus-swdir-tree

[![crates.io](https://img.shields.io/crates/v/dioxus-swdir-tree?label=rust)](https://crates.io/crates/dioxus-swdir-tree)
[![License](https://img.shields.io/github/license/nabbisen/dioxus-swdir-tree)](https://github.com/nabbisen/dioxus-swdir-tree/blob/main/LICENSE)
[![Documentation](https://docs.rs/dioxus-swdir-tree/badge.svg?version=latest)](https://docs.rs/dioxus-swdir-tree)
[![Dependency Status](https://deps.rs/crate/dioxus-swdir-tree/latest/status.svg)](https://deps.rs/crate/dioxus-swdir-tree)

A directory-tree explorer widget for [Dioxus](https://dioxuslabs.com) GUI apps —
lazy loading, display filters, multi-selection, keyboard navigation, drag & drop —
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

**v0.1.0 — core state machine.** This release ships the framework-free heart of
the widget as `dioxus-swdir-tree-core`, fully tested against the upstream
feature specification (Features 1–2, clauses S1.x / S2.x). The flagship
`dioxus-swdir-tree` crate currently re-exports the core; the Dioxus
`DirectoryTreeView` component arrives in v0.3.0. See [ROADMAP.md](ROADMAP.md).

The crate never reaches **v1.0.0 without explicit confirmation by the project
owner** — this is recorded policy (RFC 001), not just convention.

## Why this design

The widget is a **viewer with gestures, not a file manager**:

- **Lazy by contract.** One `swdir::scan_dir` per expansion gesture, one level
  deep, never recursive. A million-file home directory costs only what you
  actually open.
- **Side effects as data.** State transitions never spawn tasks. Expanding an
  unloaded directory returns a `ScanRequest`; *you* run it on whatever async
  story your app has (a Dioxus coroutine in v0.3, a thread, or inline in tests)
  and feed the `LoadPayload` back through `on_loaded`.
- **Stale results can't corrupt the tree.** Every scan carries a generation tag;
  payloads are accepted only on strict generation equality. Collapse during an
  in-flight scan, re-expand, and the late first result is silently discarded —
  the tree is bit-identical afterwards (and the test suite proves it with
  `PartialEq`).
- **Filters are free.** Raw, unfiltered scan results are cached per directory,
  so switching between *FoldersOnly* / *FilesAndFolders* / *AllIncludingHidden*
  rebuilds the tree with zero I/O and preserves your expansion state.

## Quickstart (v0.1: core API)

```rust
use dioxus_swdir_tree::{DirectoryTree, DisplayFilter, scan};
use std::path::Path;

let mut tree = DirectoryTree::new("/home/me/projects")
    .with_filter(DisplayFilter::FilesAndFolders);

// A click on a collapsed, unloaded directory produces a scan request…
if let Some(request) = tree.on_toggled(Path::new("/home/me/projects")) {
    // …which you execute off the UI thread, then merge:
    let payload = scan::run(&request); // blocking
    tree.on_loaded(payload);
}

// One flat, indented row list drives rendering (and later: keyboard
// navigation and range selection — they can never disagree).
for (node, depth) in tree.visible_rows() {
    println!("{}{}", "  ".repeat(depth as usize), node.file_name().display());
}
```

In tests and quick scripts, `tree.expand_blocking(path)` collapses the three
steps above into one synchronous call.

## Workspace layout

| Crate | Contents | Dependencies |
|---|---|---|
| [`dioxus-swdir-tree-core`](crates/dioxus-swdir-tree-core) | Framework-free state machine: `DirectoryTree`, `TreeNode`, generations, cache, filters | `swdir` only |
| [`dioxus-swdir-tree`](crates/dioxus-swdir-tree) | Flagship crate; Dioxus `DirectoryTreeView` from v0.3.0 | core (+ `dioxus` from v0.3) |

## What this widget is *not*

It never moves, renames, deletes, or watches files. Drag & drop (v0.5) ends in
a `DragCompleted { sources, destination }` event — performing the move and
refreshing the affected folders is the application's job. There is no search
indexer either: incremental search (v0.7) matches only what is already loaded.

## Design record

Every feature is specified by an RFC before it is built — see
[rfcs/README.md](rfcs/README.md) for the index and
[rfcs/done/000-rfc-lifecycle-policy.md](rfcs/done/000-rfc-lifecycle-policy.md)
for the process. Longer-form guides live in [docs/](docs/src/SUMMARY.md).

## License

Apache-2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
