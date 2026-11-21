//! Text buttons and icon buttons with shared variants and sizes.

use egui::{Color32, CornerRadius, Response, RichText, Role, Sense, Stroke, Ui, Vec2, WidgetInfo};

use crate::Icons;
use crate::close_glyph;
use crate::components::style::{ButtonVariant, IconButtonSize, Size};
use crate::components::{Component, IconComponent};

/// A text button.
///
/// ```
/// # use egui_workbench::components::{Button, ButtonVariant, prelude::*};
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// if Button::new("Delete").variant(ButtonVariant::Destructive).show(ui).clicked() {
///     // The click is the event; the caller owns the action.
/// }
/// # });
/// # output.textures_delta.clear();
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Button<'a> {
    label: &'a str,
    variant: ButtonVariant,
    size: Size,
    enabled: bool,
    selected: bool,
    width: Option<f32>,
    tooltip: Option<&'a str>,
    fill: Option<Color32>,
    text_color: Option<Color32>,
    stroke: Option<Stroke>,
    radius: Option<u8>,
}

impl<'a> Button<'a> {
    /// A primary button at the default size.
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            variant: ButtonVariant::Primary,
            size: Size::Md,
            enabled: true,
            selected: false,
            width: None,
            tooltip: None,
            fill: None,
            text_color: None,
            stroke: None,
            radius: None,
        }
    }

    /// Select the visual role.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Select the height and type scale.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Enable or disable the button. Disabled buttons never report clicks.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Paint the button as a selected toggle.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Minimum width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Tooltip and accessible description.
    pub fn tooltip(mut self, tooltip: &'a str) -> Self {
        self.tooltip = Some(tooltip);
        self
    }

    /// Override the background (the variant's fill otherwise).
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Override the label color.
    pub fn text_color(mut self, color: Color32) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Override the border (`Stroke::NONE` removes it).
    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    /// Override the corner radius (the style's `control_radius` otherwise).
    pub fn radius(mut self, radius: u8) -> Self {
        self.radius = Some(radius);
        self
    }

    /// The visible label.
    pub fn label(&self) -> &'a str {
        self.label
    }
}

impl Component for Button<'_> {
    type Output = Response;

    fn show(self, ui: &mut Ui) -> Response {
        let visuals = ui.visuals().clone();
        let (fill, stroke, text_color) = match self.variant {
            ButtonVariant::Primary => (visuals.selection.bg_fill, Stroke::NONE, visuals.selection.stroke.color),
            ButtonVariant::Secondary => (
                visuals.widgets.inactive.weak_bg_fill,
                Stroke::new(1.0, visuals.widgets.inactive.bg_stroke.color),
                visuals.text_color(),
            ),
            ButtonVariant::Outline => (
                Color32::TRANSPARENT,
                Stroke::new(1.0, visuals.widgets.inactive.bg_stroke.color),
                visuals.text_color(),
            ),
            ButtonVariant::Ghost => (Color32::TRANSPARENT, Stroke::NONE, visuals.text_color()),
            ButtonVariant::Destructive => {
                // Light danger tokens (dark themes) need dark text to stay readable.
                let fill = visuals.error_fg_color;
                (fill, Stroke::NONE, crate::theme::readable_on(fill, visuals.panel_fill))
            }
            ButtonVariant::Link => (Color32::TRANSPARENT, Stroke::NONE, visuals.hyperlink_color),
        };
        let (fill, text_color) = if self.selected {
            (visuals.selection.bg_fill, visuals.selection.stroke.color)
        } else {
            (fill, text_color)
        };
        let style = crate::style::get(ui.ctx());
        let fill = self.fill.unwrap_or(fill);
        let text_color = self.text_color.unwrap_or(text_color);
        // The outline is the Outline variant's identity: it keeps at least 1 pt even in
        // a borderless style.
        let border = if self.variant == ButtonVariant::Outline {
            style.border_width.max(1.0)
        } else {
            style.border_width
        };
        let stroke = self.stroke.unwrap_or(if stroke.width > 0.0 {
            Stroke::new(border, stroke.color)
        } else {
            stroke
        });
        let radius = self.radius.map_or_else(|| style.control(), CornerRadius::same);
        let width = self.width.unwrap_or_else(|| self.size.min_width());
        let rich = RichText::new(self.label).size(style.text(self.size)).color(text_color);
        if self.variant == ButtonVariant::Link {
            let mut response = ui.add_enabled(self.enabled, egui::Link::new(rich));
            if let Some(tooltip) = self.tooltip {
                response = response.on_hover_text(tooltip);
            }
            return response;
        }
        let padding = style.size_padding(self.size);
        let height = style.size_height(self.size);
        let mut button = egui::Button::new(rich)
            .wrap_mode(egui::TextWrapMode::Extend)
            .stroke(stroke)
            .corner_radius(radius)
            .min_size(Vec2::new(width, height));
        if self.variant == ButtonVariant::Ghost && !self.selected && self.fill.is_none() {
            // Keep the padding of a framed button, but show the frame (the hover fill)
            // only while hovered or pressed.
            button = button.frame_when_inactive(false);
        } else {
            button = button.fill(fill);
        }
        // egui aligns a button's label by the parent layout, so in a top-aligned row the
        // text sat 3 pt high. Measure the button, reserve its rectangle with a plain
        // allocation (which still takes part in `horizontal_wrapped`), and place it with
        // `put`, whose layout is centered. A child scope would break wrapping.
        let text_width = ui
            .painter()
            .layout_no_wrap(self.label.to_owned(), style.font(self.size.text()), text_color)
            .size()
            .x;
        // egui's frame adds the stroke outside the padding.
        let size = Vec2::new(
            (text_width + 2.0 * (padding.x + stroke.width)).ceil().max(width),
            height,
        );
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let previous = ui.spacing().button_padding;
        ui.spacing_mut().button_padding = padding;
        let enabled = self.enabled;
        let mut response = ui.put(rect, |ui: &mut Ui| ui.add_enabled(enabled, button));
        ui.spacing_mut().button_padding = previous;
        if self.selected {
            response.widget_info(|| WidgetInfo::selected(Role::Button, self.enabled, true, self.label));
        }
        if let Some(tooltip) = self.tooltip {
            response = response.on_hover_text(tooltip);
        }
        response
    }
}

/// A square icon button.
///
/// ```
/// # use egui_workbench::components::{IconButton, prelude::*};
/// # let ctx = egui::Context::default();
/// # let mut icons = egui_workbench::Icons::new();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// if IconButton::new("save", "Save").show(ui, &mut icons).clicked() {
///     // Save.
/// }
/// # });
/// # output.textures_delta.clear();
/// ```
#[derive(Clone, Copy, Debug)]
pub struct IconButton<'a> {
    icon: &'a str,
    tooltip: &'a str,
    size: IconButtonSize,
    active: bool,
    open: bool,
    enabled: bool,
    color: Option<Color32>,
    radius: Option<u8>,
}

impl<'a> IconButton<'a> {
    /// An action-size button.
    pub fn new(icon: &'a str, tooltip: &'a str) -> Self {
        Self {
            icon,
            tooltip,
            size: IconButtonSize::Action,
            active: false,
            open: false,
            enabled: true,
            color: None,
            radius: None,
        }
    }

    /// Select the square size.
    pub fn size(mut self, size: IconButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Paint the toggled state (hover fill plus the title-bar underline for actions).
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Keep a popup button lit while its popup is open.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Disable the button.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Tint the glyph (the text color otherwise).
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Override the hover-fill radius.
    pub fn radius(mut self, radius: u8) -> Self {
        self.radius = Some(radius);
        self
    }
}

impl IconComponent for IconButton<'_> {
    type Output = Response;

    fn show(self, ui: &mut Ui, icons: &mut Icons) -> Response {
        let side = self.size.side();
        let sense = if self.enabled { Sense::click() } else { Sense::hover() };
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), sense);
        response
            .widget_info(|| WidgetInfo::selected(Role::Button, self.enabled, self.active || self.open, self.tooltip));
        let lit = self.active || self.open;
        let fill = if !self.enabled {
            Color32::TRANSPARENT
        } else if response.is_pointer_button_down_on() {
            ui.visuals().widgets.active.weak_bg_fill
        } else if lit || response.hovered() {
            ui.visuals().widgets.hovered.weak_bg_fill
        } else {
            Color32::TRANSPARENT
        };
        // The style's item radius, one point tighter on the small size (5/4 by default).
        let item = crate::style::get(ui.ctx()).item_radius;
        let radius = self.radius.unwrap_or(match self.size {
            IconButtonSize::Small => item.saturating_sub(1),
            IconButtonSize::Action | IconButtonSize::Tool => item,
        });
        ui.painter().rect_filled(rect, CornerRadius::same(radius), fill);
        if self.active && self.size == IconButtonSize::Action {
            let underline = egui::Rect::from_min_max(
                egui::pos2(rect.left() + 7.0, rect.bottom() - 2.0),
                egui::pos2(rect.right() - 7.0, rect.bottom()),
            );
            ui.painter().rect_filled(
                underline,
                crate::style::get(ui.ctx()).mark(),
                ui.visuals().selection.bg_fill,
            );
        }
        let color = if self.enabled {
            self.color.unwrap_or_else(|| ui.visuals().text_color())
        } else {
            ui.visuals().weak_text_color().gamma_multiply(0.45)
        };
        icons.paint(ui, self.icon, rect.center(), self.size.glyph(), color);
        response.on_hover_text(self.tooltip)
    }
}

/// A borderless `×` close button for dialog headers.
pub struct CloseButton<'a> {
    tooltip: &'a str,
    size: f32,
}

impl<'a> CloseButton<'a> {
    /// A close glyph with a tooltip.
    pub fn new(tooltip: &'a str) -> Self {
        Self { tooltip, size: 14.0 }
    }

    /// Glyph size in points.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl Component for CloseButton<'_> {
    type Output = bool;

    fn show(self, ui: &mut Ui) -> bool {
        close_glyph(ui, self.size, self.tooltip)
    }
}
