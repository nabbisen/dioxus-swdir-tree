# Core concepts

## Lazy loading & generations

**Lazy loading.** The tree never scans recursively. Expanding a directory
lists exactly one level via `swdir::scan_dir`. Children stay in memory across
collapse (`is_loaded` never reverts), so re-expanding is a free in-memory
operation (Case C).

**Side effects as data.** `on_toggled` returns an `Option<ScanRequest>`
instead of spawning anything. You execute `scan::run(&request)` off the UI
thread and merge the resulting `LoadPayload` with `on_loaded`. The
`use_scan_driver` Dioxus hook handles this loop automatically.

**Generations.** The tree carries a wrapping `u32` counter, bumped before
each user-initiated scan. A payload is merged only if its generation strictly
equals the current counter — anything else is silently discarded, leaving
the tree bit-identical. Rapid expand/collapse/expand is always safe.

**Error nodes.** A failed scan marks the node loaded-with-error; toggling it
afterwards collapses or expands without retrying the scan. The error persists
until the parent directory is refreshed.

## Selection

Selection is held **by path** on the tree, not by node reference, so it
survives node rebuilds, filter changes, and re-merges. Three modes:

- **Replace** — plain click; selects exactly one path.
- **Toggle** — Ctrl-click; adds if absent, removes if present.
- **ExtendRange** — Shift-click; selects the contiguous range in
  `visible_rows()` order from the anchor to the target. The anchor does not
  move on ExtendRange gestures.

## Keyboard navigation

`handle_key(tree, key, mods)` is read-only: it inspects `visible_rows()` and
`active_path`, then returns a `DirectoryTreeEvent` without mutating anything.
`DirectoryTreeView` wires `onkeydown` and calls `evt.prevent_default()` only
when the key is consumed.

## Drag and drop

`on_drag_msg(msg)` drives a press → hover → release state machine. It returns
a `DragOutcome`:

- `Clicked { path, is_dir }` — mouse-up on the same row as mouse-down; host
  should call `on_selected`.
- `Completed { sources, destination }` — genuine drop; host performs the
  filesystem operation then refreshes affected folders.

The widget never moves, copies, or deletes anything.

## Prefetch

With `with_prefetch_limit(n)`, each user-initiated scan triggers up to *n*
speculative background scans of direct folder-children (`is_loaded` is set,
`is_expanded` is not). Prefetch completions never cascade. The
`DEFAULT_PREFETCH_SKIP` list excludes `.git`, `node_modules`, `target`, and
other high-noise directories.

## Incremental search

`set_search_query(q)` filters `visible_rows()` to direct basename matches
and all their ancestors, walking the entire loaded graph regardless of which
directories are expanded. No I/O is triggered; only already-loaded nodes are
searched. The search recomputes automatically when the filter changes or new
children are merged by `on_loaded`.

## Icon themes

`IconTheme` is a rendering-only trait (`glyph(role) -> IconSpec`) consulted
while building rows. `UnicodeTheme` (default, emoji glyphs, no font
registration) and `LucideTheme` (behind the `icons` feature, requires a
`lucide` CSS font-family) are the two built-in themes. Custom themes implement
one method and must include a `_ =>` fallback arm because `IconRole` is
`#[non_exhaustive]`.
