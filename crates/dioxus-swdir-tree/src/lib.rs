//! # dioxus-swdir-tree
//!
//! A lazy-loading directory-tree explorer widget for
//! [Dioxus](https://dioxuslabs.com) GUI apps, built on
//! [`swdir`](https://crates.io/crates/swdir).
//!
//! ## Architecture
//!
//! ```text
//!                ┌─────────────────────────────┐
//!   Signal       │    DirectoryTreeView         │  renders
//!  ─────────────►│    (this crate)              │──────────► HTML rows
//!                │                              │
//!  on_event ◄────│    EventHandler              │  user gestures
//!                └──────────────┬──────────────┘
//!                               │ Toggled / Selected
//!                               ▼
//!                ┌─────────────────────────────┐
//!   Signal.write │  DirectoryTree               │  pure state machine
//!  ─────────────►│  (dioxus-swdir-tree-core)    │
//!                └──────────────┬──────────────┘
//!                               │ ScanRequest (data)
//!                               ▼
//!                ┌─────────────────────────────┐
//!                │   use_scan_driver            │  coroutine
//!                │   ScanExecutor / Thread      │  executes scan
//!                └─────────────────────────────┘
//! ```
//!
//! ## Quick start
//!
//! ```no_run
//! # use dioxus::prelude::*;
//! use dioxus_swdir_tree::{DirectoryTreeView, DirectoryTreeEvent, use_scan_driver};
//! use dioxus_swdir_tree_core::{DirectoryTree, ThreadExecutor};
//! use std::sync::Arc;
//!
//! fn app() -> Element {
//!     let mut tree = use_signal(|| DirectoryTree::new("/home"));
//!     let scans = use_scan_driver(tree, Arc::new(ThreadExecutor));
//!
//!     let on_event = move |ev: DirectoryTreeEvent| match ev {
//!         DirectoryTreeEvent::Toggled(path) => {
//!             if let Some(req) = tree.write().on_toggled(&path) {
//!                 scans.send(req);
//!             }
//!         }
//!         DirectoryTreeEvent::Selected { path, is_dir, mode } => {
//!             tree.write().on_selected(&path, is_dir, mode);
//!         }
//!     };
//!
//!     rsx! { DirectoryTreeView { tree, on_event } }
//! }
//! ```
//!
//! The `default-style` feature (on by default) injects a minimal
//! `dx-swdir-*` stylesheet; disable it to theme from scratch.

pub mod driver;
pub mod event;
pub mod style;

mod row;
mod view;

// Re-export the component and hook at the crate root for ergonomics.
pub use driver::use_scan_driver;
pub use event::DirectoryTreeEvent;
pub use view::DirectoryTreeView;

// Re-export everything from core so application code can depend on this
// crate alone.
pub use dioxus_swdir_tree_core::*;
