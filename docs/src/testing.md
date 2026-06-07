# Testing against the feature-spec oracle

The upstream `iced-swdir-tree` feature specification doubles as a
cross-framework test oracle: integration tests are named after its clauses
(`s1_2_…`, `s2_7_…`) so coverage gaps are visible by inspection.
`DirectoryTree::expand_blocking` exists so the suite can exercise scan→merge
cycles synchronously. Stale-payload tests assert `PartialEq` equality of the
whole tree, not just observable rows.

Run everything with `cargo test --workspace`.
