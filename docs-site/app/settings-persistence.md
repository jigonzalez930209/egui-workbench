---
title: Settings persistence
description: Persist theme, motion, density, corners, text size, and brand accent atomically in the platform config directory.
---

# Settings persistence

The template persists appearance preferences outside the project. The library provides
no storage layer; this is one implementation you can replace.

## File location

| Platform | Path |
|---|---|
| Linux | `$XDG_CONFIG_HOME/<app>/settings.json`, or `~/.config/<app>/settings.json` |
| Windows | `%APPDATA%\<app>\settings.json` |

`<app>` is `CARGO_PKG_NAME`. A missing or invalid file falls back to defaults, so a broken
settings file never prevents startup.

## Shape

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: String,          // "github-light", "dracula", …
    pub follow_system: bool,
    pub reduced_motion: bool,
    pub density: String,        // "compact" | "default" | "comfortable"
    pub corners: String,        // "default" | "sharp" | "rounded"
    pub text_size: String,      // "small" | "default" | "large"
    pub accent: Option<String>, // "#rrggbb" or None
}
```

`#[serde(default)]` makes every field optional in the file; missing fields take the
default. The default is GitHub Light, no follow-system, no reduced motion, default
density, corners, and text size, no accent.

## Atomic writes

```rust
pub fn save(settings: &Settings) -> io::Result<()> {
    let Some(path) = path() else { return Ok(()) };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(settings).map_err(io::Error::other)?;
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, text)?;
    if let Err(error) = std::fs::rename(&temporary, &path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}
```

Write to a temporary file and rename it, so a crash mid-write cannot truncate the previous
settings.

## Apply at startup

```rust
let stored = settings::load();
let mut theme = ThemeState::with_reduced_motion(
    ctx,
    ThemeId::from_setting(&stored.theme),
    stored.follow_system,
    stored.reduced_motion,
);
let appearance = Appearance::from_settings(&stored);
appearance.apply(ctx, &mut theme);
```

`Appearance::from_settings` parses `density`, `corners`, `text_size`, and `accent`.
`Appearance::apply` installs the `WorkbenchStyle` and hands the accent to
`ThemeState::set_accent`, which keeps it across theme changes.

## Save on change

```rust
fn save_settings(&mut self) {
    let mut stored = Settings {
        theme: self.theme.theme_setting().to_owned(),
        follow_system: self.theme.follow_system(),
        reduced_motion: self.theme.reduced_motion(),
        ..Settings::default()
    };
    self.appearance.store(&mut stored);
    if let Err(error) = settings::save(&stored) {
        self.status = format!("Cannot save settings: {error}");
    }
}
```

Call it after a theme, follow-system, reduced-motion, density, corner, text-size, or accent
change;
not every frame.

## Round-trip tests

```rust
#[test]
fn settings_round_trip_through_json() {
    let settings = Settings {
        theme: "dracula".to_owned(),
        follow_system: true,
        reduced_motion: true,
        density: "compact".to_owned(),
        corners: "rounded".to_owned(),
        accent: Some("#e06c2b".to_owned()),
    };
    let text = serde_json::to_string(&settings).expect("serialize");
    let restored: Settings = serde_json::from_str(&text).expect("deserialize");
    assert_eq!(restored, settings);
}
```

Also test that an unknown theme string falls back to GitHub Light, and that malformed
JSON does not panic.

## Parsing the accent

```rust
fn parse_hex(value: &str) -> Option<Color32> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    Some(Color32::from_rgb(channel(0..2)?, channel(2..4)?, channel(4..6)?))
}
```

Store it back as `#rrggbb` with lowercase hex.

## What not to persist

- Window size and panel splits: egui remembers them under stable ids, but they are not in
  `Settings`. Persist them separately if the product needs them across launches.
- Document content and paths: that is your domain model, not appearance.
- Derived values such as the effective theme: store the requested ID and the
  follow-system flag; recompute the effective theme at startup.
- View state for plots: keep it in the tab model or a separate file, not in appearance
  settings.

## Replacing the format

The template uses JSON because it is easy to inspect. If you prefer TOML, `ron`, or a
database, implement `load`/`save` and keep the same `Settings` struct. The library does
not depend on the format.

## Failure behavior

- Missing file: defaults.
- Invalid JSON: defaults.
- Unknown theme: GitHub Light.
- Permission error on save: keep running and show the message in the status bar.
- Partial write: the temporary file is removed; the previous file remains.

Never block startup on a settings read, and never let a settings failure crash the app.
