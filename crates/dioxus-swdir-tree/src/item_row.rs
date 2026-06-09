//! The per-row component rendered by [`crate::ItemTreeView`].

use dioxus::prelude::*;
use dioxus_swdir_tree_core::{
    DropPosition, ItemDragMsg, ItemTreeEvent, NodeId, VisibleItem, icon::IconRole,
    selection::SelectionMode,
};

use crate::row::ArcTheme;
use crate::style as s;

/// Props for a single visible item-tree row.
#[derive(Props, Clone, PartialEq)]
pub(crate) struct ItemTreeRowProps {
    pub row: VisibleItem,
    pub on_event: EventHandler<ItemTreeEvent>,
    pub theme: ArcTheme,
    /// Whether drag-and-drop is enabled (selects three-zone vs. plain row).
    pub dnd_enabled: bool,
    /// Current drop hover target, for highlighting the active zone.
    pub hover: Option<(NodeId, DropPosition)>,
}

/// One visible row in an [`crate::ItemTreeView`].
///
/// With DnD disabled: a single clickable body (v0.8 behaviour).
/// With DnD enabled: a Before strip, the body (Pressed + Into), and an
/// After strip — each a distinct mouse target (RFC 013 / S11).
#[component]
pub(crate) fn ItemTreeRow(props: ItemTreeRowProps) -> Element {
    let ItemTreeRowProps {
        row,
        on_event,
        theme,
        dnd_enabled,
        hover,
    } = props;

    let id: NodeId = row.id;
    let indent_px = row.depth * 16;

    // ── Body class list ───────────────────────────────────────────────────
    let mut body_classes = s::CLASS_ROW.to_string();
    if row.is_selected {
        body_classes.push(' ');
        body_classes.push_str(s::CLASS_ROW_SELECTED);
    }
    if row.is_active {
        body_classes.push_str(" dx-swdir-row--active");
    }
    if hover == Some((id, DropPosition::Into)) {
        body_classes.push_str(" dx-swdir-drop-into");
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
        Box::leak(theme.0.glyph(role).glyph.into_owned().into_boxed_str())
    };
    let icon_role = if row.is_expanded && row.has_children {
        IconRole::FolderOpen
    } else if row.has_children {
        IconRole::FolderClosed
    } else {
        IconRole::File
    };
    let icon_str: &str = Box::leak(theme.0.glyph(icon_role).glyph.into_owned().into_boxed_str());
    let label = row.label.clone();

    // ── Caret toggles expand/collapse (both modes) ────────────────────────
    let on_caret_click = move |evt: MouseEvent| {
        evt.stop_propagation();
        on_event.call(ItemTreeEvent::Toggled(id));
    };

    let body_inner = rsx! {
        span { class: s::CLASS_CARET, onclick: on_caret_click, "{caret_str}" }
        span { class: s::CLASS_ICON, "{icon_str}" }
        span { class: s::CLASS_LABEL, "{label}" }
    };

    if !dnd_enabled {
        // ── v0.8 plain row: click selects ─────────────────────────────────
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
        return rsx! {
            div {
                class: "{body_classes}",
                style: "padding-left: {indent_px}px;",
                onclick: on_row_click,
                {body_inner}
            }
        };
    }

    // ── DnD-enabled: three zones ──────────────────────────────────────────
    let strip_class = |active: bool| {
        if active {
            "dx-swdir-drop-strip dx-swdir-drop-strip--active"
        } else {
            "dx-swdir-drop-strip"
        }
    };
    let before_class = strip_class(hover == Some((id, DropPosition::Before)));
    let after_class = strip_class(hover == Some((id, DropPosition::After)));

    rsx! {
        div { class: "dx-swdir-item-row",
            // Before strip
            div {
                class: "{before_class}",
                onmouseenter: move |_| on_event.call(ItemTreeEvent::Drag(ItemDragMsg::Entered(id, DropPosition::Before))),
                onmouseleave: move |_| on_event.call(ItemTreeEvent::Drag(ItemDragMsg::Exited(id, DropPosition::Before))),
                onmouseup: move |evt: MouseEvent| {
                    evt.stop_propagation();
                    on_event.call(ItemTreeEvent::Drag(ItemDragMsg::Released(id, DropPosition::Before)));
                },
            }
            // Body — press starts drag, Into hover target
            div {
                class: "{body_classes}",
                style: "padding-left: {indent_px}px;",
                onmousedown: move |_| on_event.call(ItemTreeEvent::Drag(ItemDragMsg::Pressed(id))),
                onmouseenter: move |_| on_event.call(ItemTreeEvent::Drag(ItemDragMsg::Entered(id, DropPosition::Into))),
                onmouseleave: move |_| on_event.call(ItemTreeEvent::Drag(ItemDragMsg::Exited(id, DropPosition::Into))),
                onmouseup: move |evt: MouseEvent| {
                    evt.stop_propagation();
                    on_event.call(ItemTreeEvent::Drag(ItemDragMsg::Released(id, DropPosition::Into)));
                },
                {body_inner}
            }
            // After strip
            div {
                class: "{after_class}",
                onmouseenter: move |_| on_event.call(ItemTreeEvent::Drag(ItemDragMsg::Entered(id, DropPosition::After))),
                onmouseleave: move |_| on_event.call(ItemTreeEvent::Drag(ItemDragMsg::Exited(id, DropPosition::After))),
                onmouseup: move |evt: MouseEvent| {
                    evt.stop_propagation();
                    on_event.call(ItemTreeEvent::Drag(ItemDragMsg::Released(id, DropPosition::After)));
                },
            }
        }
    }
}
