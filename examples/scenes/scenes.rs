//! Live documentation scenes: small, interactive pieces of the library that the docs
//! site embeds through WebAssembly, and that the test suite renders into the posters
//! shown before a demo starts.
//!
//! One scene is one `Scene` value. [`SceneApp`] hosts it with a theme, a style preset,
//! an optional brand accent, and a toast stack the scene uses to report what happened.
//! The same code runs natively (`cargo run --example scenes -- plot`), in the browser
//! (`web/`), and in `tests/doc_scenes.rs`.

#![allow(dead_code)]

mod animations;
mod board;
mod chart_gallery;
mod chart_types;
mod charting;
mod controls;
mod overlays;
mod plots;
mod shell;
mod showcase;
mod three_d;
mod viewer;

#[allow(dead_code, clippy::all)]
#[path = "../catalog/main.rs"]
mod catalog;

// The starter's Charts gallery, included as is (it refers to its sibling with `super::`).
#[allow(dead_code, clippy::all)]
#[path = "../../templates/app/src/chart_settings/mod.rs"]
mod chart_settings;
#[allow(dead_code, clippy::all)]
#[path = "../../templates/app/src/charts.rs"]
mod charts;

use std::sync::{Arc, Mutex};

use egui_workbench::app::ThemeState;
use egui_workbench::components::prelude::*;
use egui_workbench::eframe;
use egui_workbench::egui::{self, Color32};
use egui_workbench::style::{self, Density, WorkbenchStyle};
use egui_workbench::{Icons, ThemeId};

/// Every scene name, in the order the docs introduce them.
pub const SCENES: [&str; 30] = [
    "shell",
    "catalog",
    "components",
    "buttons",
    "inputs",
    "display",
    "toasts",
    "menus",
    "overlays",
    "settings",
    "icons",
    "style",
    "accent",
    "plot",
    "plot-select",
    "plot-log",
    "plot-bars",
    "plot-live",
    "plot-series",
    "plot-fits",
    "plot-big",
    "plot-types",
    "plot-animations",
    "plot-chart-gallery",
    "plot-3d",
    "plot-3d-surface",
    "plot-3d-bars",
    "plot-3d-helix",
    "charts",
    "viewer",
];

/// Every style preset name accepted by `style=`.
pub const STYLES: [&str; 4] = ["default", "compact-sharp", "comfortable-rounded", "large-borderless"];

/// What to show: parsed from `?scene=…&theme=…&style=…&accent=…`.
#[derive(Clone, Debug, PartialEq)]
pub struct Params {
    /// One of [`SCENES`].
    pub scene: String,
    /// A theme setting string (`github-light`, `dracula`, …).
    pub theme: ThemeId,
    /// One of [`STYLES`].
    pub style: String,
    /// A brand accent.
    pub accent: Option<Color32>,
    /// Embedded in a docs page: the wheel scrolls the page until the scene has focus.
    pub embedded: bool,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            scene: "catalog".to_owned(),
            theme: ThemeId::GithubDark,
            style: "default".to_owned(),
            accent: None,
            embedded: false,
        }
    }
}

impl Params {
    /// Parse a URL query (`scene=plot&theme=github-light`, with or without `?`) or
    /// command-line style `key=value` pairs. Unknown keys and values fall back to the
    /// defaults, so a typo never breaks a page.
    pub fn parse(query: &str) -> Self {
        let mut params = Self::default();
        for pair in query.trim_start_matches('?').split(['&', ' ']) {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            match key {
                "scene" if SCENES.contains(&value) => value.clone_into(&mut params.scene),
                "theme" => params.theme = ThemeId::from_setting(value),
                "style" if STYLES.contains(&value) => value.clone_into(&mut params.style),
                "accent" => params.accent = parse_hex(value),
                "embedded" => params.embedded = value == "1",
                _ => {}
            }
        }
        params
    }
}

/// A `rrggbb` color (an optional leading `#` or `%23` is accepted).
pub fn parse_hex(value: &str) -> Option<Color32> {
    let hex = value.trim_start_matches("%23").trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(hex.get(range)?, 16).ok();
    Some(Color32::from_rgb(channel(0..2)?, channel(2..4)?, channel(4..6)?))
}

/// The [`WorkbenchStyle`] of a preset name.
pub fn style_preset(name: &str) -> WorkbenchStyle {
    match name {
        "compact-sharp" => WorkbenchStyle::sharp().with_density(Density::Compact),
        "comfortable-rounded" => WorkbenchStyle::rounded().with_density(Density::Comfortable),
        "large-borderless" => WorkbenchStyle {
            text_scale: 1.25,
            border_width: 0.0,
            ..WorkbenchStyle::default()
        },
        _ => WorkbenchStyle::default(),
    }
}

/// Saves a file the user asked for (an exported chart): a browser download on the web,
/// a file in the temporary directory natively. Returns where it went.
pub type Saver = Box<dyn FnMut(&str, &str, Vec<u8>) -> String>;

/// What a scene can use besides its own state.
pub struct Host<'a> {
    /// The shared icon cache.
    pub icons: &'a mut Icons,
    /// Toasts drawn above the scene; report actions here.
    pub toasts: &'a mut Toasts,
    /// The scene's theme state (accent included).
    pub theme: &'a mut ThemeState,
    /// Saves exported files.
    pub save: &'a mut dyn FnMut(&str, &str, Vec<u8>) -> String,
    /// What the toasts reported last frame (action clicks, closes, timeouts).
    pub events: &'a [ToastEvent],
}

impl Host<'_> {
    /// Report an action as a short toast.
    pub fn report(&mut self, message: impl Into<String>) {
        self.toasts.add(Toast::new(ToastKind::Info, message).duration(2.5));
    }
}

/// One interactive piece of the docs.
pub trait Scene {
    /// Draw one frame.
    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>);

    /// Whether the scene draws its own panels (a whole workbench) instead of content
    /// inside the default padded panel.
    fn full(&self) -> bool {
        false
    }

    /// Whether the default panel scrolls (scenes that fill the height with their own
    /// scroll area or a plot return `false`).
    fn scroll(&self) -> bool {
        true
    }

    /// Animated scenes (live data) request repaints themselves; the others repaint on
    /// input only.
    fn tick(&mut self, _ctx: &egui::Context) {}
}

/// Build the scene called `name` (the catalog for an unknown name).
pub fn build(name: &str) -> Box<dyn Scene> {
    match name {
        "shell" => Box::new(shell::Shell::default()),
        "components" => Box::new(board::Board::default()),
        "style" => Box::new(board::StyleScene::default()),
        "accent" => Box::new(board::AccentScene::default()),
        "buttons" => Box::new(controls::Buttons),
        "inputs" => Box::new(controls::Inputs::default()),
        "display" => Box::new(controls::Display::default()),
        "toasts" => Box::new(overlays::ToastScene::default()),
        "menus" => Box::new(overlays::Menus::default()),
        "overlays" => Box::new(overlays::Overlays::default()),
        "settings" => Box::new(overlays::Settings::default()),
        "icons" => Box::new(showcase::IconBrowser::default()),
        "plot" => Box::new(plots::PlotScene::new(plots::Kind::Signal)),
        "plot-select" => Box::new(plots::PlotScene::new(plots::Kind::Select)),
        "plot-log" => Box::new(plots::PlotScene::new(plots::Kind::Log)),
        "plot-bars" => Box::new(plots::PlotScene::new(plots::Kind::Bars)),
        "plot-live" => Box::new(plots::PlotScene::new(plots::Kind::Live)),
        "plot-series" => Box::new(charting::SeriesScene::default()),
        "plot-fits" => Box::new(charting::FitsScene::default()),
        "plot-big" => Box::new(charting::BigScene::default()),
        "plot-types" => Box::new(charting::NewCharts::default()),
        "plot-animations" => Box::new(animations::AnimationScene::default()),
        "plot-chart-gallery" => Box::new(chart_gallery::ChartGallery::default()),
        "plot-3d" => Box::new(three_d::ThreeD::default()),
        "plot-3d-surface" => Box::new(three_d::ThreeD::surface()),
        "plot-3d-bars" => Box::new(three_d::ThreeD::bars()),
        "plot-3d-helix" => Box::new(three_d::ThreeD::helix()),
        "charts" => Box::new(charting::Gallery::default()),
        "viewer" => Box::new(viewer::ViewerScene::default()),
        _ => Box::new(showcase::Catalog::default()),
    }
}

/// A message from the page that embeds the scene.
#[derive(Clone, Debug, Default)]
pub struct Inbox {
    /// A theme to switch to (the docs site's light/dark toggle).
    pub theme: Option<ThemeId>,
    /// The context to wake when a message arrives.
    pub ctx: Option<egui::Context>,
}

/// The shared inbox: the web handle writes, the app reads at the start of each frame.
pub type SharedInbox = Arc<Mutex<Inbox>>;

/// Hosts one scene with its theme, style, and toasts.
pub struct SceneApp {
    params: Params,
    scene: Box<dyn Scene>,
    theme: Option<ThemeState>,
    icons: Icons,
    toasts: Toasts,
    events: Vec<ToastEvent>,
    save: Saver,
    inbox: SharedInbox,
}

impl SceneApp {
    /// A scene app; the theme and style are installed on the first frame.
    pub fn new(params: Params) -> Self {
        Self {
            scene: build(&params.scene),
            params,
            theme: None,
            icons: Icons::new(),
            toasts: Toasts::new().position(ToastPosition::BottomRight),
            events: Vec::new(),
            save: Box::new(save_to_temp),
            inbox: SharedInbox::default(),
        }
    }

    /// Replace how exported files are saved.
    pub fn with_saver(mut self, save: Saver) -> Self {
        self.save = save;
        self
    }

    /// The inbox the embedding page writes to.
    pub fn inbox(&self) -> SharedInbox {
        Arc::clone(&self.inbox)
    }

    /// The parameters the app was built with.
    pub fn params(&self) -> &Params {
        &self.params
    }

    /// Draw one frame.
    pub fn frame(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let theme = self.theme.get_or_insert_with(|| {
            style::set(&ctx, style_preset(&self.params.style));
            let mut theme = ThemeState::new(&ctx, self.params.theme, false);
            theme.set_accent(&ctx, self.params.accent);
            theme
        });
        if let Ok(mut inbox) = self.inbox.lock() {
            inbox.ctx.get_or_insert_with(|| ctx.clone());
            if let Some(id) = inbox.theme.take() {
                theme.set_requested(&ctx, id);
            }
        }
        self.scene.tick(&ctx);
        let mut host = Host {
            icons: &mut self.icons,
            toasts: &mut self.toasts,
            theme,
            save: &mut *self.save,
            events: &self.events,
        };
        if self.scene.full() {
            self.scene.ui(ui, &mut host);
        } else {
            let fill = ui.visuals().panel_fill;
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(fill).inner_margin(16))
                .show(ui, |ui| {
                    if self.scene.scroll() {
                        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                            self.scene.ui(ui, &mut host);
                        });
                    } else {
                        self.scene.ui(ui, &mut host);
                    }
                });
        }
        self.events = self.toasts.show(&ctx, &mut self.icons);
    }

    /// The clear color of the effective theme.
    pub fn clear(&self) -> [f32; 4] {
        self.theme
            .as_ref()
            .map_or([0.0, 0.0, 0.0, 1.0], ThemeState::clear_color)
    }
}

impl eframe::App for SceneApp {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        // In a docs page the wheel belongs to the page until the reader clicks the scene,
        // so scrolling past a plot never zooms it.
        if self.params.embedded && !raw_input.focused {
            raw_input
                .events
                .retain(|event| !matches!(event, egui::Event::MouseWheel { .. } | egui::Event::Zoom(_)));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.frame(ui);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.clear()
    }
}

/// The native saver: write into the temporary directory.
fn save_to_temp(name: &str, _mime: &str, bytes: Vec<u8>) -> String {
    let path = std::env::temp_dir().join(name);
    match std::fs::write(&path, bytes) {
        Ok(()) => path.display().to_string(),
        Err(error) => format!("not saved: {error}"),
    }
}
