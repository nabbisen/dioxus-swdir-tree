# RFC 001 — Project architecture and workspace layout

**Status.** Implemented (v0.1.0)
**Tracks.** Crate composition, dependency policy, and repository
layout for the `dioxus-swdir-tree` project.
**Touches.** Workspace `Cargo.toml`, `crates/`, `docs/`, `rfcs/`,
release packaging.

## Summary

`dioxus-swdir-tree` is a directory-tree widget library for Dioxus
GUI applications, a port of `iced-swdir-tree` v0.7.x built against
its framework-independent design documents. This RFC fixes the
crate composition before any code is written.

The project is a Cargo workspace with two library crates:

```
dioxus-swdir-tree/                  ← repository root, workspace
├── crates/
│   ├── dioxus-swdir-tree-core/    ← framework-free state machine
│   └── dioxus-swdir-tree/         ← Dioxus view layer (the flagship crate)
├── rfcs/                          ← governed by RFC 000
├── docs/src/                      ← mdbook-compatible full documentation
├── README.md  CHANGELOG.md  ROADMAP.md  LICENSE  NOTICE
└── Cargo.toml                     ← [workspace]
```

## Design

### Two crates, one hard boundary

`porting-to-dioxus.md` (the upstream design document) recommends
extracting the state machine into a core crate so that the view
framework is not a transitive dependency of state-machine
consumers, and so the framework-agnostic test suite verifies the
state machine once. We adopt this split:

- **`dioxus-swdir-tree-core`** holds `DirectoryTree`, `TreeNode`,
  `TreeConfig`, `TreeCache`, events, transitions, selection,
  search, drag state, the icon-theme trait, and the scan
  primitives over `swdir`. It depends on `swdir` only. It MUST
  NOT depend on `dioxus`, on any renderer, or on any async
  runtime. State transitions are synchronous functions that
  return side effects **as data** (`ScanRequest` values); they
  never spawn tasks.
- **`dioxus-swdir-tree`** holds the `DirectoryTreeView` component,
  row rendering, event wiring, the coroutine that executes
  `ScanRequest`s, and CSS-facing icon specs. It depends on
  `dioxus-swdir-tree-core` and `dioxus`.

The boundary is the same one the iced reference implementation
uses internally; here it is a crate boundary so the compiler
enforces it.

### Crate naming

The upstream document sketches a shared `swdir-tree-core` crate
serving both the iced and Dioxus widgets. No such crate is
published on crates.io at the time of writing; `iced-swdir-tree`
0.7.2 ships as a single crate. Claiming the framework-neutral
name from this project would be presumptuous, so the core crate
is named **`dioxus-swdir-tree-core`** — clearly owned by this
project. If upstream later publishes a shared core, migrating to
it is a contained change (the core's public surface is written
against the same design documents) and will be handled by a
follow-up RFC.

### Dependency policy

- `swdir` `^0.11` — the only runtime dependency of the core
  crate. Scans go through `scan_dir_with_options` with
  `SortOrder::NameAscDirsFirst`, the layout tree widgets expect.
- `dioxus` `^0.7` — view crate only. Introduced by RFC 006, not
  before.
- Dev-dependencies: `tempfile` for filesystem fixtures in tests.
- Anything else requires an RFC amendment. The widget is
  infrastructure; a small dependency tree is a feature.

### Toolchain and conventions

- Rust **2024 edition**, MSRV **1.85** (the edition floor;
  comfortably above dioxus 0.7's 1.83 requirement).
- Rust 2018+ module style: `foo.rs` plus a `foo/` subdirectory
  for submodules; no `mod.rs`.
- File splitting at 300 ELOC (consider) / 500 ELOC (strongly
  recommended), applied to test files too.
- Unit tests inside a module live in a sibling `tests.rs`;
  specification-level tests live as integration tests under the
  crate's `tests/` directory, one file per feature, named after
  the feature they oracle.
- License Apache-2.0, author nabbisen, all documentation and
  comments in English.

### Releases

Phased releases per `ROADMAP.md`, packaged as archives of the
Cargo workspace named `dioxus-swdir-tree-<version>.tar.gz`.
Version numbers follow the roadmap; **v1.0.0 is gated on
explicit confirmation from the project owner** and is never cut
unilaterally.

## Alternatives considered

- **Single crate with a `core` module.** Simpler, but the
  compiler would not enforce the framework-free boundary, and
  the state machine could not be consumed without pulling
  `dioxus`. Rejected.
- **Depending on `iced-swdir-tree` for state types.** Would drag
  `iced` into every Dioxus app's dependency graph — exactly what
  the upstream porting document warns against. Rejected.
- **Three crates (core / view / icons).** Icon themes are a
  trait plus two small implementations; a feature flag
  (`icons`) on the existing crates is enough. Rejected as
  premature.

## Test plan

This RFC is structural; it is verified by the workspace building
(`cargo build --workspace`) and by the dependency rules holding
(`cargo tree -p dioxus-swdir-tree-core` shows no `dioxus`).

## Open questions

None.
