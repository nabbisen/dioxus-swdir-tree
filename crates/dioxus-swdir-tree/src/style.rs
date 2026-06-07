//! CSS class names (the public theming surface) and the optional default
//! stylesheet bundled behind the `default-style` feature.
//!
//! All class names are prefixed `dx-swdir-` to avoid collisions.

/// Container class — applied to the root `<div>` of
/// [`crate::DirectoryTreeView`].
pub const CLASS_TREE: &str = "dx-swdir-tree";

/// Row class — applied to every visible row div.
pub const CLASS_ROW: &str = "dx-swdir-row";

/// Modifier: applied to rows whose path is in the selection set.
pub const CLASS_ROW_SELECTED: &str = "dx-swdir-row--selected";

/// Modifier: applied to the most recently activated row (`active_path`).
pub const CLASS_ROW_ACTIVE: &str = "dx-swdir-row--active";

/// Modifier: applied to rows that have a scan error.
pub const CLASS_ROW_ERROR: &str = "dx-swdir-row--error";

/// Modifier: applied to the current drag-and-drop drop target
/// (wired by RFC 008).
pub const CLASS_ROW_DROP_TARGET: &str = "dx-swdir-row--drop-target";

/// Caret span class (expand / collapse triangle).
pub const CLASS_CARET: &str = "dx-swdir-caret";

/// Icon span class (file / folder / error glyph).
pub const CLASS_ICON: &str = "dx-swdir-icon";

/// Label span class (basename text).
pub const CLASS_LABEL: &str = "dx-swdir-label";

/// Baseline stylesheet shipped behind the `default-style` feature.
///
/// Provides layout and minimal visual cues; no colours beyond transparent
/// selection highlight. Applications can disable the feature and write
/// their own rules for the `dx-swdir-*` classes.
#[cfg(feature = "default-style")]
pub const DEFAULT_CSS: &str = r#"
.dx-swdir-tree {
    overflow-y: auto;
    height: 100%;
    font-family: system-ui, sans-serif;
    font-size: 0.875rem;
    line-height: 1.5;
    user-select: none;
}
.dx-swdir-row {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.125rem 0.25rem;
    border-radius: 3px;
    cursor: default;
    white-space: nowrap;
}
.dx-swdir-row:hover {
    background: rgba(0, 0, 0, 0.06);
}
.dx-swdir-row--selected {
    background: rgba(66, 133, 244, 0.18);
}
.dx-swdir-row--active {
    outline: 1px solid rgba(66, 133, 244, 0.5);
    outline-offset: -1px;
}
.dx-swdir-row--error .dx-swdir-label {
    opacity: 0.55;
    font-style: italic;
}
.dx-swdir-caret {
    display: inline-block;
    width: 1em;
    text-align: center;
    font-size: 0.7em;
    flex-shrink: 0;
    cursor: pointer;
}
.dx-swdir-icon {
    flex-shrink: 0;
}
.dx-swdir-label {
    overflow: hidden;
    text-overflow: ellipsis;
}
"#;
