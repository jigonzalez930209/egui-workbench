---
title: Buttons
description: "Button, IconButton, and CloseButton: variants, sizes, disabled, selected, and per-instance overrides."
---

# Buttons

<LiveDemo
  scene="buttons"
  height="240"
  alt="Primary, secondary, outline, ghost, destructive, link, and disabled buttons"
  caption="Every ButtonVariant plus a disabled primary button and three icon buttons."
/>

## Button

```rust
use egui_workbench::components::prelude::*;

if Button::new("Save").variant(ButtonVariant::Primary).show(ui).clicked() {
    save();
}

Button::new("Cancel").variant(ButtonVariant::Ghost).show(ui);
Button::new("Delete").variant(ButtonVariant::Destructive).show(ui);
Button::new("Documentation").variant(ButtonVariant::Link).show(ui);
```

### Variants

| Variant | Fill | Border | Text |
|---|---|---|---|
| `Primary` (default) | Selection token | None | Selection text token |
| `Secondary` | Inactive widget fill | 1 pt | Primary text |
| `Outline` | Transparent | 1 pt | Primary text |
| `Ghost` | Transparent; hover fill only | None | Primary text |
| `Destructive` | Danger token | None | White or black, whichever reads better |
| `Link` | Transparent | None | Hyperlink token |

`selected(true)` paints the selection fill and text regardless of the variant. A ghost
button keeps its padding but shows no frame until hover.

### API

<ApiTable :rows="[
  { name: 'Button::new(label)', type: 'Button', default: 'Primary, Md', description: 'A primary button at the default size.' },
  { name: '.variant(v)', type: 'ButtonVariant', default: 'Primary', description: 'Visual role.' },
  { name: '.size(s)', type: 'Size', default: 'Md', description: 'Sm = 24 pt, Md = 30 pt, Lg = 36 pt.' },
  { name: '.enabled(bool)', type: 'bool', default: 'true', description: 'Disabled buttons are faded and never report clicks.' },
  { name: '.selected(bool)', type: 'bool', default: 'false', description: 'Paint as a selected toggle.' },
  { name: '.width(pt)', type: 'f32', default: 'Size::min_width', description: 'Minimum width in points.' },
  { name: '.tooltip(text)', type: '&str', default: 'None', description: 'Hover text and accessible description.' },
  { name: '.fill(color)', type: 'Color32', default: 'Variant fill', description: 'Override the background.' },
  { name: '.text_color(color)', type: 'Color32', default: 'Variant text', description: 'Override the label color.' },
  { name: '.stroke(stroke)', type: 'Stroke', default: 'Variant stroke', description: 'Override the border; Stroke::NONE removes it.' },
  { name: '.radius(r)', type: 'u8', default: 'Style control radius', description: 'Override the corner radius.' },
  { name: '.show(ui)', type: 'Response', default: '—', description: 'Paint and return the egui response.' }
]" />

## IconButton

An icon-only button is named by its tooltip in AccessKit. The glyph comes from the shared
`Icons` cache.

```rust
if IconButton::new("save", "Save").show(ui, &mut icons).clicked() {
    save();
}

IconButton::new("sidebar", "Explorer")
    .active(self.sidebar)
    .show(ui, &mut icons);

IconButton::new("close", "Close tab")
    .size(IconButtonSize::Tool)
    .show(ui, &mut icons);
```

| Size | Side | Glyph | Typical use |
|---|---|---|---|
| `Action` (default) | 28 pt | 16 pt | Title-bar actions |
| `Tool` | 26 pt | 18 pt | Tab strip and toolbars |
| `Small` | 21 pt | 16 pt | Explorer and card headers |

`active(true)` paints the toggled fill; on `Action` size it also draws the title-bar
underline. `open(true)` keeps a popup button lit while its popup is open. `color` tints
the glyph; `radius` overrides the hover-fill radius.

<ApiTable :rows="[
  { name: 'IconButton::new(icon, tooltip)', type: 'IconButton', default: 'Action', description: 'An icon button; the tooltip is the accessible name and hover text.' },
  { name: '.size(s)', type: 'IconButtonSize', default: 'Action', description: '28, 26, or 21 pt side.' },
  { name: '.active(bool)', type: 'bool', default: 'false', description: 'Toggled fill (and title-bar underline).' },
  { name: '.open(bool)', type: 'bool', default: 'false', description: 'Lit while a popup is open.' },
  { name: '.enabled(bool)', type: 'bool', default: 'true', description: 'Faded and inert when false.' },
  { name: '.color(color)', type: 'Color32', default: 'Text color', description: 'Tint the glyph.' },
  { name: '.radius(r)', type: 'u8', default: 'Style item radius', description: 'Override the hover-fill radius.' },
  { name: '.show(ui, icons)', type: 'Response', default: '—', description: 'Paint and return the response.' }
]" />

## CloseButton

A borderless `×` for dialog headers and custom panels. It returns `bool`, not a `Response`.

```rust
if CloseButton::new("Close").size(14.0).show(ui) {
    open = false;
}
```

## Accessibility

- Text buttons expose `Role::Button` and their label.
- Icon buttons expose `Role::Button` and their tooltip; the glyph itself is decorative.
- Selected buttons announce the selected state.
- Disabled buttons are faded and ignored by pointer and keyboard input.

## Common mistakes

- **No tooltip on an icon button.** The AccessKit name comes from it, and kittest fails
  tests when it is missing.
- **A raw `ui.button`.** It bypasses the style's density, radius, and variant tokens.
- **Hard-coded width for a label that can grow.** Let the button measure its text, or
  set a minimum width and let it wrap in `horizontal_wrapped`.
