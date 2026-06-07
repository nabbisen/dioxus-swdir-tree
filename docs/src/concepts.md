# Concepts: lazy loading & generations

**Lazy loading.** The tree never scans recursively. Expanding a directory
lists exactly one level via `swdir::scan_dir`. Children stay in memory across
collapse (`is_loaded` never reverts), so re-expanding is free.

**Side effects as data.** `on_toggled` returns an `Option<ScanRequest>`
instead of spawning anything. You execute `scan::run(&request)` off the UI
thread and merge the resulting `LoadPayload` with `on_loaded`.

**Generations.** The tree carries a wrapping `u32` counter, bumped *before*
each scan is issued. A payload is merged only if its generation equals the
tree's current one — anything else is silently discarded, leaving the tree
bit-identical. This is what makes rapid expand/collapse/expand safe.

**Error nodes.** A failed scan marks the node loaded-with-error; toggling it
afterwards collapses/expands without retrying. Refresh semantics arrive with
later RFCs.
