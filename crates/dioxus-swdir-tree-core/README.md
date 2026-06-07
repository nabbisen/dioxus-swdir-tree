# dioxus-swdir-tree-core

The framework-free state machine behind
[`dioxus-swdir-tree`](https://crates.io/crates/dioxus-swdir-tree): a lazily
loading directory tree over [`swdir`](https://crates.io/crates/swdir).

- `DirectoryTree` / `TreeNode` — the tree, its four toggle cases, and the
  flat `visible_rows()` draw model.
- `ScanRequest` / `LoadPayload` / `scan::run` — side effects as data: the
  embedding layer decides where the blocking scan runs.
- Generation-tagged staleness protocol — late scan results are discarded
  without corrupting state.
- `DisplayFilter` + `TreeCache` — zero-I/O filter switching.

This crate depends only on `swdir` — no Dioxus, no GUI framework — so it is
testable everywhere and reusable from other frontends. Most applications
should depend on the flagship `dioxus-swdir-tree` crate instead.

License: Apache-2.0.
