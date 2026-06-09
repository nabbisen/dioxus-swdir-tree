# RFC 012 — Generic item tree (`ItemTree<T>`)

**Status.** Implemented (v0.8.0)
**Tracks.** Feature 11 — Generic item tree.
**Touches.** `dioxus-swdir-tree-core` (new `item_tree` module, `item_event`
module, `keyboard` additions) and `dioxus-swdir-tree` (new `ItemTreeView<T>`
component and `ItemTreeRow<T>`).
**Upstream reference.** `iced-swdir-tree` RFC 001 / `docs/design/feature-specs.md`
§11 (S11.x clauses). The iced reference implementation ships in v0.8.0.
**Downstream driver.** `layered` — a Dioxus Markdown editor whose section-outline
panel needs a keyboard-navigable, expandable, selectable tree over in-memory
`NodeId`-keyed data with no filesystem dependency.

## Summary

This RFC adds `ItemTree<T>` — a sibling widget to `DirectoryTree` that provides
the same keyboard navigation, multi-select, expand/collapse, and incremental
search for caller-supplied, in-memory node data. No async I/O, no generation
counter, no `swdir` dependency.

The existing `DirectoryTree` API is entirely unchanged. This is a purely
additive change.

## Core design decisions

**`display_fn` over `T: Display`.** The iced reference splits its API into
a base impl and a `T: Display` impl block. For the Dioxus port, a
`display_fn: Option<Arc<dyn Fn(&T) -> String + Send + Sync>>` stored at
construction time is cleaner: it avoids the split, decouples the display
representation from the type's own `Display` impl, and removes the need
for `ItemTree<T>` to be parameterized by a rendering bound. Search and
`VisibleItem::label` both use this function.

**`TreeKey` / `Modifiers` reuse.** The handoff mentions mapping from Dioxus
key events to `iced::keyboard::Key`. We already have framework-neutral `TreeKey`
and `Modifiers` in `keyboard.rs`; `handle_key_item` is implemented as a method
on `ItemTree<T>` using those same types.

**Key-based diffing in `set_tree`.** `set_tree(root: ItemNode<T>)` snapshots
`{ NodeId → (is_expanded, is_selected) }` before replacing the tree, then
copies surviving state into the rebuilt node graph. Position changes (a node
moving to a different parent) preserve state (S11.5). Disappeared keys are
silently dropped; surviving but moved keys are not (S11.4).

**`VisibleItem` as the row type.** `visible_rows()` returns
`Vec<VisibleItem>` — a fully pre-computed, view-friendly struct — rather
than references into internal storage. This avoids lifetime entanglement in
the Dioxus component.

**`T` bounds.** `T: Clone + Debug + Send + Sync + 'static`, matching iced.

## Deferred (out of scope for this RFC)

- Drag-and-drop for `ItemTree<T>`. The ancestry check for `NodeId`-keyed trees
  is O(depth) vs O(1) for `PathBuf`'s `starts_with`; the "drop between siblings"
  event shape is also unsettled. Deferred to a future RFC.
- `tree-nav-core` extraction. Deferred until both the iced and Dioxus
  implementations are stable and the degree of convergence can be measured.
- `NodeRemoved` event in `ItemTreeEvent`. If `layered` needs to react to removed
  nodes it should diff before calling `set_tree` itself.
