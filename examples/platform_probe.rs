//! Manual platform probe for the native shell.
//!
//! It draws the same custom title bar and a grid of buttons, overlays the pointer
//! coordinates, and writes a PNG + JSON report. Run it on each OS/compositor from the
//! manual `platform-pixels` workflow and inspect/compare the artifacts.
//!
//! ```bash
//! cargo run --example platform_probe -- --screenshot /tmp/workbench-probe.png
//! ```

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use egui_workbench::app::ThemeState;
use egui_workbench::egui;
use egui_workbench::{Icons, ThemeId, TitleBar, WindowAction, show_title_bar};

fn main() -> eframe::Result {
    let mut screenshot = None;
    let mut theme = "github-dark".to_owned();
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--screenshot" => screenshot = args.next().map(PathBuf::from),
            "--theme" => theme = args.next().unwrap_or(theme),
            "--help" | "-h" => {
                println!("platform_probe [--screenshot report.png] [--theme github-dark]");
                return Ok(());
            }
            other => {
                eprintln!("unknown argument `{other}`");
                std::process::exit(2);
            }
        }
    }
    let mut options = egui_workbench::native_options_sized("Workbench platform probe", [900.0, 600.0], [640.0, 420.0]);
    #[cfg(feature = "wgpu-renderer")]
    if std::env::var_os("WORKBENCH_WGPU").is_some() {
        options.renderer = eframe::Renderer::Wgpu;
    }
    eframe::run_native(
        "Workbench platform probe",
        options,
        Box::new(move |cc| {
            let theme_state = ThemeState::new(&cc.egui_ctx, ThemeId::from_setting(&theme), false);
            Ok(Box::new(Probe::new(theme_state, screenshot)))
        }),
    )
}

struct Probe {
    theme: ThemeState,
    icons: Icons,
    screenshot: Option<PathBuf>,
    frames: u32,
    finished: Arc<AtomicBool>,
    pointer: Option<egui::Pos2>,
    clicks: Vec<(String, egui::Pos2)>,
    hovered: String,
}

impl Probe {
    fn new(theme: ThemeState, screenshot: Option<PathBuf>) -> Self {
        Self {
            theme,
            icons: Icons::new(),
            screenshot,
            frames: 0,
            finished: Arc::new(AtomicBool::new(false)),
            pointer: None,
            clicks: Vec::new(),
            hovered: String::new(),
        }
    }
}

#[derive(serde::Serialize)]
struct Report {
    os: String,
    arch: String,
    pixels_per_point: f32,
    content_rect: [f32; 4],
    screen_rect: [f32; 4],
    viewport_inner_rect: Option<[f32; 4]>,
    pointer: Option<[f32; 2]>,
    clicks: Vec<(String, [f32; 2])>,
}

fn rect_json(rect: egui::Rect) -> [f32; 4] {
    [rect.left(), rect.top(), rect.width(), rect.height()]
}

fn point_json(point: egui::Pos2) -> [f32; 2] {
    [point.x, point.y]
}

impl eframe::App for Probe {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme.sync_system(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.frames += 1;
        self.pointer = ctx.pointer_latest_pos();
        egui::Panel::top("probe_title").exact_size(34.0).show(ui, |ui| {
            let bar = TitleBar::new("Platform probe");
            let output = show_title_bar(
                ui,
                &mut self.icons,
                &bar,
                |ui, _icons| {
                    for label in ["File", "Edit", "View"] {
                        let _ = ui.button(label);
                    }
                },
                |ui, _icons| {
                    let _ = ui.button("Probe");
                },
            );
            if let Some(WindowAction::Close) = output.window {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
        egui::CentralPanel::default().show(ui, |ui| {
            ui.label(format!(
                "os={} arch={} ppp={:.3} content={:?} pointer={:?}",
                std::env::consts::OS,
                std::env::consts::ARCH,
                ctx.pixels_per_point(),
                ctx.content_rect(),
                self.pointer
            ));
            ui.label("Click the grid: the recorded id and pointer must match the drawn row.");
            ui.separator();
            for row in 0..6 {
                ui.horizontal(|ui| {
                    for column in 0..8 {
                        let id = format!("r{row}c{column}");
                        let response = ui.add_sized([88.0, 28.0], egui::Button::new(&id));
                        if response.hovered() {
                            self.hovered = id.clone();
                        }
                        if response.clicked() {
                            self.clicks
                                .push((id.clone(), response.interact_pointer_pos().unwrap_or_default()));
                            eprintln!("clicked {id} at {:?}", response.interact_pointer_pos());
                        }
                        ui.painter().text(
                            response.rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &id,
                            egui::FontId::monospace(11.0),
                            ui.visuals().text_color(),
                        );
                    }
                });
            }
            if let Some(pointer) = self.pointer {
                ui.painter()
                    .circle_stroke(pointer, 8.0, egui::Stroke::new(2.0, ui.visuals().hyperlink_color));
                ui.painter().line_segment(
                    [
                        egui::pos2(pointer.x - 16.0, pointer.y),
                        egui::pos2(pointer.x + 16.0, pointer.y),
                    ],
                    egui::Stroke::new(1.0, ui.visuals().hyperlink_color),
                );
                ui.painter().line_segment(
                    [
                        egui::pos2(pointer.x, pointer.y - 16.0),
                        egui::pos2(pointer.x, pointer.y + 16.0),
                    ],
                    egui::Stroke::new(1.0, ui.visuals().hyperlink_color),
                );
            }
            ui.label(format!("hovered: {}", self.hovered));
        });

        if self.frames > 20 && self.screenshot.is_some() && !self.finished.load(Ordering::Relaxed) {
            self.finished.store(true, Ordering::Relaxed);
            let finished = Arc::clone(&self.finished);
            let Some(path) = self.screenshot.clone() else {
                return;
            };
            let report = Report {
                os: std::env::consts::OS.to_owned(),
                arch: std::env::consts::ARCH.to_owned(),
                pixels_per_point: ctx.pixels_per_point(),
                content_rect: rect_json(ctx.content_rect()),
                screen_rect: rect_json(ctx.input(|input| input.viewport_rect())),
                viewport_inner_rect: ctx.input(|input| input.viewport().inner_rect.map(rect_json)),
                pointer: self.pointer.map(point_json),
                clicks: self
                    .clicks
                    .iter()
                    .map(|(id, pos)| (id.clone(), point_json(*pos)))
                    .collect(),
            };
            ctx.request_screenshot(move |image| {
                let _ = image::save_buffer(
                    &path,
                    &image
                        .pixels
                        .iter()
                        .flat_map(|color| color.to_array())
                        .collect::<Vec<_>>(),
                    image.size[0] as u32,
                    image.size[1] as u32,
                    image::ColorType::Rgba8,
                );
                let _ = std::fs::write(
                    path.with_extension("json"),
                    serde_json::to_string_pretty(&report).unwrap_or_default(),
                );
                eprintln!("wrote {}", path.display());
                finished.store(true, Ordering::Relaxed);
            });
        }
        if self.finished.load(Ordering::Relaxed) && self.screenshot.is_some() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.theme.clear_color()
    }
}
