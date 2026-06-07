//! Feature 10 — Icon themes (specification clauses S10.1–S10.7).

use dioxus_swdir_tree_core::icon::{IconRole, IconSpec, IconTheme, UnicodeTheme};

// ── S10.1 — Six icon roles ────────────────────────────────────────────────────

/// S10.1 — All six roles are defined and queryable.
#[test]
fn s10_1_all_six_roles_defined() {
    let theme = UnicodeTheme;
    let roles = [
        IconRole::FolderClosed,
        IconRole::FolderOpen,
        IconRole::File,
        IconRole::Error,
        IconRole::CaretRight,
        IconRole::CaretDown,
    ];
    for role in roles {
        let spec = theme.glyph(role);
        assert!(
            !spec.glyph.is_empty(),
            "role {role:?} must produce a non-empty glyph"
        );
    }
}

// ── S10.2 — IconRole is #[non_exhaustive] ─────────────────────────────────────

/// S10.2 — A custom theme can compile with a `_ =>` fallback arm.
struct MyTheme;
impl IconTheme for MyTheme {
    fn glyph(&self, role: IconRole) -> IconSpec {
        let glyph = match role {
            IconRole::FolderClosed => "D",
            IconRole::FolderOpen => "d",
            IconRole::File => "f",
            IconRole::Error => "!",
            IconRole::CaretRight => ">",
            IconRole::CaretDown => "v",
            _ => "?", // Required: non_exhaustive (S10.2)
        };
        IconSpec {
            glyph: std::borrow::Cow::Borrowed(glyph),
            font: None,
            size: None,
        }
    }
}

#[test]
fn s10_2_custom_theme_compiles_with_fallback_arm() {
    let theme = MyTheme;
    let spec = theme.glyph(IconRole::FolderClosed);
    assert_eq!(&*spec.glyph, "D");
}

// ── S10.3 — IconSpec fields ───────────────────────────────────────────────────

/// S10.3 — IconSpec has glyph, font, and size fields.
#[test]
fn s10_3_iconspec_fields_accessible() {
    let spec = IconSpec {
        glyph: std::borrow::Cow::Borrowed("▸"),
        font: Some("custom-icons"),
        size: Some(16.0),
    };
    assert_eq!(&*spec.glyph, "▸");
    assert_eq!(spec.font, Some("custom-icons"));
    assert_eq!(spec.size, Some(16.0));
}

// ── S10.4 — Theme is rendering-only ──────────────────────────────────────────

/// S10.4 — The theme interface is pure: glyph() is called with just a role,
/// never with mutable tree state, confirming it is rendering-only.
#[test]
fn s10_4_glyph_is_pure_and_takes_no_tree_state() {
    // This is a compile-time / API-shape test:
    // IconTheme::glyph takes only `&self` and `IconRole`.
    let _spec: IconSpec = UnicodeTheme.glyph(IconRole::File);
}

// ── S10.5 — Default theme: UnicodeTheme without icons feature ────────────────

/// S10.5 — Without the `icons` feature, UnicodeTheme is the default.
/// It uses ambient font (no font registration required).
#[test]
fn s10_5_unicode_theme_uses_no_font() {
    let theme = UnicodeTheme;
    for role in [
        IconRole::FolderClosed,
        IconRole::FolderOpen,
        IconRole::File,
        IconRole::Error,
        IconRole::CaretRight,
        IconRole::CaretDown,
    ] {
        let spec = theme.glyph(role);
        assert!(
            spec.font.is_none(),
            "UnicodeTheme must use None font (ambient font, no registration)"
        );
        assert!(spec.size.is_none(), "UnicodeTheme returns None size");
    }
}

// ── S10.6 — LucideTheme with icons feature ───────────────────────────────────

#[cfg(feature = "icons")]
mod icons_feature {
    use dioxus_swdir_tree_core::icon::{IconRole, IconTheme, LUCIDE_FONT_BYTES, LucideTheme};

    /// S10.5/S10.6 — With the icons feature, LucideTheme returns glyphs
    /// with `font: Some("lucide")`.
    #[test]
    fn s10_6_lucide_theme_uses_lucide_font_family() {
        let theme = LucideTheme;
        let spec = theme.glyph(IconRole::FolderClosed);
        assert_eq!(
            spec.font,
            Some("lucide"),
            "LucideTheme must declare font-family 'lucide'"
        );
        assert!(!spec.glyph.is_empty());
    }

    /// S10.6 — LUCIDE_FONT_BYTES is exposed for app-side @font-face registration.
    #[test]
    fn s10_6_font_bytes_constant_exists() {
        // The constant exists. Apps should provide real bytes via @font-face.
        let _: &[u8] = LUCIDE_FONT_BYTES;
    }
}

// ── S10.7 — Custom theme need only implement one method ───────────────────────

/// S10.7 — Custom themes only need to implement `glyph(&self, role) -> IconSpec`.
/// This test confirms the trait compiles correctly for `MyTheme` above.
#[test]
fn s10_7_custom_theme_implements_single_method() {
    let theme: &dyn IconTheme = &MyTheme;
    let spec = theme.glyph(IconRole::CaretDown);
    assert_eq!(&*spec.glyph, "v");
}

/// S10.7 — Object-safe: usable behind Arc<dyn IconTheme>.
#[test]
fn s10_7_theme_is_object_safe() {
    use std::sync::Arc;
    let theme: Arc<dyn IconTheme> = Arc::new(UnicodeTheme);
    let spec = theme.glyph(IconRole::FolderOpen);
    assert!(!spec.glyph.is_empty());
}
