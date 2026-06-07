//! Icon theme types for [`crate::DirectoryTree`] rendering.
//!
//! Themes are a **rendering-only** concern (S10.4): they are consulted
//! while building rows, never during state transitions. The view layer
//! holds the theme outside the reactive state.

use std::borrow::Cow;

/// The six logical icon positions in a tree row (S10.1).
///
/// `#[non_exhaustive]` — future minor releases may add variants; external
/// theme implementations must include a `_ =>` fallback arm (S10.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IconRole {
    /// A collapsed directory.
    FolderClosed,
    /// An expanded directory.
    FolderOpen,
    /// A non-directory entry.
    File,
    /// A directory whose scan failed.
    Error,
    /// Caret for a collapsed directory.
    CaretRight,
    /// Caret for an expanded directory (or loading indicator).
    CaretDown,
}

/// The rendering specification for one icon position (S10.3).
#[derive(Debug, Clone, PartialEq)]
pub struct IconSpec {
    /// Text to render in the icon span.
    pub glyph: Cow<'static, str>,
    /// CSS `font-family` value for the span, or `None` to inherit the
    /// row's ambient font.
    pub font: Option<&'static str>,
    /// Font size in CSS pixels, or `None` to use the widget default (14 px).
    pub size: Option<f32>,
}

/// Plug-in icon rendering (S10.7).
///
/// Implementations should be cheap and pure — build any glyph map at
/// construction time. The trait is object-safe; pass `Arc<dyn IconTheme>`.
pub trait IconTheme: Send + Sync {
    /// Return the rendering spec for `role`.
    fn glyph(&self, role: IconRole) -> IconSpec;
}

// ── UnicodeTheme ──────────────────────────────────────────────────────────────

/// Default theme: Unicode/emoji glyphs in the ambient system font.
/// No font registration required (S10.5, without `icons` feature).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UnicodeTheme;

impl IconTheme for UnicodeTheme {
    fn glyph(&self, role: IconRole) -> IconSpec {
        let glyph: &'static str = match role {
            IconRole::FolderClosed => "📁",
            IconRole::FolderOpen => "📂",
            IconRole::File => "📄",
            IconRole::Error => "⚠",
            IconRole::CaretRight => "▸",
            IconRole::CaretDown => "▾",
        };
        IconSpec {
            glyph: Cow::Borrowed(glyph),
            font: None,
            size: None,
        }
    }
}

// ── LucideTheme (icons feature) ───────────────────────────────────────────────

/// Lucide vector-glyph theme, gated by the `icons` feature (S10.5).
///
/// Requires the `lucide` CSS font-family to be registered with the
/// rendering engine (S10.6). Without registration, glyphs render as
/// tofu while the widget keeps functioning. Use [`LUCIDE_FONT_BYTES`]
/// with a `@font-face` rule in your application's stylesheet.
///
/// # Lucide licence
///
/// Lucide is distributed under the ISC licence; see `NOTICE` for the
/// full attribution.
#[cfg(feature = "icons")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LucideTheme;

#[cfg(feature = "icons")]
impl IconTheme for LucideTheme {
    fn glyph(&self, role: IconRole) -> IconSpec {
        // Codepoints from the @lucide/font package (lucide 0.441.0).
        // Private-Use-Area entries mapped sequentially from U+E000.
        let glyph: &'static str = match role {
            IconRole::FolderClosed => "\u{ea83}", // folder
            IconRole::FolderOpen => "\u{ea84}",   // folder-open
            IconRole::File => "\u{ea7f}",         // file
            IconRole::Error => "\u{ea78}",        // circle-alert
            IconRole::CaretRight => "\u{ea59}",   // chevron-right
            IconRole::CaretDown => "\u{ea56}",    // chevron-down
            _ => "\u{ea30}",                      // square (fallback)
        };
        IconSpec {
            glyph: Cow::Borrowed(glyph),
            font: Some("lucide"),
            size: Some(14.0),
        }
    }
}

/// Raw bytes of the Lucide icon font TTF (lucide 0.441.0, ISC licence).
///
/// Use these bytes to register the `lucide` CSS font-family in your app.
/// Without registration, [`LucideTheme`] glyphs render as tofu but the
/// widget continues to function correctly (S10.6).
///
/// # Example (`@font-face` via inline CSS)
///
/// ```html
/// <style>
///   @font-face {
///     font-family: "lucide";
///     src: url("/assets/lucide.ttf") format("truetype");
///   }
/// </style>
/// ```
/// Placeholder — substitute the real Lucide TTF bytes in your application.
/// Download the font from <https://github.com/lucide-icons/lucide/releases>
/// and register it with a `@font-face` CSS rule pointing to `font-family: "lucide"`.
#[cfg(feature = "icons")]
pub const LUCIDE_FONT_BYTES: &[u8] = &[];
