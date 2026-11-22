//! The chart settings dialog: edits a draft [`PlotConfig`] and applies it to the chart
//! on screen. Demo-only, like the rest of the gallery.

use egui_workbench::components::plot::{ExportOptions, PlotConfig};
use egui_workbench::components::prelude::*;
use egui_workbench::egui;

mod axes;
pub mod chart;
mod sections;

use super::charts::{Chart, ChartsDemo};

/// How charts are exported.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportSettings {
    /// Use the chart's on-screen size instead of `options.width × options.height`.
    pub screen_size: bool,
    /// Size, scale, and transparency.
    pub options: ExportOptions,
    /// Where files are written.
    pub folder: String,
}

impl Default for ExportSettings {
    fn default() -> Self {
        let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
        let folder = home
            .as_ref()
            .map(|home| home.join("Pictures"))
            .filter(|pictures| pictures.is_dir())
            .or(home)
            // No temporary directory exists in a browser (where exports are collected).
            .unwrap_or_else(|| {
                if cfg!(target_arch = "wasm32") {
                    std::path::PathBuf::from("Downloads")
                } else {
                    std::env::temp_dir()
                }
            });
        Self {
            screen_size: true,
            options: ExportOptions::default(),
            folder: folder.display().to_string(),
        }
    }
}

/// An exported chart handed to the host instead of the export folder.
///
/// The desktop app writes to the folder; hosts that set [`ChartsDemo::collected`] (the
/// docs' web demo, which turns them into downloads) read these fields.
#[allow(dead_code)]
pub struct ExportedFile {
    /// File name, such as `signal.svg`.
    pub name: String,
    /// Media type: `image/svg+xml` or `image/png`.
    pub mime: &'static str,
    /// The file's contents.
    pub bytes: Vec<u8>,
}

/// The dialog's sections.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    General,
    Titles,
    Axes,
    Grid,
    Colors,
    Series,
    Export,
}

const SECTIONS: [(Section, &str); 7] = [
    (Section::General, "General"),
    (Section::Titles, "Titles"),
    (Section::Axes, "Axes"),
    (Section::Grid, "Grid & ticks"),
    (Section::Colors, "Colors"),
    (Section::Series, "Series"),
    (Section::Export, "Export"),
];

/// An open settings dialog for one chart.
pub struct SettingsDialog {
    chart: Chart,
    draft: PlotConfig,
    section: Section,
}

impl SettingsDialog {
    /// Edit a copy of `config`.
    pub fn new(chart: Chart, config: &PlotConfig) -> Self {
        Self {
            chart,
            draft: config.clone(),
            section: Section::General,
        }
    }
}

pub(super) enum Action {
    Apply,
    Done,
    Cancel,
    Defaults,
    ResetLegend,
    Switch(Chart),
    Export(egui_workbench::components::plot::ExportFormat),
}

/// Show the dialog when `demo.settings` is set, and apply what the user chose.
pub fn show(ctx: &egui::Context, demo: &mut ChartsDemo, toasts: &mut Toasts) {
    let Some(mut dialog) = demo.settings.take() else {
        return;
    };
    let series = demo.plot(dialog.chart).series_names();
    let view = demo.current_view(dialog.chart);
    let y2_view = demo.current_y2_view(dialog.chart);
    let visuals = ctx.style_of(ctx.theme()).visuals.clone();
    let title = format!("Chart settings · {}", dialog.chart.label());
    let mut open = true;
    let mut action = None;
    Dialog::new("chart_settings", &title, &mut open)
        .width(660.0)
        .show(ctx, |ui| {
            header(ui, &mut dialog, &mut action);
            ui.add_space(10.0);
            egui::ScrollArea::vertical()
                .id_salt("chart_settings_body")
                // A constant body height: switching sections never moves the dialog.
                .max_height((ctx.content_rect().height() - 260.0).clamp(160.0, 440.0))
                .min_scrolled_height((ctx.content_rect().height() - 260.0).clamp(160.0, 440.0))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let draft = &mut dialog.draft;
                    match dialog.section {
                        Section::General => sections::general(ui, draft, &mut action),
                        Section::Titles => axes::titles(ui, draft, &visuals),
                        Section::Axes => axes::axes(ui, draft, view, y2_view, &visuals),
                        Section::Grid => sections::grid(ui, draft),
                        Section::Colors => sections::colors(ui, draft, &visuals),
                        Section::Series => sections::series_section(ui, draft, &series, &visuals),
                        Section::Export => sections::export(ui, demo, &mut action),
                    }
                });
            ui.add_space(12.0);
            Separator::new().show(ui);
            ui.add_space(10.0);
            footer(ui, &mut action);
        });
    if !open {
        action = action.or(Some(Action::Cancel));
    }
    match action {
        Some(Action::Apply) => apply(demo, &mut dialog),
        Some(Action::Done) => {
            apply(demo, &mut dialog);
            return;
        }
        Some(Action::Cancel) => return,
        Some(Action::Defaults) => dialog.draft = dialog.chart.default_config(),
        Some(Action::ResetLegend) => demo.reset_legend(dialog.chart),
        Some(Action::Switch(chart)) => {
            dialog = SettingsDialog {
                section: dialog.section,
                ..SettingsDialog::new(chart, &demo.configs[chart.index()])
            }
        }
        Some(Action::Export(format)) => demo.export(ctx, dialog.chart, format, toasts),
        None => {}
    }
    demo.settings = Some(dialog);
}

fn apply(demo: &mut ChartsDemo, dialog: &mut SettingsDialog) {
    dialog.draft.sanitize();
    let index = dialog.chart.index();
    let old = &demo.configs[index];
    let new = &dialog.draft;
    let range = |axis: &egui_workbench::components::plot::AxisConfig| (axis.min, axis.max, axis.scale);
    // Moving a series to the other axis changes both ranges too.
    let axes = |config: &PlotConfig| {
        config
            .series
            .iter()
            .map(|style| (style.name.clone(), style.y_axis))
            .collect::<Vec<_>>()
    };
    let limits_changed = (range(&old.x), range(&old.y), range(&old.y2), axes(old))
        != (range(&new.x), range(&new.y), range(&new.y2), axes(new));
    let legend_moved = old.legend != new.legend;
    demo.configs[index] = dialog.draft.clone();
    if limits_changed {
        // New limits or scales only show in the automatic view.
        demo.reset_view(dialog.chart);
    }
    if legend_moved {
        // A new corner replaces a dragged position.
        demo.reset_legend(dialog.chart);
    }
}

fn header(ui: &mut egui::Ui, dialog: &mut SettingsDialog, action: &mut Option<Action>) {
    ui.horizontal_wrapped(|ui| {
        ui.set_row_height(Size::Md.height());
        Label::new("Chart").tone(Tone::Muted).show(ui);
        let charts: Vec<(usize, String)> = Chart::ALL
            .iter()
            .map(|chart| (chart.index(), chart.label().to_owned()))
            .collect();
        let mut selected = dialog.chart.index();
        if Select::new("Chart", &mut selected, &charts).width(190.0).show(ui) {
            *action = Some(Action::Switch(Chart::ALL[selected]));
        }
    });
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (section, label) in SECTIONS {
            if Button::new(label)
                .size(Size::Sm)
                .variant(ButtonVariant::Ghost)
                .selected(dialog.section == section)
                .show(ui)
                .clicked()
            {
                dialog.section = section;
            }
        }
    });
}

fn footer(ui: &mut egui::Ui, action: &mut Option<Action>) {
    ui.horizontal(|ui| {
        if Button::new("Reset to defaults")
            .size(Size::Md)
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked()
        {
            *action = Some(Action::Defaults);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if Button::new("Done").show(ui).clicked() {
                *action = Some(Action::Done);
            }
            if Button::new("Apply")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                *action = Some(Action::Apply);
            }
            if Button::new("Cancel").variant(ButtonVariant::Outline).show(ui).clicked() {
                *action = Some(Action::Cancel);
            }
        });
    });
}
