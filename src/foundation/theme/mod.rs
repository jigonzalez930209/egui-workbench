//! Theme tokens and their application to egui.
//!
//! Nine stable theme identifiers cover the GitHub, Dracula, and One Dark Pro families.
//! Every color the shell paints comes from [`Theme`] or from the egui visuals that
//! [`apply_id`] derives from it; components never invent a literal color (the only
//! exceptions are the window close button's red hover and white glyph, as on Windows
//! and GNOME). Contrast is checked in tests against WCAG 2.2 AA.

mod apply;
mod palette;

use egui::{Color32, CornerRadius, Stroke};

pub use apply::{
    ANIMATION_TIME, apply, apply_id, apply_theme, apply_tokens, current, effective_id, motion_enabled,
    set_reduced_motion, system_is_dark,
};
pub use palette::ThemeId;

/// The light or dark base of a theme.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ThemeMode {
    /// Light surfaces.
    #[default]
    Light,
    /// Dark surfaces.
    Dark,
}

impl ThemeMode {
    /// Parse a value stored in settings.
    pub fn from_setting(value: &str) -> Self {
        if value == "dark" { Self::Dark } else { Self::Light }
    }

    /// The value stored in settings.
    pub fn as_setting(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// The color tokens of one theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    /// Window, editor, and panel background (`visuals.panel_fill`).
    pub background: Color32,
    /// Raised chrome: menus, dialogs, the sidebar, the tab strip (`visuals.window_fill`).
    pub surface: Color32,
    /// A second raised surface for cards.
    pub surface_raised: Color32,
    /// Sunken surfaces: text fields, steppers, segmented controls (`extreme_bg_color`).
    pub sunken: Color32,
    /// Primary text.
    pub text: Color32,
    /// Secondary text: hints, descriptions, inactive tabs.
    pub text_muted: Color32,
    /// Brand accent.
    pub accent: Color32,
    /// Text drawn on the accent.
    pub accent_text: Color32,
    /// Borders and separators.
    pub border: Color32,
    /// Selection background: selected rows, active segments, primary buttons.
    pub selection: Color32,
    /// Hover background for rows and icon buttons.
    pub hover: Color32,
    /// Errors and destructive actions.
    pub danger: Color32,
    /// Warnings and conflict banners.
    pub warning: Color32,
    /// Success messages.
    pub success: Color32,
    /// "Done" purple.
    pub done: Color32,
    /// Hyperlinks.
    pub link: Color32,
    /// Scrollbar handle.
    pub scrollbar: Color32,
    /// Keyboard focus ring, hovered borders, and the active-row marker.
    pub focus: Color32,
}

mod studio;
#[cfg(feature = "serde")]
pub use studio::ThemeImportError;

impl Theme {
    /// GitHub Light.
    pub fn light() -> Self {
        palette::theme(ThemeId::GithubLight)
    }

    /// GitHub Dark.
    pub fn dark() -> Self {
        palette::theme(ThemeId::GithubDark)
    }

    /// The tokens for a theme identifier.
    pub fn for_id(id: ThemeId) -> Self {
        palette::theme(id)
    }

    /// Whether the surface is dark.
    pub fn is_dark(&self) -> bool {
        relative_luminance(self.background) < 0.5
    }

    /// The same tokens with a new brand color: `accent`, `focus`, `link`, `selection`,
    /// and `accent_text` are derived from it so they keep AA contrast on this base.
    ///
    /// ```
    /// use egui_workbench::theme::{Theme, contrast_ratio};
    /// use egui_workbench::egui::Color32;
    /// let brand = Theme::dark().with_accent(Color32::from_rgb(0xe0, 0x6c, 0x2b));
    /// assert!(contrast_ratio(brand.link, brand.background) >= 4.5);
    /// assert!(contrast_ratio(brand.accent_text, brand.accent) >= 4.5);
    /// ```
    pub fn with_accent(mut self, accent: Color32) -> Self {
        let towards = if self.is_dark() { Color32::WHITE } else { Color32::BLACK };
        self.accent = accent;
        self.focus = until_contrast(accent, self.background, towards, 3.0);
        self.link = until_contrast(accent, self.background, towards, 4.5);
        self.selection = mix(self.background, accent, if self.is_dark() { 0.35 } else { 0.28 });
        // White or black text, whichever reads better on the accent.
        self.accent_text = if contrast_ratio(Color32::WHITE, accent) >= contrast_ratio(Color32::BLACK, accent) {
            Color32::WHITE
        } else {
            Color32::BLACK
        };
        if contrast_ratio(self.accent_text, self.accent) < 4.5 {
            // A mid-tone accent: darken (or lighten) it until its text passes.
            let away = if self.accent_text == Color32::WHITE {
                Color32::BLACK
            } else {
                Color32::WHITE
            };
            self.accent = until_contrast(self.accent, self.accent_text, away, 4.5);
        }
        self
    }

    /// A high-contrast variant of this theme: pure black or white surfaces, maximum text
    /// contrast, stronger borders, and the brand accent kept readable.
    /// The pairs that miss WCAG AA (4.5:1 for text, 3:1 for the focus ring), with
    /// their ratio. Empty for every built-in theme; check your own themes with it.
    pub fn accessibility_issues(&self) -> Vec<(&'static str, f32)> {
        let mut issues = Vec::new();
        for (label, foreground, background, minimum) in [
            ("text on background", self.text, self.background, 4.5),
            ("text on surface", self.text, self.surface, 4.5),
            ("muted text on background", self.text_muted, self.background, 4.5),
            ("link on background", self.link, self.background, 4.5),
            ("accent text on accent", self.accent_text, self.accent, 4.5),
            ("danger on background", self.danger, self.background, 4.5),
            ("focus on background", self.focus, self.background, 3.0),
        ] {
            let ratio = contrast_ratio(foreground, background);
            if ratio < minimum {
                issues.push((label, ratio));
            }
        }
        issues
    }

    /// The 1 px border stroke.
    pub fn border_stroke(&self) -> Stroke {
        Stroke::new(1.0, self.border)
    }

    /// The corner radius of inputs, buttons, and cards in the default style.
    #[deprecated(note = "radii belong to the style: use `style::get(ctx).control()`")]
    pub fn radius(&self) -> CornerRadius {
        CornerRadius::same(6) // style: fixed
    }
}

/// `a` blended towards `b` by `t` (0 = `a`, 1 = `b`), in sRGB.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let channel = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Color32::from_rgb(channel(a.r(), b.r()), channel(a.g(), b.g()), channel(a.b(), b.b()))
}

/// `color` moved step by step towards `towards` until it reaches `minimum` contrast
/// against `background` (or becomes `towards`).
fn until_contrast(color: Color32, background: Color32, towards: Color32, minimum: f32) -> Color32 {
    let mut candidate = color;
    for step in 0..=20 {
        candidate = mix(color, towards, step as f32 / 20.0);
        if contrast_ratio(candidate, background) >= minimum {
            break;
        }
    }
    candidate
}

/// Relative luminance of an sRGB color (WCAG 2.2).
pub fn relative_luminance(color: Color32) -> f32 {
    let channel = |value: u8| {
        let value = f32::from(value) / 255.0;
        if value <= 0.040_45 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
}

/// White or `dark`, whichever reads better on `fill` (for text on colored buttons).
pub fn readable_on(fill: Color32, dark: Color32) -> Color32 {
    if contrast_ratio(Color32::WHITE, fill) >= contrast_ratio(dark, fill) {
        Color32::WHITE
    } else {
        dark
    }
}

/// Contrast ratio between two opaque colors, from 1.0 to 21.0.
pub fn contrast_ratio(a: Color32, b: Color32) -> f32 {
    let (first, second) = (relative_luminance(a), relative_luminance(b));
    let (light, dark) = if first > second {
        (first, second)
    } else {
        (second, first)
    };
    (light + 0.05) / (dark + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_has_consistent_darkness() {
        for id in ThemeId::ALL {
            assert_eq!(Theme::for_id(id).is_dark(), id.mode() == ThemeMode::Dark, "{id:?}");
        }
    }

    #[test]
    fn contrast_meets_aa_for_primary_pairs() {
        for id in ThemeId::ALL {
            let theme = Theme::for_id(id);
            for (foreground, background, label) in [
                (theme.text, theme.background, "text/background"),
                (theme.text, theme.surface, "text/surface"),
                (theme.text_muted, theme.background, "muted/background"),
                (theme.link, theme.background, "link/background"),
                (theme.accent_text, theme.accent, "accent text/accent"),
                (theme.danger, theme.background, "danger/background"),
            ] {
                let ratio = contrast_ratio(foreground, background);
                assert!(ratio >= 4.5, "{id:?} {label}: {ratio:.2}");
            }
            assert!(contrast_ratio(theme.focus, theme.background) >= 3.0, "{id:?} focus");
        }
    }

    #[test]
    fn destructive_text_meets_aa_in_every_theme() {
        for id in ThemeId::ALL {
            let theme = Theme::for_id(id);
            let text = readable_on(theme.danger, theme.background);
            let ratio = contrast_ratio(text, theme.danger);
            assert!(ratio >= 3.0, "{id:?} destructive: {ratio:.2}");
        }
    }

    #[test]
    fn built_in_themes_have_no_accessibility_issues() {
        for id in ThemeId::ALL {
            assert_eq!(Theme::for_id(id).accessibility_issues(), Vec::new(), "{id:?}");
        }
    }

    #[test]
    fn accents_keep_aa_contrast_on_every_base() {
        let accents = [
            Color32::from_rgb(0xff, 0xd7, 0x00),
            Color32::from_rgb(0x00, 0x7a, 0xcc),
            Color32::from_rgb(0xe0, 0x6c, 0x2b),
            Color32::from_rgb(0x8a, 0x2b, 0xe2),
            Color32::from_rgb(0x20, 0xc0, 0x60),
        ];
        for id in ThemeId::ALL {
            for accent in accents {
                let theme = Theme::for_id(id).with_accent(accent);
                assert_eq!(theme.accessibility_issues(), Vec::new(), "{id:?} {accent:?}");
            }
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn vscode_themes_import_and_export() {
        let text = r##"{
            "colors": {
                "editor.background": "#1e1e1e",
                "foreground": "#d4d4d4",
                "focusBorder": "#007acc",
                "list.activeSelectionBackground": "#264f78",
                "descriptionForeground": "#9d9d9d"
            }
        }"##;
        let theme = Theme::from_vscode_json(text).expect("import");
        assert_eq!(theme.background, Color32::from_rgb(0x1e, 0x1e, 0x1e));
        assert_eq!(theme.text, Color32::from_rgb(0xd4, 0xd4, 0xd4));
        assert!(theme.accessibility_issues().is_empty());
        let css = theme.to_css_variables();
        assert!(css.contains("--wb-background: #1e1e1e;"));
        assert_eq!(theme.to_json()["background"], "#1e1e1e");
        assert!(Theme::from_vscode_json("{\"colors\":{}}").is_err());
    }

    #[test]
    fn high_contrast_keeps_aa() {
        for theme in [Theme::dark().high_contrast(), Theme::light().high_contrast()] {
            assert!(theme.accessibility_issues().is_empty());
            assert_eq!(
                theme.background,
                if theme.is_dark() {
                    Color32::BLACK
                } else {
                    Color32::WHITE
                }
            );
        }
    }

    #[test]
    fn black_and_white_are_twenty_one_to_one() {
        assert!((contrast_ratio(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01);
    }

    #[test]
    fn modes_round_trip() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            assert_eq!(ThemeMode::from_setting(mode.as_setting()), mode);
        }
    }
}
