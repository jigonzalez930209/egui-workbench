//! Buttons, inputs, and display components, each as the docs page shows them.

use egui_workbench::components::prelude::*;
use egui_workbench::egui;

use super::{Host, Scene};

/// Variants, sizes, states, and icon buttons.
pub struct Buttons;

impl Scene for Buttons {
    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        Label::new("Variants").strong().show(ui);
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
                    host.report(format!("ButtonVariant::{variant:?} clicked"));
                }
            }
        });
        ui.add_space(10.0);
        Label::new("Sizes and states").strong().show(ui);
        ui.horizontal_wrapped(|ui| {
            for size in [Size::Sm, Size::Md, Size::Lg] {
                if Button::new(&format!("Size::{size:?}")).size(size).show(ui).clicked() {
                    host.report(format!("Size::{size:?} clicked"));
                }
            }
            Button::new("Disabled").enabled(false).show(ui);
            if Button::new("Selected")
                .variant(ButtonVariant::Ghost)
                .selected(true)
                .show(ui)
                .clicked()
            {
                host.report("Selected clicked");
            }
            if Button::new("With tooltip")
                .variant(ButtonVariant::Outline)
                .tooltip("Tooltips come from the builder")
                .show(ui)
                .clicked()
            {
                host.report("Tooltip button clicked");
            }
        });
        ui.add_space(10.0);
        Label::new("Icon buttons").strong().show(ui);
        ui.horizontal_wrapped(|ui| {
            for (icon, tooltip, size) in [
                ("settings", "IconButtonSize::Action", IconButtonSize::Action),
                ("plus", "IconButtonSize::Tool", IconButtonSize::Tool),
                ("refresh", "IconButtonSize::Small", IconButtonSize::Small),
            ] {
                if IconButton::new(icon, tooltip).size(size).show(ui, host.icons).clicked() {
                    host.report(format!("{tooltip} clicked"));
                }
            }
            IconButton::new("search", "Disabled")
                .enabled(false)
                .show(ui, host.icons);
        });
    }
}

/// Text fields, switches, checkboxes, selects, steppers, and text areas.
pub struct Inputs {
    name: String,
    notes: String,
    preview: bool,
    wrap: bool,
    mode: usize,
    zoom: f32,
}

impl Default for Inputs {
    fn default() -> Self {
        Self {
            name: "Workbench".to_owned(),
            notes: "Multi-line content\nwraps and scrolls.".to_owned(),
            preview: true,
            wrap: false,
            mode: 1,
            zoom: 1.0,
        }
    }
}

impl Scene for Inputs {
    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        ui.horizontal_wrapped(|ui| {
            ui.vertical(|ui| {
                if TextField::new(&mut self.name)
                    .label("Name")
                    .hint("Type a name")
                    .width(240.0)
                    .show(ui)
                    .changed()
                {
                    host.toasts.dismiss_all();
                    host.report(format!("Name: “{}”", self.name));
                }
                ui.add_space(8.0);
                let mut locked = "Read-only".to_owned();
                TextField::new(&mut locked)
                    .label("Disabled")
                    .width(240.0)
                    .enabled(false)
                    .show(ui);
                ui.add_space(8.0);
                if Switch::new(&mut self.preview).label("Enable preview").show(ui) {
                    host.report(format!("Switch: {}", self.preview));
                }
                if Checkbox::new(&mut self.wrap).label("Wrap long lines").show(ui) {
                    host.report(format!("Checkbox: {}", self.wrap));
                }
            });
            ui.add_space(16.0);
            ui.vertical(|ui| {
                let options = [
                    (0, "Automatic".to_owned()),
                    (1, "Light".to_owned()),
                    (2, "Dark".to_owned()),
                ];
                if Select::new("Theme mode", &mut self.mode, &options).show(ui) {
                    host.report(format!("Select: {}", options[self.mode].1));
                }
                ui.add_space(8.0);
                if Stepper::new(&mut self.zoom, 0.5..=2.0, 0.1)
                    .decimals(1)
                    .suffix("×")
                    .show(ui, host.icons)
                {
                    host.report(format!("Stepper: {:.1}×", self.zoom));
                }
                ui.add_space(8.0);
                TextArea::new(&mut self.notes)
                    .label("Notes")
                    .rows(3)
                    .width(260.0)
                    .show(ui);
            });
        });
    }
}

/// Badges, banners, labels, cards, tabs, and a tree.
#[derive(Default)]
pub struct Display {
    tab: usize,
    tree: TreeState<&'static str>,
    banner: bool,
}

impl Scene for Display {
    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
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
                .tone(Tone::Custom(host.theme.tokens().success))
                .dot(true)
                .show(ui);
        });
        ui.add_space(10.0);
        if self.banner {
            if Button::new("Show the banner again")
                .variant(ButtonVariant::Outline)
                .size(Size::Sm)
                .show(ui)
                .clicked()
            {
                self.banner = false;
            }
        } else {
            let mut dismissed = false;
            Banner::new("The file changed on disk.")
                .tone(Tone::Warning)
                .show(ui, |ui| {
                    if Button::new("Reload").size(Size::Sm).show(ui).clicked() {
                        host.report("Banner action: Reload");
                        dismissed = true;
                    }
                    if Button::new("Keep mine")
                        .variant(ButtonVariant::Ghost)
                        .size(Size::Sm)
                        .show(ui)
                        .clicked()
                    {
                        host.report("Banner action: Keep mine");
                        dismissed = true;
                    }
                });
            self.banner = dismissed;
        }
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            Label::new("Default label").show(ui);
            Label::new("Muted").tone(Tone::Muted).show(ui);
            Label::new("Accent").tone(Tone::Accent).show(ui);
            Label::new("Danger").tone(Tone::Danger).show(ui);
            Label::new("Strong").strong().show(ui);
        });
        ui.add_space(10.0);
        let labels = ["Overview", "Details", "Logs"];
        let tabs: Vec<TabItem<'_>> = labels
            .iter()
            .map(|label| TabItem::new(label, TabIcon::Glyph("files")))
            .collect();
        // TabStrip paints its whole max rect: give it a strip of its own.
        let strip = egui::vec2(ui.available_width(), egui_workbench::metrics::TAB_HEIGHT);
        let event = ui
            .allocate_ui(strip, |ui| {
                ui.set_max_size(strip);
                TabStrip::new(&tabs, self.tab).show(ui, host.icons, |_, _| {})
            })
            .inner;
        match event {
            Some(TabEvent::Select(index)) => {
                self.tab = index;
                host.report(format!("TabEvent::Select({index})"));
            }
            Some(TabEvent::Close(index)) => host.report(format!("TabEvent::Close({index}): the app decides")),
            None => {}
        }
        ui.add_space(10.0);
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                Card::new()
                    .title("Card")
                    .description("Grouped content with the shared border.")
                    .width(260.0)
                    .show(ui, |ui| {
                        Label::new(&format!("Active tab: {}", labels[self.tab]))
                            .tone(Tone::Muted)
                            .show(ui);
                    });
            });
            ui.add_space(12.0);
            ui.vertical(|ui| {
                let nodes = vec![TreeNode::folder(
                    "src",
                    "src",
                    RowIcon::Glyph("folder-open".to_owned()),
                    true,
                    vec![
                        TreeNode::leaf("src/main.rs", "main.rs", RowIcon::Glyph("files".to_owned())),
                        TreeNode::leaf("src/model.rs", "model.rs", RowIcon::Glyph("files".to_owned())),
                        TreeNode::leaf("src/view.rs", "view.rs", RowIcon::Glyph("files".to_owned())),
                    ],
                )];
                ui.set_width(220.0);
                for event in Tree::new(&nodes, &mut self.tree, "scene_tree").show(ui, host.icons, |_, _, _| {}) {
                    host.report(format!("{event:?}"));
                }
            });
        });
    }
}
