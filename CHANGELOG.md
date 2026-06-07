# Changelog

All notable changes to this project are documented here. The format is based
on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-07

First release: the framework-free core state machine (RFCs 001–003).

### Added

- **Workspace** with two crates: `dioxus-swdir-tree-core` (state machine,
  depends only on `swdir ^0.11`) and `dioxus-swdir-tree` (flagship crate,
  currently a re-export of the core; the Dioxus component lands in v0.3.0).
- **Lazy loading** (Feature 1): `DirectoryTree::on_toggled` covering all four
  toggle cases, optimistic expansion, one-level non-recursive scans via
  `scan::run`, and `on_loaded` merging with the generation protocol —
  strict-equality staleness checks, silent discard of stale payloads,
  `max_depth` capping, and sticky (non-auto-retried) error nodes.
- **Display filters** (Feature 2): `FoldersOnly`, `FilesAndFolders` (default),
  `AllIncludingHidden`; hiddenness derived at scan time (dotfiles on Unix,
  `FILE_ATTRIBUTE_HIDDEN` or dotfile on Windows); `set_filter` rebuilds from
  the raw-entry `TreeCache` with zero I/O while preserving expansion state.
- **Row model**: `visible_rows()` — the single flat, depth-annotated draw list
  that rendering, keyboard navigation, and range selection will all share.
- **Test-oracle suite**: 13 integration tests named after specification
  clauses S1.1–S1.6 and S2.1–S2.7 (S2.6 deferred to RFC 004 with the selection
  model), plus unit tests and doctests. `expand_blocking` ports upstream's
  `__test_expand_blocking` as a supported public helper.
- **Design record**: RFC lifecycle policy adopted (RFC 000); RFCs 001–003
  implemented; RFCs 004–011 proposed, covering selection, async executors, the
  Dioxus view component, keyboard navigation, drag & drop, prefetch,
  incremental search, and icon themes.

[0.1.0]: https://github.com/nabbisen/dioxus-swdir-tree/releases/tag/0.1.0

## [0.2.0] - 2026-06-07

### Added

- **Selection model** (Feature 3 + Feature 6, RFC 004):
  - `SelectionMode` enum (`Replace` / `Toggle` / `ExtendRange`).
  - `DirectoryTree::on_selected(path, is_dir, mode)` — all three modes,
    anchor semantics, and per-call flag sync.
  - `selected_paths() -> &[PathBuf]` — insertion-ordered, duplicate-free.
  - `selected_path() -> Option<&Path>` — `active_path` view (S3.3).
  - `is_selected(&Path) -> bool` — authoritative query.
  - `TreeNode::is_selected` — derived view hint, re-synced by every
    selection mutation, `on_loaded` merge, and `set_filter` call.
  - Selection survives filter changes, node rebuilds, and re-merges
    (selected paths stay authoritative even while their nodes are
    filtered out or not yet loaded).

[0.2.0]: https://github.com/nabbisen/dioxus-swdir-tree/releases/tag/v0.2.0

## [0.3.0] - 2026-06-07

### Added

- **Pluggable async scanning** (Feature 5, RFC 005):
  - `ScanExecutor` trait — object-safe (`Arc<dyn ScanExecutor>`), one
    `spawn_blocking(job)` per `ScanRequest` (S5.2).
  - `ThreadExecutor` (default, S5.3) — one OS thread per scan via a
    `Mutex`/`Waker`-based future with no external runtime dependency.
  - `ScanJob` and `ScanFuture` type aliases.
- **Dioxus view component** (RFC 006):
  - `dioxus-swdir-tree` now depends on `dioxus ^0.7` (minimal feature
    set — no platform renderer in the library).
  - `DirectoryTreeView` component — flat row list from `visible_rows()`,
    caret + icon + label per row, `dx-swdir-*` CSS class names.
  - `use_scan_driver` hook — wraps `use_coroutine`; wires `ScanRequest`s
    from event handlers through the executor and back into the signal.
  - `DirectoryTreeEvent` enum (`Toggled`, `Selected`) — click-to-select
    placeholder with `SelectionMode::Replace` until RFC 007/008.
  - `default-style` feature (on by default) — injects a minimal baseline
    stylesheet; disable for full theming control.
  - `examples/explorer` — standalone desktop app (not a workspace member;
    requires `dioxus` with `features = ["desktop"]`).

[0.3.0]: https://github.com/nabbisen/dioxus-swdir-tree/releases/tag/v0.3.0

## [0.4.0] - 2026-06-07

### Added

- **Keyboard navigation** (Feature 4, RFC 007):
  - `TreeKey` enum — framework-neutral key representation (Up, Down,
    Home, End, Enter, Space, Left, Right, Escape).
  - `Modifiers` struct — `{ shift: bool, ctrl: bool }`.
  - `handle_key(tree, key, mods) -> Option<DirectoryTreeEvent>` —
    read-only; inspects `visible_rows()` and `active_path` to produce
    the correct event for all ten S4.x bindings without mutating state.
  - `DirectoryTreeEvent` moved from the view crate to `dioxus-swdir-tree-core`
    (re-exported in both places — no breaking API change).
  - `DirectoryTreeView` wires `onkeydown` on the focusable container:
    maps Dioxus `KeyboardEvent` → `TreeKey`, calls `handle_key`, calls
    `evt.prevent_default()` only when the key was consumed.
  - 26 integration tests covering every S4.x clause including no-wrap
    boundaries, Left/Right tri-state behaviour, and the Escape no-op.

[0.4.0]: https://github.com/nabbisen/dioxus-swdir-tree/releases/tag/v0.4.0
