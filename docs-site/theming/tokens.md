---
title: Color tokens
description: Every Theme field, what it paints, how tokens map to egui visuals, and the contrast guarantee.
---

# Color tokens

`Theme` is a flat set of 19 color tokens. Components read tokens through
`ui.visuals()` or `theme::current(ctx)`; they never define a literal color.

## GitHub Light

| Token | Value | Paints |
|---|---|---|
| `background` | <TokenSwatch name="background" value="#ffffff" /> | Window, editor, panels (`panel_fill`) |
| `surface` | <TokenSwatch name="surface" value="#f6f8fa" /> | Menus, dialogs, sidebar, tab strip (`window_fill`) |
| `surface_raised` | <TokenSwatch name="surface_raised" value="#ffffff" /> | Second raised surface (toasts) |
| `sunken` | <TokenSwatch name="sunken" value="#f6f8fa" /> | Text fields, steppers, segmented (`extreme_bg_color`) |
| `text` | <TokenSwatch name="text" value="#1f2328" /> | Primary text |
| `text_muted` | <TokenSwatch name="text_muted" value="#596069" /> | Hints, descriptions, inactive tabs |
| `accent` | <TokenSwatch name="accent" value="#0969da" /> | Brand accent, active state |
| `accent_text` | <TokenSwatch name="accent_text" value="#ffffff" /> | Text on the accent |
| `border` | <TokenSwatch name="border" value="#d1d9e0" /> | Borders and separators |
| `selection` | <TokenSwatch name="selection" value="#b6d7ff" /> | Selected rows, segments, primary buttons |
| `hover` | <TokenSwatch name="hover" value="#eaeef2" /> | Row and icon hover |
| `danger` | <TokenSwatch name="danger" value="#cf222e" /> | Errors, destructive actions |
| `warning` | <TokenSwatch name="warning" value="#9a6700" /> | Warnings, conflict banners |
| `success` | <TokenSwatch name="success" value="#1a7f37" /> | Success messages |
| `done` | <TokenSwatch name="done" value="#8250df" /> | “Done” purple |
| `link` | <TokenSwatch name="link" value="#0969da" /> | Hyperlinks |
| `scrollbar` | <TokenSwatch name="scrollbar" value="#8c959f" /> | Scrollbar handle |
| `focus` | <TokenSwatch name="focus" value="#0969da" /> | Focus ring, hovered borders, active-row marker |

## GitHub Dark

| Token | Value |
|---|---|
| `background` | <TokenSwatch name="background" value="#0d1117" /> |
| `surface` | <TokenSwatch name="surface" value="#161b22" /> |
| `surface_raised` | <TokenSwatch name="surface_raised" value="#21262d" /> |
| `sunken` | <TokenSwatch name="sunken" value="#161b22" /> |
| `text` | <TokenSwatch name="text" value="#e6edf3" /> |
| `text_muted` | <TokenSwatch name="text_muted" value="#9fa7b2" /> |
| `accent` | <TokenSwatch name="accent" value="#1f6feb" /> |
| `accent_text` | <TokenSwatch name="accent_text" value="#ffffff" /> |
| `border` | <TokenSwatch name="border" value="#30363d" /> |
| `selection` | <TokenSwatch name="selection" value="#264f78" /> |
| `hover` | <TokenSwatch name="hover" value="#21262d" /> |
| `danger` | <TokenSwatch name="danger" value="#ff7b72" /> |
| `warning` | <TokenSwatch name="warning" value="#e3b341" /> |
| `success` | <TokenSwatch name="success" value="#3fb950" /> |
| `done` | <TokenSwatch name="done" value="#ab7df8" /> |
| `link` | <TokenSwatch name="link" value="#58a6ff" /> |
| `scrollbar` | <TokenSwatch name="scrollbar" value="#6e7681" /> |
| `focus` | <TokenSwatch name="focus" value="#58a6ff" /> |

The other seven themes use the same roles with their own values. `Theme::for_id(id)`
returns them; `foundation/theme/palette.rs` is the single source of truth.

## How tokens reach egui

`theme::apply_tokens(ctx, &theme)`:

| egui field | Token |
|---|---|
| `visuals.panel_fill` | `background` |
| `visuals.window_fill` | `surface` |
| `visuals.faint_bg_color` | `hover` |
| `visuals.extreme_bg_color` | `sunken` |
| `visuals.hyperlink_color` | `link` |
| `visuals.selection.bg_fill` | `selection` |
| `visuals.selection.stroke` | `text` |
| `visuals.window_stroke` | `border` |
| `visuals.error_fg_color` | `danger` |
| `visuals.warn_fg_color` | `warning` |
| `visuals.override_text_color` | `text` |
| widget `inactive` fill | `surface` |
| widget `hovered` fill | `hover` |
| widget `active` fill | `accent` |

The function also installs spacing, scrollbar metrics, and the type scale. The installed
`WorkbenchStyle` is re-applied after the visuals, so a theme change cannot reset density
or radii.

## Reading tokens in components

```rust
use egui_workbench::theme;

let tokens = theme::current(ui.ctx());
let success = tokens.success;   // egui visuals does not carry this role
let muted = tokens.text_muted;
```

`theme::current` returns the tokens last applied with `apply_id` or `apply_tokens`. If
none were applied, it falls back to GitHub Light or Dark based on the current visuals, so
a test harness always has a sensible theme.

## Contrast guarantee

`Theme::accessibility_issues()` checks the pairs that matter:

| Pair | Minimum |
|---|---|
| `text` on `background` | 4.5:1 |
| `text` on `surface` | 4.5:1 |
| `text_muted` on `background` | 4.5:1 |
| `link` on `background` | 4.5:1 |
| `accent_text` on `accent` | 4.5:1 |
| `danger` on `background` | 4.5:1 |
| `focus` on `background` | 3:1 |

Every built-in theme returns an empty vector, and the unit tests enforce it. Use the
same method after defining a custom theme.

## Helpers

| Function | Purpose |
|---|---|
| `theme::mix(a, b, t)` | Blend two colors in sRGB (`0 = a`, `1 = b`) |
| `theme::relative_luminance(color)` | WCAG relative luminance |
| `theme::contrast_ratio(a, b)` | WCAG contrast ratio |
| `theme::readable_on(fill, dark)` | White or the supplied dark color, whichever reads better |
| `theme::current(ctx)` | Installed tokens |
| `theme::motion_enabled(ctx)` | Whether animations are on |

`Theme::border_stroke()` returns a 1 pt border in the `border` token. For borders and
radii that follow the installed shape, use the style instead:
`style::get(ctx).border(theme.border)` and `style::get(ctx).control()`. `Theme::radius()`
is deprecated for that reason.
