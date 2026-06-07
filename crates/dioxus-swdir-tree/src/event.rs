//! Re-export [`DirectoryTreeEvent`] from `dioxus-swdir-tree-core`.
//!
//! The type now lives in core so that [`dioxus_swdir_tree_core::keyboard::handle_key`]
//! can produce it without a circular dependency.

pub use dioxus_swdir_tree_core::DirectoryTreeEvent;
