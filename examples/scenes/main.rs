//! Run one documentation scene natively, exactly as the docs site embeds it.
//!
//! ```bash
//! cargo run --example scenes -- scene=plot-live theme=dracula
//! cargo run --example scenes -- scene=style style=compact-sharp
//! ```

#[path = "scenes.rs"]
mod scenes;

use egui_workbench::eframe;

fn main() -> eframe::Result {
    let query: Vec<String> = std::env::args().skip(1).collect();
    let mut query = query.join("&");
    if !query.is_empty() && !query.contains('=') {
        query = format!("scene={query}");
    }
    let params = scenes::Params::parse(&query);
    let title = format!("Scene: {}", params.scene);
    eframe::run_native(
        &title,
        egui_workbench::native_options_sized(&title, [960.0, 640.0], [360.0, 240.0]),
        Box::new(move |_| Ok(Box::new(scenes::SceneApp::new(params)))),
    )
}
