# RFC 010 — Incremental search

**Status.** Proposed
**Tracks.** Feature 9 (`feature-specs.md` §9); search-state
section of `data-model.md`; search-aware `visible_rows()`
dispatch in `state-machine.md`.
**Touches.** `crates/dioxus-swdir-tree-core/src/` — new
`search.rs` (`SearchState`, `recompute_search_visibility`,
`walk_for_search`); `tree.rs` (`set_search_query`,
`clear_search`, `search_match_count`); `visible_rows()`
dispatch in `transitions.rs`; tests `tests/search.rs`.

## Summary

`set_search_query(q)` activates a live filter over the
**already-loaded** node graph — search never scans (S9.9). The
application owns the text input and wires keystrokes to
`set_search_query`; the widget owns the matching and the
visible set.

```
SearchState {
    query:         String            ← original casing
    query_lower:   String            ← for comparisons
    visible_paths: HashSet<PathBuf>  ← matches ∪ ancestors-of-matches
    match_count:   usize             ← direct matches only
}
```

## Design

- **Match rule (S9.1):** case-insensitive substring on the
  **basename** only; `/src/` in a parent component does not
  match `"src"`.
- **Visible set (S9.2):** every matching path plus every proper
  ancestor of a match.
- **Sees through collapse (S9.3):** the recompute walks the
  loaded tree ignoring `is_expanded`; matches inside a
  collapsed-but-loaded subtree surface, ancestors force-shown.
- **Empty query clears (S9.4):** `set_search_query("")` ≡
  `clear_search()`; no empty-string-active state exists.
- **Recompute triggers:** `set_search_query`, `set_filter`
  (S9.6 — filter first, then search over the survivors), and
  every accepted `on_loaded` (S9.7 — new children may match).
  RFCs 002/003 left this hook in place.
- **`visible_rows()` dispatch:** when search is active, the
  pre-order walk gates on `visible_paths` instead of
  `is_expanded` and always descends. View, keyboard, and range
  selection automatically become search-aware because they all
  consume `visible_rows()` (composability rules).
- **Orthogonality (S9.5):** search never touches
  `selected_paths`; hidden-by-search selections persist.
- **`match_count` (S9.8):** direct matches only — what apps
  show as "N results"; ancestors are context, not results.

**Documented limitation (upstream-acknowledged):** expanding a
folder during search does not escape the filter; clearing the
search is the way back to explore mode.

## Alternatives considered

- **Incremental (delta) recompute.** Rejected for now: the
  full-walk recompute is O(N_loaded), N_loaded is bounded by
  what the user has expanded plus prefetch, and the upstream
  reference uses the same strategy. Optimize behind the same
  API if profiling ever demands it.
- **Matching full relative paths.** Rejected: contradicts S9.1
  and produces confusing matches via ancestor directory names.
- **Search triggering scans of unloaded subtrees.** Rejected:
  core principle — the widget is not a search indexer.

## Test plan

`tests/search.rs` encodes S9.1–S9.9 per clause, including
see-through-collapse, filter+search composition order,
recompute-on-load revealing new matches, match-count vs
visible-count divergence, and selection persistence under
search.

## Open questions

None.
