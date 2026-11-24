//! {{app_title}}: a generic desktop workbench built on `egui-workbench`.
//!
//! The shell lives in `ui.rs` and `app.rs`. The product screen starts in `content.rs`;
//! the per-tab model starts in `model.rs`.

mod app;
mod appearance;
mod chart_settings;
mod charts;
mod commands;
mod content;
mod gallery;
mod icon_browser;
mod model;
mod panels;
mod settings;
mod ui;

use egui_workbench::eframe;

fn main() -> eframe::Result {
    let title = "{{app_title}}";
    eframe::run_native(
        title,
        egui_workbench::native_options(title),
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
