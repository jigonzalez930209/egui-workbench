//! The Icons gallery section: a searchable, responsive browser of the Lucide set.

use egui_workbench::Icons;
use egui_workbench::components::prelude::*;
use egui_workbench::egui::{self, RichText};

/// Draw the section; the grid owns its scroll area and fills the remaining height.
pub fn show(ui: &mut egui::Ui, icons: &mut Icons, demo: &mut crate::model::DemoState) {
    ui.horizontal_wrapped(|ui| {
        TextField::new(&mut demo.icon_query)
            .hint("Search icons by name…")
            .width(ui.available_width().clamp(160.0, 320.0))
            .show(ui);
        ui.add_space(8.0);
        let sizes = [
            (0, "Auto".to_owned()),
            (1, "16 pt".to_owned()),
            (2, "24 pt".to_owned()),
            (3, "32 pt".to_owned()),
            (4, "48 pt".to_owned()),
        ];
        Label::new("Size").tone(Tone::Muted).show(ui);
        Select::new("Size", &mut demo.icon_size, &sizes).width(88.0).show(ui);
        let layouts = [(0, "Grid".to_owned()), (1, "List".to_owned())];
        Label::new("Layout").tone(Tone::Muted).show(ui);
        Select::new("Layout", &mut demo.icon_layout, &layouts)
            .width(96.0)
            .show(ui);
    });

    let names = Icons::lucide_search(&demo.icon_query);
    let size = match demo.icon_size {
        1 => IconSize::Fixed(16.0),
        2 => IconSize::Fixed(24.0),
        3 => IconSize::Fixed(32.0),
        4 => IconSize::Fixed(48.0),
        _ => IconSize::Auto,
    };
    let layout = if demo.icon_layout == 1 {
        IconLayout::List
    } else {
        IconLayout::Grid
    };
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{} icons", names.len())).small().weak());
        if let Some(selected) = &demo.icon_selected {
            ui.label(RichText::new("·").small().weak());
            ui.label(RichText::new(format!("lucide-{selected}")).small().monospace());
        }
    });
    ui.add_space(6.0);

    if names.is_empty() {
        ui.add_space(24.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("No icons match the search.").weak());
        });
        return;
    }
    if let Some(index) = IconGrid::new("lucide_icon_browser", &names)
        .prefix("lucide-")
        .size(size)
        .layout(layout)
        .selected(demo.icon_selected.as_deref())
        .show(ui, icons)
    {
        demo.message = format!("Lucide: {}", names[index]);
        demo.icon_selected = Some(names[index].clone());
    }
}
