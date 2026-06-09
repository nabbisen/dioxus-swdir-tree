//! Event type for [`crate::ItemTree`].

use crate::item_tree::{ItemDragMsg, NodeId};
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
///         ItemTreeEvent::Drag(msg) => { tree.on_drag_msg(msg); }
///     }
/// }
/// ```
///
/// `Drag` carries an opaque [`ItemDragMsg`]; the host routes it back through
/// [`crate::ItemTree::on_drag_msg`] and acts on the returned
/// [`crate::item_tree::ItemDragOutcome`] (RFC 013).
#[derive(Debug, Clone, PartialEq)]
pub enum ItemTreeEvent {
    /// Expand or collapse the node.
    Toggled(NodeId),
    /// Change the selection.
    Selected(NodeId, SelectionMode),
    /// A drag-and-drop gesture (only emitted when drag-and-drop is enabled).
    Drag(ItemDragMsg),
}
