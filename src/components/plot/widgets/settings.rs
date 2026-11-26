//! The analysis menu and the pinned chart-settings panel.

use egui::{Align, Layout, Rect, RichText, Sense, Ui, pos2};

use super::super::{FilterKind, MeasurementAreaStyle, MeasurementMode, PlotState, TransformKind};

/// The filters and transforms shown by the analysis icon.
pub(super) fn analysis_menu(ui: &mut Ui, state: &mut PlotState) {
    ui.set_min_width(260.0);
    ui.label(RichText::new("Filters").strong());
    for filter in FilterKind::ALL {
        let mut enabled = state.pipeline().is_filter_enabled(filter);
        if ui.checkbox(&mut enabled, filter.label()).changed() {
            state.pipeline_mut().set_filter(filter, enabled);
        }
    }
    ui.separator();
    ui.label(RichText::new("Transforms").strong());
    transform_radios(ui, state);
    ui.separator();
    if ui.button("Clear analysis").clicked() {
        state.clear_analysis();
    }
    ui.label(
        RichText::new(
            "Filters apply to every visible series. Transforms use the first series; \
             cross-correlation uses the first two.",
        )
        .small()
        .weak(),
    );
}

/// The hovered/pinned settings panel. Returns whether the pointer is over it.
pub(super) fn panel(ui: &Ui, state: &mut PlotState, anchor: Rect) -> bool {
    let width = 320.0;
    let screen = ui.ctx().input(|input| input.content_rect());
    let position = pos2(
        (anchor.left() - width - 8.0 + state.settings_offset[0]).clamp(8.0, (screen.right() - width - 8.0).max(8.0)),
        (anchor.top() + state.settings_offset[1]).clamp(8.0, (screen.bottom() - 80.0).max(8.0)),
    );
    let id = ui.scope_id().with("chart_settings_panel");
    let area = egui::Area::new(id)
        .order(egui::Order::Foreground)
        .fixed_pos(position)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(width);
                ui.horizontal(|ui| {
                    let handle = ui.add(egui::Label::new("::").selectable(false).sense(Sense::drag()));
                    if handle.dragged() {
                        state.settings_offset[0] += handle.drag_delta().x;
                        state.settings_offset[1] += handle.drag_delta().y;
                    }
                    ui.label(RichText::new("Chart settings").strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            state.settings_pinned = false;
                            state.settings_hovered = false;
                        }
                    });
                });
                ui.separator();
                measurement_controls(ui, state);
                ui.separator();
                ui.label(RichText::new("Analysis").strong());
                analysis_controls(ui, state);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Clear measurements").clicked() {
                        state.clear_measurements();
                    }
                    if ui.button("Clear analysis").clicked() {
                        state.clear_analysis();
                    }
                });
                ui.label(
                    RichText::new(
                        "Hover the settings icon to preview; click it to keep this panel open. \
                         Drag the panel by the :: handle.",
                    )
                    .small()
                    .weak(),
                );
            });
        });
    area.response.hovered()
}

fn measurement_controls(ui: &mut Ui, state: &mut PlotState) {
    let mut measurement = state.measurement;
    ui.label(RichText::new("Measurement").strong());
    ui.label("Area tool");
    ui.selectable_value(&mut measurement.mode, MeasurementMode::Rectangle, "Rectangle");
    ui.selectable_value(&mut measurement.mode, MeasurementMode::CurveSegment, "Curve vs chord");
    ui.label("Area label");
    ui.selectable_value(
        &mut measurement.area_style,
        MeasurementAreaStyle::Absolute,
        "Absolute area",
    );
    ui.selectable_value(&mut measurement.area_style, MeasurementAreaStyle::Signed, "Signed area");
    ui.add(egui::Slider::new(&mut measurement.snap_radius, 4.0..=30.0).text("Snap radius (pt)"));
    ui.checkbox(&mut measurement.show_labels, "Show labels");
    if measurement != state.measurement {
        state.set_measurement(measurement);
    }
}

fn analysis_controls(ui: &mut Ui, state: &mut PlotState) {
    for filter in FilterKind::ALL {
        let mut enabled = state.pipeline().is_filter_enabled(filter);
        if ui.checkbox(&mut enabled, filter.label()).changed() {
            state.pipeline_mut().set_filter(filter, enabled);
        }
    }
    let mut settings = state.pipeline().filter_settings;
    let mut changed = false;
    changed |= ui
        .add(
            egui::Slider::new(&mut settings.savgol_window, 5..=31)
                .step_by(2.0)
                .text("Savitzky–Golay window"),
        )
        .changed();
    changed |= ui
        .add(egui::Slider::new(&mut settings.savgol_order, 1..=5).text("Savitzky–Golay order"))
        .changed();
    changed |= ui
        .add(
            egui::Slider::new(&mut settings.moving_window, 3..=31)
                .step_by(2.0)
                .text("Moving-average window"),
        )
        .changed();
    changed |= ui
        .add(egui::Slider::new(&mut settings.gaussian_sigma, 0.5..=10.0).text("Gaussian sigma"))
        .changed();
    changed |= ui
        .add(
            egui::Slider::new(&mut settings.median_window, 3..=31)
                .step_by(2.0)
                .text("Median window"),
        )
        .changed();
    changed |= ui
        .add(egui::Slider::new(&mut settings.lowpass_alpha, 0.01..=1.0).text("Low-pass alpha"))
        .changed();
    if changed {
        state.pipeline_mut().filter_settings = settings;
    }
    ui.separator();
    ui.label(RichText::new("Transforms").strong());
    transform_radios(ui, state);
}

fn transform_radios(ui: &mut Ui, state: &mut PlotState) {
    let current = state.pipeline().transform;
    for transform in TransformKind::ALL {
        if ui.radio(current == Some(transform), transform.label()).clicked() {
            state.set_transform(if current == Some(transform) {
                None
            } else {
                Some(transform)
            });
        }
    }
}
