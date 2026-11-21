---
title: Own brand
description: Persist a brand accent, density, corner preset, and text size; the accent follows theme changes on its own.
---

# Own brand

<LiveDemo
  scene="accent"
  height="600"
  alt="The component board with a black and orange brand palette"
  caption="Pick a swatch or any color: ThemeState::set_accent derives focus, links, selection, and accent text."
/>

The template's `appearance.rs` is a complete implementation of this recipe.

## Preferences

```rust
use egui_workbench::egui::Color32;
use egui_workbench::style::Density;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Appearance {
    pub density: Density,
    pub corners: Corners,
    pub text_size: TextSize, // Small 0.9 | Default 1.0 | Large 1.15
    pub accent: Option<Color32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Corners {
    #[default]
    Default,
    Sharp,
    Rounded,
}
```

Store the pieces as strings:

```rust
#[derive(Serialize, Deserialize)]
struct Settings {
    theme: String,
    follow_system: bool,
    reduced_motion: bool,
    density: String,
    corners: String,
    text_size: String,      // small | default | large
    accent: Option<String>, // "#rrggbb"
}
```

## Apply

```rust
impl Appearance {
    pub fn style(&self) -> WorkbenchStyle {
        WorkbenchStyle {
            text_scale: self.text_size.scale(),
            ..self.corners.style().with_density(self.density)
        }
    }

    pub fn apply(&self, ctx: &egui::Context, theme_state: &mut ThemeState) {
        style::set(ctx, self.style());
        theme_state.set_accent(ctx, self.accent);
    }
}
```

1. `style::set` installs shape and text scale; theme changes preserve it.
2. `set_accent` stores the brand color in the theme state, which derives it on the
   effective built-in theme now and after every later change.

Call `apply` at startup and when the user edits the appearance. Theme commands only call
`theme_state.set_requested(ctx, id)`.

## System light/dark

Nothing to do. `ThemeState::sync_system` (already called from `App::logic`) re-applies
the effective theme with the accent when the desktop switches between light and dark
while following the system.

## Settings UI

```rust
let mut custom = appearance.accent.is_some();
settings_switch(ui, "Brand accent", "Replace the theme accent; contrast stays AA.", &mut custom);
if custom {
    let mut color = appearance.accent.unwrap_or(Color32::from_rgb(0xe0, 0x6c, 0x2b));
    settings_row(ui, "Accent color", "Buttons, links, focus, and selection.", |ui| {
        egui::color_picker::color_edit_button_srgba(ui, &mut color, egui::color_picker::Alpha::Opaque);
    });
    appearance.accent = Some(color);
} else {
    appearance.accent = None;
}
```

Also offer density, corners, and text size in the Shape tab:

```rust
settings_row(ui, "Density", "Height of controls and rows.", |ui| {
    combo(ui, "Density", &mut appearance.density, &densities);
});
settings_row(ui, "Corners", "Radius of controls, cards, and dialogs.", |ui| {
    combo(ui, "Corners", &mut appearance.corners, &corners);
});
settings_row(ui, "Text size", "Scale of every label, menu, and title.", |ui| {
    combo(ui, "Text size", &mut appearance.text_size, &sizes);
});
```

## Persist

Store `density.as_setting()`, `corners` as `default|sharp|rounded`, `text_size` as
`small|default|large`, and the accent as lowercase `#rrggbb`. On load, parse unknown values to defaults; a broken accent should not
prevent startup.

## Check contrast

```rust
let brand = Theme::dark().with_accent(Color32::from_rgb(0xe0, 0x6c, 0x2b));
assert!(brand.accessibility_issues().is_empty());
```

`with_accent` (and therefore `set_accent`) derives focus, link, selection, and accent text
to meet AA on every built-in base; a test checks all nine. If the base theme is custom,
run `accessibility_issues()` after applying the accent too.

## Logo and icons

- Register the product mark as `logo`; the title bar draws it at 20 pt.
- Draw custom monochrome glyphs in white so tinting works.
- Use `lucide-` names for general icons and Material for file trees.
- Replace `assets/logo.svg` and the template's neutral mark.

## What not to brand

- **Window controls.** Keep the standard close hover and hit areas.
- **Status bar and tab heights.** They align with native decorations.
- **Error and warning semantics.** Derive them from the theme; do not paint errors in the
  brand color.
- **Disabled states.** A brand accent on a disabled control looks enabled.

## Checklist

- [ ] Accent installed with `ThemeState::set_accent`, not a custom theme built from a
      hard-coded base.
- [ ] Contrast checked.
- [ ] Density, corners, and text size persisted as stable strings.
- [ ] Large text paired with a density that leaves room for it.
- [ ] Logo and custom glyphs registered before the first frame.
