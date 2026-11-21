---
title: Themes
description: The nine built-in themes, theme identifiers, follow-system behavior, and live theme switching.
---

# Themes

The crate ships nine stable themes in three families. Every color in the shell comes
from the active `Theme` tokens or from the egui visuals derived from them.

<LiveDemo
  scene="shell"
  height="460"
  alt="The same workbench in GitHub Light and GitHub Dark"
  caption="Toggle the site's light/dark mode and the demo follows; pick other themes from View."
/>

| Theme | `ThemeId` | Setting | Family | Mode |
|---|---|---|---|---|
| GitHub Light | `GithubLight` | `github-light` | GitHub | Light |
| GitHub Dark | `GithubDark` | `github-dark` | GitHub | Dark |
| GitHub Dark Dimmed | `GithubDarkDimmed` | `github-dark-dimmed` | GitHub | Dark |
| Dracula | `Dracula` | `dracula` | Dracula | Dark |
| Dracula Soft | `DraculaSoft` | `dracula-soft` | Dracula | Dark |
| Alucard | `Alucard` | `alucard` | Dracula | Light |
| One Dark Pro | `OneDarkPro` | `one-dark-pro` | One Dark Pro | Dark |
| One Dark Pro Darker | `OneDarkProDarker` | `one-dark-pro-darker` | One Dark Pro | Dark |
| One Dark Pro Light | `OneDarkProLight` | `one-dark-pro-light` | One Dark Pro | Light |

`ThemeId::ALL` lists them in display order. `label()`, `family()`, and `mode()` drive
menus and settings. `from_setting` accepts the old `light`/`dark` values; unknown values
fall back to GitHub Light.

<LiveDemo
  scene="shell"
  theme="dracula"
  height="460"
  alt="The workbench in the Dracula theme"
  caption="Dracula: the same tokens, a different family."
/>

## Apply a theme

```rust
use egui_workbench::{ThemeId, theme};

// Directly, for one-off screens and tests.
theme::apply_id(ctx, ThemeId::GithubDark);

// Follow the desktop light/dark preference, keeping the family.
let effective = theme::apply_theme(ctx, ThemeId::Dracula, true);
```

`apply_id` installs visuals, widget states, spacing, scrollbars, and the type scale.
`apply_theme` resolves the system preference first and returns the effective ID.

## ThemeState

Applications should keep a `ThemeState`; it remembers the request, the effective theme,
follow-system, reduced motion, and an optional custom theme.

```rust
use egui_workbench::app::ThemeState;

let mut theme = ThemeState::new(ctx, ThemeId::GithubLight, false);
theme.set_requested(ctx, ThemeId::OneDarkPro);
theme.set_follow_system(ctx, true);

// Call every frame in App::logic; it only re-applies when the system changed.
theme.sync_system(ctx);

assert!(theme.is_dark());
let clear = theme.clear_color(); // for eframe::App::clear_color
```

| Method | Purpose |
|---|---|
| `new(ctx, requested, follow_system)` | Apply and remember the preference |
| `with_reduced_motion(…)` | Same, with animations disabled |
| `from_settings(ctx, setting, follow, reduced)` | Restore from persisted strings |
| `requested()` / `effective()` | Stored preference / installed theme |
| `follow_system()` / `reduced_motion()` | Current flags |
| `tokens()` | The installed `Theme` (custom when set, else built-in + accent) |
| `accent()` / `set_accent(ctx, Option<Color32>)` | Brand color re-derived on every theme change |
| `custom()` / `set_custom(ctx, Option<Theme>)` | Own tokens or back to built-ins |
| `is_light()` / `is_dark()` | Base of the effective theme |
| `clear_color()` | `[f32; 4]` for `eframe::App::clear_color` |
| `apply(ctx)` | Re-resolve and install |
| `set_requested(ctx, id)` | Change and apply |
| `set_follow_system(ctx, bool)` | Change and apply |
| `set_reduced_motion(ctx, bool)` | Toggle animations |
| `sync_system(ctx)` | Re-apply when the desktop preference changed |
| `theme_setting()` | Stable string for settings files |

`sync_system` is cheap: it returns `false` without touching styles when the effective
theme did not change. While a custom theme is set, following the system has no effect
until the custom theme is cleared.

## Follow system

`follow_system(true)` keeps the family and picks its light or dark base:

| Requested family | Desktop light | Desktop dark |
|---|---|---|
| GitHub | GitHub Light | GitHub Dark |
| Dracula | Alucard | Dracula |
| One Dark Pro | One Dark Pro Light | One Dark Pro |

`theme::effective_id(id, follow_system, system_dark)` performs the resolution without
applying anything. `theme::system_is_dark(ctx)` reads egui's raw system preference.

## Reduced motion

`theme::set_reduced_motion(ctx, true)` sets egui's animation time to zero.
`theme::motion_enabled(ctx)` reports the current state. `ThemeState` applies the flag on
every theme change, so an animation cannot reappear after switching themes.

## Changing themes live

```rust
match command {
    Command::SetTheme(id) => {
        self.theme.set_requested(ctx, id);
        self.save_settings();
    }
    // ...
}
```

A brand accent installed with `set_accent` is re-derived on the new base automatically;
see [Custom themes](/theming/custom-themes) and the template's `appearance.rs`.

## Adding a theme

1. Add the variant to `ThemeId`.
2. Add its tokens to `foundation/theme/palette.rs`.
3. Add `as_setting`, `label`, `family`, and `mode` arms.
4. The contrast tests iterate `ThemeId::ALL`, so they cover the new theme automatically.
5. Add a snapshot if it is meant to be a flagship.

The API map and the settings menu pick it up from `ThemeId::ALL` without further wiring.
