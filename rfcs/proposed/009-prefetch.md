# RFC 009 — Parallel pre-expansion (prefetch)

**Status.** Proposed
**Tracks.** Feature 8 (`feature-specs.md` §8); prefetch-state
section of `data-model.md`.
**Touches.** `crates/dioxus-swdir-tree-core/src/` — `config.rs`
(`prefetch_per_parent`, `prefetch_skip`,
`DEFAULT_PREFETCH_SKIP`), `tree.rs` (`prefetching_paths`),
`transitions.rs` (step-7 cascade logic, `Toggled` interplay);
driver fan-out in `crates/dioxus-swdir-tree/src/driver.rs`;
tests `tests/prefetch.rs`.

## Summary

With `prefetch_per_parent = N > 0`, completing a
**user-initiated** scan speculatively issues background scans
for up to N of the just-loaded folder's direct folder-children.
Prefetch sets `is_loaded` without touching `is_expanded`
(exploiting the loaded/expanded split from RFC 002), so the
user's later click is a zero-I/O fast path.

Disabled by default (`N = 0`): a tree without
`with_prefetch_limit` behaves exactly as before this RFC.

## Design

### Target selection (on user-initiated `on_loaded`)

Folder-children of the merged path that are: not yet loaded, not
in `prefetch_skip` (basename, exact, ASCII case-insensitive),
within `max_depth` — taken in child order up to N. For each
target: insert into `prefetching_paths`, bump the generation
(each prefetch gets its **own** generation), and append a
`ScanRequest` to `LoadedOutcome.prefetch_requests`. RFC 002
shaped the return type for exactly this; the RFC 005 driver
already fans the requests back into its channel, so the driver
needs no change beyond what it does today.

### Cascade prevention

`prefetching_paths: HashSet<PathBuf>` is the one-level
registry. When `on_loaded` merges a result whose path is in the
set, the path is removed and **no further prefetch is
triggered** — preventing `N^depth` fan-out (S8.3).

### User wins (S8.7)

`Toggled(P)` while `P ∈ prefetching_paths`: remove P from the
set, bump the generation, issue a user-initiated scan. The
in-flight prefetch result arrives stale and is discarded by the
generation check; the user-initiated result then triggers its
own prefetch wave normally.

### Default skip list

`.git`, `.hg`, `.svn`, `node_modules`, `__pycache__`, `.venv`,
`venv`, `target`, `build`, `dist`. The skip list never blocks a
user-initiated scan — it only filters speculation.

## Alternatives considered

- **Recursive prefetch with a depth budget.** Rejected:
  exponential fan-out risk is exactly what the one-level
  registry exists to prevent; the upstream spec forbids
  cascading.
- **Shared generation per wave.** Rejected: per-scan generations
  keep the staleness rule uniform (strict equality against the
  latest counter) with no special wave bookkeeping.
- **Prefetch on hover.** Out of scope; gesture-driven
  speculation is a view-layer policy that could feed the same
  core mechanism later.

## Test plan

`tests/prefetch.rs` encodes S8.1–S8.7: default-off equivalence,
exactly-one-wave per user scan, no cascade, loaded-not-expanded,
skip-list filtering (case-insensitivity included), `max_depth`
bound, and the user-wins race via manual payload injection.

## Open questions

None — the upstream spec is fully prescriptive here.
