//! # dioxus-swdir-tree
//!
//! A directory-tree explorer widget for [Dioxus](https://dioxuslabs.com)
//! GUI apps, built on [`swdir`](https://crates.io/crates/swdir).
//!
//! ## Status: v0.1 — core only
//!
//! This release ships the **framework-free state machine** (re-exported
//! from [`dioxus_swdir_tree_core`]): lazy one-level-per-click loading,
//! display filters, the generation-tagged stale-scan protocol, and the
//! `visible_rows()` flat row model that every later feature builds on.
//!
//! The `DirectoryTreeView` Dioxus component lands in **v0.3.0**
//! (RFC 006), together with pluggable async scanning (RFC 005). Until
//! then this crate adds no Dioxus dependency — you can already drive the
//! tree from any event loop:
//!
//! ```no_run
//! use dioxus_swdir_tree::{DirectoryTree, scan};
//!
//! let mut tree = DirectoryTree::new("/home/me/projects");
//!
//! // A click on a collapsed, unloaded directory…
//! if let Some(request) = tree.on_toggled(std::path::Path::new("/home/me/projects")) {
//!     // …produces a scan request. Run it off the UI thread, then merge:
//!     let payload = scan::run(&request); // blocking — worker thread!
//!     let outcome = tree.on_loaded(payload);
//!     assert!(outcome.accepted);
//! }
//!
//! for (node, depth) in tree.visible_rows() {
//!     println!("{}{}", "  ".repeat(depth as usize), node.file_name().display());
//! }
//! ```
//!
//! See the repository `ROADMAP.md` for the feature schedule and the
//! `rfcs/` directory for the full design record.

pub use dioxus_swdir_tree_core::*;
