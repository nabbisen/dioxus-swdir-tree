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

## [0.5.0] - 2026-06-07

### Added

- **Drag and drop** (Feature 7, RFC 008):
  - `DragState { sources, hovered_target, started_at, started_is_dir }` — the
    active drag session held on `DirectoryTree`.
  - `DragMsg` enum — `Pressed`, `Entered`, `Exited`, `Released`, `Cancelled`.
  - `DragOutcome` enum — `None`, `Clicked { path, is_dir }`, `Completed { sources, destination }`.
  - `DirectoryTree::on_drag_msg(msg) -> DragOutcome` — all five transitions
    with correct target validity (component-wise prefix, not string prefix, S7.3).
  - `DirectoryTree::drag_state() -> Option<&DragState>` accessor.
  - `DirectoryTreeEvent::Drag(DragMsg)` variant — the single event channel
    carries all gesture types.
  - `Escape` key now live: `handle_key` returns `Drag(Cancelled)` when a
    drag is active, `None` otherwise (completes S4.10 / S7.4).
  - View: rows use `onmousedown`/`onmouseenter`/`onmouseleave`/`onmouseup`
    (replaces RFC 006's `onclick` placeholder, per S7.2). Container
    `onmouseup` fires `Cancelled` when mouse-up misses all rows.
    `dx-swdir-row--drop-target` CSS class applied to the valid hovered target.
    Fixed-position ghost badge shows dragging item count while drag is active.
  - 15 integration tests covering S7.1–S7.6 including descendant validity,
    out-of-order Exited guard, and Escape key duality.

[0.5.0]: https://github.com/nabbisen/dioxus-swdir-tree/releases/tag/v0.5.0

## [0.6.0] - 2026-06-07

### Added

- **Prefetch / parallel pre-expansion** (Feature 8, RFC 009):
  - `TreeConfig::prefetch_per_parent: u32` (default `0` — disabled, S8.1).
  - `TreeConfig::prefetch_skip: Vec<String>` — default skip list (S8.5):
    `.git`, `.hg`, `.svn`, `node_modules`, `__pycache__`, `.venv`, `venv`,
    `target`, `build`, `dist` (exported as `DEFAULT_PREFETCH_SKIP`).
  - `DirectoryTree::with_prefetch_limit(n)` and
    `DirectoryTree::with_prefetch_skip(iter)` builder methods.
  - `DirectoryTree::prefetching_paths() -> &HashSet<PathBuf>` accessor.
  - `on_loaded` Step 7: after a **user-initiated** scan, issues up to N
    speculative `ScanRequest`s for not-yet-loaded, non-skip, within-depth
    folder-children (S8.2). All requests in one wave share a single bumped
    generation so each result independently passes the staleness check.
  - **No cascade**: completions of prefetch scans remove the path from
    `prefetching_paths` and return no further requests (S8.3).
  - **Loads, does not expand**: `is_loaded` set; `is_expanded` unchanged;
    subsequent user click is an instant Case-C no-op (S8.4).
  - **User wins** (S8.7): `on_toggled` on a prefetching path removes it from
    the registry and issues a fresh user-initiated scan; the in-flight
    prefetch result arrives stale and is silently discarded.
  - `use_scan_driver` fans prefetch requests out as concurrent `spawn`-ed
    tasks; no driver API change needed for applications.
  - 9 integration tests covering S8.1–S8.7 including case-insensitive skip
    list, max_depth bound, and the user-wins race.

[0.6.0]: https://github.com/nabbisen/dioxus-swdir-tree/releases/tag/v0.6.0

## [0.7.0] - 2026-06-07

**Feature parity with `iced-swdir-tree` 0.7** — all ten features implemented.

### Added

- **Incremental search** (Feature 9, RFC 010):
  - `SearchState { query, query_lower, visible_paths, match_count }` held as
    `Option<SearchState>` on the tree.
  - `set_search_query(q)` — activates/updates the live filter; empty string
    clears (S9.4). No I/O, no generation bump (S9.9).
  - `clear_search()` — alias for `set_search_query("")`.
  - `search_query() -> Option<&str>`, `search_state() -> Option<&SearchState>`,
    `search_match_count() -> usize` accessors.
  - `visible_rows()` dispatches on `visible_paths` when search is active;
    descends into all loaded dirs regardless of `is_expanded` (S9.3). All
    consumers (keyboard nav, range selection) automatically become search-aware.
  - Search recomputes on `set_filter` (filter first, S9.6) and on every
    accepted `on_loaded` (new children may match, S9.7).
  - 12 integration tests covering S9.1–S9.9 including see-through-collapse,
    filter+search composition, load-time recompute, and match count vs
    visible count divergence.

- **Icon themes** (Feature 10, RFC 011):
  - `IconRole` — six logical icon positions (S10.1), `#[non_exhaustive]`
    so minor releases may add roles (S10.2).
  - `IconSpec { glyph: Cow<'static, str>, font: Option<&'static str>,
    size: Option<f32> }` — CSS-native rendering spec (S10.3).
  - `IconTheme` trait — object-safe (`Arc<dyn IconTheme>`), one method
    (S10.7): `fn glyph(&self, role: IconRole) -> IconSpec`.
  - `UnicodeTheme` — emoji glyphs in the ambient system font; default without
    the `icons` feature (S10.5).
  - `LucideTheme` — Lucide vector glyphs with `font: Some("lucide")`; default
    with the `icons` feature (S10.5, S10.6). `LUCIDE_FONT_BYTES: &[u8]`
    placeholder exported for app-side `@font-face` registration.
  - `icons` feature on both crates; off by default.
  - View: optional `theme: Option<ArcTheme>` prop on `DirectoryTreeView`;
    rows render all six roles through the theme with correct `font-family` /
    `font-size` CSS styles.
  - 7 integration tests covering S10.1–S10.7.

[0.7.0]: https://github.com/nabbisen/dioxus-swdir-tree/releases/tag/v0.7.0

## [0.7.1] - 2026-06-07

### Changed

- **MSRV raised from 1.85 to 1.87.** Rust 1.87 lifts the floor for
  `wasip2` and `wit-bindgen` transitive dependencies, keeping the
  full dependency graph current.
  - `wasip2` updated 1.0.1+wasi-0.2.4 → 1.0.3+wasi-0.2.9
  - `wit-bindgen` updated 0.46.0 → 0.57.1

[0.7.1]: https://github.com/nabbisen/dioxus-swdir-tree/releases/tag/v0.7.1

## [0.7.2] - 2026-06-07

### Changed

- **MSRV removed.** The `rust-version` field has been dropped from all
  manifests. Minimum supported Rust is now determined in practice by the
  dependency graph rather than a hard-coded floor, giving users more
  flexibility and keeping the project unencumbered by an explicit constraint.

### Fixed

- Collapsed a redundant nested `if let` / `if` in `DragMsg::Exited`
  handling (clippy `collapsible_if`).

[0.7.2]: https://github.com/nabbisen/dioxus-swdir-tree/releases/tag/v0.7.2

## [0.8.0] - 2026-06-09

### Added

- **Generic item tree** (Feature 11, RFC 012), in parity with
  `iced-swdir-tree` v0.8.0 RFC 001 / spec §11:

  **Core (`dioxus-swdir-tree-core`):**
  - `NodeId(pub u64)` — opaque, caller-assigned node identity.
  - `ItemNode<T> { id, data, children }` — recursive caller input for
    `set_tree`.
  - `ItemTree<T: Clone + Debug + Send + Sync + 'static>` — in-memory tree
    with `set_tree`, `on_toggled`, `on_selected`, `handle_key`, and
    `set_search_query` / `clear_search`.
  - **Key-based diffing** (S11.4/S11.5): `set_tree` snapshots
    `NodeId → (is_expanded, is_selected)` before replacing the tree; state
    is preserved for surviving keys regardless of position changes; disappeared
    keys are silently dropped.
  - `display_fn: Option<Arc<dyn Fn(&T) -> String>>` stored at construction
    via `with_display(f)` — used for search and label rendering, avoiding the
    `T: Display` split used in the iced reference.
  - `VisibleItem` — pre-computed, owned row struct returned by `visible_rows()`,
    suitable for direct use as Dioxus component props.
  - `ItemTreeEvent { Toggled(NodeId), Selected(NodeId, SelectionMode) }` —
    no drag variant (drag deferred, see below).
  - `ItemSearchState` — incremental search state parallel to `SearchState`.
  - `handle_key` on `ItemTree<T>` reuses `TreeKey`/`Modifiers` from
    `keyboard.rs`; bindings are identical to `DirectoryTree`.

  **View (`dioxus-swdir-tree`):**
  - `ItemTreeView<T>` component — `Signal<ItemTree<T>>` + `EventHandler` +
    optional `ArcTheme`; wires `onkeydown` using the existing key mapping.
    No coroutine or scan driver needed.
  - `ItemTreeRow` — click fires `Selected(Replace)`; Ctrl-click
    `Toggle`; Shift-click `ExtendRange`; caret click `Toggled`.

  **21 integration tests** covering S11.2–S11.8 (expand/collapse, key-based
  diffing, position-change preservation, selection modes, ExtendRange, search).

### Deferred

- Drag-and-drop for `ItemTree<T>` — the `PathBuf::starts_with` ancestry check
  is O(1); the `NodeId` equivalent requires O(depth) tree traversal, and
  "drop between siblings" needs a distinct event shape. Deferred to a later RFC.
- `tree-nav-core` extraction — deferred until both iced and Dioxus
  implementations can be compared for convergence.

[0.8.0]: https://github.com/nabbisen/dioxus-swdir-tree/releases/tag/v0.8.0

## [0.9.0] - 2026-06-09

### Added

- **ItemTree drag-and-drop** (Feature 11 continued, RFC 013), in parity with
  `iced-swdir-tree` v0.9.0 RFC 002 / spec S11.9–S11.16:

  **Core (`dioxus-swdir-tree-core`):**
  - `DropPosition { Before, Into, After }` — reorder (sibling) and nest (child)
    with unambiguous parent/index mapping (S11.15).
  - `ItemDragMsg { Pressed, Entered, Exited, Released, Cancelled }` and
    `ItemDragOutcome { None, Clicked(NodeId), Completed { sources, target, position } }`.
  - `ItemTree::on_drag_msg(msg) -> ItemDragOutcome` drives the drag state
    machine; the widget mutates no node structure — the host rebuilds its
    model from `Completed` and calls `set_tree` (S11.14).
  - Validity check (S11.12) over the live arena via native `parent_id` links —
    O(depth) ancestor walk, no parent-map snapshot.
  - Deferred selection (S11.11): release on the press row returns `Clicked`;
    selection is never mutated on press.
  - `with_drag_and_drop(bool)` builder (opt-in, off by default, S11.9);
    accessors `is_drag_and_drop_enabled`, `is_dragging`, `drag_sources`,
    `drop_target`.
  - `Escape` cancels an active drag in `handle_key` (S11.13); drag survives
    `set_search_query` (S11.16).
  - `ItemTreeEvent::Drag(ItemDragMsg)` variant.
  - 12 validity unit tests + 17 state-machine integration tests.

  **View (`dioxus-swdir-tree`):**
  - `ItemTreeView` renders three drop zones per row when DnD is enabled — a
    Before strip, the body (Pressed + Into), and an After strip; plain
    clickable rows when disabled (v0.8 behaviour unchanged).
  - Active Before/After strips paint an insertion bar; the active Into body
    paints a nest outline. Container-level mouse-up cancels a stray drag.

### Notes

- **`tree-nav-core` extraction declined** (informational). The ecosystem
  decision is to share the *design* (the spec), not a crate: the navigation
  logic is written against each project's data structure (our arena vs. iced's
  nested tree) and the async models differ. `dioxus-swdir-tree-core` stays
  self-contained. Mirrors `iced-swdir-tree`'s withdrawn RFC 003.
- **No `NodeRemoved` event** (informational, unchanged). `set_tree`'s key-based
  diffing silently drops disappeared ids; the application owns the before/after
  diff. Matches RFC 012's out-of-scope decision and `iced-swdir-tree` RFC 001 [D4].

[0.9.0]: https://github.com/nabbisen/dioxus-swdir-tree/releases/tag/v0.9.0
