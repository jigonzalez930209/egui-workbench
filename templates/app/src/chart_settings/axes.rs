//! The Titles and Axes sections of the chart settings dialog.

use egui_workbench::components::plot::{AxisConfig, Bounds, PlotConfig, Scale, TitleAlign};
use egui_workbench::components::prelude::*;
use egui_workbench::egui::{self, Color32, RichText};

use super::sections::{color_row, form, heading, number, optional_number, optional_text, row};

/// One title: its text, size (`None` = the shared axis title size), and color.
fn title_rows(
    ui: &mut egui::Ui,
    text: &mut Option<String>,
    placeholder: &str,
    size: &mut Option<f32>,
    shared: f32,
    color: &mut Option<Color32>,
    theme: Color32,
) {
    row(ui, "Text", |ui| optional_text(ui, text, placeholder));
    row(ui, "Size", |ui| {
        let mut value = size.map(f64::from);
        optional_number(ui, &mut value, f64::from(shared), 0.25);
        *size = value.map(|value| value.clamp(6.0, 36.0) as f32);
    });
    color_row(ui, "Color", color, theme);
}

pub(super) fn titles(ui: &mut egui::Ui, draft: &mut PlotConfig, visuals: &egui::Visuals) {
    let theme = draft.colors.text.unwrap_or(visuals.text_color());
    heading(ui, "Chart title");
    form(ui, "chart_titles_main", |ui| {
        row(ui, "Text", |ui| optional_text(ui, &mut draft.title, "Chart"));
        row(ui, "Size", |ui| {
            number(ui, &mut draft.title_size, 6.0..=48.0, 0.25, " pt")
        });
        color_row(ui, "Color", &mut draft.title_color, theme);
        row(ui, "Alignment", |ui| {
            let aligns = [TitleAlign::Left, TitleAlign::Center, TitleAlign::Right];
            let options = [
                (0, "Left".to_owned()),
                (1, "Center".to_owned()),
                (2, "Right".to_owned()),
            ];
            let mut index = aligns.iter().position(|a| *a == draft.title_align).unwrap_or(1);
            if Select::new("Title alignment", &mut index, &options)
                .width(120.0)
                .show(ui)
            {
                draft.title_align = aligns[index];
            }
        });
    });
    heading(ui, "Axis titles");
    form(ui, "chart_titles_shared", |ui| {
        row(ui, "Default size", |ui| {
            number(ui, &mut draft.label_size, 6.0..=36.0, 0.25, " pt")
        });
    });
    let shared = draft.label_size;
    for (name, axis, placeholder) in [
        ("X axis", &mut draft.x, "x"),
        ("Left Y axis", &mut draft.y, "y"),
        ("Right Y axis", &mut draft.y2, "y₂"),
    ] {
        heading(ui, name);
        form(ui, &format!("chart_titles_{name}"), |ui| {
            title_rows(
                ui,
                &mut axis.label,
                placeholder,
                &mut axis.label_size,
                shared,
                &mut axis.label_color,
                theme,
            );
        });
    }
    ui.label(
        RichText::new("The right axis appears when a series uses it (Series → Y axis).")
            .small()
            .weak(),
    );
}

pub(super) fn axes(
    ui: &mut egui::Ui,
    draft: &mut PlotConfig,
    view: Option<Bounds>,
    y2_view: Option<[f64; 2]>,
    visuals: &egui::Visuals,
) {
    let view = view.unwrap_or(Bounds::from_corners([0.0, 0.0], [1.0, 1.0]));
    let [y2_low, y2_high] = y2_view.unwrap_or([0.0, 1.0]);
    let tick_theme = visuals.weak_text_color();
    for (name, axis, (low, high)) in [
        ("X axis", &mut draft.x, (view.min_x, view.max_x)),
        ("Left Y axis", &mut draft.y, (view.min_y, view.max_y)),
        ("Right Y axis", &mut draft.y2, (y2_low, y2_high)),
    ] {
        heading(ui, name);
        form(ui, &format!("chart_axis_{name}"), |ui| {
            axis_rows(ui, name, axis, low, high, tick_theme)
        });
    }
}

fn axis_rows(ui: &mut egui::Ui, name: &str, axis: &mut AxisConfig, low: f64, high: f64, tick_theme: Color32) {
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
    color_row(ui, "Tick color", &mut axis.tick_color, tick_theme);
    row(ui, "Label decimals", |ui| {
        let mut auto = axis.decimals.is_none();
        if Checkbox::new(&mut auto).label("Auto").show(ui) {
            axis.decimals = if auto { None } else { Some(2) };
        }
        if let Some(decimals) = &mut axis.decimals {
            ui.add(egui::DragValue::new(decimals).range(0..=10));
        }
    });
}
