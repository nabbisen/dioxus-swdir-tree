# dioxus-swdir-tree-core

The framework-free state machine behind
[`dioxus-swdir-tree`](https://crates.io/crates/dioxus-swdir-tree): a lazily
loading, searchable, drag-and-drop-aware directory tree over
[`swdir`](https://crates.io/crates/swdir).

**This crate depends only on `swdir`** — no Dioxus, no GUI framework — so it
is testable anywhere and reusable from other frontends. Most applications
should depend on the flagship `dioxus-swdir-tree` crate instead.

## What's inside

- `DirectoryTree` / `TreeNode` — the tree, the four toggle cases, and the flat
  `visible_rows()` draw model used by every consumer.
- `ScanRequest` / `LoadPayload` / `scan::run` — side effects as data; the
  embedding layer decides where blocking scans run.
- `ScanExecutor` / `ThreadExecutor` — pluggable async scan dispatch.
- `DisplayFilter` / `TreeCache` — zero-I/O filter switching with a raw-entry cache.
- `SelectionMode` / `on_selected` — insertion-ordered multi-selection with
  Replace, Toggle, and ExtendRange modes.
- `handle_key` / `TreeKey` / `Modifiers` — read-only keyboard navigation over
  `visible_rows()`.
- `DragMsg` / `DragOutcome` / `on_drag_msg` — press → hover → release drag
  protocol; emits `DragCompleted { sources, destination }`.
- Prefetch — `with_prefetch_limit`, `DEFAULT_PREFETCH_SKIP`, parallel
  speculative pre-expansion.
- `set_search_query` / `SearchState` — incremental basename substring search
  that sees through collapsed subtrees, zero I/O.
- `IconTheme` / `IconRole` / `UnicodeTheme` — pluggable icon rendering;
  `LucideTheme` available behind the `icons` feature.

License: Apache-2.0.
