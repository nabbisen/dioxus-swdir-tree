# RFC 003 — Display filters

**Status.** Implemented (v0.1.0)
**Tracks.** Feature 2 (display filters) from the upstream design
documents (`feature-specs.md` §2).
**Touches.** `crates/dioxus-swdir-tree-core/src/config.rs`
(`DisplayFilter`), `entry.rs` (hidden detection), `tree.rs`
(`set_filter`), child-rebuild logic in `transitions.rs`;
integration tests `tests/display_filters.rs`.

## Summary

Three display modes control which scanned entries become visible
tree nodes:

| Mode | Shows |
|---|---|
| `FoldersOnly` | non-hidden directories only |
| `FilesAndFolders` *(default)* | non-hidden files and directories |
| `AllIncludingHidden` | everything `scan_dir` returned |

Switching modes at runtime is instant and issues **zero I/O**:
the `TreeCache` (RFC 002) stores the raw, unfiltered entry list
of every completed scan, and `set_filter` re-derives every loaded
node's child list from it in memory.

The test oracle is `feature-specs.md` S2.1–S2.7.

## Design

### Hiddenness

Defined per OS, computed once at scan time and stored on
`LoadedEntry`:

- Unix: basename starts with `.`.
- Windows: `FILE_ATTRIBUTE_HIDDEN` (0x2) from the entry's cached
  metadata, with the dotfile rule as fallback.
- Other platforms: dotfile rule.

### Filter predicate

```
FoldersOnly        → is_dir && !is_hidden
FilesAndFolders    → !is_hidden
AllIncludingHidden → true
```

Note S2.2's subtlety: a *hidden directory* is hidden under
`FoldersOnly` too — the mode is "folders only", not "everything
that is a folder".

The filter applies to children only; the root node is always
visible regardless of mode (S2.1).

### `set_filter(filter)`

- Equal to current filter → no-op.
- Otherwise: store the new filter; for every cached path, rebuild
  that node's children as `cache[path].entries.filter(pred)`.
  The rebuild path-matches against the node's previous children
  so expansion and loaded state survive (S2.7) — the same
  `rebuild_children` helper `on_loaded` uses (RFC 002 step 3).
- No generation bump; no `ScanRequest` produced.
- Later dimensions re-sync afterwards once they exist: selection
  flags (RFC 004), search visibility (RFC 010). Their hooks are
  called from `set_filter` as those RFCs land.

Selection survival (S2.6) is specified here but verified fully
when RFC 004 introduces `selected_paths`; the structural
guarantee — filters never delete state, only derive views — is
established now.

### Interaction with `on_loaded`

The merge step always caches the **raw** entry list and derives
children through the *current* filter. Consequence: a folder
expanded under `FoldersOnly` already has its files in cache, and
flipping to `FilesAndFolders` reveals them instantly.

## Alternatives considered

- **Re-scanning on filter change.** Rejected: turns a pure view
  concern into I/O, breaks instant switching, and races with the
  generation protocol for no benefit.
- **Filtering inside `swdir`** (passing filter rules to the scan).
  Rejected for this widget: the cache must hold the unfiltered
  truth, otherwise mode switches need re-scans.
- **A user-supplied filter closure.** Deferred. The three fixed
  modes match the upstream contract; an open predicate is a
  post-1.0 extension candidate.

## Test plan

`tests/display_filters.rs` encodes S2.1–S2.7 one test per clause,
including: hidden-dir-under-`FoldersOnly`, instant re-derivation
without new scans (asserted via generation stability and absence
of produced `ScanRequest`s), and expansion-state survival across
a filter flip.

## Open questions

None.
