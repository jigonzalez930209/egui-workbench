//! Theme Studio helpers: VS Code import, JSON/CSS export, and contrast adjustment.

use egui::Color32;

#[cfg(feature = "serde")]
use super::mix;
use super::{Theme, contrast_ratio, until_contrast};

impl Theme {
    /// A high-contrast variant of this theme: pure black or white surfaces, maximum text
    /// contrast, stronger borders, and a readable brand accent.
    pub fn high_contrast(mut self) -> Self {
        let dark = self.is_dark();
        self.background = if dark { Color32::BLACK } else { Color32::WHITE };
        self.surface = if dark {
            Color32::from_rgb(0x0a, 0x0a, 0x0a)
        } else {
            Color32::from_rgb(0xf5, 0xf5, 0xf5)
        };
        self.surface_raised = if dark {
            Color32::from_rgb(0x14, 0x14, 0x14)
        } else {
            Color32::WHITE
        };
        self.sunken = if dark {
            Color32::from_rgb(0x18, 0x18, 0x18)
        } else {
            Color32::from_rgb(0xea, 0xea, 0xea)
        };
        self.text = if dark { Color32::WHITE } else { Color32::BLACK };
        self.text_muted = if dark {
            Color32::from_rgb(0xd0, 0xd0, 0xd0)
        } else {
            Color32::from_rgb(0x30, 0x30, 0x30)
        };
        self.border = self.text;
        self.focus = if dark {
            Color32::from_rgb(0xff, 0xd7, 0x00)
        } else {
            Color32::from_rgb(0x00, 0x40, 0x90)
        };
        self.scrollbar = self.text_muted;
        self.ensure_aa()
    }

    /// Adjust text and accent pairs until they reach WCAG AA (4.5:1 for text, 3:1 for
    /// large text and controls), preserving the theme's light/dark polarity.
    pub fn ensure_aa(mut self) -> Self {
        let towards = if self.is_dark() { Color32::WHITE } else { Color32::BLACK };
        if contrast_ratio(self.text, self.background) < 4.5 {
            self.text = until_contrast(self.text, self.background, towards, 4.5);
        }
        if contrast_ratio(self.text_muted, self.background) < 4.5 {
            self.text_muted = until_contrast(self.text_muted, self.background, towards, 4.5);
        }
        if contrast_ratio(self.text, self.surface) < 4.5 {
            self.surface = until_contrast(self.surface, self.text, towards, 4.5);
        }
        if contrast_ratio(self.accent_text, self.accent) < 4.5 {
            self = self.with_accent(self.accent);
        }
        if contrast_ratio(self.link, self.background) < 4.5 {
            self.link = until_contrast(self.link, self.background, towards, 4.5);
        }
        if contrast_ratio(self.focus, self.background) < 3.0 {
            self.focus = until_contrast(self.focus, self.background, towards, 3.0);
        }
        self
    }

    /// The theme as `#rrggbb` CSS custom properties.
    pub fn to_css_variables(&self) -> String {
        let mut css = String::from(":root {\n");
        for (name, color) in [
            ("background", self.background),
            ("surface", self.surface),
            ("surface-raised", self.surface_raised),
            ("sunken", self.sunken),
            ("text", self.text),
            ("text-muted", self.text_muted),
            ("accent", self.accent),
            ("accent-text", self.accent_text),
            ("border", self.border),
            ("selection", self.selection),
            ("hover", self.hover),
            ("danger", self.danger),
            ("warning", self.warning),
            ("success", self.success),
            ("done", self.done),
            ("link", self.link),
            ("scrollbar", self.scrollbar),
            ("focus", self.focus),
        ] {
            css.push_str(&format!("  --wb-{name}: {};\n", hex(color)));
        }
        css.push_str("}\n");
        css
    }

    /// Import a VS Code color theme (`colors` object); syntax token colors are accepted
    /// but do not replace the workbench tokens.
    ///
    /// # Errors
    /// When the JSON is malformed or lacks `editor.background` and `foreground`.
    #[cfg(feature = "serde")]
    pub fn from_vscode_json(text: &str) -> Result<Self, ThemeImportError> {
        let value: serde_json::Value =
            serde_json::from_str(text).map_err(|error| ThemeImportError::Json(error.to_string()))?;
        let colors = value
            .get("colors")
            .and_then(serde_json::Value::as_object)
            .ok_or(ThemeImportError::Missing("colors"))?;
        let get = |key: &str| {
            colors
                .get(key)
                .and_then(serde_json::Value::as_str)
                .and_then(parse_color)
        };
        let background = get("editor.background").ok_or(ThemeImportError::Missing("editor.background"))?;
        let text = get("foreground").ok_or(ThemeImportError::Missing("foreground"))?;
        let accent = get("focusBorder")
            .or_else(|| get("activityBarBadge.background"))
            .or_else(|| get("button.background"))
            .unwrap_or(Color32::from_rgb(0x2f, 0x81, 0xf7));
        let theme = Self {
            background,
            surface: get("sideBar.background").unwrap_or(mix(background, text, 0.06)),
            surface_raised: get("panel.background").unwrap_or(mix(background, text, 0.09)),
            sunken: get("input.background").unwrap_or(mix(background, text, 0.04)),
            text,
            text_muted: get("descriptionForeground").unwrap_or(mix(text, background, 0.35)),
            accent,
            accent_text: get("button.foreground").unwrap_or(Color32::WHITE),
            border: get("panel.border").unwrap_or(mix(background, text, 0.22)),
            selection: get("list.activeSelectionBackground").unwrap_or(mix(background, accent, 0.35)),
            hover: get("list.hoverBackground").unwrap_or(mix(background, text, 0.1)),
            danger: get("errorForeground").unwrap_or(Color32::from_rgb(0xf8, 0x51, 0x49)),
            warning: get("editorWarning.foreground").unwrap_or(Color32::from_rgb(0xd2, 0x9a, 0x00)),
            success: get("terminal.ansiGreen").unwrap_or(Color32::from_rgb(0x3f, 0xb9, 0x50)),
            done: get("terminal.ansiMagenta").unwrap_or(Color32::from_rgb(0xa3, 0x71, 0xf7)),
            link: get("textLink.foreground").unwrap_or(accent),
            scrollbar: get("scrollbarSlider.background").unwrap_or(mix(background, text, 0.3)),
            focus: get("focusBorder").unwrap_or(accent),
        };
        Ok(theme.ensure_aa())
    }

    /// The theme as JSON tokens.
    #[cfg(feature = "serde")]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "background": hex(self.background),
            "surface": hex(self.surface),
            "surface_raised": hex(self.surface_raised),
            "sunken": hex(self.sunken),
            "text": hex(self.text),
            "text_muted": hex(self.text_muted),
            "accent": hex(self.accent),
            "accent_text": hex(self.accent_text),
            "border": hex(self.border),
            "selection": hex(self.selection),
            "hover": hex(self.hover),
            "danger": hex(self.danger),
            "warning": hex(self.warning),
            "success": hex(self.success),
            "done": hex(self.done),
            "link": hex(self.link),
            "scrollbar": hex(self.scrollbar),
            "focus": hex(self.focus),
        })
    }
}

fn hex(color: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}

#[cfg(feature = "serde")]
fn parse_color(text: &str) -> Option<Color32> {
    let text = text.trim().trim_start_matches('#');
    if text.len() == 8 {
        return Some(Color32::from_rgba_unmultiplied(
            u8::from_str_radix(&text[0..2], 16).ok()?,
            u8::from_str_radix(&text[2..4], 16).ok()?,
            u8::from_str_radix(&text[4..6], 16).ok()?,
            u8::from_str_radix(&text[6..8], 16).ok()?,
        ));
    }
    if text.len() != 6 {
        return None;
    }
    Some(Color32::from_rgb(
        u8::from_str_radix(&text[0..2], 16).ok()?,
        u8::from_str_radix(&text[2..4], 16).ok()?,
        u8::from_str_radix(&text[4..6], 16).ok()?,
    ))
}

/// Why a VS Code theme could not be imported.
#[cfg(feature = "serde")]
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ThemeImportError {
    /// The JSON is malformed.
    Json(String),
    /// A required color key is missing.
    Missing(&'static str),
}

#[cfg(feature = "serde")]
impl std::fmt::Display for ThemeImportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(error) => formatter.write_str(error),
            Self::Missing(key) => write!(formatter, "missing VS Code color `{key}`"),
        }
    }
}

#[cfg(feature = "serde")]
impl std::error::Error for ThemeImportError {}
