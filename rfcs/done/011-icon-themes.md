# RFC 011 — Icon themes

**Status.** Implemented (v0.7.0)
**Tracks.** Feature 10 (`feature-specs.md` §10); icon-theme
section of `data-model.md`; the `iced::Font` → CSS decoupling
mandated by `porting-to-dioxus.md`.
**Touches.** `crates/dioxus-swdir-tree-core/src/` — new
`icon.rs` (`IconRole`, `IconSpec`, `IconTheme`, `UnicodeTheme`);
optional `icons` cargo feature adding `LucideTheme` +
`LUCIDE_FONT_BYTES`; glyph rendering in
`crates/dioxus-swdir-tree/src/row.rs`; tests
`tests/icon_theme.rs`.

## Summary

An `IconTheme` trait controls the glyph, font, and size rendered
for each logical icon position. Themes are a **rendering-only**
concern (S10.4): they are consulted while building rows, never
during state transitions, so the view layer may hold the theme
outside the reactive state.

```rust
#[non_exhaustive]
pub enum IconRole { FolderClosed, FolderOpen, File, Error, CaretRight, CaretDown }

pub struct IconSpec {
    pub glyph: Cow<'static, str>,
    pub font:  Option<&'static str>,  // CSS font-family — NOT iced::Font
    pub size:  Option<f32>,           // None → widget default (14.0)
}

pub trait IconTheme: Send + Sync {
    fn glyph(&self, role: IconRole) -> IconSpec;   // cheap and pure
}
```

`IconRole` is `#[non_exhaustive]` (S10.2): minor releases may
add roles; external themes must carry a `_ =>` fallback arm.

## Design

- **`UnicodeTheme`** (always available, default without the
  `icons` feature): Unicode glyphs in the ambient font; zero
  registration. This is the formalization of RFC 006's interim
  glyph set.
- **`LucideTheme`** (behind the `icons` feature, default when
  enabled): lucide codepoints with `font:
  Some("lucide")`. The TTF bytes ship as `LUCIDE_FONT_BYTES`;
  the **application** registers the font with its Dioxus asset
  pipeline / `@font-face` — without registration the glyphs
  render as tofu while the widget keeps functioning (S10.6).
  Lucide's ISC license is recorded in `NOTICE`.
- The view crate renders `IconSpec` as a span with
  `font-family` / `font-size` styles, falling back to the
  row's inherited font when `font` is `None`.
- Theme injection: `with_icon_theme(Arc<dyn IconTheme>)` on the
  view component (not on `DirectoryTree`), reflecting the
  rendering-only contract. The default is chosen by feature
  flag at the view layer.

This deviates from the upstream data model in one deliberate
way: upstream stores `icon_theme` on the tree struct; the
Dioxus port moves it to the component, which the porting guide
explicitly sanctions ("a port can store it separately from the
reactive state"). Recording the deviation here so it is a
decision, not drift.

## Alternatives considered

- **Keeping the theme on `DirectoryTree`.** Rejected: forces
  `Arc<dyn>` into a struct that otherwise derives cleanly, and
  couples re-renders to a pure-render concern.
- **SVG icons instead of font glyphs.** Tempting on a web
  renderer, but breaks `IconSpec`'s upstream shape and the
  theme-portability story; an `IconSpec::Svg` variant can be a
  post-1.0 extension of the non-exhaustive design.
- **Generic `F: IconFont` bound.** Rejected: a CSS font-family
  string is the natural currency on every Dioxus renderer.

## Test plan

`tests/icon_theme.rs` encodes S10.1–S10.7: role coverage of both
built-in themes, default-size fallback, feature-gated default
selection, and a custom single-method theme compiling against
the non-exhaustive enum.

## Open questions

- Which lucide release to vendor the TTF from — pin at
  implementation time and record in `NOTICE`.
