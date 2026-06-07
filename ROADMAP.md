# Roadmap

Each minor release implements the listed RFCs; an RFC moves from
`rfcs/proposed/` to `rfcs/done/` only when its release ships. Feature numbers
refer to the upstream `iced-swdir-tree` feature specification, which serves as
the cross-framework test oracle.

| Version | RFCs | Features | Status |
|---|---|---|---|
| **0.1.0** | 001 architecture, 002 core state machine, 003 display filters | F1 lazy loading, F2 display filters | **Shipped** |
| **0.2.0** | 004 selection model | F3 single selection, F6 multi-selection | **Shipped** |
| 0.3.0 | 005 async scanning, 006 Dioxus view component | F5 pluggable `ScanExecutor`; `DirectoryTreeView` (first visual release) | Proposed |
| 0.4.0 | 007 keyboard navigation | F4 | Proposed |
| 0.5.0 | 008 drag & drop | F7 | Proposed |
| 0.6.0 | 009 prefetch | F8 | Proposed |
| 0.7.0 | 010 incremental search, 011 icon themes | F9, F10 — feature parity with `iced-swdir-tree` 0.7 | Proposed |
| 1.0.0 | API freeze review | — | **Gated: published only with explicit confirmation from the project owner. Never auto-released.** |

Out of scope before 1.0: web/WASM scan executors, file operations (move,
rename, delete), filesystem watching, and recursive search indexing.
