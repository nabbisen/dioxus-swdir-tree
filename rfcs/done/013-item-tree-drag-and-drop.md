# RFC 013 — ItemTree drag-and-drop

**Status.** Implemented (v0.9.0)
**Tracks.** Feature 11 (continued) — drag-and-drop reorder/nest for `ItemTree<T>`.
**Touches.** `dioxus-swdir-tree-core` (`item_tree/drag.rs`, `ItemTree` drag state
+ transitions, `item_event` Drag variant, `keyboard` Escape) and
`dioxus-swdir-tree` (`ItemTreeView` / `ItemTreeRow` three-zone rows).
**Upstream reference.** `iced-swdir-tree` RFC 002 / `docs/design/feature-specs.md`
S11.9–S11.16. Reference implementation ships in `iced-swdir-tree` v0.9.0.
**Downstream driver.** `layered` — structural section-outline editing (reorder /
nest headings).

## Summary

Adds opt-in drag-and-drop to `ItemTree<T>`. Unlike `DirectoryTree` (drop is
always *into* a folder), an item-tree drop is placed relative to a target at
one of three [`DropPosition`]s — `Before` / `Into` / `After` — expressing both
reorder (sibling) and nest (child). The widget mutates nothing; it emits the
drop *intent* and the host rebuilds its model and calls `set_tree`.

`DirectoryTree` drag is unchanged. This is additive.

## Core design decisions

**`DropPosition { Before, Into, After }`** with unambiguous parent/index
mapping (S11.15): `Before` → (parent_of(target), index_of(target)); `Into` →
(target, end); `After` → (parent_of(target), index_of(target)+1).

**Outcome via `on_drag_msg`, not an event variant.** Mirroring our existing
`DirectoryTree::on_drag_msg`, `ItemTree::on_drag_msg(msg) -> ItemDragOutcome`
returns `Clicked(NodeId)` / `Completed { sources, target, position }` / `None`.
The host acts on the outcome. (iced models `DragCompleted` as an `update`
event because of its Elm architecture; the outcome-return shape is idiomatic
for our synchronous core, identical to how we already handle `DirectoryTree`.)

**Validity via the live arena (S11.12).** Our `InternalItem` stores `parent_id`
natively, so the cycle check is a direct O(depth) ancestor walk over the live
store — no parent-map snapshot (which iced needs only because its nested tree
lacks parent links). The four rules: target live; target ∉ sources;
Before/After requires non-root target; effective new parent is neither a
source nor a descendant of any source.

**Deferred selection (S11.11).** `Pressed` never mutates selection.
`Released(id, _)` with `id == primary` returns `Clicked(id)`; the host applies
`Selected(id, Replace)`. The `Released` position is ignored — the drop target
is the stored hover set by `Entered` (S11.10–S11.12).

**Opt-in (S11.9).** `ItemTree::with_drag_and_drop(true)`; off by default.
`on_drag_msg` is a no-op while disabled. The view reads
`is_drag_and_drop_enabled()` to decide between three-zone drag rows and plain
v0.8 click selection.

**Escape (S11.13).** `handle_key` returns `Drag(Cancelled)` while a drag is
active, `None` otherwise.

**Drag survives search (S11.16).** `set_search_query` does not touch drag state.

## View

Three mouse zones per row: a thin Before strip, the body (Pressed + Into), and
a thin After strip. Each emits `Entered` / `Exited` for its position; the body
also emits `Pressed`. `Released` fires on mouse-up over a zone; a container-level
mouse-up emits `Cancelled` as a fallback (consistent with `DirectoryTree`'s
drag). Active Before/After strips paint an insertion bar; the active Into body
paints a nest outline.

## Accessors (testing + view)

`is_drag_and_drop_enabled()`, `is_dragging()`, `drag_sources() -> &[NodeId]`,
`drop_target() -> Option<(NodeId, DropPosition)>`.

## Out of scope

Cross-tree drag, drag between an `ItemTree` and a `DirectoryTree`, and
multi-position cursor-geometry heuristics. The three discrete zones are the
whole interaction surface.
