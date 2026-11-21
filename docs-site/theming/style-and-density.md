---
title: Style and density
description: WorkbenchStyle, density factors, corner presets, border width, text scale, and what the style does not change.
---

# Style and density

`WorkbenchStyle` is the global shape layer: control heights, row heights, corner radii,
border width, and text scale. It is separate from `Theme`, which owns colors, and it
survives theme changes. Every component in the crate reads it: there are no literal radii
or font sizes outside `style.rs` (`cargo xtask style-lint` enforces this).

<LiveDemo
  scene="style"
  look="compact-sharp"
  height="540"
  alt="The component board at compact density with square corners"
  caption="Starts at compact density + sharp corners; change density, corners, text size, and borders live."
/>

<LiveDemo
  scene="style"
  look="comfortable-rounded"
  height="660"
  alt="The component board at comfortable density with rounded corners"
  caption="Comfortable density + rounded corners."
/>

<LiveDemo
  scene="style"
  look="large-borderless"
  height="660"
  alt="The component board with 125 % text and no borders"
  caption="text_scale 1.25 + border_width 0: unchecked checkboxes get a fill, Outline buttons keep their line."
/>

## Install

```rust
use egui_workbench::style::{self, Density, WorkbenchStyle};

style::set(ctx, WorkbenchStyle {
    density: Density::Compact,
    control_radius: 3,
    ..WorkbenchStyle::default()
});

assert_eq!(style::get(ctx).control_height(), 26.0);
```

Call `style::set` once at startup and after a settings change. `theme::apply_tokens`
re-applies the installed style, so switching themes does not lose it.

## Fields

| Field | Default | Affects |
|---|---|---|
| `density` | `Default` | Control and row heights, vertical button padding |
| `control_radius` | 6 | Buttons, text fields, selects, steppers, context menus, egui widgets |
| `item_radius` | 5 | Hover fills of icon buttons, nav items, segmented controls |
| `inner_radius` | 4 | Shapes inside other shapes: menu and tree rows, checkboxes, segments, plot toolbar buttons |
| `container_radius` | 8 | Cards, popovers, legends, egui windows |
| `dialog_radius` | 10 | Modal dialogs and toasts |
| `pill_radius` | 12 | Capsules: badges, switches, title-bar search (clamped to half the height) |
| `border_width` | 1.0 | Borders of controls, cards, popovers, dialogs (`0.0` removes them) |
| `text_scale` | 1.0 | Multiplies every font size; clamped to `0.75..=1.5` |

Separators, focus rings, and the 2 pt active-tab indicator keep their own widths.

### Radius helpers

Components never build a `CornerRadius` from a number; they ask the style for a role:

| Helper | Value | Used by |
|---|---|---|
| `control()` | `control_radius` | Buttons, fields, menus |
| `item()` | `item_radius` | Icon-button and nav hover fills |
| `inner()` | `inner_radius` | Menu and tree rows, checkboxes |
| `tight()` | `inner_radius − 1` | Tiny hover targets (tab close, stepper arrows) |
| `mark()` | `min(inner_radius, 1)` | Indicator bars and underlines |
| `container()` | `container_radius` | Cards, popovers |
| `dialog()` | `dialog_radius` | Dialogs, toasts |
| `pill(height)` | `min(pill_radius, height / 2)` | Capsules |
| `border(color)` | `Stroke` of `border_width` | Every bordered surface |

Use the same helpers in your own widgets so they follow the product's style:

```rust
let style = egui_workbench::style::get(ui.ctx());
ui.painter().rect(rect, style.container(), fill, style.border(stroke_color), egui::StrokeKind::Inside);
```

## Density

| Density | Factor | Control height | Intent |
|---|---|---|---|
| `Compact` | 0.86 | 26 pt | Data-heavy tools |
| `Default` | 1.0 | 30 pt | Reference sizes in `metrics` |
| `Comfortable` | 1.16 | 35 pt | Touch screens, presentations |

`Density::ALL`, `as_setting()`, and `from_setting()` round-trip through settings. The
factor is applied to heights and vertical padding, rounded to whole points; horizontal
padding stays fixed and text sizes follow `text_scale`, not the density.

`WorkbenchStyle::scaled(height)` is public if you need the same rounding elsewhere.

## Text scale

`text_scale` multiplies the whole type scale: egui's text styles (`TYPE_SCALE`: body 14,
button 12.5, small 11, monospace 13, heading 20), builder labels (`Size::text`), and the
chrome's titles, tabs, menus, and shortcuts.

```rust
style::set(ctx, WorkbenchStyle { text_scale: 1.15, ..WorkbenchStyle::default() });
```

Heights do not grow with the text, so pair a large text scale with
`Density::Comfortable` when labels start to touch the edges. Values outside
`0.75..=1.5` (and NaN) are clamped by `text_scale()`.

For text in your own widgets, use the font helpers instead of `FontId::proportional(…)`:

| Helper | Returns |
|---|---|
| `font_size(points)` | `points × text_scale` |
| `font(points)` / `mono(points)` | Proportional / monospace `FontId` |
| `text(size)` | Label size of a builder `Size` |

## Presets

```rust
let sharp = WorkbenchStyle::sharp();       // square controls, 2 pt pills
let rounded = WorkbenchStyle::rounded();   // larger radii everywhere
let compact = WorkbenchStyle::default().with_density(Density::Compact);
```

`sharp()` uses radius 0 for every role (controls, items, inner shapes, containers,
dialogs) and 2 pt pills, so switches and checkboxes are square too. `rounded()` uses
9/7/5/12/14 and full capsules (`pill_radius: u8::MAX`). Both keep the default density,
border, and text scale.

## What reads the style

Everything the crate draws:

- components: `Button`, `IconButton`, `TextField`, `TextArea`, `Select`, `Stepper`,
  `Checkbox`, `Switch`, `Label`, `Card`, `Badge`, `Banner`, `Nav`, `IconGrid`, `Toasts`;
- widgets: menus, the keyboard menu, the command palette, dialogs, settings rows, the
  explorer tree, segmented controls;
- chrome: tab strip (radii, fonts), title bar (search pill, menus, title font);
- the plot's toolbar, legend, and hover popups (the data area is configured by
  `PlotConfig`);
- egui's own widgets: `style::set` copies radii, border width, interact height, button
  padding, and the scaled type scale into egui's styles, and every theme change re-applies
  them.

Two rules keep a borderless style usable: an unchecked `Checkbox` gets a subtle fill when
`border_width` is 0, and `ButtonVariant::Outline` always keeps a 1 pt line (otherwise it
would look like `Ghost`).

## What does not scale

The window chrome keeps fixed `metrics`:

- title bar 34 pt;
- tab strip 34 pt;
- status bar 24 pt;
- window control cell 44 × 32 pt.

These surfaces align with native window decorations and the app icon. Scaling them
independently makes the window controls float or clip. If a product needs different
chrome sizes, change the panel heights at the shell level rather than mutating `metrics`
at runtime.

## Persist a style

The template stores `density`, `corners`, and `text_size` as strings:

```rust
#[derive(Serialize, Deserialize)]
struct Settings {
    density: String,    // compact | default | comfortable
    corners: String,    // default | sharp | rounded
    text_size: String,  // small | default | large  (0.9 / 1.0 / 1.15)
    accent: Option<String>,
}
```

At startup, parse them into `Density`, a corner preset, and a text scale, then call
`style::set`:

```rust
pub fn style(&self) -> WorkbenchStyle {
    WorkbenchStyle {
        text_scale: self.text_size.scale(),
        ..self.corners.style().with_density(self.density)
    }
}
``` See
[Settings persistence](/app/settings-persistence) and the template's `appearance.rs`.

## Keeping it complete

`cargo xtask style-lint` (part of `cargo xtask check`) fails on a non-zero literal in
`CornerRadius::same(…)`, `.corner_radius(…)`, `FontId::proportional/monospace/new(…)`, or
the radius argument of `rect`, `rect_filled`, and `rect_stroke` anywhere under `src/`,
including calls split over several lines. A value that really is fixed (a 2 pt indicator,
a size the user configured) ends its line with `// style: fixed`.

## Testing a style

The crate's snapshot tests render the component board in default, compact + sharp,
comfortable + rounded, large text + borderless, and with a custom accent. The default
style is pixel-identical to the reference shell, so adding the style layer changed no
existing snapshot. `style::get` reads from egui's temporary data, so a test context
gets the default when nothing was set. Use `style::set` in `with_context_setup` and reset
it after the harness if other tests share the context.
