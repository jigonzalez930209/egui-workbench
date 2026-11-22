# Custom themes

> Build tokens from a brand accent, check contrast, install them with ThemeState, and keep them across theme changes. Live page: <https://jigonzalez930209.github.io/egui-workbench/theming/custom-themes.html>

There are two levels of customization:

1. **A brand accent on a built-in base.** `Theme::with_accent` derives the accent, focus,
   link, selection, and accent text colors while keeping the rest of the theme.
2. **A fully custom `Theme`.** All fields are public, so a struct-update expression can
   replace any token.

## with_accent

```rust
use egui_workbench::theme::{contrast_ratio, Theme};
use egui_workbench::egui::Color32;

let brand = Theme::dark().with_accent(Color32::from_rgb(0xe0, 0x6c, 0x2b));
assert!(brand.accessibility_issues().is_empty());
assert!(contrast_ratio(brand.link, brand.background) >= 4.5);
```

`with_accent`:

- sets `accent` to the supplied color;
- derives `focus` and `link` until they meet 3:1 and 4.5:1 against the background;
- blends `selection` between the background and the accent;
- picks white or black for `accent_text`, whichever reads better;
- adjusts the accent itself if its text would miss 4.5:1.

The result keeps the base's light/dark mode and all other tokens.

## Fully custom tokens

```rust
use egui_workbench::theme::Theme;
use egui_workbench::egui::Color32;

let custom = Theme {
    surface: Color32::from_rgb(0x14, 0x18, 0x1f),
    border: Color32::from_rgb(0x2a, 0x31, 0x3d),
    ..Theme::dark()
};
```

Keep `background` and `text` consistent with `is_dark()`: the installed egui base
(`Visuals::dark()` or `Visuals::light()`) is chosen from the background luminance.

## Install

```rust
theme::apply_tokens(ctx, &brand);          // one-off
state.set_custom(ctx, Some(brand));        // through ThemeState
state.set_custom(ctx, None);               // back to the requested built-in
```

`ThemeState::set_custom` applies immediately when the value changes and remembers the
custom theme in `custom()`. While a custom theme is set, `sync_system` and
`set_follow_system` do not change the effective theme. If all you want is a brand color,
use `set_accent` instead (next section): it keeps following the system.

## Keep the accent across theme changes

`ThemeState::set_accent` stores a brand color and applies it on top of whatever built-in
theme is effective. Changing the theme, toggling follow-system, or a desktop light/dark
switch picked up by `sync_system` re-derives it on the new base; you do not re-apply
anything:

```rust
let mut theme = ThemeState::new(ctx, ThemeId::GithubDark, false);
theme.set_accent(ctx, Some(Color32::from_rgb(0xe0, 0x6c, 0x2b)));

theme.set_requested(ctx, ThemeId::GithubLight); // accent re-derived on GitHub Light
theme.set_follow_system(ctx, true);             // and on every desktop switch
theme.sync_system(ctx);                         // (call this every frame in `logic`)
assert_eq!(theme.accent(), Some(Color32::from_rgb(0xe0, 0x6c, 0x2b)));
```

`tokens()` returns `custom` when set, otherwise `Theme::for_id(effective).with_accent(accent)`.
A custom theme takes precedence over the accent.

The template's `Appearance::apply` is two lines:

```rust
pub fn apply(&self, ctx: &Context, theme_state: &mut ThemeState) {
    style::set(ctx, self.style());
    theme_state.set_accent(ctx, self.accent);
}
```

Call it at startup and when the appearance settings change; `App::logic` only calls
`theme.sync_system(ctx)`.

## Check contrast

`Theme::accessibility_issues()` returns `(label, ratio)` pairs below WCAG AA. An empty
vector means the theme passes:

```rust
for (pair, ratio) in custom.accessibility_issues() {
    eprintln!("{pair}: {ratio:.2}:1");
}
```

The built-in themes return empty, and the unit tests iterate all nine. Add a test for
your own palette; do not ship a theme that fails.

## Helpers

| Function | Use |
|---|---|
| `theme::mix(a, b, t)` | Blend colors in sRGB |
| `theme::readable_on(fill, dark)` | White or `dark`, whichever contrasts more |
| `theme::contrast_ratio(a, b)` | WCAG ratio |
| `theme::relative_luminance(color)` | WCAG luminance |
| `theme::apply_tokens(ctx, &theme)` | Install tokens and re-apply the style |
| `theme::current(ctx)` | Read the installed tokens |

## Persisting a custom accent

Store the accent as `#rrggbb` (the template uses `Option<String>`), parse it at startup,
and derive the `Theme` from the effective built-in theme. If parsing fails, fall back to
the built-in accent instead of refusing to start.

## Limits

- A custom theme is not a `ThemeId`; it cannot be serialized as a theme name. Persist the
  accent or the full token set separately.
- `ThemeState::is_light()` / `is_dark()` follow the custom background, not the requested
  ID.
- A theme change re-applies the style, so `style::set` must have been called (or the
  default will be used). `WorkbenchStyle` survives theme changes.
