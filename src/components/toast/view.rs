//! Painting one toast card.

use egui::epaint::Shadow;
use egui::{Align, Color32, Frame, Layout, Margin, RichText, Ui, vec2};

use super::{Toast, ToastEvent, ToastId, ToastKind};
use crate::Icons;
use crate::components::Component;
use crate::components::button::Button;
use crate::components::style::{ButtonVariant, Size};
use crate::theme;

/// Cubic ease-out for the enter animation.
pub(super) fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

pub(super) fn paint(
    ui: &mut Ui,
    icons: &mut Icons,
    toast: &Toast,
    tokens: &theme::Theme,
    width: f32,
) -> Option<ToastEvent> {
    let accent = toast.kind.color(tokens);
    let mut event = None;
    Frame::new()
        .fill(tokens.surface_raised)
        .stroke(crate::style::get(ui.ctx()).border(tokens.border))
        .corner_radius(crate::style::get(ui.ctx()).dialog())
        .inner_margin(Margin {
            left: 14,
            right: 10,
            top: 12,
            bottom: 12,
        })
        .shadow(Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(if tokens.is_dark() { 90 } else { 38 }),
        })
        .show(ui, |ui| {
            let inner = width - 24.0;
            ui.set_width(inner);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let (slot, _) = ui.allocate_exact_size(vec2(18.0, 18.0), egui::Sense::hover());
                if toast.kind == ToastKind::Loading {
                    egui::Spinner::new().size(16.0).color(accent).paint_at(ui, slot);
                } else {
                    icons.paint(ui, toast.kind.icon(), slot.center(), 18.0, accent);
                }
                let trailing = if toast.closable { 22.0 } else { 0.0 }
                    + toast
                        .action
                        .as_ref()
                        .map_or(0.0, |label| 24.0 + 7.0 * label.chars().count() as f32);
                let text_width = (inner - 28.0 - trailing).max(80.0);
                ui.vertical(|ui| {
                    ui.set_width(text_width);
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.add(
                        egui::Label::new(
                            RichText::new(&toast.title)
                                .size(crate::style::get(ui.ctx()).font_size(13.0))
                                .strong()
                                .color(tokens.text),
                        )
                        .wrap(),
                    );
                    if let Some(description) = &toast.description {
                        ui.add(
                            egui::Label::new(
                                RichText::new(description)
                                    .size(crate::style::get(ui.ctx()).font_size(12.0))
                                    .color(tokens.text_muted),
                            )
                            .wrap(),
                        );
                    }
                });
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    if toast.closable && crate::close_glyph(ui, 12.0, "Dismiss") {
                        event = Some(ToastEvent::Closed(ToastId(0)));
                    }
                    if let Some(label) = &toast.action
                        && Button::new(label)
                            .size(Size::Sm)
                            .variant(ButtonVariant::Secondary)
                            .show(ui)
                            .clicked()
                    {
                        event = Some(ToastEvent::Action(ToastId(0)));
                    }
                });
            });
        });
    event
}
