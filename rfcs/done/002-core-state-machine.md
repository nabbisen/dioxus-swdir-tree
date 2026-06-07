# RFC 002 — Core data model and lazy-loading state machine

**Status.** Implemented (v0.1.0)
**Tracks.** Features 1 (lazy loading) and the shared data model /
generation protocol from the upstream design documents
(`data-model.md`, `state-machine.md`, `feature-specs.md` §1).
**Touches.** `crates/dioxus-swdir-tree-core/src/` — `tree.rs`,
`node.rs`, `config.rs`, `cache.rs`, `entry.rs`, `scan.rs`,
`error.rs`, `transitions.rs`; integration tests
`tests/lazy_loading.rs`.

## Summary

This RFC defines the heart of the widget: the in-memory data
model and the synchronous state machine that loads a directory
tree lazily — one level per user gesture — without ever blocking
on I/O inside a transition and without ever accepting a stale
scan result.

It implements the upstream specification S1.1–S1.6 verbatim. The
test oracle is `feature-specs.md` Feature 1.

## Data model (v0.1.0 subset)

```
DirectoryTree
├── root: TreeNode            ← created eagerly; never removed (invariant)
├── config: TreeConfig        ← root_path, filter, max_depth
├── cache: TreeCache          ← path → (generation, raw entries)
└── generation: u32           ← wrapping; bumped before each scan

TreeNode
├── path: PathBuf             ← absolute
├── is_dir: bool
├── is_expanded: bool         ← drawn open (is_expanded ⟹ is_dir)
├── is_loaded: bool           ← children populated by a scan
├── children: Vec<TreeNode>   ← empty iff !is_loaded (or genuinely empty)
├── error: Option<ScanIssue>  ← error ⟹ is_loaded ∧ children = []

LoadedEntry { path, is_dir, is_hidden }   ← owned, Send + 'static
ScanIssue   { path, kind: io::ErrorKind, message }  ← Clone + PartialEq
```

Fields defined by the upstream data model but owned by later
features arrive with their feature's RFC: `is_selected`,
`selected_paths`, `active_path`, `anchor_path` (RFC 004), `drag`
(RFC 008), `prefetching_paths`, `prefetch_per_parent`,
`prefetch_skip` (RFC 009), `search` (RFC 010), `icon_theme`
(RFC 011). Pre-1.0 this staging is acceptable and keeps every
release honest about what it contains.

`ScanIssue` replaces `swdir::ScanError` on the node because node
graphs must be `Clone` and comparable in tests; `io::Error` is
neither. The conversion preserves the failing path, the
`io::ErrorKind`, and the rendered message.

`LoadedEntry::is_hidden` is derived at scan time: dotfile rule on
Unix; on Windows the `FILE_ATTRIBUTE_HIDDEN` bit from the cached
`Metadata` plus the dotfile rule as fallback; dotfile rule
elsewhere. `swdir::DirEntry` carries cached `FileType` and
`Metadata`, so this costs no extra syscalls.

## State machine

Transitions are methods on `DirectoryTree`. They mutate state
synchronously and return side effects as data; they never spawn.

### `on_toggled(path) -> Option<ScanRequest>`

Implements the four cases of the upstream `Toggled` event:

- **A** — path is not a directory (or not found): no-op, `None`.
- **B** — expanded directory: collapse. `is_expanded = false`.
  Generation untouched (an in-flight descendant scan stays
  valid). `None`.
- **C** — collapsed, already loaded: fast path. `is_expanded =
  true`, no I/O, `None`.
- **D** — collapsed, not loaded: if `depth_of(root, path) >
  max_depth`, mark loaded-empty and return `None` (S1.5).
  Otherwise bump `generation` (wrapping), set `is_expanded =
  true`, return `Some(ScanRequest { path, generation, depth })`.

The caller (test helper now; Dioxus coroutine from RFC 005)
executes the request off the UI thread and feeds the result back.

### `on_loaded(payload: LoadPayload) -> LoadedOutcome`

```
LoadPayload   { path, generation, depth, result: Result<Vec<LoadedEntry>, ScanIssue> }
LoadedOutcome { accepted: bool, prefetch_requests: Vec<ScanRequest> }
```

Steps, in order (upstream `Loaded` steps 1–7):

1. **Staleness:** `payload.generation != self.generation` →
   discard silently; `accepted = false`; state bit-identical.
2. **Find node** by path; not found → discard silently.
3. **Merge:** `Ok(entries)` → children rebuilt from the filtered
   entry list; `Err(e)` → `children = []`, `error = Some(e)`.
   Either way `is_loaded = true`. Rebuilding **path-matches
   against the previous children** so an existing child's
   subtree, expansion, and loaded flags survive a re-merge.
4. **Cache:** on `Ok`, store `(generation, raw unfiltered
   entries)` under the path — the substrate for RFC 003's
   zero-I/O filter switching.
5–7. Selection sync, search recompute, prefetch cascade: no-ops
   until RFCs 004 / 010 / 009 land. `prefetch_requests` is
   always empty in v0.1.0; the return type is shaped for RFC 009
   now so the signature never breaks.

### Generation protocol

`u32`, `wrapping_add(1)` **before** each issued scan; acceptance
is **strict equality** (not `>=`, because of wrapping). Bumped
by: user expansion (and, later, each prefetch scan). Not bumped
by: collapse, `set_filter`, selection, search, drag.

### `visible_rows() -> Vec<(&TreeNode, depth)>`

Depth-first pre-order: visit root, then for each
filter-surviving child recurse iff `is_dir && is_expanded &&
is_loaded`. The single source of draw order for the view
(RFC 006), keyboard navigation (RFC 007), and range selection
(RFC 004). Search-aware dispatch is added by RFC 010.

### Blocking helpers

`scan::run(request) -> LoadPayload` performs the actual
`swdir::scan_dir_with_options(NameAscDirsFirst)` call and entry
conversion. `DirectoryTree::expand_blocking(path)` chains
`on_toggled` → `run` → `on_loaded` synchronously — the port of
upstream's `__test_expand_blocking`, letting the whole spec
suite bypass async infrastructure. It is public (not
test-gated): scripts and examples legitimately want it.

## Error-node policy (S1.6)

A failed scan marks the node loaded with `error = Some(..)`.
Following the reference implementation, a subsequent `Toggled`
does **not** automatically retry; refresh remains an explicit
application concern. Recorded here as the chosen option of the
spec's allowed variants.

## Alternatives considered

- **`on_toggled` spawning its own task** (executor injected into
  the tree). Rejected: couples the core to a runtime and makes
  transitions untestable without one. Side-effects-as-data is
  the upstream recommendation and keeps the Dioxus wiring free
  to choose `use_coroutine`.
- **Accepting `generation >= issued`** instead of strict
  equality. Rejected: wrapping makes ordering meaningless;
  strict equality is the documented protocol.
- **Storing `swdir::ScanError` on the node.** Rejected: not
  `Clone`/`PartialEq`; poisons every derived impl upward.

## Test plan

`tests/lazy_loading.rs` encodes S1.1–S1.6 one test per clause,
over `tempfile` fixtures, using `expand_blocking` and manual
`on_loaded` injection for the staleness/generation cases.

## Open questions

None.
