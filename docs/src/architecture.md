# Architecture & the RFC process

The workspace holds two crates: `dioxus-swdir-tree-core` (state machine,
depends only on `swdir`) and `dioxus-swdir-tree` (flagship; gains the Dioxus
view in v0.3.0). Every feature is specified by an RFC before implementation —
see `rfcs/README.md` for the index and RFC 000 for the lifecycle policy
(proposed → done/archive, folders are the source of truth).
