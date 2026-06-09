# Architecture & the RFC process

## Workspace structure

```
dioxus-swdir-tree/
  crates/
    dioxus-swdir-tree-core/   ← framework-free state machine (swdir only)
    dioxus-swdir-tree/        ← Dioxus component + scan driver (core + dioxus 0.7)
  examples/
    explorer/                 ← standalone desktop app (not a workspace member)
  rfcs/
    done/                     ← 12 implemented RFCs
  docs/src/                   ← mdBook source
```

`dioxus-swdir-tree-core` intentionally carries no UI dependency: every
transition is a synchronous method on `DirectoryTree` that returns data (a
`ScanRequest`, a `DragOutcome`, etc.) rather than performing side effects.
This keeps the state machine testable without a runtime and reusable from
frontends other than Dioxus.

## RFC process

Every feature is specified by an RFC before implementation. The lifecycle
(Draft → Proposed → Implemented/Withdrawn/Superseded) is defined in
[RFC 000](../../../rfcs/done/000-rfc-lifecycle-policy.md). Folder location is
the source of truth for an RFC's state; see
[rfcs/README.md](../../../rfcs/README.md) for the full index.

All 12 feature RFCs (001–011 plus the lifecycle policy RFC 000) are now in
`rfcs/done/`. The next RFC will be the v1.0.0 API freeze review.
