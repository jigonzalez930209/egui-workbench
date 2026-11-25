//! The fixed workbench layout.
//!
//! [`show_workbench`] draws the panels in the order the specification requires:
//! title bar, explorer, tab strip, status bar, central content, then overlays. It
//! returns every command the sections produced so the application can apply them
//! after the frame, exactly like the component API.
//!
//! ```
//! # use egui_workbench::app::{WorkbenchApp, show_workbench};
//! # use egui_workbench::egui;
//! enum Command { Quit }
//!
//! struct App { sidebar: bool }
//!
//! impl WorkbenchApp for App {
//!     type Command = Command;
//!     fn sidebar_visible(&self) -> bool { self.sidebar }
//!     fn tab_strip_visible(&self) -> bool { true }
//!     fn status_bar_visible(&self) -> bool { true }
//!     fn central(&mut self, ui: &mut egui::Ui) -> Vec<Command> {
//!         ui.label("Content");
//!         Vec::new()
//!     }
//! }
//!
//! fn ui(ui: &mut egui::Ui, app: &mut App) {
//!     let commands = show_workbench(ui, app);
//!     for command in commands {
//!         match command { Command::Quit => {} }
//!     }
//! }
//! ```

use egui::{CentralPanel, Frame, Panel, Ui};

use crate::foundation::metrics::{SIDEBAR_WIDTH, STATUS_HEIGHT, TAB_HEIGHT, TITLE_HEIGHT};

/// The stable egui id of the title-bar panel.
pub const TITLE_BAR_PANEL: &str = "workbench_title_bar";
/// The stable egui id of the explorer panel.
pub const SIDEBAR_PANEL: &str = "workbench_sidebar";
/// The narrowest the explorer gets, in points.
pub const SIDEBAR_MIN_WIDTH: f32 = 160.0;
/// The content width the explorer yields to on narrow windows, in points.
pub const CONTENT_MIN_WIDTH: f32 = 480.0;
/// The stable egui id of the tab-strip panel.
pub const TAB_STRIP_PANEL: &str = "workbench_tab_strip";
/// The stable egui id of the status-bar panel.
pub const STATUS_BAR_PANEL: &str = "workbench_status_bar";
/// The stable egui id of the right-hand panel.
pub const RIGHT_PANEL: &str = "workbench_right_panel";
/// The stable egui id of the bottom panel.
pub const BOTTOM_PANEL: &str = "workbench_bottom_panel";
/// The narrowest the right panel gets, in points.
pub const RIGHT_PANEL_MIN_WIDTH: f32 = 180.0;
/// The lowest the bottom panel gets, in points.
pub const BOTTOM_PANEL_MIN_HEIGHT: f32 = 72.0;

/// The application side of [`show_workbench`].
///
/// Every drawing method returns the commands it produced. The shell only lays out
/// panels and never applies a command itself, so state changes stay in one place.
/// Implement `sidebar_visible`, `tab_strip_visible`, and `status_bar_visible` to
/// compose the shell; the defaults show only the title bar and central content.
pub trait WorkbenchApp {
    /// The command type shared by menus, the palette, shortcuts, and panels.
    type Command;

    /// Whether the title bar occupies the top panel. Defaults to `true`.
    fn title_bar_visible(&self) -> bool {
        true
    }

    /// Whether the explorer occupies the left panel. Defaults to `false`.
    fn sidebar_visible(&self) -> bool {
        false
    }

    /// Whether the tab strip occupies the top panel of the editor column.
    fn tab_strip_visible(&self) -> bool {
        false
    }

    /// Whether the status bar occupies the bottom panel of the editor column.
    fn status_bar_visible(&self) -> bool {
        false
    }

    /// The preferred explorer width. egui remembers later resizes under a stable id; on
    /// narrow windows the shell shrinks it (down to `SIDEBAR_MIN_WIDTH`) so the content
    /// keeps `CONTENT_MIN_WIDTH`, and never lets it exceed half of the window.
    fn sidebar_width(&self) -> f32 {
        SIDEBAR_WIDTH
    }

    /// Draw the title bar.
    fn title_bar(&mut self, _ui: &mut Ui) -> Vec<Self::Command> {
        Vec::new()
    }

    /// Draw the explorer.
    fn sidebar(&mut self, _ui: &mut Ui) -> Vec<Self::Command> {
        Vec::new()
    }

    /// Draw the tab strip.
    fn tab_strip(&mut self, _ui: &mut Ui) -> Vec<Self::Command> {
        Vec::new()
    }

    /// Draw the application's central content.
    fn central(&mut self, _ui: &mut Ui) -> Vec<Self::Command> {
        Vec::new()
    }

    /// Draw the status bar.
    fn status_bar(&mut self, _ui: &mut Ui) -> Vec<Self::Command> {
        Vec::new()
    }

    /// Whether a resizable panel occupies the right edge (an inspector, outline,
    /// properties, or chat). It spans from the title bar to the bottom of the window.
    /// Defaults to `false`.
    fn right_panel_visible(&self) -> bool {
        false
    }

    /// The initial right-panel width; egui remembers later resizes.
    fn right_panel_width(&self) -> f32 {
        280.0
    }

    /// Draw the right panel.
    fn right_panel(&mut self, _ui: &mut Ui) -> Vec<Self::Command> {
        Vec::new()
    }

    /// Whether a resizable panel sits under the content, above the status bar (a
    /// terminal, output, problems, or a timeline). Defaults to `false`.
    fn bottom_panel_visible(&self) -> bool {
        false
    }

    /// The initial bottom-panel height; egui remembers later resizes.
    fn bottom_panel_height(&self) -> f32 {
        220.0
    }

    /// Draw the bottom panel.
    fn bottom_panel(&mut self, _ui: &mut Ui) -> Vec<Self::Command> {
        Vec::new()
    }

    /// Draw dialogs, the palette, prompts, and any other foreground layer.
    fn overlays(&mut self, _ui: &mut Ui) -> Vec<Self::Command> {
        Vec::new()
    }
}

/// Draw the full workbench and return the commands produced this frame.
///
/// This calls [`crate::handle_window_edges`] and uses the fixed panel order from
/// `WORKBENCH.md` §3. Apply the returned commands after this function returns.
pub fn show_workbench<A: WorkbenchApp>(ui: &mut Ui, app: &mut A) -> Vec<A::Command> {
    crate::ensure_undecorated(ui.ctx());
    crate::handle_window_edges(ui.ctx());
    let mut commands = Vec::new();
    if app.title_bar_visible() {
        Panel::top(TITLE_BAR_PANEL)
            .exact_size(TITLE_HEIGHT)
            .show(ui, |ui| commands.extend(app.title_bar(ui)));
    }
    if app.sidebar_visible() {
        // On narrow windows the explorer gives way so the content keeps at least
        // `CONTENT_MIN_WIDTH`, and it can never take more than half of the window.
        let available = ui.available_width();
        let max = (available * 0.5).max(SIDEBAR_MIN_WIDTH);
        let width = app
            .sidebar_width()
            .min(available - CONTENT_MIN_WIDTH)
            .clamp(SIDEBAR_MIN_WIDTH, max);
        let fill = ui.visuals().window_fill;
        Panel::left(SIDEBAR_PANEL)
            .resizable(true)
            .default_size(width)
            .size_range(SIDEBAR_MIN_WIDTH..=max)
            .frame(Frame::new().fill(fill))
            .show(ui, |ui| commands.extend(app.sidebar(ui)));
    }
    // The right and bottom panels have the content's 8 pt inner margin so their
    // contents line up with it; egui draws the separator line on the shared edge.
    if app.right_panel_visible() {
        // Like the explorer, it yields to the content on narrow windows.
        let available = ui.available_width();
        let max = (available * 0.5).max(RIGHT_PANEL_MIN_WIDTH);
        let width = app
            .right_panel_width()
            .min(available - CONTENT_MIN_WIDTH)
            .clamp(RIGHT_PANEL_MIN_WIDTH, max);
        let fill = ui.visuals().window_fill;
        Panel::right(RIGHT_PANEL)
            .resizable(true)
            .default_size(width)
            .size_range(RIGHT_PANEL_MIN_WIDTH..=max)
            .frame(Frame::new().fill(fill).inner_margin(8))
            .show(ui, |ui| commands.extend(app.right_panel(ui)));
    }
    if app.tab_strip_visible() {
        Panel::top(TAB_STRIP_PANEL)
            .exact_size(TAB_HEIGHT)
            .show(ui, |ui| commands.extend(app.tab_strip(ui)));
    }
    if app.status_bar_visible() {
        Panel::bottom(STATUS_BAR_PANEL)
            .exact_size(STATUS_HEIGHT)
            .show(ui, |ui| commands.extend(app.status_bar(ui)));
    }
    if app.bottom_panel_visible() {
        // Never more than 60 % of the editor column, so the content stays usable.
        let max = (ui.available_height() * 0.6).max(BOTTOM_PANEL_MIN_HEIGHT);
        let height = app.bottom_panel_height().clamp(BOTTOM_PANEL_MIN_HEIGHT, max);
        let fill = ui.visuals().window_fill;
        Panel::bottom(BOTTOM_PANEL)
            .resizable(true)
            .default_size(height)
            .size_range(BOTTOM_PANEL_MIN_HEIGHT..=max)
            .frame(Frame::new().fill(fill).inner_margin(8))
            .show(ui, |ui| commands.extend(app.bottom_panel(ui)));
    }
    CentralPanel::default().show(ui, |ui| commands.extend(app.central(ui)));
    commands.extend(app.overlays(ui));
    commands
}
