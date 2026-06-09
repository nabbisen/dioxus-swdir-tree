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
//!                               │ Toggled / Selected / Drag
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
//! use dioxus_swdir_tree_core::{DirectoryTree, SelectionMode, ThreadExecutor};
//! use dioxus_swdir_tree_core::drag::DragOutcome;
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
//!         DirectoryTreeEvent::Drag(msg) => {
//!             let outcome = tree.write().on_drag_msg(msg);
//!             if let DragOutcome::Clicked { path, is_dir } = outcome {
//!                 tree.write().on_selected(&path, is_dir, SelectionMode::Replace);
//!             }
//!             // DragOutcome::Completed { sources, destination } → app handles it
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

mod item_row;
mod item_view;
mod row;
mod view;

pub use driver::use_scan_driver;
pub use event::DirectoryTreeEvent;
pub use item_view::ItemTreeView;
pub use row::{ArcTheme, default_theme};
pub use view::DirectoryTreeView;

// Re-export everything from core so application code can depend on this
// crate alone.
pub use dioxus_swdir_tree_core::*;
