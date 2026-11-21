//! Appearance preferences stored outside the project.
//!
//! The file lives at `$XDG_CONFIG_HOME/__APP_NAME__/settings.json` on Linux (or
//! `~/.config/__APP_NAME__/settings.json`) and `%APPDATA%\__APP_NAME__\settings.json`
//! on Windows. A missing or invalid file falls back to the defaults, so a broken
//! settings file never prevents startup. Writes go to a temporary file and are renamed,
//! so a crash mid-write cannot truncate the previous settings.

use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Persisted appearance preferences.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Theme identifier; unknown values fall back to GitHub Light.
    pub theme: String,
    /// Follow the desktop light/dark preference.
    pub follow_system: bool,
    /// Disable animations.
    pub reduced_motion: bool,
    /// `compact`, `default`, or `comfortable`.
    pub density: String,
    /// `default`, `sharp`, or `rounded`.
    pub corners: String,
    /// `small`, `default`, or `large`.
    pub text_size: String,
    /// A brand accent as `#rrggbb`, or none for the theme's own.
    pub accent: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "github-light".to_owned(),
            follow_system: false,
            reduced_motion: false,
            density: "default".to_owned(),
            corners: "default".to_owned(),
            text_size: "default".to_owned(),
            accent: None,
        }
    }
}

/// Read the stored settings, or the defaults.
pub fn load() -> Settings {
    let Some(path) = path() else {
        return Settings::default();
    };
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Write the settings, creating the config directory.
pub fn save(settings: &Settings) -> io::Result<()> {
    let Some(path) = path() else {
        return Ok(());
    };
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

fn path() -> Option<PathBuf> {
    let app = env!("CARGO_PKG_NAME");
    if cfg!(windows) {
        return std::env::var_os("APPDATA").map(|dir| PathBuf::from(dir).join(app).join("settings.json"));
    }
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME").filter(|dir| !dir.is_empty()) {
        return Some(PathBuf::from(dir).join(app).join("settings.json"));
    }
    std::env::var_os("HOME").map(|dir| PathBuf::from(dir).join(".config").join(app).join("settings.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_github_light() {
        let settings = Settings::default();
        assert_eq!(settings.theme, "github-light");
        assert!(!settings.follow_system);
        assert!(!settings.reduced_motion);
    }

    #[test]
    fn settings_round_trip_through_json() {
        let settings = Settings {
            theme: "dracula".to_owned(),
            follow_system: true,
            reduced_motion: true,
            density: "compact".to_owned(),
            corners: "rounded".to_owned(),
            text_size: "large".to_owned(),
            accent: Some("#e06c2b".to_owned()),
        };
        let text = serde_json::to_string(&settings).expect("serialize");
        let restored: Settings = serde_json::from_str(&text).expect("deserialize");
        assert_eq!(restored, settings);
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let restored: Settings = serde_json::from_str(r#"{"theme":"dracula"}"#).expect("deserialize");
        assert_eq!(restored.theme, "dracula");
        assert!(!restored.follow_system);
        assert!(!restored.reduced_motion);
    }
}
