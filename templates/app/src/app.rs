//! Application state and command handling.
//!
//! The state is one struct. Menus, the palette, shortcuts, and panels all produce
//! [`Command`] values, and [`App::handle`] is the only place that mutates state.

use egui_workbench::app::{CloseGuard, Documents, ThemeState, show_workbench};
use egui_workbench::app::{CommandJournal, DevToolsState, show_devtools};
use egui_workbench::commands::{CommandId, CommandRegistry};
use egui_workbench::components::{ToastEvent, Toasts};
use egui_workbench::eframe;
use egui_workbench::{
    Icons, KeyboardMenuState, PaletteState, ThemeId, TreeState, WindowAction, apply_window_action, set_window_title,
};

use crate::appearance::Appearance;
use crate::commands::{self, Command};
use crate::model::{GallerySection, Tab};
use crate::settings::{self, Settings};

/// What a close prompt will close.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseRequest {
    /// The window.
    Window,
    /// One dirty tab.
    Tab(usize),
}

/// The user's answer to a close prompt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseChoice {
    /// Save first.
    Save,
    /// Close without saving.
    Discard,
    /// Keep everything open.
    Cancel,
}

/// The whole application.
pub struct App {
    /// Shell glyph cache with the product mark registered.
    pub icons: Icons,
    /// Transient notifications, drawn above every panel.
    pub toasts: Toasts,
    /// Requested and effective theme.
    pub theme: ThemeState,
    /// Open tabs and the active tab.
    pub tabs: Documents<Tab>,
    /// Command palette state.
    pub palette: PaletteState,
    /// Settings dialog visibility.
    pub settings_open: bool,
    /// About window visibility.
    pub about_open: bool,
    /// Pending close prompt, for the window or one tab.
    pub close_request: Option<CloseRequest>,
    /// Demo tree state (replace with the product navigation).
    pub tree: TreeState<&'static str>,
    /// Demo keyboard menu state.
    pub keyboard_menu: KeyboardMenuState,
    /// Custom dialog demo visibility.
    pub demo_dialog: bool,
    /// Confirm dialog demo visibility.
    pub demo_confirm: bool,
    /// Name dialog demo visibility.
    pub demo_name: bool,
    /// Progress dialog demo visibility.
    pub demo_progress: bool,
    /// Name dialog value.
    pub demo_name_value: String,
    /// Navigator visibility.
    pub sidebar_open: bool,
    /// Inspector (right panel) visibility.
    pub inspector_open: bool,
    /// Output (bottom panel) visibility.
    pub output_open: bool,
    /// Every handled command, newest last, for the Output panel.
    pub log: Vec<String>,
    /// Density, corners, and brand accent on top of the theme.
    pub appearance: Appearance,
    /// One-line feedback for the status bar.
    pub status: String,
    /// Cancels window close requests while tabs are dirty.
    pub guard: CloseGuard,
    /// The registry every chrome surface reads.
    pub registry: CommandRegistry,
    /// Every handled command, for the journal and macros.
    pub journal: CommandJournal,
    /// The F12 journal viewer and state inspector.
    pub devtools: DevToolsState,
    window_title: String,
    next_tab: u64,
}

impl App {
    /// Build the application from stored settings.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::with_context(&cc.egui_ctx)
    }

    fn with_context(ctx: &egui_workbench::egui::Context) -> Self {
        let stored = settings::load();
        let mut icons = Icons::new();
        icons.register("logo", include_bytes!("../assets/logo.svg").to_vec());
        let mut app = Self {
            icons,
            // Clear the status bar in the bottom-right corner.
            toasts: Toasts::new().offset(0.0, egui_workbench::metrics::STATUS_HEIGHT),
            theme: ThemeState::with_reduced_motion(
                ctx,
                ThemeId::from_setting(&stored.theme),
                stored.follow_system,
                stored.reduced_motion,
            ),
            tabs: Documents::new(),
            palette: PaletteState::default(),
            settings_open: false,
            about_open: false,
            close_request: None,
            tree: TreeState::default(),
            keyboard_menu: KeyboardMenuState::default(),
            demo_dialog: false,
            demo_confirm: false,
            demo_name: false,
            demo_progress: false,
            demo_name_value: "New document".to_owned(),
            sidebar_open: true,
            inspector_open: false,
            output_open: false,
            log: Vec::new(),
            appearance: Appearance::from_settings(&stored),
            status: String::new(),
            registry: commands::registry(),
            journal: CommandJournal::new(),
            devtools: DevToolsState::default(),
            guard: CloseGuard::new(),
            window_title: String::new(),
            next_tab: 0,
        };
        app.appearance.apply(ctx, &mut app.theme);
        app.next_tab += 1;
        app.tabs.push_active(Tab::new(app.next_tab));
        app
    }

    /// Publish the checked and enabled state the chrome should show this frame.
    fn sync_registry(&mut self) {
        let sidebar = self.sidebar_open;
        let inspector = self.inspector_open;
        let output = self.output_open;
        let follow = self.theme.follow_system();
        let requested = self.theme.requested();
        let tabs = self.tabs.len();
        let set = |registry: &mut CommandRegistry, id: &str, checked: bool| {
            let _ = registry.set_checked(&CommandId::new(id), checked);
        };
        set(&mut self.registry, commands::ids::TOGGLE_SIDEBAR, sidebar);
        set(&mut self.registry, commands::ids::TOGGLE_INSPECTOR, inspector);
        set(&mut self.registry, commands::ids::TOGGLE_OUTPUT, output);
        set(&mut self.registry, commands::ids::FOLLOW_SYSTEM, follow);
        for theme in ThemeId::ALL {
            set(
                &mut self.registry,
                &format!("theme.{}", theme.as_setting()),
                theme == requested,
            );
        }
        for (id, enabled) in [
            (commands::ids::NEXT_TAB, tabs > 1),
            (commands::ids::PREVIOUS_TAB, tabs > 1),
            (commands::ids::SAVE, tabs > 0),
            (commands::ids::CLOSE_TAB, tabs > 0),
        ] {
            let _ = self.registry.set_enabled(&CommandId::new(id), enabled);
        }
    }

    /// Apply one command.
    pub fn handle(&mut self, ctx: &egui_workbench::egui::Context, command: Command) {
        if let Some(id) = commands::id_for(command) {
            self.journal
                .append_at(ctx.input(|input| input.time), id, None, Some(format!("{command:?}")));
        }
        self.log.push(format!("{command:?}"));
        if self.log.len() > 500 {
            self.log.drain(..100);
        }
        match command {
            Command::NewTab => {
                if let Some(section) = GallerySection::ALL
                    .into_iter()
                    .find(|section| !self.tabs.items().iter().any(|tab| tab.section == *section))
                {
                    self.open_section(section);
                }
            }
            Command::OpenExample(section) => self.open_section(section),
            Command::Save => self.save_active(),
            Command::CloseTab => self.close_tab(self.tabs.active_index()),
            Command::NextTab => {
                self.tabs.select_next();
            }
            Command::PreviousTab => {
                self.tabs.select_previous();
            }
            Command::ToggleSidebar => self.sidebar_open = !self.sidebar_open,
            Command::ToggleInspector => self.inspector_open = !self.inspector_open,
            Command::ToggleOutput => self.output_open = !self.output_open,
            Command::Palette => self.palette.open(),
            Command::Settings => self.settings_open = true,
            Command::About => self.about_open = true,
            Command::ShowDialog => self.demo_dialog = true,
            Command::ShowConfirm => self.demo_confirm = true,
            Command::ShowName => self.demo_name = true,
            Command::ShowProgress => self.demo_progress = true,
            Command::KeyboardMenu => self.keyboard_menu.open(0),
            Command::SetTheme(id) => {
                self.theme.set_requested(ctx, id);
                self.save_settings();
            }
            Command::ToggleFollowSystem => {
                self.theme.set_follow_system(ctx, !self.theme.follow_system());
                self.save_settings();
            }
            Command::Quit => apply_window_action(ctx, WindowAction::Close),
        }
    }

    /// Open a gallery section in the editor, or select its existing tab.
    pub fn open_section(&mut self, section: GallerySection) {
        if let Some(index) = self.tabs.items().iter().position(|tab| tab.section == section) {
            self.tabs.set_active(index);
        } else {
            self.next_tab += 1;
            self.tabs.push_active(Tab::example(self.next_tab, section));
        }
    }

    /// Close a tab, asking first when it has unsaved changes.
    pub fn close_tab(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }
        if self.tabs.get(index).is_some_and(|tab| tab.dirty) {
            self.close_request = Some(CloseRequest::Tab(index));
        } else {
            self.tabs.remove(index);
        }
    }

    /// Save the active tab.
    pub fn save_active(&mut self) {
        if self.tabs.is_empty() {
            self.status = "No tab is open.".to_owned();
            return;
        }
        let index = self.tabs.active_index();
        match self.save_tab(index) {
            Ok(()) => {
                let name = self.tabs.get(index).map_or("tab", |tab| tab.title.as_str());
                self.status = format!("Saved {name}");
                self.toasts.success(format!("Saved {name}"));
            }
            Err(message) => {
                self.toasts.error(message.clone());
                self.status = message;
            }
        }
    }

    /// Persist one tab.
    ///
    /// This stub only clears the dirty flag. Replace it with the application's
    /// persistence (file, database, remote session, …). Returning `Err` keeps the tab
    /// open and shows the message in the status bar or close prompt.
    fn save_tab(&mut self, index: usize) -> Result<(), String> {
        let Some(tab) = self.tabs.get_mut(index) else {
            return Err("No tab is open.".to_owned());
        };
        tab.dirty = false;
        Ok(())
    }

    /// Save every dirty tab. Returns `false` when one cannot be saved.
    pub fn save_all(&mut self) -> bool {
        let mut saved = true;
        for index in 0..self.tabs.len() {
            if self.tabs.get(index).is_some_and(|tab| tab.dirty)
                && let Err(message) = self.save_tab(index)
            {
                self.status = message;
                saved = false;
            }
        }
        saved
    }

    /// Resolve the pending close prompt.
    pub fn resolve_close(&mut self, ctx: &egui_workbench::egui::Context, choice: CloseChoice) {
        let Some(request) = self.close_request.take() else {
            return;
        };
        match choice {
            CloseChoice::Cancel => self.guard.reset(),
            CloseChoice::Discard => match request {
                CloseRequest::Window => self.guard.allow_close(ctx),
                CloseRequest::Tab(index) => {
                    self.tabs.remove(index);
                }
            },
            CloseChoice::Save => match request {
                CloseRequest::Window => {
                    if self.save_all() {
                        self.guard.allow_close(ctx);
                    } else {
                        self.close_request = Some(CloseRequest::Window);
                    }
                }
                CloseRequest::Tab(index) => match self.save_tab(index) {
                    Ok(()) => {
                        self.tabs.remove(index);
                    }
                    Err(message) => {
                        self.status = message;
                        self.close_request = Some(CloseRequest::Tab(index));
                    }
                },
            },
        }
    }

    /// Whether any modal layer is open.
    pub fn overlay_open(&self) -> bool {
        self.palette.open
            || self.settings_open
            || self.about_open
            || self.close_request.is_some()
            || self.demo_dialog
            || self.demo_confirm
            || self.demo_name
            || self.demo_progress
            || self.keyboard_menu.open
    }

    /// Store the appearance preferences.
    pub fn save_settings(&mut self) {
        let mut stored = Settings {
            theme: self.theme.theme_setting().to_owned(),
            follow_system: self.theme.follow_system(),
            reduced_motion: self.theme.reduced_motion(),
            ..Settings::default()
        };
        self.appearance.store(&mut stored);
        if let Err(error) = settings::save(&stored) {
            self.status = format!("Cannot save settings: {error}");
        }
    }

    /// Whether there are unsaved tabs.
    pub fn dirty(&self) -> bool {
        self.tabs.any_dirty()
    }

    fn sync_window_title(&mut self, ctx: &egui_workbench::egui::Context) {
        let title = match self.tabs.active() {
            Some(tab) => format!("{}{} — {{app_title}}", tab.title, if tab.dirty { " •" } else { "" }),
            None => "{{app_title}}".to_owned(),
        };
        if title != self.window_title {
            set_window_title(ctx, &title);
            self.window_title = title;
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui_workbench::egui::Context, _frame: &mut eframe::Frame) {
        // Follows the desktop light/dark preference, brand accent included.
        self.theme.sync_system(ctx);
        if self.guard.should_prompt(ctx, self.dirty()) {
            self.close_request = Some(CloseRequest::Window);
        }
        self.sync_window_title(ctx);
        self.sync_registry();
        if !self.overlay_open()
            && let Some(id) = self.registry.shortcut(ctx, &commands::context())
            && let Some(command) = commands::command_from_id(&id)
        {
            self.handle(ctx, command);
        }
    }

    fn ui(&mut self, ui: &mut egui_workbench::egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let commands = show_workbench(ui, self);
        for command in commands {
            self.handle(&ctx, command);
        }
        for event in self.toasts.show(&ctx, &mut self.icons) {
            if let ToastEvent::Action(_) = event {
                self.status = "Toast action".to_owned();
            }
        }
        let snapshot = serde_json::json!({
            "tabs": self.tabs.len(),
            "dirty": self.dirty(),
            "journal": self.journal.len(),
            "theme": self.theme.requested().as_setting(),
        });
        show_devtools(&ctx, &mut self.devtools, &self.journal, &snapshot, Default::default());
    }

    fn clear_color(&self, _visuals: &egui_workbench::egui::Visuals) -> [f32; 4] {
        self.theme.clear_color()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_workbench::egui;

    fn app() -> App {
        let mut app = App::with_context(&egui::Context::default());
        app.tabs.clear();
        app.next_tab = 0;
        app
    }

    #[test]
    fn clean_tabs_close_without_a_prompt() {
        let mut app = app();
        app.tabs.push_active(Tab::new(1));
        app.close_tab(0);
        assert!(app.tabs.is_empty());
        assert_eq!(app.close_request, None);
    }

    #[test]
    fn dirty_tabs_prompt_and_discard_closes_them() {
        let ctx = egui::Context::default();
        let mut app = app();
        app.tabs.push_active(Tab::new(1));
        app.tabs.active_mut().expect("tab").dirty = true;
        app.close_tab(0);
        assert_eq!(app.close_request, Some(CloseRequest::Tab(0)));
        app.resolve_close(&ctx, CloseChoice::Discard);
        assert!(app.tabs.is_empty());
        assert_eq!(app.close_request, None);
    }

    #[test]
    fn save_clears_the_dirty_flag() {
        let mut app = app();
        app.tabs.push_active(Tab::new(1));
        app.tabs.active_mut().expect("tab").dirty = true;
        app.save_active();
        assert!(!app.tabs.active().expect("tab").dirty);
        assert_eq!(app.tabs.active_title(), Some("Buttons"));
    }
}
