//! The component board, and the style and accent playgrounds built on it.

use egui_workbench::components::prelude::*;
use egui_workbench::egui::{self, Color32};
use egui_workbench::style::{self, Density, WorkbenchStyle};
use egui_workbench::{Theme, combo};

use super::{Host, Scene};

/// Every component family on one page.
pub struct Board {
    text: String,
    area: String,
    toggle: bool,
    check: bool,
    mode: usize,
    value: f32,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            text: "Workbench".to_owned(),
            area: "Multi-line notes".to_owned(),
            toggle: true,
            check: true,
            mode: 1,
            value: 1.0,
        }
    }
}

impl Board {
    /// Draw the board; actions are reported through `host`.
    pub fn contents(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        ui.horizontal_wrapped(|ui| {
            for (label, variant) in [
                ("Primary", ButtonVariant::Primary),
                ("Secondary", ButtonVariant::Secondary),
                ("Outline", ButtonVariant::Outline),
                ("Ghost", ButtonVariant::Ghost),
                ("Destructive", ButtonVariant::Destructive),
                ("Link", ButtonVariant::Link),
            ] {
                if Button::new(label).variant(variant).show(ui).clicked() {
                    host.report(format!("{label} clicked"));
                }
            }
        });
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            Button::new("Disabled").enabled(false).show(ui);
            for (icon, tooltip, size) in [
                ("settings", "Settings", IconButtonSize::Action),
                ("plus", "New", IconButtonSize::Tool),
                ("refresh", "Refresh", IconButtonSize::Small),
            ] {
                if IconButton::new(icon, tooltip).size(size).show(ui, host.icons).clicked() {
                    host.report(format!("{tooltip} clicked"));
                }
            }
        });
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            ui.vertical(|ui| {
                TextField::new(&mut self.text)
                    .label("Name")
                    .hint("Untitled")
                    .width(240.0)
                    .show(ui);
                ui.add_space(6.0);
                if Switch::new(&mut self.toggle).label("Preview").show(ui) {
                    host.report(format!("Preview: {}", on_off(self.toggle)));
                }
                if Checkbox::new(&mut self.check).label("Word wrap").show(ui) {
                    host.report(format!("Word wrap: {}", on_off(self.check)));
                }
            });
            ui.add_space(16.0);
            ui.vertical(|ui| {
                let options = [
                    (0, "Automatic".to_owned()),
                    (1, "Light".to_owned()),
                    (2, "Dark".to_owned()),
                ];
                if Select::new("Mode", &mut self.mode, &options).show(ui) {
                    host.report(format!("Mode: {}", options[self.mode].1));
                }
                ui.add_space(6.0);
                Stepper::new(&mut self.value, 0.0..=2.0, 0.1)
                    .decimals(1)
                    .suffix("×")
                    .show(ui, host.icons);
                ui.add_space(6.0);
                TextArea::new(&mut self.area)
                    .label("Notes")
                    .rows(2)
                    .width(240.0)
                    .show(ui);
            });
        });
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            Badge::new("Saved").show(ui);
            Badge::new("Accent").tone(Tone::Accent).show(ui);
            Badge::new("Warning").tone(Tone::Warning).show(ui);
            Badge::new("Danger").tone(Tone::Danger).show(ui);
            Badge::new("Success")
                .tone(Tone::Custom(host.theme.tokens().success))
                .dot(true)
                .show(ui);
        });
        ui.add_space(10.0);
        Card::new()
            .title("Card")
            .description("Raised content with the shared border.")
            .width(360.0_f32.min(ui.available_width()))
            .show(ui, |ui| {
                Label::new("Card body").tone(Tone::Muted).show(ui);
                if Button::new("Action")
                    .variant(ButtonVariant::Outline)
                    .size(Size::Sm)
                    .show(ui)
                    .clicked()
                {
                    host.report("Card action");
                }
            });
    }
}

impl Scene for Board {
    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        self.contents(ui, host);
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

/// Corner presets offered by the style playground.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Corners {
    Default,
    Sharp,
    Rounded,
}

/// The board under live `WorkbenchStyle` controls.
#[derive(Default)]
pub struct StyleScene {
    board: Board,
    controls: Option<(Density, Corners, f32, bool)>,
}

impl Scene for StyleScene {
    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        let current = style::get(ui.ctx());
        let (mut density, mut corners, mut text, mut borders) = *self.controls.get_or_insert_with(|| {
            let corners = if current.control_radius == 0 {
                Corners::Sharp
            } else if current.control_radius > WorkbenchStyle::default().control_radius {
                Corners::Rounded
            } else {
                Corners::Default
            };
            (current.density, corners, current.text_scale, current.border_width > 0.0)
        });
        ui.horizontal_wrapped(|ui| {
            let densities: Vec<(Density, String)> = Density::ALL.iter().map(|d| (*d, format!("{d:?}"))).collect();
            combo(ui, "Density", &mut density, &densities);
            let presets = [
                (Corners::Default, "Default corners".to_owned()),
                (Corners::Sharp, "Sharp".to_owned()),
                (Corners::Rounded, "Rounded".to_owned()),
            ];
            combo(ui, "Corners", &mut corners, &presets);
            let sizes = [
                (0.9_f32, "Text 90 %".to_owned()),
                (1.0, "Text 100 %".to_owned()),
                (1.15, "Text 115 %".to_owned()),
                (1.25, "Text 125 %".to_owned()),
            ];
            combo(ui, "Text size", &mut text, &sizes);
            Switch::new(&mut borders).label("Borders").show(ui);
        });
        ui.add_space(4.0);
        Separator::new().show(ui);
        ui.add_space(8.0);
        if self.controls != Some((density, corners, text, borders)) {
            self.controls = Some((density, corners, text, borders));
            let base = match corners {
                Corners::Default => WorkbenchStyle::default(),
                Corners::Sharp => WorkbenchStyle::sharp(),
                Corners::Rounded => WorkbenchStyle::rounded(),
            };
            style::set(
                ui.ctx(),
                WorkbenchStyle {
                    text_scale: text,
                    border_width: if borders { 1.0 } else { 0.0 },
                    ..base.with_density(density)
                },
            );
            host.report(format!(
                "style::set — {density:?}, {corners:?}, text {:.0} %",
                text * 100.0
            ));
        }
        self.board.contents(ui, host);
    }
}

/// The board under a live brand accent.
pub struct AccentScene {
    board: Board,
    accent: Color32,
    enabled: bool,
}

impl Default for AccentScene {
    fn default() -> Self {
        Self {
            board: Board::default(),
            accent: Color32::from_rgb(0xe0, 0x6c, 0x2b),
            enabled: true,
        }
    }
}

const SWATCHES: [(u8, u8, u8); 6] = [
    (0xe0, 0x6c, 0x2b),
    (0x8b, 0x5c, 0xf6),
    (0x10, 0xb9, 0x81),
    (0xec, 0x48, 0x99),
    (0x0e, 0xa5, 0xe9),
    (0xea, 0xb3, 0x08),
];

impl Scene for AccentScene {
    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        if host.theme.accent().is_none() && self.enabled {
            host.theme.set_accent(ui.ctx(), Some(self.accent));
        }
        let before = (self.enabled, self.accent);
        ui.horizontal_wrapped(|ui| {
            Switch::new(&mut self.enabled).label("Brand accent").show(ui);
            for (r, g, b) in SWATCHES {
                let color = Color32::from_rgb(r, g, b);
                let (rect, response) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
                let selected = self.enabled && color == self.accent;
                ui.painter().circle_filled(rect.center(), 9.0, color);
                if selected {
                    ui.painter()
                        .circle_stroke(rect.center(), 10.5, egui::Stroke::new(1.5, ui.visuals().text_color()));
                }
                if response.on_hover_text(format!("#{r:02x}{g:02x}{b:02x}")).clicked() {
                    self.accent = color;
                    self.enabled = true;
                }
            }
            egui::color_picker::color_edit_button_srgba(ui, &mut self.accent, egui::color_picker::Alpha::Opaque);
        });
        if before != (self.enabled, self.accent) {
            let accent = self.enabled.then_some(self.accent);
            host.theme.set_accent(ui.ctx(), accent);
            let issues = host.theme.tokens().accessibility_issues();
            host.report(match accent {
                Some(color) => format!(
                    "set_accent(#{:02x}{:02x}{:02x}) — {} AA issues",
                    color.r(),
                    color.g(),
                    color.b(),
                    issues.len()
                ),
                None => "set_accent(None) — theme accent".to_owned(),
            });
        }
        ui.add_space(4.0);
        let tokens: Theme = host.theme.tokens();
        ui.horizontal_wrapped(|ui| {
            for (name, color) in [
                ("accent", tokens.accent),
                ("focus", tokens.focus),
                ("link", tokens.link),
                ("selection", tokens.selection),
            ] {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, style::get(ui.ctx()).mark(), color);
                Label::new(name).tone(Tone::Muted).size(Size::Sm).show(ui);
                ui.add_space(8.0);
            }
        });
        ui.add_space(4.0);
        Separator::new().show(ui);
        ui.add_space(8.0);
        self.board.contents(ui, host);
    }
}
