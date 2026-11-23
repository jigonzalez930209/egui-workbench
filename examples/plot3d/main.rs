//! A runnable native 3D gallery: a surface, a bar field, and a streaming helix.
//!
//! ```bash
//! cargo run --example plot3d
//! ```
//!
//! Drag to orbit, Shift-drag to pan, use the wheel to zoom, and double-click to fit.
//! Hover a legend row to play its animation. The toolbar exports SVG or PNG.

use egui_workbench::app::ThemeState;
use egui_workbench::components::plot::*;
use egui_workbench::components::prelude::*;
use egui_workbench::egui::{self, RichText};
use egui_workbench::{Icons, ThemeId, TitleBar, apply_window_action, combo, is_maximized, show_title_bar};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    eframe::run_native(
        "3D charts",
        egui_workbench::native_options_sized("3D charts", [1120.0, 780.0], [720.0, 520.0]),
        Box::new(|cc| {
            let theme = ThemeState::new(&cc.egui_ctx, ThemeId::GithubDark, false);
            Ok(Box::new(ThreeDExample::new(theme)))
        }),
    )
}

use egui_workbench::eframe;

const KINDS: [(&str, Kind); 3] = [("Surface", Kind::Surface), ("Bars", Kind::Bars), ("Helix", Kind::Helix)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Surface,
    Bars,
    Helix,
}

struct ThreeDExample {
    icons: Icons,
    theme: ThemeState,
    states: [Plot3DState; 3],
    kind: usize,
    projection: usize,
    animation: usize,
    easing: usize,
    wireframe: bool,
    direction: bool,
    status: String,
}

impl ThreeDExample {
    fn new(theme: ThemeState) -> Self {
        Self {
            icons: Icons::new(),
            theme,
            states: std::array::from_fn(|_| Plot3DState::default()),
            kind: 0,
            projection: 0,
            animation: 0,
            easing: 0,
            wireframe: true,
            direction: true,
            status: "Drag to orbit, Shift-drag to pan, wheel to zoom, double-click to fit.".to_owned(),
        }
    }

    fn config(&self) -> Plot3DConfig {
        let mut config = Plot3DConfig {
            legend: LegendPosition::BottomRight,
            ..Plot3DConfig::default()
        };
        config.legend_hover.animation = LegendAnimation::ALL[self.animation];
        config.legend_hover.easing = AnimationEasing::ALL[self.easing];
        config.projection = if self.projection == 0 {
            Projection::Perspective
        } else {
            Projection::Orthographic
        };
        config.surface.wireframe = self.wireframe;
        config.direction.enabled = self.direction;
        config
    }

    fn plot(&self) -> Plot3D<'static> {
        let plot = Plot3D::new(("native-3d", self.kind))
            .config(self.config())
            .export_button(true)
            .height(1.0);
        match KINDS[self.kind].1 {
            Kind::Surface => plot
                .title("Terrain surface")
                .x_label("Longitude")
                .y_label("Latitude")
                .z_label("Height")
                .series(Series3D::surface_fn(
                    "terrain",
                    -2.0..=2.0,
                    -2.0..=2.0,
                    [42, 42],
                    |x, y| (x * x + y * y).sin() * 0.7 + (x * 1.2).cos() * 0.3,
                )),
            Kind::Bars => {
                let cells: Vec<[f64; 3]> = (0..8)
                    .flat_map(|x| {
                        (0..8).map(move |y| {
                            let (x, y) = (f64::from(x), f64::from(y));
                            [x, y, 1.0 + (x * 0.8 + y * 0.5).sin() * 1.4 + (x + y) * 0.04]
                        })
                    })
                    .collect();
                plot.title("Revenue by region")
                    .x_label("Region")
                    .y_label("Segment")
                    .z_label("Revenue")
                    .series(Series3D::bars("revenue", cells).bar_width(0.62))
            }
            Kind::Helix => {
                let line: Vec<[f64; 3]> = (0..180)
                    .map(|index| {
                        let angle = index as f64 * 0.14;
                        [angle.cos() * 2.0, angle.sin() * 2.0, index as f64 * 0.04]
                    })
                    .collect();
                let marks: Vec<[f64; 3]> = line.iter().copied().step_by(14).collect();
                plot.title("Streaming helix")
                    .x_label("x")
                    .y_label("y")
                    .z_label("z")
                    .legend_animation(LegendAnimation::DashMarch)
                    .series(Series3D::line("path", line).width(2.0))
                    .series(Series3D::scatter("samples", marks).radius(4.0))
            }
        }
    }

    fn save(&mut self, ui: &egui::Ui, plot: &Plot3D<'_>, format: ExportFormat) {
        let options = ExportOptions {
            width: 1280.0,
            height: 720.0,
            ..ExportOptions::default()
        };
        let (name, bytes) = match format {
            ExportFormat::Svg => (
                "chart-3d.svg",
                plot.to_svg(ui.ctx(), &self.states[self.kind], &options).into_bytes(),
            ),
            ExportFormat::Png => (
                "chart-3d.png",
                plot.to_png(ui.ctx(), &self.states[self.kind], &options)
                    .unwrap_or_default(),
            ),
        };
        if bytes.is_empty() {
            self.status = "Could not render the export.".to_owned();
            return;
        }
        let path = std::env::temp_dir().join(name);
        self.status = match std::fs::write(&path, bytes) {
            Ok(()) => format!("Saved {}", path.display()),
            Err(error) => format!("Could not save {}: {error}", path.display()),
        };
    }
}

impl eframe::App for ThreeDExample {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.theme.sync_system(&ctx);
        egui::Panel::top("plot3d_title")
            .exact_size(egui_workbench::metrics::TITLE_HEIGHT)
            .show(ui, |ui| {
                let mut bar = TitleBar::new("3D charts");
                bar.maximized = is_maximized(ui.ctx());
                let result = show_title_bar(
                    ui,
                    &mut self.icons,
                    &bar,
                    |_, _| {},
                    |ui, icons| {
                        if IconButton::new("settings", "Switch light/dark")
                            .show(ui, icons)
                            .clicked()
                        {
                            let next = if self.theme.is_light() {
                                ThemeId::GithubDark
                            } else {
                                ThemeId::GithubLight
                            };
                            self.theme.set_requested(ui.ctx(), next);
                        }
                    },
                );
                if let Some(action) = result.window {
                    apply_window_action(ui.ctx(), action);
                }
            });
        egui::CentralPanel::default().show(ui, |ui| {
            let kinds: Vec<(usize, String)> = KINDS
                .iter()
                .enumerate()
                .map(|(index, (label, _))| (index, (*label).to_owned()))
                .collect();
            let projections = vec![(0, "Perspective".to_owned()), (1, "Orthographic".to_owned())];
            let animations: Vec<(usize, String)> = LegendAnimation::ALL
                .iter()
                .enumerate()
                .map(|(index, animation)| (index, animation.label().to_owned()))
                .collect();
            let easings: Vec<(usize, String)> = AnimationEasing::ALL
                .iter()
                .enumerate()
                .map(|(index, easing)| (index, easing.label().to_owned()))
                .collect();
            ui.horizontal_wrapped(|ui| {
                combo(ui, "Chart", &mut self.kind, &kinds);
                combo(ui, "Projection", &mut self.projection, &projections);
                combo(ui, "Legend animation", &mut self.animation, &animations);
                combo(ui, "Easing", &mut self.easing, &easings);
            });
            ui.horizontal_wrapped(|ui| {
                Switch::new(&mut self.wireframe).label("Wireframe").show(ui);
                Switch::new(&mut self.direction).label("Direction arrow").show(ui);
                if Button::new("Reset camera").show(ui).clicked() {
                    self.states[self.kind].reset_view();
                }
            });
            ui.add_space(4.0);
            let plot = self.plot().height((ui.available_height() - 28.0).max(260.0));
            let mut state = std::mem::take(&mut self.states[self.kind]);
            let response = plot.clone().show(ui, &mut self.icons, &mut state);
            self.states[self.kind] = state;
            if let Some(format) = response.export_requested {
                self.save(ui, &plot, format);
            }
            ui.add_space(2.0);
            ui.horizontal_wrapped(|ui| {
                Badge::new(KINDS[self.kind].0).tone(Tone::Accent).show(ui);
                Label::new(&self.status).tone(Tone::Muted).size(Size::Sm).show(ui);
                ui.label(
                    RichText::new("Hover a legend row to play the effect; the hovered series draws on top.")
                        .small()
                        .weak(),
                );
            });
        });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.theme.clear_color()
    }
}
