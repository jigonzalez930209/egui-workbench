//! A live catalog of the component layer: variants, sizes, states, and composition.
//!
//! Run with `cargo run --example catalog`.

use egui_workbench::app::{ThemeState, WorkbenchApp, show_workbench};
use egui_workbench::components::prelude::*;
use egui_workbench::egui::{self, RichText};
use egui_workbench::{Icons, StatusItem, ThemeId, TitleBar, apply_window_action, is_maximized, show_title_bar};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    eframe::run_native(
        "Component Catalog",
        egui_workbench::native_options("Component Catalog"),
        Box::new(|cc| {
            let theme = ThemeState::new(&cc.egui_ctx, ThemeId::GithubDark, false);
            Ok(Box::new(Catalog::new(theme)))
        }),
    )
}

use egui_workbench::eframe;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    Buttons,
    Inputs,
    Feedback,
    Data,
    Menus,
}

const SECTIONS: [(&str, Section); 5] = [
    ("Buttons", Section::Buttons),
    ("Inputs", Section::Inputs),
    ("Feedback", Section::Feedback),
    ("Data display", Section::Data),
    ("Menus", Section::Menus),
];

/// The live component catalog, shared by the native example and the wasm demo.
pub struct Catalog {
    theme: ThemeState,
    icons: Icons,
    section: Section,
    status: String,
    text: String,
    area: String,
    switch: bool,
    checkbox: bool,
    select: usize,
    stepper: f32,
    tab: usize,
    tree: TreeState<&'static str>,
    palette: PaletteState,
    palette_items: Vec<PaletteItem<&'static str>>,
    dialog_open: bool,
    confirm_open: bool,
    name_open: bool,
    name: String,
}

impl Catalog {
    /// Build the catalog with the given theme state.
    pub fn new(theme: ThemeState) -> Self {
        Self {
            theme,
            icons: Icons::new(),
            section: Section::Buttons,
            status: "Component catalog".to_owned(),
            text: "Workbench".to_owned(),
            area: "Multi-line content".to_owned(),
            switch: true,
            checkbox: true,
            select: 1,
            stepper: 1.0,
            tab: 0,
            tree: TreeState::default(),
            palette: PaletteState::default(),
            palette_items: vec![
                PaletteItem {
                    label: "New tab".to_owned(),
                    detail: "Ctrl+N".to_owned(),
                    value: "New tab",
                },
                PaletteItem {
                    label: "Toggle navigator".to_owned(),
                    detail: "Ctrl+B".to_owned(),
                    value: "Navigator",
                },
                PaletteItem {
                    label: "Open settings".to_owned(),
                    detail: "Ctrl+,".to_owned(),
                    value: "Settings",
                },
            ],
            dialog_open: false,
            confirm_open: false,
            name_open: false,
            name: "Untitled".to_owned(),
        }
    }

    /// The catalog's theme state, for hosts that switch themes from outside.
    pub fn theme_mut(&mut self) -> &mut ThemeState {
        &mut self.theme
    }

    fn draw_title_bar(&mut self, ui: &mut egui::Ui) {
        let mut bar = TitleBar::new("Component Catalog");
        bar.maximized = is_maximized(ui.ctx());
        let result = show_title_bar(ui, &mut self.icons, &bar, |_, _| {}, |_, _| {});
        if let Some(action) = result.window {
            apply_window_action(ui.ctx(), action);
        }
    }

    fn section_buttons(&mut self, ui: &mut egui::Ui) {
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
                    self.status = format!("Button: {variant:?}");
                }
            }
        });
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            for size in [Size::Sm, Size::Md, Size::Lg] {
                if Button::new("Sized").size(size).show(ui).clicked() {
                    self.status = format!("Button size: {size:?}");
                }
            }
            Button::new("Disabled").enabled(false).show(ui);
            Button::new("Selected").selected(true).show(ui);
        });
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            for (icon, tooltip, size) in [
                ("settings", "Action button", IconButtonSize::Action),
                ("plus", "Tool button", IconButtonSize::Tool),
                ("refresh", "Small button", IconButtonSize::Small),
            ] {
                if IconButton::new(icon, tooltip)
                    .size(size)
                    .show(ui, &mut self.icons)
                    .clicked()
                {
                    self.status = format!("Icon button: {tooltip}");
                }
            }
            IconButton::new("search", "Disabled")
                .enabled(false)
                .show(ui, &mut self.icons);
        });
    }

    fn section_inputs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                TextField::new(&mut self.text)
                    .label("Name")
                    .hint("Type a name")
                    .width(240.0)
                    .show(ui);
                ui.add_space(8.0);
                TextField::new(&mut self.area)
                    .label("Disabled")
                    .width(240.0)
                    .enabled(false)
                    .show(ui);
                ui.add_space(8.0);
                if Switch::new(&mut self.switch).label("Enable preview").show(ui) {
                    self.status = format!("Switch: {}", self.switch);
                }
                if Checkbox::new(&mut self.checkbox).label("Wrap long lines").show(ui) {
                    self.status = format!("Checkbox: {}", self.checkbox);
                }
            });
            ui.add_space(24.0);
            ui.vertical(|ui| {
                let options = [
                    (0, "Automatic".to_owned()),
                    (1, "Light".to_owned()),
                    (2, "Dark".to_owned()),
                ];
                if Select::new("Theme mode", &mut self.select, &options).show(ui) {
                    self.status = format!("Select: {}", options[self.select].1);
                }
                ui.add_space(8.0);
                if Stepper::new(&mut self.stepper, 0.5..=2.0, 0.1)
                    .decimals(1)
                    .suffix("×")
                    .show(ui, &mut self.icons)
                {
                    self.status = format!("Stepper: {:.1}", self.stepper);
                }
                ui.add_space(8.0);
                TextArea::new(&mut self.area)
                    .label("Notes")
                    .rows(4)
                    .width(300.0)
                    .show(ui);
            });
        });
    }

    fn section_feedback(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for (label, tone) in [
                ("Neutral", Tone::Neutral),
                ("Muted", Tone::Muted),
                ("Accent", Tone::Accent),
                ("Warning", Tone::Warning),
                ("Danger", Tone::Danger),
            ] {
                Badge::new(label).tone(tone).show(ui);
            }
            Badge::new("Success")
                .tone(Tone::Custom(self.theme.tokens().success))
                .dot(true)
                .show(ui);
        });
        ui.add_space(10.0);
        Banner::new("The file changed on disk.")
            .tone(Tone::Warning)
            .show(ui, |ui| {
                if Button::new("Reload").size(Size::Sm).show(ui).clicked() {
                    self.status = "Banner action: Reload".to_owned();
                }
            });
        ui.add_space(10.0);
        Label::new("Muted label").tone(Tone::Muted).show(ui);
        Label::new("Danger label").tone(Tone::Danger).show(ui);
    }

    fn section_data(&mut self, ui: &mut egui::Ui) {
        let labels = ["Overview", "Details", "Logs"];
        let tabs: Vec<TabItem<'_>> = labels
            .iter()
            .map(|label| TabItem::new(label, TabIcon::Glyph("files")))
            .collect();
        // TabStrip paints its whole max rect: give it a strip of its own.
        let strip = egui::vec2(ui.available_width(), egui_workbench::metrics::TAB_HEIGHT);
        let mut new_tab = false;
        let event = ui
            .allocate_ui(strip, |ui| {
                ui.set_max_size(strip);
                TabStrip::new(&tabs, self.tab)
                    .controls_width(80.0)
                    .show(ui, &mut self.icons, |ui, icons| {
                        new_tab = IconButton::new("plus", "New tab")
                            .size(IconButtonSize::Tool)
                            .show(ui, icons)
                            .clicked();
                    })
            })
            .inner;
        if new_tab {
            self.status = "Tab strip: New tab".to_owned();
        }
        match event {
            Some(TabEvent::Select(index)) => {
                self.tab = index;
                self.status = format!("Tab: {}", labels[index]);
            }
            Some(TabEvent::Close(index)) => self.status = format!("Close requested: {}", labels[index]),
            None => {}
        }
        ui.add_space(10.0);
        Card::new()
            .title("Card")
            .description("Raised content with the shared border and radius.")
            .width(420.0)
            .show(ui, |ui| {
                ui.label("Use Card::new().show(ui, |ui| { … }) for grouped content.");
                ui.add_space(8.0);
                if Button::new("Card action")
                    .variant(ButtonVariant::Outline)
                    .show(ui)
                    .clicked()
                {
                    self.status = "Card action".to_owned();
                }
            });
        ui.add_space(6.0);
        Separator::new().show(ui);
        Label::new("Tree").strong().show(ui);
        let nodes = vec![TreeNode::folder(
            "src",
            "src",
            RowIcon::Glyph("folder-open".to_owned()),
            true,
            vec![
                TreeNode::leaf("src/main.rs", "main.rs", RowIcon::Glyph("files".to_owned())),
                TreeNode::leaf("src/model.rs", "model.rs", RowIcon::Glyph("files".to_owned())),
            ],
        )];
        let events = Tree::new(&nodes, &mut self.tree, "catalog_tree").show(ui, &mut self.icons, |_, _, _| {});
        for event in events {
            self.status = format!("Tree: {event:?}");
        }
    }

    fn section_menus(&mut self, ui: &mut egui::Ui) {
        let mut chosen = None;
        Menu::new("Actions").width(210.0).show(ui, |ui| {
            if MenuButton::new("Open settings").show(ui) {
                chosen = Some("Open settings");
            }
            ui.separator();
            if MenuItem::new("save", "Save")
                .shortcut("Ctrl+S")
                .show(ui, &mut self.icons)
            {
                chosen = Some("Save");
            }
            if MenuItem::new("close", "Delete")
                .danger(true)
                .enabled(false)
                .show(ui, &mut self.icons)
            {
                chosen = Some("Delete");
            }
        });
        MenuIconButton::new("more", "More actions").show(ui, &mut self.icons, |ui, icons| {
            if MenuItem::new("refresh", "Refresh").show(ui, icons) {
                chosen = Some("Refresh");
            }
        });
        if let Some(chosen) = chosen {
            self.status = format!("Menu: {chosen}");
        }
        ui.add_space(12.0);
        if Button::new("Open custom dialog").show(ui).clicked() {
            self.dialog_open = true;
        }
        if Button::new("Open confirm dialog")
            .variant(ButtonVariant::Secondary)
            .show(ui)
            .clicked()
        {
            self.confirm_open = true;
        }
        if Button::new("Open name dialog")
            .variant(ButtonVariant::Secondary)
            .show(ui)
            .clicked()
        {
            self.name_open = true;
        }
    }
}

impl WorkbenchApp for Catalog {
    type Command = ();

    fn title_bar_visible(&self) -> bool {
        true
    }

    fn sidebar_visible(&self) -> bool {
        true
    }

    fn sidebar_width(&self) -> f32 {
        200.0
    }

    fn tab_strip_visible(&self) -> bool {
        false
    }

    fn status_bar_visible(&self) -> bool {
        true
    }

    fn title_bar(&mut self, ui: &mut egui::Ui) -> Vec<()> {
        self.draw_title_bar(ui);
        Vec::new()
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) -> Vec<()> {
        ui.set_min_width(ui.available_width());
        ui.add_space(6.0);
        Label::new("SECTIONS").size(Size::Sm).strong().show(ui);
        ui.add_space(4.0);
        for (label, section) in SECTIONS {
            if Button::new(label)
                .variant(ButtonVariant::Ghost)
                .selected(self.section == section)
                .show(ui)
                .clicked()
            {
                self.section = section;
            }
        }
        ui.add_space(12.0);
        Separator::new().show(ui);
        ui.add_space(6.0);
        let mut theme = self.theme.requested();
        let options: Vec<(ThemeId, String)> = ThemeId::ALL.iter().map(|id| (*id, id.label().to_owned())).collect();
        if Select::new("Theme", &mut theme, &options).width(170.0).show(ui) {
            self.theme.set_requested(ui.ctx(), theme);
        }
        let mut reduce = self.theme.reduced_motion();
        if Switch::new(&mut reduce).label("Reduced motion").show(ui) {
            self.theme.set_reduced_motion(ui.ctx(), reduce);
        }
        Vec::new()
    }

    fn central(&mut self, ui: &mut egui::Ui) -> Vec<()> {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            let title = SECTIONS
                .iter()
                .find(|(_, section)| *section == self.section)
                .map_or("Components", |(label, _)| label);
            ui.heading(title);
            ui.add_space(4.0);
            ui.label(RichText::new("Reactive builders over the workbench primitives.").weak());
            ui.add_space(16.0);
            match self.section {
                Section::Buttons => self.section_buttons(ui),
                Section::Inputs => self.section_inputs(ui),
                Section::Feedback => self.section_feedback(ui),
                Section::Data => self.section_data(ui),
                Section::Menus => self.section_menus(ui),
            }
        });
        Vec::new()
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) -> Vec<()> {
        StatusBar::new()
            .left(StatusItem::new(self.status.clone()))
            .right(StatusItem::new(self.theme.requested().label()))
            .right(StatusItem::new("components"))
            .show(ui);
        Vec::new()
    }

    fn overlays(&mut self, ui: &mut egui::Ui) -> Vec<()> {
        let ctx = ui.ctx().clone();
        let labels = PaletteLabels {
            title: "Command Palette",
            hint: "Type to filter…",
            empty: "No matches",
            close: "Close",
        };
        if let Some(chosen) = CommandPalette::new(&mut self.palette, labels, &self.palette_items).show(&ctx) {
            self.status = format!("Palette: {chosen}");
        }
        if self.dialog_open {
            Dialog::new("catalog_dialog", "Custom dialog", &mut self.dialog_open)
                .width(420.0)
                .show(&ctx, |ui| {
                    Label::new("Dialog body").strong().show(ui);
                    ui.label("Compose the body and footer with the component builders.");
                    ui.add_space(12.0);
                    ui.label("Close with ×, Escape, or a backdrop click.");
                });
        }
        if self.confirm_open
            && ConfirmDialog::new("Delete item?", "This cannot be undone.")
                .confirm("Delete")
                .danger(true)
                .show(&ctx)
                .is_some()
        {
            self.confirm_open = false;
        }
        if self.name_open && NameDialog::new("Rename", "Name", &mut self.name).show(&ctx).is_some() {
            self.name_open = false;
        }
        Vec::new()
    }
}

impl eframe::App for Catalog {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme.sync_system(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let _ = show_workbench(ui, self);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.theme.clear_color()
    }
}
