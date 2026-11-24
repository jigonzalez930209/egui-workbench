//! A vertical activity bar: one icon per view group, an active marker, and badges.

use egui::{Sense, Ui};

use crate::Icons;
use crate::components::{IconComponent, Size};
use crate::metrics::ICON_MEDIUM;

/// One item in the activity bar.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ActivityBarItem {
    /// Icon name.
    pub icon: String,
    /// Tooltip and accessible name.
    pub label: String,
    /// Optional badge text, such as an error count.
    pub badge: Option<String>,
    /// Whether the item can be selected.
    pub enabled: bool,
}

impl ActivityBarItem {
    /// An enabled item.
    pub fn new(icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            label: label.into(),
            badge: None,
            enabled: true,
        }
    }

    /// Add a badge.
    pub fn badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    /// Disable the item.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// A vertical activity bar. Place it at the far left, above the side bar.
pub struct ActivityBar<'a> {
    items: &'a [ActivityBarItem],
    active: usize,
    width: f32,
}

impl<'a> ActivityBar<'a> {
    /// An activity bar over `items` with `active` selected.
    pub fn new(items: &'a [ActivityBarItem], active: usize) -> Self {
        Self {
            items,
            active,
            width: Size::Md.height() + 8.0,
        }
    }

    /// Width in points; defaults to a 32 pt rail.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(28.0);
        self
    }
}

impl IconComponent for ActivityBar<'_> {
    type Output = Option<usize>;

    fn show(self, ui: &mut Ui, icons: &mut Icons) -> Option<usize> {
        let mut selected = None;
        ui.vertical(|ui| {
            ui.set_width(self.width);
            ui.add_space(6.0);
            for (index, item) in self.items.iter().enumerate() {
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(self.width, Size::Md.height()),
                    if item.enabled { Sense::click() } else { Sense::hover() },
                );
                response.widget_info(|| {
                    egui::WidgetInfo::selected(egui::Role::Tab, item.enabled, index == self.active, &item.label)
                });
                if index == self.active {
                    let bar = egui::Rect::from_min_size(
                        egui::pos2(rect.left() + 2.0, rect.top() + 6.0),
                        egui::vec2(2.0, rect.height() - 12.0),
                    );
                    ui.painter()
                        .rect_filled(bar, crate::style::get(ui.ctx()).mark(), ui.visuals().selection.bg_fill);
                }
                let color = if !item.enabled {
                    ui.visuals().weak_text_color().gamma_multiply(0.4)
                } else if index == self.active || response.hovered() {
                    ui.visuals().text_color()
                } else {
                    ui.visuals().weak_text_color()
                };
                icons.paint(ui, &item.icon, rect.center(), ICON_MEDIUM, color);
                if let Some(badge) = &item.badge {
                    let center = egui::pos2(rect.right() - 7.0, rect.top() + 8.0);
                    ui.painter().circle_filled(center, 6.0, ui.visuals().selection.bg_fill);
                    ui.painter().text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        badge,
                        crate::style::get(ui.ctx()).font(8.5),
                        ui.visuals().selection.stroke.color,
                    );
                }
                if item.enabled && response.clicked() {
                    selected = Some(index);
                }
                response.on_hover_text(&item.label);
            }
        });
        selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::IconComponent;
    use egui::Context;

    #[test]
    fn items_default_to_enabled_and_badges_are_optional() {
        let item = ActivityBarItem::new("folder", "Explorer");
        assert!(item.enabled);
        assert!(item.badge.is_none());
        let item = item.badge("3").enabled(false);
        assert_eq!(item.badge.as_deref(), Some("3"));
        assert!(!item.enabled);
    }

    #[test]
    fn the_bar_renders_every_item() {
        let ctx = Context::default();
        let mut icons = Icons::new();
        let items = [
            ActivityBarItem::new("folder", "Explorer"),
            ActivityBarItem::new("search", "Search"),
            ActivityBarItem::new("settings", "Settings").badge("1"),
        ];
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let _ = ActivityBar::new(&items, 0).show(ui, &mut icons);
        });
        output.textures_delta.clear();
    }
}
