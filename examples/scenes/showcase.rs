//! The icon browser and the full component catalog.

use egui_workbench::ThemeId;
use egui_workbench::app::ThemeState;
use egui_workbench::components::prelude::*;
use egui_workbench::egui;

use super::{Host, Scene, catalog};

/// Search 1,600+ Lucide icons in a responsive, virtualized grid.
pub struct IconBrowser {
    query: String,
    names: Vec<String>,
    searched: Option<String>,
    selected: Option<String>,
    list: bool,
}

impl Default for IconBrowser {
    fn default() -> Self {
        Self {
            query: "file".to_owned(),
            names: Vec::new(),
            searched: None,
            selected: None,
            list: false,
        }
    }
}

impl Scene for IconBrowser {
    fn scroll(&self) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        ui.horizontal_wrapped(|ui| {
            TextField::new(&mut self.query)
                .hint("Search icons")
                .width(220.0_f32.min(ui.available_width()))
                .show(ui);
            let mut list = usize::from(self.list);
            egui_workbench::combo(
                ui,
                "Layout",
                &mut list,
                &[(0, "Grid".to_owned()), (1, "List".to_owned())],
            );
            self.list = list == 1;
            Label::new(&format!("{} matches", self.names.len()))
                .tone(Tone::Muted)
                .size(Size::Sm)
                .show(ui);
        });
        ui.add_space(8.0);
        if self.searched.as_deref() != Some(self.query.as_str()) {
            self.names = egui_workbench::Icons::lucide_search(self.query.trim());
            self.searched = Some(self.query.clone());
        }
        let layout = if self.list { IconLayout::List } else { IconLayout::Grid };
        if let Some(index) = IconGrid::new("scene-icons", &self.names)
            .prefix("lucide-")
            .size(IconSize::Auto)
            .layout(layout)
            .selected(self.selected.as_deref())
            .show(ui, host.icons)
        {
            let name = self.names[index].clone();
            host.toasts.dismiss_all();
            host.report(format!("\"lucide-{name}\""));
            self.selected = Some(name);
        }
    }
}

/// The complete catalog (the `/guide/live-demo` page).
#[derive(Default)]
pub struct Catalog {
    catalog: Option<catalog::Catalog>,
    theme: Option<ThemeId>,
}

impl Scene for Catalog {
    fn full(&self) -> bool {
        true
    }

    fn ui(&mut self, ui: &mut egui::Ui, host: &mut Host<'_>) {
        let requested = host.theme.requested();
        let catalog = self
            .catalog
            .get_or_insert_with(|| catalog::Catalog::new(ThemeState::new(ui.ctx(), requested, false)));
        if self.theme != Some(requested) {
            self.theme = Some(requested);
            catalog.theme_mut().set_requested(ui.ctx(), requested);
        }
        let _ = egui_workbench::app::show_workbench(ui, catalog);
    }
}
