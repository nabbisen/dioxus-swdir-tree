//! Event type for [`crate::ItemTree`].

use crate::item_tree::NodeId;
use crate::selection::SelectionMode;

/// An event emitted by [`crate::ItemTree::handle_key`] or by the view
/// component's mouse handlers.
///
/// The host dispatches each event back through [`crate::ItemTree::on_toggled`]
/// or [`crate::ItemTree::on_selected`]:
///
/// ```no_run
/// # use dioxus_swdir_tree_core::item_event::ItemTreeEvent;
/// # use dioxus_swdir_tree_core::item_tree::{ItemTree, NodeId};
/// # use dioxus_swdir_tree_core::selection::SelectionMode;
/// fn handle(tree: &mut ItemTree<String>, ev: ItemTreeEvent) {
///     match ev {
///         ItemTreeEvent::Toggled(id) => tree.on_toggled(id),
///         ItemTreeEvent::Selected(id, mode) => tree.on_selected(id, mode),
///     }
/// }
/// ```
///
/// Drag-and-drop is deliberately absent (deferred to a later RFC).
#[derive(Debug, Clone, PartialEq)]
pub enum ItemTreeEvent {
    /// Expand or collapse the node.
    Toggled(NodeId),
    /// Change the selection.
    Selected(NodeId, SelectionMode),
}
