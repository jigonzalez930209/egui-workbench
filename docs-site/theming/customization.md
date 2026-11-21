---
title: Customization layers
description: Theme colors, global shape, per-instance overrides, custom fonts, icons, and layouts.
---

# Customization layers

Customization is layered from global to local. Pick the lowest layer that expresses the
intent; per-instance overrides are for exceptions, not for a whole product.

| Layer | API | Scope | Survives theme change |
|---|---|---|---|
| Colors | `ThemeState::set_accent`, `Theme::with_accent`, `apply_tokens`, `ThemeState::set_custom` | All components | Yes with `set_accent`; re-apply a custom theme |
| Shape and type | `WorkbenchStyle` (density, radii, borders, text scale), `style::set` | All components and chrome text | Yes |
| Metrics | `metrics` constants | Chrome | Recompile |
| Instance | `fill`, `text_color`, `stroke`, `radius`, `color`, `Tone::Custom` | One component | Yes |
| Content | Fonts, icons, layouts | Your screens | Yes |

## Colors

A brand color on the built-in themes is one call. `ThemeState` re-derives it whenever the
theme or the desktop's light/dark preference changes:

```rust
use egui_workbench::egui::Color32;

theme_state.set_accent(ctx, Some(Color32::from_rgb(0xe0, 0x6c, 0x2b)));
theme_state.set_accent(ctx, None); // back to the theme's own accent
```

For a fully custom palette, build a `Theme` and install it:

```rust
use egui_workbench::theme::{self, Theme};

let brand = Theme::dark().with_accent(Color32::from_rgb(0xe0, 0x6c, 0x2b));
theme::apply_tokens(ctx, &brand);          // or theme_state.set_custom(ctx, Some(brand))
```

See [Custom themes](/theming/custom-themes) for the accent derivation and contrast
checks, and the template's `appearance.rs` for persistence.

## Shape

```rust
use egui_workbench::style::{self, Density, WorkbenchStyle};

style::set(ctx, WorkbenchStyle {
    density: Density::Comfortable,
    control_radius: 9,
    item_radius: 7,
    inner_radius: 5,
    container_radius: 12,
    dialog_radius: 14,
    pill_radius: u8::MAX, // full capsules
    border_width: 1.0,
    text_scale: 1.15,
});
```

Start from a preset when you change only a few fields:
`WorkbenchStyle { text_scale: 1.15, ..WorkbenchStyle::rounded() }`.

See [Style and density](/theming/style-and-density).

## Per-instance overrides

```rust
let theme = egui_workbench::theme::current(ui.ctx());
let style = egui_workbench::style::get(ui.ctx());

Button::new("Launch")
    .fill(theme.success)
    .text_color(Color32::BLACK)
    .stroke(Stroke::NONE)
    .radius(4)
    .show(ui);

IconButton::new("star", "Favorite")
    .color(theme.warning)
    .radius(3)
    .show(ui, &mut icons);

Card::new()
    .fill(theme.surface_raised)
    .stroke(style.border(theme.accent))
    .radius(10)
    .show(ui, |ui| { /* ... */ });

Badge::new("Beta").tone(Tone::Custom(theme.done)).radius(4).show(ui);

Banner::new("Deployed").color(theme.success).show(ui, |_ui| {});
```

`Tone::Custom(color)` is the escape hatch for semantic roles egui does not carry, such as
`theme.success` and `theme.done`. A custom tone derives its subtle fill at 15 % opacity.

Use overrides for one branded control, not for every button: variants and tokens keep
hover, focus, disabled, and contrast behavior consistent.

## Fonts

The style installs a type scale (`style::TYPE_SCALE`): body 14, button 12.5, small 11,
monospace 13, heading 20, each multiplied by `WorkbenchStyle::text_scale`. To make the
whole product larger or smaller, change `text_scale`; do not edit egui's text styles
directly, because every theme change re-applies the scale. In your own widgets, take
fonts from the style (`style.font(13.0)`, `style.mono(12.0)`) so they follow it.

Add your own faces with egui's font definitions, then apply them after the theme so a
theme change does not remove them:

```rust
let mut fonts = egui::FontDefinitions::default();
fonts.font_data.insert(
    "brand".to_owned(),
    std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/brand.ttf"))),
);
fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "brand".to_owned());
ctx.set_fonts(fonts);
```

Keep the default faces as fallbacks; the workbench draws arrow glyphs, `…`, `×`, and
punctuation that a brand face may not include.

## Icons

- Embed custom monochrome SVGs with `Icons::register` and draw them in white.
- Register the product mark as `logo`; the title bar draws it at 20 pt.
- Use `lucide-` names for general-purpose glyphs and `MaterialFile` / `MaterialFolder`
  for file trees.
- See [Icons](/theming/icons).

## Layouts

`WorkbenchApp` exposes the shell's composition points:

```rust
impl WorkbenchApp for App {
    type Command = Command;

    fn sidebar_visible(&self) -> bool { self.explorer }
    fn right_panel_visible(&self) -> bool { self.inspector }
    fn bottom_panel_visible(&self) -> bool { self.output }
    fn tab_strip_visible(&self) -> bool { true }
    fn status_bar_visible(&self) -> bool { true }

    fn central(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
        crate::content::show(self, ui)
    }
    // ...
}
```

The title bar and panels are closures; you can put any widgets in them. See
[Workbench layout](/app/workbench) and [IDE layout](/recipes/ide-layout).

## A brand recipe

1. Pick a base theme and a brand color.
2. Install it with `ThemeState::set_accent(ctx, Some(brand))`.
3. Check `theme_state.tokens().accessibility_issues()` (built-in bases always pass).
4. Choose a density, corner preset, and text size.
5. Store the accent, density, corners, and text size in settings.
6. Register the logo and any monochrome marks.
7. Override only the hero controls (primary button, active tab marker) if the tokens are
   not enough.

The template implements steps 1–6 in `appearance.rs`, `settings.rs`, and the Settings →
Shape tab. See [Own brand](/recipes/brand) for the full walkthrough.

## What not to customize globally

- **Do not change `metrics` at runtime.** The chrome sizes are part of the shell's
  geometry contract.
- **Do not hard-code a color** to match a mockup; add a token or use `Tone::Custom`.
- **Do not shrink controls below their text.** `text_scale` and `Density` are
  independent; when you raise the text scale above about 1.15, move to
  `Density::Comfortable` too so labels keep their vertical padding.
- **Do not hard-code radii or font sizes** in your own widgets; use the style helpers
  (`control()`, `container()`, `font()`), or a style change will skip them.
- **Do not re-implement the title bar.** It owns the drag region, double-click maximize,
  resize edges, and window controls.
