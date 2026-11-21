//! Shared size, variant, and tone vocabulary for the component layer.
//!
//! Every builder in [`super`] uses these values, so a new component does not invent its
//! own scale. Colors still come from [`egui::Visuals`] (and thus from the active theme).

use egui::{Color32, RichText, Ui, Vec2, Visuals};

/// Control height and type scale.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    /// 24 pt high, 11.5 pt text. Dense rows and toolbars.
    Sm,
    /// 30 pt high, 13 pt text. The default control height.
    #[default]
    Md,
    /// 36 pt high, 14 pt text. Dialog and hero actions.
    Lg,
}

impl Size {
    /// Height in points.
    pub const fn height(self) -> f32 {
        match self {
            Self::Sm => 24.0,
            Self::Md => 30.0,
            Self::Lg => 36.0,
        }
    }

    /// Label size in points.
    pub const fn text(self) -> f32 {
        match self {
            Self::Sm => 11.5,
            Self::Md => 13.0,
            Self::Lg => 14.0,
        }
    }

    /// Minimum width for a text button.
    pub const fn min_width(self) -> f32 {
        match self {
            Self::Sm => 0.0,
            Self::Md => 88.0,
            Self::Lg => 104.0,
        }
    }

    /// Horizontal and vertical button padding.
    pub const fn padding(self) -> Vec2 {
        match self {
            Self::Sm => Vec2::new(8.0, 2.0),
            Self::Md => Vec2::new(12.0, 4.0),
            Self::Lg => Vec2::new(16.0, 6.0),
        }
    }
}

/// The visual role of a button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Selection fill, no border, selection text color.
    #[default]
    Primary,
    /// Surface fill with a 1 pt border.
    Secondary,
    /// Transparent fill with a 1 pt border.
    Outline,
    /// Transparent, no border; hover fill only.
    Ghost,
    /// Error fill with white text.
    Destructive,
    /// Hyperlink color with no frame.
    Link,
}

/// The square sizes used by the shell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconButtonSize {
    /// 28 pt title-bar action with a 16 pt glyph.
    #[default]
    Action,
    /// 26 pt tab-strip tool with an 18 pt glyph.
    Tool,
    /// 21 pt explorer-header button with a 16 pt glyph.
    Small,
}

impl IconButtonSize {
    /// Side length in points.
    pub const fn side(self) -> f32 {
        match self {
            Self::Action => 28.0,
            Self::Tool => 26.0,
            Self::Small => 21.0,
        }
    }

    /// Glyph size in points.
    pub const fn glyph(self) -> f32 {
        match self {
            Self::Action => 16.0,
            Self::Tool => 18.0,
            Self::Small => 16.0,
        }
    }

    /// Corner radius in points.
    pub const fn radius(self) -> u8 {
        match self {
            Self::Small => 4,
            Self::Action | Self::Tool => 5,
        }
    }
}

/// Semantic color roles for badges, banners, and labels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    /// Primary text.
    #[default]
    Neutral,
    /// Secondary text.
    Muted,
    /// Selection/brand color.
    Accent,
    /// Warning color.
    Warning,
    /// Error color.
    Danger,
    /// A caller-supplied theme token, such as `theme.success`.
    Custom(Color32),
}

impl Tone {
    /// Foreground color for this tone.
    pub fn text(self, visuals: &Visuals) -> Color32 {
        match self {
            Self::Neutral => visuals.text_color(),
            Self::Muted => visuals.weak_text_color(),
            Self::Accent => visuals.selection.stroke.color,
            Self::Warning => visuals.warn_fg_color,
            Self::Danger => visuals.error_fg_color,
            Self::Custom(color) => color,
        }
    }

    /// Subtle background for this tone.
    pub fn fill(self, visuals: &Visuals) -> Color32 {
        match self {
            Self::Neutral => visuals.extreme_bg_color,
            Self::Muted => visuals.widgets.inactive.weak_bg_fill,
            Self::Accent => visuals.selection.bg_fill,
            Self::Warning => visuals.warn_fg_color.gamma_multiply(0.15),
            Self::Danger => visuals.error_fg_color.gamma_multiply(0.15),
            Self::Custom(color) => color.gamma_multiply(0.15),
        }
    }

    /// A rich text with this tone and optional emphasis.
    pub fn rich(self, ui: &Ui, text: impl Into<String>, size: f32, strong: bool) -> RichText {
        let mut rich = RichText::new(text).size(size).color(self.text(ui.visuals()));
        if strong {
            rich = rich.strong();
        }
        rich
    }
}
