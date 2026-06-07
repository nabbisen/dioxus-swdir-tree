# RFC 006 — Dioxus view component

**Status.** Proposed
**Tracks.** The rendering layer: `porting-to-dioxus.md`
(rendering, conceptual mapping) and the view-side half of the
responsibility split in `core-design.md`.
**Touches.** `crates/dioxus-swdir-tree/src/` — `view.rs`
(`DirectoryTreeView`), `row.rs` (`TreeRow`), `style.rs`,
`lib.rs`; adds the `dioxus ^0.7` dependency; an
`examples/explorer` desktop example.

## Summary

`DirectoryTreeView` is the flagship component: it reads
`visible_rows()` from a `Signal<DirectoryTree>` and renders one
`TreeRow` per visible node — caret, icon, label, indentation —
emitting `DirectoryTreeEvent`s through an `EventHandler` prop.
The component renders state; it never owns business logic. All
behaviour stays in `dioxus-swdir-tree-core`.

```rust
#[component]
pub fn DirectoryTreeView(
    tree: Signal<DirectoryTree>,
    on_event: EventHandler<DirectoryTreeEvent>,
) -> Element
```

## Design

### Structure

- Container: `div.dx-swdir-tree` with `overflow-y: auto;
  height: 100%` (the `scrollable` ⟷ CSS mapping from the
  porting guide). Focusable (`tabindex: "0"`) so RFC 007 can
  attach `onkeydown`.
- Rows: flat `for (node, depth) in visible_rows()` — no DOM
  recursion. Key = the node path. Indentation =
  `padding-left: depth × indent`.
- Row anatomy: caret span (directories only) → icon span →
  basename label. Loading indicator while
  `is_expanded && !is_loaded`; greyed error row when
  `error.is_some()`.

### Event wiring (v0.3.0 scope)

- Caret / row activation → `on_event(Toggled(path))`; the host
  forwards into `tree.write().on_toggled(..)` and the RFC 005
  driver. Click-to-select arrives with RFC 008's press/release
  model (upstream S7.2 derives single click from drag
  release); until then a plain `onclick` → `Selected(..,
  Replace)` placeholder is acceptable and documented.
- Mouse handlers `onmousedown` / `onmouseenter` /
  `onmouseleave` / `onmouseup` are reserved for RFC 008.

### Styling contract

Class names are the public theming surface, prefixed
`dx-swdir-`: `-tree`, `-row`, `-row--selected`, `-row--active`,
`-row--error`, `-row--drop-target`, `-caret`, `-icon`,
`-label`. A small default stylesheet ships behind a
`default-style` feature (on by default); apps can disable it
and restyle everything. No inline colors beyond layout.

### Icons (baseline)

Until RFC 011, glyphs come from a built-in Unicode set (▸ ▾ 📁
📂 📄 ⚠) rendered in a span — no font registration. RFC 011
replaces this with the `IconTheme` trait without changing row
structure.

### State granularity

One monolithic `Signal<DirectoryTree>` (the porting guide's
recommended starting point). Split signals are a measured
optimisation, deferred until profiling motivates a follow-up
RFC.

## Alternatives considered

- **Recursive component per node.** Rejected: `visible_rows()`
  already linearizes draw order, the flat list keeps keyboard
  navigation and virtualization options open, and deep DOM
  nesting hurts styling.
- **Headless-only (no shipped markup).** Attractive long-term,
  but parity with `iced-swdir-tree`'s batteries-included widget
  comes first. A headless hook API can be extracted post-1.0.
- **Virtualized list now.** Deferred: correctness first;
  virtualization is invisible to the public API.

## Test plan

Headless `VirtualDom` tests in the view crate: rendered row
count and order match `visible_rows()`; caret click dispatches
`Toggled`; error and loading rows render their classes. The
`examples/explorer` desktop app serves as the manual test bed.

## Open questions

- Exact default indent (px) and row height — settle during
  implementation with the example app.
