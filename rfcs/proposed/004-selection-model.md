# RFC 004 — Selection model: single and multi-select

**Status.** Proposed
**Tracks.** Features 3 (single-path selection) and 6
(multi-select) from `feature-specs.md` §3 and §6;
selection-state section of `data-model.md`.
**Touches.** `crates/dioxus-swdir-tree-core/src/` — new
`selection.rs`; `tree.rs` (fields + accessors), `node.rs`
(`is_selected`), `transitions.rs` (`on_selected`, selection sync
hook in `on_loaded` / `set_filter`); tests
`tests/selection_single.rs`, `tests/selection_multi.rs`.

## Summary

Selection is held **by path, not by node**. The authoritative
state is an insertion-ordered, duplicate-free `Vec<PathBuf>` on
the tree root, plus two cursors:

```
selected_paths: Vec<PathBuf>     ← source of truth
active_path:    Option<PathBuf>  ← most-recently-touched row (focus styling)
anchor_path:    Option<PathBuf>  ← Shift-range pivot
```

Per-node `is_selected` flags are derived view hints, re-synced
(`sync_selection_flags`, O(N_loaded)) after every mutation that
rebuilds nodes. A path stays selected through filter changes,
re-merges, and (later) search, even while no node for it exists.

## Design

### `on_selected(path, is_dir, mode: SelectionMode)`

All modes set `active_path = Some(path)` and end with a flag
sync. No side effects, ever.

- **`Replace`** — `selected_paths = [path]`;
  `anchor_path = Some(path)`. Clicking an already-selected row
  with Replace still yields exactly that one row selected (S3.4
  — no toggle semantics).
- **`Toggle`** — remove if present, append if absent;
  `anchor_path = Some(path)`.
- **`ExtendRange`** — if `anchor_path` is `None`, behave as
  Replace. Otherwise locate anchor and target in
  `visible_rows()` and replace `selected_paths` with the
  inclusive slice between them. **The anchor does not move**
  (S6.3): repeated Shift-clicks grow or shrink the range from
  the same pivot.

### Accessors

- `selected_paths() -> &[PathBuf]`
- `selected_path() -> Option<&Path>` — the single-select
  accessor; per S3.3 this is a view onto `active_path`, *not*
  the last element of `selected_paths`.
- `is_selected(path) -> bool` — consults `selected_paths` only.

### Orthogonality obligations

- `set_filter` and `on_loaded` never modify `selected_paths`;
  they only re-derive flags (S6.4). RFC 002/003 left the sync
  hook in place; this RFC fills it in.
- A selected path hidden by filter (or unloaded) remains
  selected and resurfaces with its flag when visible again
  (S6.5, S2.6).

### Modifier mapping (for later RFCs)

The core consumes `SelectionMode`; it never inspects keyboard
modifiers. Mapping click+Ctrl → `Toggle`, click+Shift →
`ExtendRange` is component-layer work (RFC 006/007), because in
Dioxus modifier state arrives on the mouse/keyboard event, not
in widget state.

## Alternatives considered

- **`HashSet<PathBuf>` for the selection.** Rejected: insertion
  order is part of the contract (apps receive sources in the
  order the user picked them, e.g. for drag payloads).
- **Authoritative per-node flags.** Rejected: nodes are
  ephemeral; selection must survive node rebuilds. This is core
  principle 3 of the upstream design.
- **Anchor moving on ExtendRange.** Rejected: breaks standard
  list-range UX and contradicts S6.3.

## Test plan

`tests/selection_single.rs` encodes S3.1–S3.4;
`tests/selection_multi.rs` encodes S6.1–S6.5, including
range-over-`visible_rows()` with collapsed branches and
selection survival across `set_filter` and re-merge.

## Open questions

None.
