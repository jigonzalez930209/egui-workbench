---
title: Settings dialog
description: Build a tabbed settings surface with rows, switches, selects, and an appearance tab.
---

# Settings dialog

<LiveDemo
  scene="settings"
  height="460"
  alt="A settings dialog with a left rail and appearance controls"
  caption="A tab rail, rows, switches, and a theme select that applies immediately."
/>

## Dialog definition

```rust
use egui_workbench::SettingsDialog;

let dialog = SettingsDialog {
    title: "Settings",
    tabs: &["Appearance", "Shape", "Editor"],
    note: "My App 0.4.0",
    done: "Done",
    close: "Close",
};
let mut open = self.settings_open;
show_settings_dialog(ctx, &mut open, &dialog, |ui, tab| {
    match tab {
        0 => appearance_tab(ui, self),
        1 => shape_tab(ui, self),
        _ => editor_tab(ui, self),
    }
});
self.settings_open = open;
```

`show_settings_dialog` owns the 560–620 pt frame, the left rail, Done/×/Esc, and the
width clamp for small windows. It returns the body value (`Option<R>`, usually
`Option<()>`); write changes into your state inside the closure or collect them in
locals and apply after.

## Sections and rows

```rust
settings_section(ui, "Appearance", "Theme and motion.");
settings_row(ui, "Theme", "Color family for the whole workbench.", |ui| {
    combo(ui, "Theme", &mut theme, &options);
});
settings_switch(ui, "Follow system", "Match the desktop light or dark preference.", &mut follow);
settings_switch(ui, "Reduce motion", "Pause non-essential animation.", &mut reduce);
```

| Helper | Layout |
|---|---|
| `settings_section(ui, title, description)` | Heading + muted description |
| `settings_row(ui, title, description, control)` | Label column + control column |
| `settings_switch(ui, title, description, &mut bool)` | Row with a switch |

Descriptions wrap; controls align on the right. On narrow windows the dialog shrinks and
the control column yields.

## Shape tab

```rust
settings_section(ui, "Shape", "Density, corners, text size, and brand color.");
let densities: Vec<(Density, String)> = Density::ALL
    .iter()
    .map(|density| (*density, format!("{density:?}")))
    .collect();
settings_row(ui, "Density", "Height of controls and rows.", |ui| {
    combo(ui, "Density", &mut appearance.density, &densities);
});
let sizes: Vec<(TextSize, String)> = TextSize::ALL
    .iter()
    .map(|size| (*size, size.label().to_owned()))
    .collect();
settings_row(ui, "Text size", "Scale of every label, menu, and title.", |ui| {
    combo(ui, "Text size", &mut appearance.text_size, &sizes);
});

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

## Apply after the dialog

```rust
let theme_changed = (theme, follow, reduce)
    != (self.theme.requested(), self.theme.follow_system(), self.theme.reduced_motion());

if theme_changed || appearance != self.appearance {
    self.theme.set_requested(ctx, theme);
    self.theme.set_follow_system(ctx, follow);
    self.theme.set_reduced_motion(ctx, reduce);
    self.appearance = appearance;
    self.appearance.apply(ctx, &mut self.theme);
    self.save_settings();
}
```

Apply only when something changed; `ThemeState` already avoids rebuilding styles when the
effective theme is unchanged, but the settings write should not run every frame.

## A settings screen without the dialog

The same rows work in a normal page:

```rust
egui::ScrollArea::vertical().show(ui, |ui| {
    settings_section(ui, "Editor", "Typography and behavior.");
    settings_row(ui, "Font size", "Body text in the editor.", |ui| {
        Stepper::new(&mut self.font_size, 10.0..=24.0, 1.0).show(ui, &mut self.icons);
    });
    settings_switch(ui, "Word wrap", "Wrap long lines.", &mut self.wrap);
});
```

## Checklist

- [ ] One tab per concern, not one giant page.
- [ ] Descriptions explain the consequence, not the control.
- [ ] Apply and persist only on change.
- [ ] Keep the dialog usable at 640×480.
- [ ] `Esc` closes it; Done is on the right.
- [ ] Destructive resets are confirmed with `ConfirmDialog`.
