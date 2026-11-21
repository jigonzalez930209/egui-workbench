//! Toasts, menus, dialogs, the palette, and the settings dialog.

use egui_workbench::components::ToastId;
use egui_workbench::components::prelude::*;
use egui_workbench::egui;
use egui_workbench::{SettingsDialog, ThemeId, combo, settings_row, settings_section, settings_switch};

use super::{Host, Scene};

/// Every toast kind, positions, actions, and a loading toast updated in place.
#[derive(Default)]
pub struct ToastScene {
    started: bool,
    position: usize,
    loading: Option<(ToastId, f64)>,
    action: Option<ToastId>,
}

const POSITIONS: [(ToastPosition, &str); 6] = [
    (ToastPosition::BottomRight, "Bottom right"),
    (ToastPosition::BottomCenter, "Bottom center"),
    (ToastPosition::BottomLeft, "Bottom left"),
    (ToastPosition::TopRight, "Top right"),
    (ToastPosition::TopCenter, "Top center"),
    (ToastPosition::TopLeft, "Top left"),
];

impl Scene for ToastScene {
    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        let now = ui.input(|input| input.time);
        if !self.started {
            self.started = true;
            let id = host.toasts.add(
                Toast::new(ToastKind::Error, "Could not save the file")
                    .description("Permission denied")
                    .action("Retry")
                    .persistent(),
            );
            self.action = Some(id);
        }
        for event in host.events {
            if let ToastEvent::Action(id) = event
                && Some(*id) == self.action
            {
                host.toasts.add(Toast::new(ToastKind::Success, "Done: the action ran"));
            }
        }
        Label::new("Kinds").strong().show(ui);
        ui.horizontal_wrapped(|ui| {
            if Button::new("Info").variant(ButtonVariant::Secondary).show(ui).clicked() {
                host.toasts
                    .add(Toast::new(ToastKind::Info, "Indexing 1,204 files").description("Search is ready when done."));
            }
            if Button::new("Success")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                host.toasts.add(Toast::new(ToastKind::Success, "Saved “guide.md”"));
            }
            if Button::new("Warning")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                host.toasts
                    .add(Toast::new(ToastKind::Warning, "Disk almost full").description("2.1 GB left."));
            }
            if Button::new("Error")
                .variant(ButtonVariant::Destructive)
                .show(ui)
                .clicked()
            {
                host.toasts.add(
                    Toast::new(ToastKind::Error, "Could not save the file")
                        .description("Permission denied")
                        .persistent(),
                );
            }
        });
        ui.add_space(10.0);
        Label::new("Flows").strong().show(ui);
        ui.horizontal_wrapped(|ui| {
            if Button::new("Loading, then success")
                .variant(ButtonVariant::Outline)
                .enabled(self.loading.is_none())
                .show(ui)
                .clicked()
            {
                let id = host.toasts.add(Toast::new(ToastKind::Loading, "Exporting chart…"));
                self.loading = Some((id, now));
            }
            if Button::new("With action")
                .variant(ButtonVariant::Outline)
                .show(ui)
                .clicked()
            {
                let id = host.toasts.add(
                    Toast::new(ToastKind::Info, "Deleted “notes.md”")
                        .description("Moved to the trash.")
                        .action("Undo"),
                );
                self.action = Some(id);
            }
            if Button::new("Dismiss all")
                .variant(ButtonVariant::Ghost)
                .show(ui)
                .clicked()
            {
                host.toasts.dismiss_all();
            }
        });
        ui.add_space(10.0);
        let mut position = self.position;
        let options: Vec<(usize, String)> = POSITIONS
            .iter()
            .enumerate()
            .map(|(index, (_, label))| (index, (*label).to_owned()))
            .collect();
        settings_row(ui, "Position", "Where the stack grows from.", |ui| {
            combo(ui, "Position", &mut position, &options);
        });
        if position != self.position {
            self.position = position;
            host.toasts.set_position(POSITIONS[position].0);
            host.report(format!("Toasts::set_position({:?})", POSITIONS[position].0));
        }
        ui.add_space(6.0);
        Label::new("Hover a toast to pause its timer.")
            .tone(Tone::Muted)
            .size(Size::Sm)
            .show(ui);
        if let Some((id, start)) = self.loading {
            if now - start > 2.0 {
                host.toasts.update(
                    id,
                    Toast::new(ToastKind::Success, "Exported chart.svg").description("1 file, 48 KB"),
                );
                self.loading = None;
            } else {
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
            }
        }
    }
}

/// Dropdown menus, the overflow menu, and a context menu.
#[derive(Default)]
pub struct Menus {
    wrap: bool,
    minimap: bool,
}

impl Scene for Menus {
    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        let mut chosen: Option<String> = None;
        ui.horizontal(|ui| {
            Menu::new("File").width(220.0).show(ui, |ui| {
                if MenuItem::new("file-plus", "New file")
                    .shortcut("Ctrl+N")
                    .show(ui, host.icons)
                {
                    chosen = Some("New file".into());
                }
                if MenuItem::new("save", "Save").shortcut("Ctrl+S").show(ui, host.icons) {
                    chosen = Some("Save".into());
                }
                ui.separator();
                if MenuItem::new("close", "Delete").danger(true).show(ui, host.icons) {
                    chosen = Some("Delete".into());
                }
            });
            Menu::new("View").width(220.0).show(ui, |ui| {
                if MenuToggle::new("Word wrap", self.wrap).show(ui) {
                    self.wrap = !self.wrap;
                    chosen = Some(format!("Word wrap: {}", self.wrap));
                }
                if MenuToggle::new("Minimap", self.minimap).show(ui) {
                    self.minimap = !self.minimap;
                    chosen = Some(format!("Minimap: {}", self.minimap));
                }
                ui.separator();
                if MenuButton::new("Reset layout").show(ui) {
                    chosen = Some("Reset layout".into());
                }
            });
            MenuIconButton::new("more", "More actions").show(ui, host.icons, |ui, icons| {
                if MenuItem::new("refresh", "Refresh").show(ui, icons) {
                    chosen = Some("Refresh".into());
                }
                if MenuItem::new("search", "Find…").shortcut("Ctrl+F").show(ui, icons) {
                    chosen = Some("Find".into());
                }
            });
        });
        ui.add_space(14.0);
        let height = 120.0_f32.min(ui.available_height().max(80.0));
        let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), height), egui::Sense::click());
        let style = egui_workbench::style::get(ui.ctx());
        let visuals = ui.visuals();
        ui.painter().rect(
            rect,
            style.container(),
            visuals.extreme_bg_color,
            style.border(visuals.widgets.noninteractive.bg_stroke.color),
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Right-click here for a context menu",
            style.font(13.0),
            visuals.weak_text_color(),
        );
        response.context_menu(|ui| {
            if MenuItem::new("eye", "Open").shortcut("Enter").show(ui, host.icons) {
                chosen = Some("Open".into());
            }
            if MenuItem::new("edit", "Rename…").shortcut("F2").show(ui, host.icons) {
                chosen = Some("Rename".into());
            }
            ui.separator();
            if MenuItem::new("close", "Delete…").danger(true).show(ui, host.icons) {
                chosen = Some("Delete".into());
            }
        });
        if let Some(chosen) = chosen {
            host.report(format!("Menu: {chosen}"));
        }
    }
}

/// Dialogs, confirmations, prompts, progress, popovers, and the command palette.
pub struct Overlays {
    dialog: bool,
    confirm: bool,
    name_open: bool,
    name: String,
    progress: Option<f64>,
    popover: bool,
    palette: PaletteState,
}

impl Default for Overlays {
    fn default() -> Self {
        Self {
            dialog: false,
            confirm: false,
            name_open: false,
            name: "Untitled".to_owned(),
            progress: None,
            popover: false,
            palette: PaletteState::default(),
        }
    }
}

impl Scene for Overlays {
    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        let ctx = ui.ctx().clone();
        let now = ctx.input(|input| input.time);
        ui.horizontal_wrapped(|ui| {
            if Button::new("Dialog").show(ui).clicked() {
                self.dialog = true;
            }
            if Button::new("Confirm")
                .variant(ButtonVariant::Destructive)
                .show(ui)
                .clicked()
            {
                self.confirm = true;
            }
            if Button::new("Name prompt")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                self.name_open = true;
            }
            if Button::new("Progress")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                self.progress = Some(now);
            }
            if Button::new("Popover")
                .variant(ButtonVariant::Outline)
                .show(ui)
                .clicked()
            {
                self.popover = !self.popover;
            }
            if Button::new("Command palette")
                .variant(ButtonVariant::Outline)
                .show(ui)
                .clicked()
            {
                self.palette.open = true;
            }
        });
        ui.add_space(8.0);
        Label::new("Escape or a backdrop click closes modal overlays.")
            .tone(Tone::Muted)
            .size(Size::Sm)
            .show(ui);
        if self.dialog {
            Dialog::new("scene_dialog", "Custom dialog", &mut self.dialog)
                .width(420.0)
                .show(&ctx, |ui| {
                    Label::new("Compose any body").strong().show(ui);
                    ui.label("Dialog::show takes a closure; the frame, header, and close button are shared.");
                });
        }
        if self.confirm
            && let Some(action) = ConfirmDialog::new("Delete “notes.md”?", "This cannot be undone.")
                .confirm("Delete")
                .danger(true)
                .show(&ctx)
        {
            self.confirm = false;
            host.report(format!("PromptAction::{action:?}"));
        }
        if self.name_open
            && let Some(action) = NameDialog::new("Rename", "Name", &mut self.name).show(&ctx)
        {
            self.name_open = false;
            host.report(format!("PromptAction::{action:?} — “{}”", self.name));
        }
        if let Some(start) = self.progress {
            let progress = ((now - start) / 3.0).min(1.0) as f32;
            let cancelled = ProgressDialog::new("Exporting", "chart.png", progress).show(&ctx);
            if cancelled || progress >= 1.0 {
                self.progress = None;
                host.report(if cancelled {
                    "Progress cancelled"
                } else {
                    "Progress finished"
                });
            } else {
                ctx.request_repaint();
            }
        }
        if self.popover {
            Popover::new("scene_popover").width(300.0).show(&ctx, |ui| {
                Label::new("Popover").strong().show(ui);
                ui.label("Non-modal; it stays until you close it.");
                if Button::new("Close").size(Size::Sm).show(ui).clicked() {
                    self.popover = false;
                }
            });
        }
        let items: Vec<PaletteItem<&'static str>> = [
            ("New file", "Ctrl+N"),
            ("Save", "Ctrl+S"),
            ("Toggle explorer", "Ctrl+B"),
            ("Open settings", "Ctrl+,"),
            ("Find in files", "Ctrl+Shift+F"),
        ]
        .into_iter()
        .map(|(label, detail)| PaletteItem {
            label: label.to_owned(),
            detail: detail.to_owned(),
            value: label,
        })
        .collect();
        let labels = PaletteLabels {
            title: "Command Palette",
            hint: "Type to filter…",
            empty: "No matches",
            close: "Close",
        };
        if let Some(chosen) = CommandPalette::new(&mut self.palette, labels, &items).show(&ctx) {
            host.report(format!("Palette: {chosen}"));
        }
    }
}

/// The settings dialog with live theme controls.
pub struct Settings {
    open: bool,
    wrap: bool,
    minimap: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            open: true,
            wrap: true,
            minimap: false,
        }
    }
}

impl Scene for Settings {
    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        let ctx = ui.ctx().clone();
        if !self.open {
            Label::new("The dialog is closed.").tone(Tone::Muted).show(ui);
            if Button::new("Open settings").show(ui).clicked() {
                self.open = true;
            }
            return;
        }
        let dialog = SettingsDialog {
            title: "Settings",
            tabs: &["Appearance", "Editor", "About"],
            note: "egui-workbench",
            done: "Done",
            close: "Close",
        };
        let theme = &mut *host.theme;
        let mut report = None;
        egui_workbench::show_settings_dialog(&ctx, &mut self.open, &dialog, |ui, tab| match tab {
            0 => {
                settings_section(ui, "Appearance", "Applied immediately.");
                let mut id = theme.requested();
                let options: Vec<(ThemeId, String)> =
                    ThemeId::ALL.iter().map(|id| (*id, id.label().to_owned())).collect();
                settings_row(ui, "Theme", "Color family for the whole workbench.", |ui| {
                    combo(ui, "Theme", &mut id, &options);
                });
                if id != theme.requested() {
                    theme.set_requested(ui.ctx(), id);
                    report = Some(format!("set_requested({})", id.label()));
                }
                let mut reduce = theme.reduced_motion();
                if settings_switch(ui, "Reduced motion", "Turn every animation off.", &mut reduce) {
                    theme.set_reduced_motion(ui.ctx(), reduce);
                }
            }
            1 => {
                settings_section(ui, "Editor", "Plain state owned by the application.");
                settings_switch(ui, "Word wrap", "Wrap lines at the window edge.", &mut self.wrap);
                settings_switch(ui, "Minimap", "Show an overview on the right.", &mut self.minimap);
            }
            _ => {
                settings_section(ui, "About", "The dialog frame, rail, and rows come from the library.");
                ui.label("Rows wrap their descriptions and keep the control column on the right.");
            }
        });
        if let Some(message) = report {
            host.report(message);
        }
    }
}
