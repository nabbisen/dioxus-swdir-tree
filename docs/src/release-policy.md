# Release policy

## Versioning

Releases follow semantic versioning. Each minor version implemented one or
more RFCs. Archives ship as `dioxus-swdir-tree-v<version>.tar.gz` (note the
`v` prefix in the archive's top-level directory name).

When a release ships, its RFCs move from `rfcs/proposed/` to `rfcs/done/`
with their `Status` field updated to `Implemented (v<version>)`.

## v1.0.0 gate

**v1.0.0 is published only with explicit confirmation from the project owner.**
No automation, schedule, or parity milestone overrides this. Until then the
public API may change between minor versions without a major version bump.

## MSRV

There is no declared `rust-version` in the manifests. The effective minimum
Rust version is determined by the dependency graph. Run `cargo msrv` or check
`cargo tree` if you need a precise floor for your CI.
