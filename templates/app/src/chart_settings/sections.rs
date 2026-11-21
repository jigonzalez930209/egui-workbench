//! The form sections of the chart settings dialog.

use egui_workbench::components::plot::{LegendPosition, LineStyle, Marker, Minor, PlotConfig, Scale};
use egui_workbench::components::prelude::*;
use egui_workbench::egui::{self, Color32, RichText};

use super::super::charts::ChartsDemo;
use super::Action;

/// A two-column form with aligned labels.
fn form(ui: &mut egui::Ui, id: &str, add: impl FnOnce(&mut egui::Ui)) {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([18.0, 8.0])
        .min_col_width(150.0)
        .show(ui, add);
}

fn row(ui: &mut egui::Ui, label: &str, add: impl FnOnce(&mut egui::Ui)) {
    // Both cells share one control-height row, centered, so labels sit on the
    // control's midline.
    let height = Size::Md.height();
    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.set_min_height(height);
        ui.label(RichText::new(label).color(ui.visuals().weak_text_color()));
    });
    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.set_min_height(height);
        add(ui);
    });
    ui.end_row();
}

fn heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(10.0);
    Label::new(text).strong().show(ui);
    ui.add_space(6.0);
}

fn optional_text(ui: &mut egui::Ui, value: &mut Option<String>, placeholder: &str) {
    let mut shown = value.is_some();
    if Checkbox::new(&mut shown).show(ui) {
        *value = shown.then(|| placeholder.to_owned());
    }
    if let Some(text) = value {
        TextField::new(text).width(300.0).show(ui);
    }
}

fn number(ui: &mut egui::Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>, speed: f64, suffix: &str) {
    ui.add(egui::DragValue::new(value).range(range).speed(speed).suffix(suffix));
}

/// "Auto" or a number; the placeholder is what auto currently means.
fn optional_number(ui: &mut egui::Ui, value: &mut Option<f64>, placeholder: f64, speed: f64) {
    let mut auto = value.is_none();
    if Checkbox::new(&mut auto).label("Auto").show(ui) {
        *value = if auto { None } else { Some(placeholder) };
    }
    match value {
        Some(number) => {
            ui.add(egui::DragValue::new(number).speed(speed).max_decimals(6));
        }
        None => {
            ui.add_enabled(false, egui::DragValue::new(&mut placeholder.clone()).max_decimals(4));
        }
    }
}

pub(super) fn general(ui: &mut egui::Ui, draft: &mut PlotConfig) {
    heading(ui, "Text");
    form(ui, "chart_general_text", |ui| {
        row(ui, "Title", |ui| optional_text(ui, &mut draft.title, "Chart"));
        row(ui, "X axis title", |ui| optional_text(ui, &mut draft.x.label, "x"));
        row(ui, "Y axis title", |ui| optional_text(ui, &mut draft.y.label, "y"));
        row(ui, "Title size", |ui| {
            number(ui, &mut draft.title_size, 6.0..=48.0, 0.25, " pt")
        });
        row(ui, "Axis title size", |ui| {
            number(ui, &mut draft.label_size, 6.0..=36.0, 0.25, " pt")
        });
        row(ui, "Tick label size", |ui| {
            number(ui, &mut draft.tick_size, 6.0..=30.0, 0.25, " pt")
        });
        row(ui, "Legend text size", |ui| {
            number(ui, &mut draft.legend_size, 6.0..=30.0, 0.25, " pt")
        });
    });
    heading(ui, "Lines and markers");
    form(ui, "chart_general_lines", |ui| {
        row(ui, "Line width ×", |ui| {
            number(ui, &mut draft.line_scale, 0.25..=8.0, 0.02, "")
        });
        row(ui, "Marker size ×", |ui| {
            number(ui, &mut draft.marker_scale, 0.25..=8.0, 0.02, "")
        });
    });
    heading(ui, "Legend and interaction");
    form(ui, "chart_general_legend", |ui| {
        row(ui, "Legend", |ui| {
            let options = [
                (0, "Top left".to_owned()),
                (1, "Bottom left".to_owned()),
                (2, "Bottom right".to_owned()),
                (3, "Hidden".to_owned()),
            ];
            let positions = [
                LegendPosition::TopLeft,
                LegendPosition::BottomLeft,
                LegendPosition::BottomRight,
                LegendPosition::Hidden,
            ];
            let mut index = positions.iter().position(|p| *p == draft.legend).unwrap_or(0);
            if Select::new("Legend", &mut index, &options).width(160.0).show(ui) {
                draft.legend = positions[index];
            }
            Checkbox::new(&mut draft.legend_frame).label("Box").show(ui);
        });
        row(ui, "Toolbar", |ui| {
            Switch::new(&mut draft.toolbar).show(ui);
        });
        row(ui, "Hover popup", |ui| {
            Switch::new(&mut draft.hover_popup).show(ui);
        });
        row(ui, "Hover guides", |ui| {
            Switch::new(&mut draft.hover_guides).show(ui);
        });
    });
}

pub(super) fn axes(ui: &mut egui::Ui, draft: &mut PlotConfig, view: Option<egui_workbench::components::plot::Bounds>) {
    let view = view.unwrap_or(egui_workbench::components::plot::Bounds::from_corners(
        [0.0, 0.0],
        [1.0, 1.0],
    ));
    for (name, axis, (low, high)) in [
        ("X axis", &mut draft.x, (view.min_x, view.max_x)),
        ("Y axis", &mut draft.y, (view.min_y, view.max_y)),
    ] {
        heading(ui, name);
        form(ui, &format!("chart_axis_{name}"), |ui| {
            row(ui, "Scale", |ui| {
                let options = [(0, "Linear".to_owned()), (1, "Log10".to_owned())];
                let mut index = usize::from(axis.scale == Scale::Log10);
                if Select::new(&format!("{name} scale"), &mut index, &options)
                    .width(120.0)
                    .show(ui)
                {
                    axis.scale = if index == 1 { Scale::Log10 } else { Scale::Linear };
                }
            });
            row(ui, "Minimum", |ui| {
                optional_number(ui, &mut axis.min, low, (high - low).abs() / 200.0)
            });
            row(ui, "Maximum", |ui| {
                optional_number(ui, &mut axis.max, high, (high - low).abs() / 200.0)
            });
            row(ui, "", |ui| {
                if Button::new("Use current view")
                    .size(Size::Sm)
                    .variant(ButtonVariant::Outline)
                    .show(ui)
                    .clicked()
                {
                    (axis.min, axis.max) = (Some(low), Some(high));
                }
                if Button::new("Automatic")
                    .size(Size::Sm)
                    .variant(ButtonVariant::Ghost)
                    .show(ui)
                    .clicked()
                {
                    (axis.min, axis.max) = (None, None);
                }
            });
            row(ui, "Tick labels", |ui| {
                Switch::new(&mut axis.tick_labels).show(ui);
            });
            row(ui, "Label decimals", |ui| {
                let mut auto = axis.decimals.is_none();
                if Checkbox::new(&mut auto).label("Auto").show(ui) {
                    axis.decimals = if auto { None } else { Some(2) };
                }
                if let Some(decimals) = &mut axis.decimals {
                    ui.add(egui::DragValue::new(decimals).range(0..=10));
                }
            });
        });
    }
}

pub(super) fn grid(ui: &mut egui::Ui, draft: &mut PlotConfig) {
    for (name, axis) in [
        ("Vertical lines (x)", &mut draft.x),
        ("Horizontal lines (y)", &mut draft.y),
    ] {
        heading(ui, name);
        form(ui, &format!("chart_grid_{name}"), |ui| {
            row(ui, "Major step", |ui| {
                let unit = if axis.scale == Scale::Log10 { " decades" } else { "" };
                optional_number(ui, &mut axis.major_step, 1.0, 0.01);
                ui.label(RichText::new(unit).weak());
            });
            row(ui, "Minor divisions", |ui| {
                let options = [
                    (0, "Auto".to_owned()),
                    (1, "Off".to_owned()),
                    (2, "2".to_owned()),
                    (3, "4".to_owned()),
                    (4, "5".to_owned()),
                    (5, "10".to_owned()),
                ];
                let mut index = match axis.minor {
                    Minor::Auto => 0,
                    Minor::Off => 1,
                    Minor::Divisions(2) => 2,
                    Minor::Divisions(4) => 3,
                    Minor::Divisions(5) => 4,
                    Minor::Divisions(_) => 5,
                };
                if Select::new(&format!("{name} minor"), &mut index, &options)
                    .width(110.0)
                    .show(ui)
                {
                    axis.minor = match index {
                        0 => Minor::Auto,
                        1 => Minor::Off,
                        2 => Minor::Divisions(2),
                        3 => Minor::Divisions(4),
                        4 => Minor::Divisions(5),
                        _ => Minor::Divisions(10),
                    };
                }
            });
            row(ui, "Grid", |ui| {
                Checkbox::new(&mut axis.grid_major).label("Major").show(ui);
                Checkbox::new(&mut axis.grid_minor).label("Minor").show(ui);
            });
        });
    }
    heading(ui, "Lines");
    form(ui, "chart_grid_lines", |ui| {
        row(ui, "Grid width", |ui| {
            number(ui, &mut draft.grid_width, 0.5..=6.0, 0.05, " pt")
        });
        row(ui, "Major opacity", |ui| {
            number(ui, &mut draft.grid_major_opacity, 0.0..=1.0, 0.01, "");
        });
        row(ui, "Minor opacity", |ui| {
            number(ui, &mut draft.grid_minor_opacity, 0.0..=1.0, 0.01, "");
        });
        row(ui, "Frame width", |ui| {
            number(ui, &mut draft.frame_width, 0.0..=6.0, 0.05, " pt")
        });
        row(ui, "Tick length", |ui| {
            number(ui, &mut draft.tick_length, 0.0..=16.0, 0.1, " pt")
        });
    });
}

fn color_row(ui: &mut egui::Ui, label: &str, value: &mut Option<Color32>, theme: Color32) {
    row(ui, label, |ui| {
        let mut custom = value.is_some();
        if Checkbox::new(&mut custom).label("Custom").show(ui) {
            *value = custom.then_some(theme);
        }
        let mut color = value.unwrap_or(theme);
        // A 1 pt border keeps swatches visible when they match the dialog surface.
        let border = ui.visuals().window_stroke.color;
        let response = ui.add_enabled_ui(custom, |ui| {
            egui::Frame::new()
                .stroke(egui::Stroke::new(1.0, border))
                .corner_radius(4)
                .show(ui, |ui| {
                    egui::color_picker::color_edit_button_srgba(ui, &mut color, egui::color_picker::Alpha::OnlyBlend)
                })
                .inner
        });
        if response.inner.changed() {
            *value = Some(color);
        }
        if !custom {
            ui.label(RichText::new("theme").small().weak());
        }
    });
}

pub(super) fn colors(ui: &mut egui::Ui, draft: &mut PlotConfig, visuals: &egui::Visuals) {
    heading(ui, "Surfaces and text");
    let frame = visuals.window_stroke.color;
    form(ui, "chart_colors", |ui| {
        let colors = &mut draft.colors;
        color_row(ui, "Data area", &mut colors.background, visuals.extreme_bg_color);
        color_row(ui, "Around the frame", &mut colors.canvas, visuals.panel_fill);
        color_row(ui, "Frame and ticks", &mut colors.frame, frame);
        color_row(ui, "Grid", &mut colors.grid, frame);
        color_row(ui, "Titles", &mut colors.text, visuals.text_color());
        color_row(ui, "Tick labels", &mut colors.tick_text, visuals.weak_text_color());
        color_row(ui, "Selection", &mut colors.selection, visuals.widgets.active.bg_fill);
    });
}

pub(super) fn series_section(ui: &mut egui::Ui, draft: &mut PlotConfig, names: &[String], visuals: &egui::Visuals) {
    let styles = [
        (0, "Auto".to_owned()),
        (1, "Solid".to_owned()),
        (2, "Dashed".to_owned()),
        (3, "Dotted".to_owned()),
    ];
    let markers = [
        Marker::Circle,
        Marker::Square,
        Marker::Diamond,
        Marker::Triangle,
        Marker::Cross,
        Marker::Plus,
        Marker::Ring,
    ];
    let marker_options: Vec<(usize, String)> = std::iter::once((0, "Auto".to_owned()))
        .chain(
            markers
                .iter()
                .enumerate()
                .map(|(index, marker)| (index + 1, format!("{marker:?}"))),
        )
        .collect();
    for name in names {
        heading(ui, name);
        let style = draft.series_style_mut(name);
        form(ui, &format!("chart_series_{name}"), |ui| {
            color_row(ui, "Color", &mut style.color, visuals.text_color());
            row(ui, "Width", |ui| {
                let mut value = style.width.map(f64::from);
                optional_number(ui, &mut value, 1.5, 0.05);
                style.width = value.map(|value| value as f32);
            });
            row(ui, "Line style", |ui| {
                let mut index = match style.line_style {
                    None => 0,
                    Some(LineStyle::Solid) => 1,
                    Some(LineStyle::Dashed) => 2,
                    Some(LineStyle::Dotted) => 3,
                };
                if Select::new(&format!("{name} style"), &mut index, &styles)
                    .width(120.0)
                    .show(ui)
                {
                    style.line_style = [
                        None,
                        Some(LineStyle::Solid),
                        Some(LineStyle::Dashed),
                        Some(LineStyle::Dotted),
                    ][index];
                }
            });
            row(ui, "Marker", |ui| {
                let mut index = style
                    .marker
                    .and_then(|marker| markers.iter().position(|m| *m == marker))
                    .map_or(0, |position| position + 1);
                if Select::new(&format!("{name} marker"), &mut index, &marker_options)
                    .width(120.0)
                    .show(ui)
                {
                    style.marker = index.checked_sub(1).map(|index| markers[index]);
                }
            });
            row(ui, "Marker size", |ui| {
                let mut value = style.marker_size.map(f64::from);
                optional_number(ui, &mut value, 3.0, 0.05);
                style.marker_size = value.map(|value| value as f32);
            });
            row(ui, "Fill opacity", |ui| {
                let mut value = style.fill_opacity.map(f64::from);
                optional_number(ui, &mut value, 0.2, 0.01);
                style.fill_opacity = value.map(|value| value.clamp(0.0, 1.0) as f32);
            });
        });
    }
    // Drop empty overrides so the config stays small.
    draft.series.retain(|style| {
        style.color.is_some()
            || style.width.is_some()
            || style.line_style.is_some()
            || style.marker.is_some()
            || style.marker_size.is_some()
            || style.fill_opacity.is_some()
    });
}

pub(super) fn export(ui: &mut egui::Ui, demo: &mut ChartsDemo, action: &mut Option<Action>) {
    let settings = &mut demo.export;
    heading(ui, "Image");
    form(ui, "chart_export", |ui| {
        row(ui, "Size", |ui| {
            Checkbox::new(&mut settings.screen_size).label("As on screen").show(ui);
        });
        row(ui, "Custom size", |ui| {
            ui.add_enabled_ui(!settings.screen_size, |ui| {
                ui.add(
                    egui::DragValue::new(&mut settings.options.width)
                        .range(120.0..=8000.0)
                        .suffix(" pt"),
                );
                ui.label("×");
                ui.add(
                    egui::DragValue::new(&mut settings.options.height)
                        .range(120.0..=8000.0)
                        .suffix(" pt"),
                );
            });
        });
        row(ui, "PNG scale", |ui| {
            let options = [
                (1, "1×".to_owned()),
                (2, "2×".to_owned()),
                (3, "3×".to_owned()),
                (4, "4×".to_owned()),
            ];
            let mut scale = settings.options.scale.round().clamp(1.0, 4.0) as usize;
            if Select::new("PNG scale", &mut scale, &options).width(90.0).show(ui) {
                settings.options.scale = scale as f32;
            }
        });
        row(ui, "Background", |ui| {
            Checkbox::new(&mut settings.options.transparent)
                .label("Transparent")
                .show(ui);
        });
        row(ui, "Folder", |ui| {
            TextField::new(&mut settings.folder).width(320.0).show(ui);
        });
    });
    ui.add_space(8.0);
    ui.label(
        RichText::new("Exports use the applied settings, the current view, and the selection.")
            .small()
            .weak(),
    );
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if Button::new("Export SVG")
            .variant(ButtonVariant::Secondary)
            .show(ui)
            .clicked()
        {
            *action = Some(Action::Export(egui_workbench::components::plot::ExportFormat::Svg));
        }
        if Button::new("Export PNG")
            .variant(ButtonVariant::Secondary)
            .show(ui)
            .clicked()
        {
            *action = Some(Action::Export(egui_workbench::components::plot::ExportFormat::Png));
        }
    });
}
