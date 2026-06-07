# RFC 007 — Keyboard navigation

**Status.** Proposed
**Tracks.** Feature 4 (`feature-specs.md` §4).
**Touches.** `crates/dioxus-swdir-tree-core/src/` — new
`keyboard.rs` (`TreeKey`, `Modifiers`, `handle_key`);
`crates/dioxus-swdir-tree/src/view.rs` (`onkeydown` bridge);
tests `tests/keyboard.rs`.

## Summary

`handle_key(key, modifiers) -> Option<DirectoryTreeEvent>`
translates a key press into a tree event, or `None` when the key
is unbound — so host applications can layer their own shortcuts
safely. The core consumes a framework-neutral `TreeKey` enum;
the view crate maps `dioxus` keyboard events
(`evt.key()` / `evt.modifiers()`) onto it.

All movement is computed over `visible_rows()` relative to the
current row (the row for `active_path`); if `active_path` is not
visible, movement keys no-op. Movement never wraps.

## Bindings (S4.1–S4.10)

| Key | Event produced |
|---|---|
| `↑` / `↓` | `Selected(prev/next, Replace)`; no wrap |
| `Shift+↑` / `Shift+↓` | `Selected(prev/next, ExtendRange)` |
| `Home` / `End` | `Selected(first/last, Replace)` |
| `Shift+Home` / `Shift+End` | `Selected(first/last, ExtendRange)` |
| `Enter` | `Toggled(active)` iff active is a directory; else `None` |
| `Space`, `Ctrl+Space` | `Selected(active, Toggle)` |
| `←` | expanded dir → `Toggled` (collapse); collapsed dir or file → `Selected(parent, Replace)`; at root → `None` |
| `→` | collapsed dir → `Toggled` (expand); expanded dir → `Selected(first child, Replace)`; file → `None` |
| `Escape` | cancels drag iff a drag is active (RFC 008); otherwise `None` — deliberately unbound so hosts may use it |

`handle_key` is read-only: it inspects state and *produces* an
event; the host dispatches that event back through the normal
`on_selected` / `on_toggled` path. This keeps a single mutation
funnel and makes the function trivially testable.

## View-crate bridge

The focusable tree container (RFC 006) attaches `onkeydown`,
maps the web `Key`/`Modifiers` to `TreeKey`, calls
`handle_key`, and forwards any produced event to `on_event`.
Arrow/Space/Enter call `prevent_default()` only when the key
was actually consumed, so page scrolling still works when the
tree ignores a key.

## Alternatives considered

- **Global document listener** (porting guide's alternative).
  Deferred: the focusable-container approach composes better
  with multiple widgets per page; a global option can be an
  opt-in helper later.
- **Letting `handle_key` mutate directly.** Rejected: two
  mutation entry points means two code paths to test; the
  event-producing design matches the upstream API.

## Test plan

`tests/keyboard.rs` encodes S4.1–S4.10 per clause against a
fixture tree, including: no-wrap at boundaries, `←`/`→`
tri-state behaviour, Escape's bound/unbound duality (the drag
half lands with RFC 008 and extends this file).

## Open questions

- Type-ahead (jump-to-row by typed prefix) is mentioned in
  upstream prose but is not part of the ten-feature spec; out
  of scope, possible post-1.0 RFC.
