//! The starter gallery: every component, variant, and state in one page.
//!
//! This module is demo-only. Delete it and replace `content.rs` when starting the real
//! product; the shell, commands, and model do not depend on it.

use egui_workbench::components::prelude::*;
use egui_workbench::egui::{self, RichText};
use egui_workbench::{Icons, Segment, TabEvent, TabIcon, TabItem, TreeNode, TreeState};

use crate::app::App;
use crate::commands::Command;

/// Draw the gallery section of the active tab.
pub fn show(app: &mut App, ui: &mut egui::Ui) -> Vec<Command> {
    let mut commands = Vec::new();
    let Some(tab) = app.tabs.active_mut() else {
        return commands;
    };
    let icons = &mut app.icons;
    let tree_state: &mut TreeState<&'static str> = &mut app.tree;

    let toasts = &mut app.toasts;

    ui.heading(tab.section.label());
    ui.label(RichText::new(tab.section.description()).small().weak());
    ui.add_space(10.0);
    if tab.section == crate::model::GallerySection::Icons {
        // The icon browser owns its scroll area so the grid can fill the remaining
        // height and only lay out the visible rows.
        crate::icon_browser::show(ui, icons, &mut tab.data.demo);
    } else {
        egui::ScrollArea::vertical()
            .id_salt(("gallery_scroll", tab.id))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                match tab.section {
                    crate::model::GallerySection::Icons => {}
                    crate::model::GallerySection::Buttons => buttons(ui, icons, tab.data.demo.message.as_str()),
                    crate::model::GallerySection::Inputs => inputs(ui, tab, icons),
                    crate::model::GallerySection::Feedback => feedback(ui, toasts, &mut tab.data.demo),
                    crate::model::GallerySection::Menus => menus(ui, icons, &mut tab.data.demo),
                    crate::model::GallerySection::Data => data(ui, icons, tree_state, &mut tab.data.demo),
                    crate::model::GallerySection::Charts => {
                        crate::charts::show(ui, icons, &mut tab.data.demo.charts, toasts)
                    }
                    crate::model::GallerySection::Actions => actions(ui, &mut commands, &mut tab.data.demo),
                }
                if !tab.data.demo.message.is_empty() {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("Last event: {}", tab.data.demo.message))
                            .color(ui.visuals().selection.stroke.color),
                    );
                }
            });
    }
    if tab.data.demo.popover {
        let mut close = false;
        Popover::new("gallery_popover").width(360.0).show(ui.ctx(), |ui| {
            Label::new("Popover").strong().show(ui);
            ui.label("Anchored below the title bar, above every panel.");
            if Button::new("Close").size(Size::Sm).show(ui).clicked() {
                close = true;
            }
        });
        if close {
            tab.data.demo.popover = false;
        }
    }
    commands
}

fn buttons(ui: &mut egui::Ui, icons: &mut Icons, message: &str) {
    ui.horizontal_wrapped(|ui| {
        for (label, variant) in [
            ("Primary", ButtonVariant::Primary),
            ("Secondary", ButtonVariant::Secondary),
            ("Outline", ButtonVariant::Outline),
            ("Ghost", ButtonVariant::Ghost),
            ("Destructive", ButtonVariant::Destructive),
            ("Link", ButtonVariant::Link),
        ] {
            Button::new(label).variant(variant).show(ui);
        }
    });
    ui.add_space(6.0);
    // Mixed heights: a fixed row height centers Sm/Md/Lg on one midline.
    ui.horizontal_wrapped(|ui| {
        ui.set_row_height(Size::Lg.height());
        for size in [Size::Sm, Size::Md, Size::Lg] {
            Button::new("Sized").size(size).show(ui);
        }
        Button::new("Disabled").enabled(false).show(ui);
        Button::new("Selected").selected(true).show(ui);
    });
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        for (icon, tooltip, size) in [
            ("lucide-heart", "Lucide heart", IconButtonSize::Action),
            ("lucide-trash-2", "Delete", IconButtonSize::Tool),
            ("lucide-circle-help", "Help", IconButtonSize::Small),
        ] {
            IconButton::new(icon, tooltip).size(size).show(ui, icons);
        }
        IconButton::new("lucide-settings", "Disabled")
            .enabled(false)
            .show(ui, icons);
        CloseButton::new("Close").size(14.0).show(ui);
        if !message.is_empty() {
            ui.label(RichText::new(message).small().weak());
        }
    });
}

fn inputs(ui: &mut egui::Ui, tab: &mut crate::model::Tab, icons: &mut Icons) {
    // Two columns when they fit side by side; one column on narrow windows.
    let wide = ui.available_width() >= 620.0;
    let layout = if wide {
        egui::Layout::left_to_right(egui::Align::Min)
    } else {
        egui::Layout::top_down(egui::Align::Min)
    };
    let field = ui.available_width().min(300.0);
    ui.with_layout(layout, |ui| {
        ui.vertical(|ui| {
            if TextField::new(&mut tab.data.demo.text)
                .label("Name")
                .hint("Type a name")
                .width(field)
                .show(ui)
                .changed()
            {
                tab.dirty = true;
            }
            ui.add_space(6.0);
            if TextArea::new(&mut tab.data.demo.area)
                .label("Notes")
                .rows(3)
                .width(field)
                .show(ui)
                .changed()
            {
                tab.dirty = true;
            }
        });
        ui.add_space(if wide { 24.0 } else { 12.0 });
        ui.vertical(|ui| {
            if Switch::new(&mut tab.data.demo.switch).label("Enable preview").show(ui) {
                tab.data.demo.message = format!("Switch: {}", tab.data.demo.switch);
            }
            if Checkbox::new(&mut tab.data.demo.check)
                .label("Wrap long lines")
                .show(ui)
            {
                tab.data.demo.message = format!("Checkbox: {}", tab.data.demo.check);
            }
            let options = [
                (0, "Automatic".to_owned()),
                (1, "Light".to_owned()),
                (2, "Dark".to_owned()),
            ];
            if Select::new("Theme mode", &mut tab.data.demo.select, &options).show(ui) {
                tab.data.demo.message = format!("Select: {}", options[tab.data.demo.select].1);
            }
            if Stepper::new(&mut tab.data.demo.number, 0.5..=2.0, 0.1)
                .suffix("×")
                .show(ui, icons)
            {
                tab.data.demo.message = format!("Stepper: {:.1}", tab.data.demo.number);
            }
            ui.add_space(6.0);
            let segments = [
                Segment {
                    icon: "lucide-eye",
                    tooltip: "View",
                    enabled: true,
                },
                Segment {
                    icon: "lucide-pencil",
                    tooltip: "Edit",
                    enabled: true,
                },
                Segment {
                    icon: "lucide-columns-2",
                    tooltip: "Split",
                    enabled: false,
                },
            ];
            if let Some(index) = SegmentedControl::new("gallery_mode", &segments, tab.data.demo.mode).show(ui, icons) {
                tab.data.demo.mode = index;
            }
        });
    });
}

fn feedback(ui: &mut egui::Ui, toasts: &mut Toasts, demo: &mut crate::model::DemoState) {
    Label::new("Toasts").strong().show(ui);
    ui.label(
        RichText::new("Stacked in the bottom-right corner; hover to pause the timer.")
            .small()
            .weak(),
    );
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        if Button::new("Success")
            .variant(ButtonVariant::Outline)
            .show(ui)
            .clicked()
        {
            toasts.add(Toast::new(ToastKind::Success, "Changes saved").description("Your workspace is up to date."));
        }
        if Button::new("Info").variant(ButtonVariant::Outline).show(ui).clicked() {
            toasts.add(
                Toast::new(ToastKind::Info, "New version available")
                    .description("Restart the application to update.")
                    .action("Restart"),
            );
        }
        if Button::new("Warning")
            .variant(ButtonVariant::Outline)
            .show(ui)
            .clicked()
        {
            toasts.add(Toast::new(ToastKind::Warning, "Disk space is running low").duration(6.0));
        }
        if Button::new("Error").variant(ButtonVariant::Outline).show(ui).clicked() {
            toasts.add(
                Toast::new(ToastKind::Error, "Could not save the file")
                    .description("Permission denied: /etc/hosts")
                    .action("Retry"),
            );
        }
        if Button::new("Loading")
            .variant(ButtonVariant::Outline)
            .show(ui)
            .clicked()
        {
            demo.toast_loading = Some(toasts.loading("Exporting report…"));
        }
        if let Some(id) = demo.toast_loading
            && Button::new("Finish export")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
        {
            toasts.update(
                id,
                Toast::new(ToastKind::Success, "Report exported").description("report.pdf · 2.4 MB"),
            );
            demo.toast_loading = None;
        }
        if Button::new("Dismiss all")
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked()
        {
            toasts.dismiss_all();
            demo.toast_loading = None;
        }
    });
    ui.add_space(12.0);
    Separator::new().show(ui);
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        Badge::new("Neutral").show(ui);
        Badge::new("Muted").tone(Tone::Muted).show(ui);
        Badge::new("Accent").tone(Tone::Accent).show(ui);
        Badge::new("Warning").tone(Tone::Warning).show(ui);
        Badge::new("Danger").tone(Tone::Danger).show(ui);
    });
    ui.add_space(6.0);
    Banner::new("A banner sits above content.")
        .tone(Tone::Warning)
        .show(ui, |ui| {
            if Button::new("Reload").size(Size::Sm).show(ui).clicked() {
                demo.message = "Banner: Reload".to_owned();
            }
            if CloseButton::new("Dismiss").show(ui) {
                demo.message = "Banner: Dismiss".to_owned();
            }
        });
    Card::new()
        .title("Card")
        .description("A raised surface with a title, description, and body.")
        .width(420.0)
        .show(ui, |ui| {
            Label::new("Card body").tone(Tone::Muted).show(ui);
            if Button::new("Card action")
                .variant(ButtonVariant::Outline)
                .show(ui)
                .clicked()
            {
                demo.message = "Card action".to_owned();
            }
        });
    ui.add_space(6.0);
    Separator::new().show(ui);
    Label::new("Muted label").tone(Tone::Muted).show(ui);
    Label::new("Danger label").tone(Tone::Danger).show(ui);
}

fn menus(ui: &mut egui::Ui, icons: &mut Icons, demo: &mut crate::model::DemoState) {
    let mut chosen = None;
    ui.horizontal(|ui| {
        Menu::new("Actions").width(220.0).show(ui, |ui| {
            if MenuButton::new("Open settings").show(ui) {
                chosen = Some("Settings");
            }
            if MenuButton::new("Show dialog demo").show(ui) {
                chosen = Some("Dialog");
            }
            ui.separator();
            if MenuItem::new("lucide-save", "Save").shortcut("Ctrl+S").show(ui, icons) {
                chosen = Some("Save");
            }
            if MenuItem::new("lucide-trash-2", "Delete")
                .danger(true)
                .enabled(false)
                .show(ui, icons)
            {
                chosen = Some("Delete");
            }
            if MenuToggle::new("Word wrap", demo.check).show(ui) {
                demo.check = !demo.check;
                chosen = Some("Word wrap");
            }
        });
        MenuIconButton::new("lucide-ellipsis", "More actions").show(ui, icons, |ui, icons| {
            if MenuItem::new("lucide-refresh", "Refresh").show(ui, icons) {
                chosen = Some("Refresh");
            }
            if MenuItem::new("lucide-heart", "Favorite").show(ui, icons) {
                chosen = Some("Favorite");
            }
        });
    });
    if let Some(chosen) = chosen {
        demo.message = format!("Menu: {chosen}");
    }
}

fn data(ui: &mut egui::Ui, icons: &mut Icons, tree: &mut TreeState<&'static str>, demo: &mut crate::model::DemoState) {
    let nodes = vec![TreeNode::folder(
        "src",
        "src",
        RowIcon::Glyph("lucide-folder-open".to_owned()),
        true,
        vec![
            TreeNode::leaf("src/main.rs", "main.rs", RowIcon::Glyph("lucide-file-text".to_owned())),
            TreeNode::leaf(
                "src/model.rs",
                "model.rs",
                RowIcon::Glyph("lucide-file-text".to_owned()),
            ),
        ],
    )];
    ui.allocate_ui(egui::vec2(ui.available_width(), 160.0), |ui| {
        let events = Tree::new(&nodes, tree, "gallery_tree").show(ui, icons, |_, _, _| {});
        for event in events {
            demo.message = format!("Tree: {event:?}");
        }
    });

    ui.add_space(8.0);
    ui.label(RichText::new("Split panes").small().weak());
    ui.allocate_ui(egui::vec2(ui.available_width(), 180.0), |ui| {
        let mut ratio = demo.split;
        egui_workbench::split_panes(
            ui,
            &mut ratio,
            |ui| {
                egui::Frame::new().inner_margin(8).show(ui, |ui| {
                    ui.label("First pane");
                });
            },
            |ui| {
                egui::Frame::new().inner_margin(8).show(ui, |ui| {
                    ui.label("Second pane");
                });
            },
        );
        demo.split = ratio;
    });
    ui.add_space(8.0);
    let labels = ["One", "Two", "Three"];
    let tabs: Vec<TabItem<'_>> = labels
        .iter()
        .map(|label| TabItem::new(label, TabIcon::Glyph("lucide-file-text")))
        .collect();
    let event = ui
        .allocate_ui(egui::vec2(ui.available_width(), 34.0), |ui| {
            TabStrip::new(&tabs, demo.mini_tab)
                .controls_width(40.0)
                .show(ui, icons, |ui, icons| {
                    if IconButton::new("lucide-plus", "New")
                        .size(IconButtonSize::Tool)
                        .show(ui, icons)
                        .clicked()
                    {
                        demo.message = "Tab strip: New".to_owned();
                    }
                })
        })
        .inner;
    if let Some(TabEvent::Select(index)) = event {
        demo.mini_tab = index;
    }
}

fn actions(ui: &mut egui::Ui, commands: &mut Vec<Command>, demo: &mut crate::model::DemoState) {
    ui.horizontal_wrapped(|ui| {
        if Button::new("Command palette")
            .variant(ButtonVariant::Secondary)
            .show(ui)
            .clicked()
        {
            commands.push(Command::Palette);
        }
        if Button::new("Settings")
            .variant(ButtonVariant::Secondary)
            .show(ui)
            .clicked()
        {
            commands.push(Command::Settings);
        }
        if Button::new("About")
            .variant(ButtonVariant::Secondary)
            .show(ui)
            .clicked()
        {
            commands.push(Command::About);
        }
        if Button::new("Custom dialog").show(ui).clicked() {
            commands.push(Command::ShowDialog);
        }
        if Button::new("Confirm dialog").show(ui).clicked() {
            commands.push(Command::ShowConfirm);
        }
        if Button::new("Name dialog").show(ui).clicked() {
            commands.push(Command::ShowName);
        }
        if Button::new("Progress").show(ui).clicked() {
            commands.push(Command::ShowProgress);
        }
        if Button::new("Keyboard menu").show(ui).clicked() {
            commands.push(Command::KeyboardMenu);
        }
        if Button::new("Popover")
            .variant(ButtonVariant::Secondary)
            .show(ui)
            .clicked()
        {
            demo.popover = true;
        }
    });
}
