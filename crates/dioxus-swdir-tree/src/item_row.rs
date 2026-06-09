//! The per-row component rendered by [`crate::ItemTreeView`].

use dioxus::prelude::*;
use dioxus_swdir_tree_core::{
    ItemTreeEvent, NodeId, VisibleItem, icon::IconRole, selection::SelectionMode,
};

use crate::row::ArcTheme;
use crate::style as s;

/// Props for a single visible item-tree row.
#[derive(Props, Clone, PartialEq)]
pub(crate) struct ItemTreeRowProps {
    pub row: VisibleItem,
    pub on_event: EventHandler<ItemTreeEvent>,
    pub theme: ArcTheme,
}

/// One visible row in an [`crate::ItemTreeView`].
///
/// Click fires `Selected(Replace)`. Ctrl-click fires `Toggle`. Shift-click
/// fires `ExtendRange`. Caret click fires `Toggled`.
#[component]
pub(crate) fn ItemTreeRow(props: ItemTreeRowProps) -> Element {
    let ItemTreeRowProps {
        row,
        on_event,
        theme,
    } = props;

    let id: NodeId = row.id;
    let indent_px = row.depth * 16;

    // ── CSS class list ────────────────────────────────────────────────────
    let mut classes = s::CLASS_ROW.to_string();
    if row.is_selected {
        classes.push(' ');
        classes.push_str(s::CLASS_ROW_SELECTED);
    }
    if row.is_active {
        classes.push_str(" dx-swdir-row--active");
    }

    // ── Glyphs ────────────────────────────────────────────────────────────
    let caret_str: &str = if !row.has_children {
        " "
    } else {
        let role = if row.is_expanded {
            IconRole::CaretDown
        } else {
            IconRole::CaretRight
        };
        let spec = theme.0.glyph(role);
        Box::leak(spec.glyph.into_owned().into_boxed_str())
    };

    let icon_role = if row.is_expanded && row.has_children {
        IconRole::FolderOpen
    } else if row.has_children {
        IconRole::FolderClosed
    } else {
        IconRole::File
    };
    let icon_spec = theme.0.glyph(icon_role);
    let icon_str: &str = Box::leak(icon_spec.glyph.into_owned().into_boxed_str());

    let label = row.label.clone();

    // ── Event handlers ────────────────────────────────────────────────────
    let on_caret_click = move |evt: MouseEvent| {
        evt.stop_propagation();
        on_event.call(ItemTreeEvent::Toggled(id));
    };

    let on_row_click = move |evt: MouseEvent| {
        let mode = if evt.modifiers().shift() {
            SelectionMode::ExtendRange
        } else if evt.modifiers().ctrl() {
            SelectionMode::Toggle
        } else {
            SelectionMode::Replace
        };
        on_event.call(ItemTreeEvent::Selected(id, mode));
    };

    rsx! {
        div {
            class: "{classes}",
            style: "padding-left: {indent_px}px;",
            onclick: on_row_click,

            span {
                class: s::CLASS_CARET,
                onclick: on_caret_click,
                "{caret_str}"
            }
            span {
                class: s::CLASS_ICON,
                "{icon_str}"
            }
            span {
                class: s::CLASS_LABEL,
                "{label}"
            }
        }
    }
}
