# Driving the tree from your own event loop

The core crate is framework-free. Every transition is synchronous and returns
side effects as data. Here is the full event contract:

## Scan lifecycle

1. A row click or keyboard event produces a `DirectoryTreeEvent::Toggled(path)`.
2. The host calls `tree.write().on_toggled(&path)`.
3. If it returns `Some(request)`, execute `scan::run(&request)` on a worker thread.
4. Feed the result back: `tree.write().on_loaded(payload)`.
5. If `outcome.prefetch_requests` is non-empty, dispatch each as an additional
   background scan (the `use_scan_driver` hook handles this with `spawn`).
6. Re-render from `tree.read().visible_rows()`.

## Selection

Call `tree.write().on_selected(&path, is_dir, mode)` in response to
`DirectoryTreeEvent::Selected { path, is_dir, mode }`.

## Drag and drop

Call `tree.write().on_drag_msg(msg)` in response to
`DirectoryTreeEvent::Drag(msg)` and dispatch on the returned `DragOutcome`.

## Keyboard

`DirectoryTreeView` wires `onkeydown` internally and emits the resulting
`Toggled` / `Selected` / `Drag` events through the `on_event` handler —
no extra wiring needed.

## Search

Search is application-driven: call `tree.write().set_search_query(q)` in
response to text-input events in your own search bar. Clear with
`tree.write().clear_search()` or `set_search_query("")`.
`visible_rows()` automatically returns the filtered set when a query is active.

## Using use_scan_driver

The `use_scan_driver(tree, executor)` Dioxus hook wraps steps 1–5 into a
`Coroutine<ScanRequest>`. Call `.send(req)` from your event handler whenever
`on_toggled` returns `Some(req)`:

```rust
let scans = use_scan_driver(tree, Arc::new(ThreadExecutor));

// in your on_event handler:
if let Some(req) = tree.write().on_toggled(&path) {
    scans.send(req);
}
```
