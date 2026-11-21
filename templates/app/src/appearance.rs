//! Shape, density, text size, and brand color on top of the selected theme.
//!
//! The library keeps colors ([`egui_workbench::Theme`]) and shape ([`WorkbenchStyle`])
//! apart; this module is the application's choice of both. Replace the presets with
//! your brand.

use egui_workbench::app::ThemeState;
use egui_workbench::egui::{Color32, Context};
use egui_workbench::style::{self, Density, WorkbenchStyle};

use crate::settings::Settings;

/// Corner presets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Corners {
    /// The workbench radii (6 pt controls, 10 pt dialogs).
    #[default]
    Default,
    /// Square corners.
    Sharp,
    /// Larger radii.
    Rounded,
}

impl Corners {
    /// Every preset.
    pub const ALL: [Self; 3] = [Self::Default, Self::Sharp, Self::Rounded];

    /// The settings label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Sharp => "Sharp",
            Self::Rounded => "Rounded",
        }
    }

    fn as_setting(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Sharp => "sharp",
            Self::Rounded => "rounded",
        }
    }

    fn from_setting(value: &str) -> Self {
        match value {
            "sharp" => Self::Sharp,
            "rounded" => Self::Rounded,
            _ => Self::Default,
        }
    }

    fn style(self) -> WorkbenchStyle {
        match self {
            Self::Default => WorkbenchStyle::default(),
            Self::Sharp => WorkbenchStyle::sharp(),
            Self::Rounded => WorkbenchStyle::rounded(),
        }
    }
}

/// Text size presets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextSize {
    /// 90 %.
    Small,
    /// The reference type scale.
    #[default]
    Default,
    /// 115 %.
    Large,
}

impl TextSize {
    /// Every preset.
    pub const ALL: [Self; 3] = [Self::Small, Self::Default, Self::Large];

    /// The settings label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Default => "Default",
            Self::Large => "Large",
        }
    }

    /// The factor for [`WorkbenchStyle::text_scale`].
    pub const fn scale(self) -> f32 {
        match self {
            Self::Small => 0.9,
            Self::Default => 1.0,
            Self::Large => 1.15,
        }
    }

    fn as_setting(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Default => "default",
            Self::Large => "large",
        }
    }

    fn from_setting(value: &str) -> Self {
        match value {
            "small" => Self::Small,
            "large" => Self::Large,
            _ => Self::Default,
        }
    }
}

/// The application's appearance on top of the theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Appearance {
    /// Control and row heights.
    pub density: Density,
    /// Corner radii.
    pub corners: Corners,
    /// Font sizes.
    pub text_size: TextSize,
    /// A brand color that replaces the theme's accent, link, focus, and selection.
    pub accent: Option<Color32>,
}

impl Appearance {
    /// Read the stored preferences.
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            density: Density::from_setting(&settings.density),
            corners: Corners::from_setting(&settings.corners),
            text_size: TextSize::from_setting(&settings.text_size),
            accent: settings.accent.as_deref().and_then(parse_hex),
        }
    }

    /// Write the preferences into `settings`.
    pub fn store(&self, settings: &mut Settings) {
        settings.density = self.density.as_setting().to_owned();
        settings.corners = self.corners.as_setting().to_owned();
        settings.text_size = self.text_size.as_setting().to_owned();
        settings.accent = self.accent.map(to_hex);
    }

    /// The style these preferences describe.
    pub fn style(&self) -> WorkbenchStyle {
        WorkbenchStyle {
            text_scale: self.text_size.scale(),
            ..self.corners.style().with_density(self.density)
        }
    }

    /// Install the style and the brand accent. The theme state keeps the accent across
    /// theme changes and desktop light/dark switches, so call this only when the
    /// appearance itself changes.
    pub fn apply(&self, ctx: &Context, theme_state: &mut ThemeState) {
        style::set(ctx, self.style());
        theme_state.set_accent(ctx, self.accent);
    }
}

fn parse_hex(value: &str) -> Option<Color32> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).ok();
    Some(Color32::from_rgb(channel(0..2)?, channel(2..4)?, channel(4..6)?))
}

fn to_hex(color: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_workbench::Theme;

    #[test]
    fn appearance_round_trips_through_settings() {
        let appearance = Appearance {
            density: Density::Compact,
            corners: Corners::Rounded,
            text_size: TextSize::Large,
            accent: Some(Color32::from_rgb(0xe0, 0x6c, 0x2b)),
        };
        let mut settings = Settings::default();
        appearance.store(&mut settings);
        assert_eq!(settings.accent.as_deref(), Some("#e06c2b"));
        assert_eq!(Appearance::from_settings(&settings), appearance);
    }

    #[test]
    fn invalid_accents_are_ignored() {
        assert_eq!(parse_hex("e06c2b"), None);
        assert_eq!(parse_hex("#zz0000"), None);
        assert_eq!(parse_hex("#123"), None);
    }

    #[test]
    fn an_accent_follows_the_selected_theme() {
        let ctx = Context::default();
        let mut theme_state = ThemeState::new(&ctx, egui_workbench::ThemeId::GithubDark, false);
        let appearance = Appearance {
            text_size: TextSize::Large,
            accent: Some(Color32::from_rgb(0xe0, 0x6c, 0x2b)),
            ..Appearance::default()
        };
        appearance.apply(&ctx, &mut theme_state);
        assert!(theme_state.is_dark());
        assert_ne!(theme_state.tokens().accent, Theme::dark().accent);
        assert_eq!(style::get(&ctx).text_scale, 1.15);
        theme_state.set_requested(&ctx, egui_workbench::ThemeId::GithubLight);
        assert!(theme_state.is_light(), "the accent is re-derived on the new base");
        assert_ne!(theme_state.tokens().accent, Theme::light().accent);
    }
}
