# Buttons

> Button, IconButton, and CloseButton: variants, sizes, disabled, selected, and per-instance overrides. Live page: <https://jigonzalez930209.github.io/egui-workbench/components/buttons.html>

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

| Item | Type | Default | Description |
|---|---|---|---|
| `Button::new(label)` | `Button` | Primary, Md | A primary button at the default size. |
| `.variant(v)` | `ButtonVariant` | Primary | Visual role. |
| `.size(s)` | `Size` | Md | Sm = 24 pt, Md = 30 pt, Lg = 36 pt. |
| `.enabled(bool)` | `bool` | true | Disabled buttons are faded and never report clicks. |
| `.selected(bool)` | `bool` | false | Paint as a selected toggle. |
| `.width(pt)` | `f32` | Size::min_width | Minimum width in points. |
| `.tooltip(text)` | `&str` | None | Hover text and accessible description. |
| `.fill(color)` | `Color32` | Variant fill | Override the background. |
| `.text_color(color)` | `Color32` | Variant text | Override the label color. |
| `.stroke(stroke)` | `Stroke` | Variant stroke | Override the border; Stroke::NONE removes it. |
| `.radius(r)` | `u8` | Style control radius | Override the corner radius. |
| `.show(ui)` | `Response` | — | Paint and return the egui response. |

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

| Item | Type | Default | Description |
|---|---|---|---|
| `IconButton::new(icon, tooltip)` | `IconButton` | Action | An icon button; the tooltip is the accessible name and hover text. |
| `.size(s)` | `IconButtonSize` | Action | 28, 26, or 21 pt side. |
| `.active(bool)` | `bool` | false | Toggled fill (and title-bar underline). |
| `.open(bool)` | `bool` | false | Lit while a popup is open. |
| `.enabled(bool)` | `bool` | true | Faded and inert when false. |
| `.color(color)` | `Color32` | Text color | Tint the glyph. |
| `.radius(r)` | `u8` | Style item radius | Override the hover-fill radius. |
| `.show(ui, icons)` | `Response` | — | Paint and return the response. |

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
