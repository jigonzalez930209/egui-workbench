//! Application-wide shape, density, and type scale: control heights, row heights,
//! corner radii, border width, and text size.
//!
//! Colors come from the [`crate::theme`]; this module holds everything else that makes
//! the workbench look compact, roomy, sharp, or round. Install a [`WorkbenchStyle`]
//! once with [`set`] (it survives theme changes) and every component reads it with
//! [`get`]. The default reproduces the shell exactly, pixel for pixel.
//!
//! ```
//! use egui_workbench::style::{self, Density, WorkbenchStyle};
//! # let ctx = egui_workbench::egui::Context::default();
//! style::set(&ctx, WorkbenchStyle {
//!     density: Density::Compact,
//!     control_radius: 3,
//!     ..WorkbenchStyle::default()
//! });
//! assert_eq!(style::get(&ctx).control_height(), 26.0);
//! ```
//!
//! The window chrome (title bar, tab strip, status bar) keeps its fixed heights from
//! [`crate::metrics`]: it matches the platform's window decorations and the hit areas
//! of the window controls. Its radii, borders, and text still follow the style.
//!
//! Components never hard-code a radius, border width, or font size: they ask the
//! style ([`WorkbenchStyle::item`], [`WorkbenchStyle::border`],
//! [`WorkbenchStyle::font`], …). `cargo xtask check` rejects literal radii and font
//! sizes outside this module, so a custom style reaches every pixel.

use egui::{Color32, Context, CornerRadius, FontId, Id, Stroke, TextStyle, Vec2};

use crate::components::Size;
use crate::foundation::metrics;

/// How much vertical room controls and rows take.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Density {
    /// About 15 % tighter: data-heavy tools.
    Compact,
    /// The reference sizes in [`crate::metrics`].
    #[default]
    Default,
    /// About 15 % roomier: touch screens and presentation.
    Comfortable,
}

impl Density {
    /// The factor applied to heights and vertical padding.
    pub const fn factor(self) -> f32 {
        match self {
            Self::Compact => 0.86,
            Self::Default => 1.0,
            Self::Comfortable => 1.16,
        }
    }

    /// Every density, from tight to roomy.
    pub const ALL: [Self; 3] = [Self::Compact, Self::Default, Self::Comfortable];

    /// A stable value for settings files.
    pub const fn as_setting(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Default => "default",
            Self::Comfortable => "comfortable",
        }
    }

    /// Parse [`Self::as_setting`]; unknown values give [`Density::Default`].
    pub fn from_setting(value: &str) -> Self {
        match value {
            "compact" => Self::Compact,
            "comfortable" => Self::Comfortable,
            _ => Self::Default,
        }
    }
}

/// Shape and density shared by every component.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkbenchStyle {
    /// Control and row heights.
    pub density: Density,
    /// Buttons, text fields, selects, steppers, context menus, and egui widgets.
    pub control_radius: u8,
    /// Hover fills of icon buttons, nav items, and segmented controls.
    pub item_radius: u8,
    /// Small shapes inside other shapes: menu and tree rows, checkboxes, segments,
    /// toolbar buttons. Tiny hover targets use one point less ([`Self::tight`]).
    pub inner_radius: u8,
    /// Cards, popovers, legends, and egui windows.
    pub container_radius: u8,
    /// Modal dialogs and toasts.
    pub dialog_radius: u8,
    /// Capsules: badges, switches, and the title-bar search. Clamped to half the
    /// shape's height, so the default draws full capsules at any height.
    pub pill_radius: u8,
    /// Borders of controls, cards, popovers, and dialogs (`0.0` removes them).
    /// Separators and focus rings keep their own widths.
    pub border_width: f32,
    /// Multiplies every font size (body, buttons, menus, chrome). Clamped to
    /// `0.75..=1.5`; heights follow the density, not the text.
    pub text_scale: f32,
}

impl Default for WorkbenchStyle {
    fn default() -> Self {
        Self {
            density: Density::Default,
            control_radius: 6,
            item_radius: 5,
            inner_radius: 4,
            container_radius: 8,
            dialog_radius: 10,
            pill_radius: 12,
            border_width: 1.0,
            text_scale: 1.0,
        }
    }
}

impl WorkbenchStyle {
    /// Square corners everywhere, for a flat or "technical" look.
    pub fn sharp() -> Self {
        Self {
            control_radius: 0,
            item_radius: 0,
            inner_radius: 0,
            container_radius: 0,
            dialog_radius: 0,
            pill_radius: 2,
            ..Self::default()
        }
    }

    /// Larger radii for a softer look.
    pub fn rounded() -> Self {
        Self {
            control_radius: 9,
            item_radius: 7,
            inner_radius: 5,
            container_radius: 12,
            dialog_radius: 14,
            pill_radius: u8::MAX,
            ..Self::default()
        }
    }

    /// The same style with another density.
    pub fn with_density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }

    /// A reference height scaled by the density, on whole points.
    pub fn scaled(&self, height: f32) -> f32 {
        (height * self.density.factor()).round()
    }

    /// Height of text fields, selects, steppers, and dialog buttons.
    pub fn control_height(&self) -> f32 {
        self.scaled(metrics::CONTROL_HEIGHT)
    }

    /// Height of a builder button of `size`.
    pub fn size_height(&self, size: Size) -> f32 {
        self.scaled(size.height())
    }

    /// Button padding of `size`, vertical part scaled by the density.
    pub fn size_padding(&self, size: Size) -> Vec2 {
        let padding = size.padding();
        Vec2::new(padding.x, (padding.y * self.density.factor()).round())
    }

    /// Height of a context-menu row.
    pub fn menu_row_height(&self) -> f32 {
        self.scaled(metrics::MENU_ROW_HEIGHT)
    }

    /// Height of a palette or keyboard-menu row.
    pub fn list_row_height(&self) -> f32 {
        self.scaled(metrics::LIST_ROW_HEIGHT)
    }

    /// Height of an explorer tree row.
    pub fn tree_row_height(&self) -> f32 {
        self.scaled(metrics::TREE_ROW_HEIGHT)
    }

    /// Corner radius of controls.
    pub fn control(&self) -> CornerRadius {
        CornerRadius::same(self.control_radius)
    }

    /// Corner radius of row and icon hover fills.
    pub fn item(&self) -> CornerRadius {
        CornerRadius::same(self.item_radius)
    }

    /// Corner radius of cards, popovers, and menus.
    pub fn container(&self) -> CornerRadius {
        CornerRadius::same(self.container_radius)
    }

    /// Corner radius of dialogs and toasts.
    pub fn dialog(&self) -> CornerRadius {
        CornerRadius::same(self.dialog_radius)
    }

    /// Corner radius of shapes nested in other shapes (menu rows, checkboxes).
    pub fn inner(&self) -> CornerRadius {
        CornerRadius::same(self.inner_radius)
    }

    /// One point under [`Self::inner`]: tiny hover targets such as a tab's close
    /// button or a stepper arrow.
    pub fn tight(&self) -> CornerRadius {
        CornerRadius::same(self.inner_radius.saturating_sub(1))
    }

    /// Indicator bars and underlines (1 pt, square in a sharp style).
    pub fn mark(&self) -> CornerRadius {
        CornerRadius::same(self.inner_radius.min(1))
    }

    /// Corner radius of a capsule `height` points tall.
    pub fn pill(&self, height: f32) -> CornerRadius {
        let half = (height / 2.0).round().clamp(0.0, 255.0) as u8;
        CornerRadius::same(self.pill_radius.min(half))
    }

    /// A border of the style's width.
    pub fn border(&self, color: Color32) -> Stroke {
        Stroke::new(self.border_width.max(0.0), color)
    }

    /// The effective text scale.
    pub fn text_scale(&self) -> f32 {
        if self.text_scale.is_finite() {
            self.text_scale.clamp(0.75, 1.5)
        } else {
            1.0
        }
    }

    /// A reference font size (in points) scaled by [`Self::text_scale`].
    pub fn font_size(&self, points: f32) -> f32 {
        points * self.text_scale()
    }

    /// The proportional font of a reference size.
    pub fn font(&self, points: f32) -> FontId {
        FontId::proportional(self.font_size(points))
    }

    /// The monospace font of a reference size.
    pub fn mono(&self, points: f32) -> FontId {
        FontId::monospace(self.font_size(points))
    }

    /// The label size of a builder [`Size`].
    pub fn text(&self, size: Size) -> f32 {
        self.font_size(size.text())
    }
}

/// The reference type scale installed into egui's text styles (before
/// [`WorkbenchStyle::text_scale`]).
pub const TYPE_SCALE: [(TextStyle, f32); 5] = [
    (TextStyle::Body, 14.0),
    (TextStyle::Button, 12.5),
    (TextStyle::Small, 11.0),
    (TextStyle::Monospace, 13.0),
    (TextStyle::Heading, 20.0),
];

fn key() -> Id {
    Id::unique("egui_workbench::style")
}

/// The installed style (the default when none was set).
pub fn get(ctx: &Context) -> WorkbenchStyle {
    ctx.data(|data| data.get_temp::<WorkbenchStyle>(key()))
        .unwrap_or_default()
}

/// Install a style and update egui's own widgets (radius, interact height) to match.
/// Themes applied later keep it.
pub fn set(ctx: &Context, style: WorkbenchStyle) {
    ctx.data_mut(|data| data.insert_temp(key(), style));
    apply_to_egui(ctx, &style);
}

/// Copy the shape and type scale into egui's styles; called by [`set`] and after
/// every theme change.
pub(crate) fn apply_to_egui(ctx: &Context, style: &WorkbenchStyle) {
    let radius = style.control();
    let border = style.border_width.max(0.0);
    ctx.all_styles_mut(|egui_style| {
        let widgets = &mut egui_style.visuals.widgets;
        for state in [
            &mut widgets.noninteractive,
            &mut widgets.inactive,
            &mut widgets.hovered,
            &mut widgets.active,
            &mut widgets.open,
        ] {
            state.corner_radius = radius;
        }
        // Borders follow the style; the 2 pt focus ring of the active state stays.
        for state in [&mut widgets.noninteractive, &mut widgets.inactive, &mut widgets.hovered] {
            state.bg_stroke.width = border;
        }
        egui_style.visuals.window_stroke.width = border;
        egui_style.visuals.window_corner_radius = style.container();
        egui_style.visuals.menu_corner_radius = style.control();
        egui_style.spacing.interact_size.y = style.scaled(24.0);
        egui_style.spacing.button_padding.y = style.scaled(3.0);
        for (text_style, points) in TYPE_SCALE {
            let font = if text_style == TextStyle::Monospace {
                style.mono(points)
            } else {
                style.font(points)
            };
            egui_style.text_styles.insert(text_style, font);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_style_reproduces_the_metrics() {
        let style = WorkbenchStyle::default();
        assert_eq!(style.control_height(), metrics::CONTROL_HEIGHT);
        assert_eq!(style.menu_row_height(), metrics::MENU_ROW_HEIGHT);
        assert_eq!(style.list_row_height(), metrics::LIST_ROW_HEIGHT);
        assert_eq!(style.tree_row_height(), metrics::TREE_ROW_HEIGHT);
        for size in [Size::Sm, Size::Md, Size::Lg] {
            assert_eq!(style.size_height(size), size.height());
            assert_eq!(style.size_padding(size), size.padding());
        }
    }

    #[test]
    fn capsules_clamp_to_half_the_height() {
        let style = WorkbenchStyle::default();
        assert_eq!(style.pill(20.0), CornerRadius::same(10));
        assert_eq!(style.pill(24.0), CornerRadius::same(12));
        assert_eq!(WorkbenchStyle::sharp().pill(24.0), CornerRadius::same(2));
        assert_eq!(WorkbenchStyle::rounded().pill(30.0), CornerRadius::same(15));
    }

    #[test]
    fn the_text_scale_reaches_egui_and_is_clamped() {
        let ctx = Context::default();
        set(
            &ctx,
            WorkbenchStyle {
                text_scale: 1.2,
                border_width: 0.0,
                ..WorkbenchStyle::default()
            },
        );
        let style = ctx.style_of(egui::Theme::Light);
        assert_eq!(style.text_styles[&TextStyle::Body].size, 14.0 * 1.2);
        assert_eq!(style.visuals.widgets.inactive.bg_stroke.width, 0.0);
        let huge = WorkbenchStyle {
            text_scale: 9.0,
            ..WorkbenchStyle::default()
        };
        assert_eq!(huge.font_size(10.0), 15.0);
    }

    #[test]
    fn densities_order_the_heights() {
        let heights: Vec<f32> = Density::ALL
            .iter()
            .map(|density| WorkbenchStyle::default().with_density(*density).control_height())
            .collect();
        assert!(heights[0] < heights[1] && heights[1] < heights[2], "{heights:?}");
        for density in Density::ALL {
            assert_eq!(Density::from_setting(density.as_setting()), density);
        }
    }

    #[test]
    fn the_style_survives_in_the_context_and_reaches_egui() {
        let ctx = Context::default();
        set(&ctx, WorkbenchStyle::sharp());
        assert_eq!(get(&ctx).control_radius, 0);
        assert_eq!(
            ctx.style_of(egui::Theme::Light).visuals.widgets.inactive.corner_radius,
            CornerRadius::ZERO
        );
    }
}
