# RFC 005 — Async scanning: executor seam and coroutine wiring

**Status.** Implemented (v0.3.0)
**Tracks.** Feature 5 (pluggable scan executor) from
`feature-specs.md` §5; the async-scanning section of
`porting-to-dioxus.md`; core principle 1 (non-blocking I/O).
**Touches.** `crates/dioxus-swdir-tree-core/src/` — new
`executor.rs` (`ScanExecutor`, `ThreadExecutor`);
`crates/dioxus-swdir-tree/src/` — new `driver.rs` (the
`use_coroutine` loop); tests `tests/executor.rs`.

## Summary

The UI thread never blocks on disk I/O. RFC 002 already shaped
transitions as "mutate synchronously, return `ScanRequest` as
data"; this RFC defines **who runs the request** and **how the
result re-enters the reactive cycle** in Dioxus.

Two layers:

1. **Core:** an object-safe `ScanExecutor` trait — the seam
   between "how to run a blocking scan" and "what the widget
   does with the result" — with a default `ThreadExecutor`
   (one `std::thread::spawn` per scan).
2. **View crate:** a scan driver built on `use_coroutine`. Event
   handlers send `ScanRequest`s into the channel; the coroutine
   executes them via the configured executor and merges each
   `LoadPayload` back through `tree.write().on_loaded(..)` —
   always inside the reactive update cycle, never concurrently
   with other state mutations.

## Design

### Core trait

```rust
pub trait ScanExecutor: Send + Sync {
    fn spawn_blocking(&self, job: ScanJob) -> ScanFuture;
}
pub type ScanJob    = Box<dyn FnOnce() -> LoadPayload + Send>;
pub type ScanFuture = Pin<Box<dyn Future<Output = LoadPayload> + Send>>;
```

- Object-safe; usable behind `Arc<dyn ScanExecutor>` (S5.1).
- The widget issues exactly one `spawn_blocking` per scan (S5.2).
- `ThreadExecutor` (default, S5.3): spawns an OS thread, hands
  the result back over a oneshot channel the future awaits.
  Correct everywhere; one thread per expansion. High-throughput
  apps (heavy prefetch) plug in a pooled executor
  (tokio `spawn_blocking`, rayon, smol) by implementing the
  trait.

The executor lives in the **driver**, not in `DirectoryTree`
state: transitions stay pure, and the same tree state is usable
in tests with no executor at all (`expand_blocking`).

### Dioxus driver

```rust
let tree = use_signal(|| DirectoryTree::new(root));
let scans = use_scan_driver(tree, executor);   // wraps use_coroutine
// in handlers:
if let Some(req) = tree.write().on_toggled(path) { scans.send(req); }
```

The coroutine loop: receive request → `executor.spawn_blocking`
→ await payload → `tree.write().on_loaded(payload)` → forward
any `prefetch_requests` (empty until RFC 009) back into its own
channel. Single channel ⟹ merges are serialized; the generation
protocol (RFC 002) handles arbitrary completion order.

### Web target note

`ThreadExecutor` requires OS threads, so the default targets
desktop (`dioxus-desktop`). On wasm there is no local
filesystem to scan in the first place; web support, if it ever
makes sense (virtual filesystems), needs its own RFC and is
explicitly out of scope pre-1.0.

## Alternatives considered

- **`spawn` per request instead of one coroutine.** Workable,
  but loses serialized merging and makes prefetch fan-out
  (RFC 009) harder to reason about. The upstream porting guide
  recommends the coroutine channel; adopted.
- **Async trait (`async fn` in trait).** Rejected for now:
  object safety with `async fn` still needs boxing anyway; the
  explicit `ScanFuture` alias keeps MSRV head-room and mirrors
  the upstream signature.
- **Executor stored in `DirectoryTree`.** Rejected: drags
  `Send + Sync + 'static` bounds and runtime concerns into the
  pure state machine.

## Test plan

`tests/executor.rs`: `ThreadExecutor` resolves with a correct
payload off-thread; a custom recording executor proves exactly
one `spawn_blocking` per request; an out-of-order delivery test
proves stale discard end-to-end. Driver-level behaviour is
covered in the view crate with a headless `VirtualDom` once
RFC 006 lands.

## Open questions

- Whether to ship a `tokio` feature with a pooled executor in
  v0.3.0 or defer to demand. Leaning: defer.
