# Driving the tree from your own event loop

The core crate is framework-free. The contract:

1. Forward a click on a row to `tree.on_toggled(path)`.
2. If it returns `Some(request)`, run `scan::run(&request)` on a worker.
3. Feed the payload back: `tree.on_loaded(payload)`.
4. Re-render from `tree.visible_rows()`.

From v0.3.0, RFC 005 wraps steps 2–3 in a pluggable `ScanExecutor` and a
`use_scan_driver` Dioxus hook so applications never write this loop by hand.
