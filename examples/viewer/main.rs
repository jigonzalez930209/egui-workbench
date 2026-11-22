//! The native data viewer as a runnable example: open an Excel workbook, CSV, TSV, or
//! text file by dragging it into the window, or pass a path as the first argument.
//!
//! ```bash
//! cargo run --example viewer
//! cargo run --example viewer -- tests/fixtures/sales.xlsx
//! ```

use egui_workbench::app::ThemeState;
use egui_workbench::components::prelude::*;
use egui_workbench::components::viewer::FileViewer;
use egui_workbench::egui::{self, RichText};
use egui_workbench::{Icons, ThemeId, TitleBar, apply_window_action, is_maximized, show_title_bar};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let path = std::env::args().nth(1);
    eframe::run_native(
        "Data viewer",
        egui_workbench::native_options("Data viewer"),
        Box::new(move |cc| {
            let theme = ThemeState::new(&cc.egui_ctx, ThemeId::GithubDark, false);
            Ok(Box::new(ViewerExample::new(theme, path)))
        }),
    )
}

use egui_workbench::eframe;

/// The example application: one viewer, drag-and-drop, and a theme toggle.
struct ViewerExample {
    icons: Icons,
    theme: ThemeState,
    viewer: FileViewer,
    source: String,
    error: Option<String>,
}

impl ViewerExample {
    fn new(theme: ThemeState, path: Option<String>) -> Self {
        let (viewer, source) = match path {
            Some(path) => match FileViewer::open(&path) {
                Ok(viewer) => (viewer, path),
                Err(error) => (sample(), format!("Could not open {path}: {error}")),
            },
            None => (sample(), "Embedded sample: sales.xlsx".to_owned()),
        };
        Self {
            icons: Icons::new(),
            theme,
            viewer,
            source,
            error: None,
        }
    }

    fn open(&mut self, path: &std::path::Path) {
        match FileViewer::open(path) {
            Ok(viewer) => {
                self.viewer = viewer;
                self.source = path.display().to_string();
                self.error = None;
            }
            Err(error) => self.error = Some(format!("Could not open {}: {error}", path.display())),
        }
    }

    fn take_dropped(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|input| input.raw.dropped_files.clone());
        if let Some(path) = dropped.into_iter().map(|file| file.path().to_path_buf()).next() {
            self.open(&path);
        }
    }
}

/// The sample workbook, embedded so the example works without arguments.
fn sample() -> FileViewer {
    FileViewer::from_bytes("sales.xlsx", include_bytes!("../../tests/fixtures/sales.xlsx"))
        .unwrap_or_else(|error| FileViewer::text("sales-error.txt", format!("Could not open the sample: {error}")))
}

impl eframe::App for ViewerExample {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.theme.sync_system(&ctx);
        self.take_dropped(&ctx);
        egui::Panel::top("viewer_title")
            .exact_size(egui_workbench::metrics::TITLE_HEIGHT)
            .show(ui, |ui| {
                let mut bar = TitleBar::new(self.viewer.name());
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
            ui.horizontal_wrapped(|ui| {
                Badge::new("Native").tone(Tone::Accent).show(ui);
                Label::new("Drag an XLSX, XLS, ODS, CSV, TSV, or UTF-8 text file here")
                    .tone(Tone::Muted)
                    .size(Size::Sm)
                    .show(ui);
                Label::new(&self.source).tone(Tone::Muted).size(Size::Sm).show(ui);
            });
            if let Some(error) = &self.error {
                Banner::new(error).tone(Tone::Danger).show(ui, |_| {});
            }
            ui.add_space(4.0);
            self.viewer.show(ui);
            ui.add_space(2.0);
            ui.label(
                RichText::new("Only rows inside the viewport are laid out.")
                    .small()
                    .weak(),
            );
        });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.theme.clear_color()
    }
}
