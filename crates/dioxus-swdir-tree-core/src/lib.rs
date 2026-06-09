//! Framework-free state machine for a lazy-loading directory-tree widget.
//!
//! This crate powers the `dioxus-swdir-tree` widget crate but depends on no UI framework
//! and no async runtime. It models a navigable tree of files and
//! directories over the [`swdir`] crate's single-level `scan_dir`
//! primitive, following the design documents of `iced-swdir-tree` v0.7.
//!
//! # Architecture
//!
//! State transitions are synchronous methods on [`DirectoryTree`]. They
//! never block on I/O and never spawn tasks; instead, transitions that
//! require disk access return a [`ScanRequest`] **as data**. The caller —
//! a Dioxus coroutine, a thread pool, or a test — executes the request
//! (see [`scan::run`]) off the UI thread and feeds the resulting
//! [`LoadPayload`] back through [`DirectoryTree::on_loaded`].
//!
//! Every scan is tagged with a wrapping `u32` generation. A result is
//! merged if and only if its generation strictly equals the tree's
//! current counter; anything else is stale and silently discarded. This
//! makes the widget safe against out-of-order delivery.
//!
//! # Quick start (blocking helper)
//!
//! ```no_run
//! use dioxus_swdir_tree_core::{DirectoryTree, DisplayFilter, SelectionMode};
//! use dioxus_swdir_tree_core::keyboard::{handle_key, TreeKey, Modifiers};
//!
//! let mut tree = DirectoryTree::new("/some/project")
//!     .with_filter(DisplayFilter::FilesAndFolders);
//!
//! // Synchronous convenience path (tests, scripts, examples):
//! tree.expand_blocking(&tree.config().root_path.clone());
//!
//! // Select the root:
//! tree.on_selected(&tree.config().root_path.clone(), true, SelectionMode::Replace);
//!
//! // Navigate down one row with the keyboard:
//! if let Some(ev) = handle_key(&tree, TreeKey::Down, Modifiers::default()) {
//!     println!("keyboard event: {ev:?}");
//! }
//! ```

pub mod cache;
pub mod config;
pub mod drag;
pub mod entry;
pub mod error;
pub mod event;
pub mod executor;
pub mod icon;
pub mod item_event;
pub mod item_tree;
pub mod keyboard;
pub mod node;
pub mod scan;
pub mod search;
pub mod selection;
pub mod tree;

pub use cache::{CachedScan, TreeCache};
pub use config::{DEFAULT_PREFETCH_SKIP, DisplayFilter, TreeConfig};
pub use drag::{DragMsg, DragOutcome, DragState};
pub use entry::LoadedEntry;
pub use error::ScanIssue;
pub use event::DirectoryTreeEvent;
pub use executor::{ScanExecutor, ScanFuture, ScanJob, ThreadExecutor};
pub use icon::{IconRole, IconSpec, IconTheme, UnicodeTheme};
#[cfg(feature = "icons")]
pub use icon::{LUCIDE_FONT_BYTES, LucideTheme};
pub use item_event::ItemTreeEvent;
pub use item_tree::ItemSearchState;
pub use item_tree::{ItemNode, ItemTree, NodeId, VisibleItem};
pub use keyboard::{Modifiers, TreeKey, handle_key};
pub use node::TreeNode;
pub use scan::{LoadPayload, LoadedOutcome, ScanRequest};
pub use search::SearchState;
pub use selection::SelectionMode;
pub use tree::DirectoryTree;

#[cfg(test)]
mod tests;
