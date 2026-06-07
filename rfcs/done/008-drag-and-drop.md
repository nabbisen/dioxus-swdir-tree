# RFC 008 — Drag-and-drop

**Status.** Implemented (v0.5.0)
**Tracks.** Feature 7 (`feature-specs.md` §7); drag transitions
in `state-machine.md`; drag-state section of `data-model.md`.
**Touches.** `crates/dioxus-swdir-tree-core/src/` — new
`drag.rs` (`DragState`, `DragMsg`, transitions, target
validity); `crates/dioxus-swdir-tree/src/row.rs` + `view.rs`
(synthetic mouse-event wiring, drop overlay); tests
`tests/drag_drop.rs`.

## Summary

The widget tracks press → hover → release and emits
`DragCompleted { sources, destination }` on a valid drop. **It
never performs filesystem operations** — moving, copying, or
rejecting is the application's decision, and the application
must re-toggle affected folders to refresh afterwards (core
principle 4).

```
DragState { sources: Vec<PathBuf>, hovered_target: Option<PathBuf>, started_at: PathBuf }
tree.drag: Option<DragState>
```

## Core transitions (upstream verbatim)

- **`Pressed(path, is_dir)`** — sources = the whole selection if
  `path ∈ selected_paths`, else `[path]`. Drag becomes active.
- **`Entered(path)`** — `hovered_target = Some(path)` iff valid:
  a directory, not a source, not a descendant of any source
  (prefix check on components, not string prefix). Else `None`.
- **`Exited(path)`** — clears `hovered_target` only if it still
  equals `path` (guards against out-of-order enter/leave).
- **`Released(path)`** — if `path == started_at`: this was a
  click, not a drag; clear drag and emit a deferred
  `Selected(path, Replace)` (S7.2 — single-click selection is
  *derived* from press/release, replacing RFC 006's interim
  `onclick`). Otherwise capture sources + destination, clear
  drag, emit `DragCompleted`.
- **`Cancelled`** — clear drag (Escape via RFC 007, or
  mouse-up over no valid target).

Orthogonality: `set_filter` and `set_search_query` never clear
`drag` (S7.6); drop targets are constrained to currently visible
rows.

## View wiring: synthetic mouse events

HTML5 native DnD is rejected (upstream guidance: no custom drag
image for multi-selection, browser-owned cursors, poor fit with
validity logic). Instead:

- Rows attach `onmousedown` → `Pressed`, `onmouseenter` →
  `Entered`, `onmouseleave` → `Exited`.
- While a drag is active the component mounts a full-viewport
  overlay capturing `onmousemove` (ghost badge follows cursor;
  shows source count) and `onmouseup`. Mouse-up resolves the
  current `hovered_target`: over a row → `Released(row_path)`;
  over nothing valid → `Cancelled`.
- The hovered valid target row gets `dx-swdir-row--drop-target`.
- A small movement threshold (a few px from press) distinguishes
  intentional drags before showing the ghost, while the
  press/release state machine still resolves clicks correctly
  even without the threshold.

## Alternatives considered

- **HTML5 drag events.** Rejected, reasons above.
- **Auto-refresh destination after `DragCompleted`.** Rejected:
  the widget cannot know whether the app accepted the drop; the
  hard widget/application line says refresh is explicit.
- **Click selection as a separate `onclick`.** Rejected once
  this RFC lands: press/release already disambiguates click vs
  drag; a parallel click handler would double-fire.

## Test plan

`tests/drag_drop.rs` encodes S7.1–S7.6: source-set derivation
(selected vs unselected press), the three-clause target
validity (including descendant-of-source via component-wise
prefix), click-vs-drop on `Released`, Escape cancellation, and
drag survival across `set_filter`. View-level ghost/overlay
behaviour is exercised in the example app.

## Open questions

- Whether the ghost badge is part of the default style feature
  or always rendered. Leaning: part of `default-style`.
