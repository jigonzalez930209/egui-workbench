//! The native host view and its eframe runner.

use egui::{RichText, Ui};
#[cfg(not(target_arch = "wasm32"))]
use egui_workbench::app::ThemeState;
use egui_workbench::components::prelude::*;
use egui_workbench::components::{DataGrid, DataGridEvent};
use egui_workbench::title_menu;

use crate::runtime::{Host, HostEvent};
use crate::spec::{HostSpec, SeriesSpec, WidgetSpec};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) type EventCallback = Box<dyn FnMut(&HostEvent)>;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type FrameCallback = Box<dyn FnMut()>;

/// The eframe application that owns a [`Host`].
#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct HostApp {
    pub(crate) host: Host,
    pub(crate) theme: ThemeState,
    pub(crate) on_event: Option<EventCallback>,
    pub(crate) on_frame: Option<FrameCallback>,
}

#[cfg(not(target_arch = "wasm32"))]
impl eframe::App for HostApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.theme.sync_system(ctx);
        let _ = self.host.reload_if_changed();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.host.apply_actions() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.host.ui(ui);
        if let Some(callback) = &mut self.on_event {
            for event in self.host.poll_events() {
                callback(&event);
            }
        }
        if let Some(callback) = &mut self.on_frame {
            callback();
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.theme.clear_color()
    }
}

impl Host {
    /// Draw the host in a root `Ui` and collect its events.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let spec = self.spec.clone();
        let mut events = Vec::new();
        if spec.panels.sidebar {
            egui::Panel::left("host_sidebar").default_size(220.0).show(ui, |ui| {
                ui.add_space(6.0);
                ui.label(RichText::new(&spec.window.title).strong());
                ui.separator();
                for node in &spec.explorer {
                    tree(ui, node, &mut events, "explorer");
                }
            });
        }
        if spec.panels.bottom {
            egui::Panel::bottom("host_bottom").default_size(140.0).show(ui, |ui| {
                ui.label(RichText::new("Output").strong());
                ui.separator();
                ui.label(RichText::new("Host events appear in the journal.").weak());
            });
        }
        if spec.panels.status {
            egui::Panel::bottom("host_status").exact_size(24.0).show(ui, |ui| {
                ui.horizontal(|ui| {
                    for item in &spec.status.left {
                        ui.label(RichText::new(item).small());
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        for item in spec.status.right.iter().rev() {
                            ui.label(RichText::new(item).small().weak());
                        }
                    });
                });
            });
        }
        egui::Panel::top("host_menu").exact_size(30.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                for menu in &spec.menus {
                    title_menu(ui, &menu.title, 180.0, |ui| {
                        for item in &menu.items {
                            if item.separator {
                                ui.separator();
                            } else if let Some(command) = &item.command {
                                if ui.button(&item.label).clicked() {
                                    events.push(HostEvent::Command { id: command.clone() });
                                    ui.close();
                                }
                            } else {
                                ui.label(&item.label);
                            }
                        }
                    });
                }
            });
        });
        egui::CentralPanel::default().show(ui, |ui| {
            let group = spec.layout.groups.get(spec.layout.active);
            if let Some(group) = group {
                ui.horizontal(|ui| {
                    for (index, tab) in group.tabs.iter().enumerate() {
                        if ui.selectable_label(index == group.active, &tab.title).clicked() {
                            events.push(HostEvent::TabChanged {
                                id: group.id.clone(),
                                index,
                            });
                        }
                    }
                });
                ui.separator();
            }
            let widgets = spec.widgets.clone();
            render_widgets(self, ui, &widgets, &mut events);
        });
        self.show_dialogs(ui.ctx(), &spec, &mut events);
        for event in self.toasts.show(ui.ctx(), &mut self.icons) {
            if let egui_workbench::components::ToastEvent::Action(_) = event {
                events.push(HostEvent::Custom {
                    kind: "toast".to_owned(),
                    payload: serde_json::Value::String("action".to_owned()),
                });
            }
        }
        self.events.extend(events);
    }

    fn show_dialogs(&mut self, ctx: &egui::Context, spec: &HostSpec, events: &mut Vec<HostEvent>) {
        for dialog in &spec.dialogs {
            if !dialog.open {
                continue;
            }
            let mut open = true;
            egui::Window::new(&dialog.title)
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(&dialog.message);
                    ui.horizontal(|ui| {
                        if ui.button(&dialog.cancel).clicked() {
                            events.push(HostEvent::Command {
                                id: format!("{}.cancel", dialog.id),
                            });
                            ui.close();
                        }
                        if ui.button(&dialog.confirm).clicked() {
                            events.push(HostEvent::Command {
                                id: format!("{}.confirm", dialog.id),
                            });
                            ui.close();
                        }
                    });
                });
        }
    }
}

fn tree(ui: &mut Ui, node: &crate::spec::TreeSpec, events: &mut Vec<HostEvent>, id: &str) {
    let response = ui.selectable_label(false, &node.label);
    if response.clicked() {
        events.push(HostEvent::TreeSelected {
            id: id.to_owned(),
            key: node.key.clone(),
        });
    }
    if !node.children.is_empty() {
        ui.indent(&node.key, |ui| {
            for child in &node.children {
                tree(ui, child, events, id);
            }
        });
    }
}

fn render_widgets(host: &mut Host, ui: &mut Ui, widgets: &[WidgetSpec], events: &mut Vec<HostEvent>) {
    for widget in widgets {
        render_widget(host, ui, widget, events);
    }
}

fn render_widget(host: &mut Host, ui: &mut Ui, widget: &WidgetSpec, events: &mut Vec<HostEvent>) {
    match widget {
        WidgetSpec::Label {
            text,
            size,
            tone,
            strong,
        } => {
            let mut label = RichText::new(text).size(host_size(ui, *size));
            if *strong {
                label = label.strong();
            }
            if tone == "muted" {
                label = label.color(ui.visuals().weak_text_color());
            }
            ui.label(label);
        }
        WidgetSpec::Button {
            id,
            label,
            variant,
            enabled,
        } => {
            let variant = match variant.as_str() {
                "secondary" => ButtonVariant::Secondary,
                "outline" => ButtonVariant::Outline,
                "ghost" => ButtonVariant::Ghost,
                "danger" | "destructive" => ButtonVariant::Destructive,
                "link" => ButtonVariant::Link,
                _ => ButtonVariant::Primary,
            };
            if Button::new(label).variant(variant).enabled(*enabled).show(ui).clicked() {
                events.push(HostEvent::Command { id: id.clone() });
            }
        }
        WidgetSpec::TextField { id, label, hint, .. } => {
            let mut value = host.string_value(id);
            let changed = TextField::new(&mut value).label(label).hint(hint).show(ui).changed();
            if changed {
                host.values.insert(id.clone(), serde_json::Value::String(value.clone()));
                events.push(HostEvent::ValueChanged {
                    id: id.clone(),
                    value: serde_json::Value::String(value),
                });
            }
        }
        WidgetSpec::Checkbox { id, label, .. } => {
            let mut value = host.bool_value(id);
            if Checkbox::new(&mut value).label(label).show(ui) {
                host.values.insert(id.clone(), serde_json::Value::Bool(value));
                events.push(HostEvent::ValueChanged {
                    id: id.clone(),
                    value: serde_json::Value::Bool(value),
                });
            }
        }
        WidgetSpec::Switch { id, label, .. } => {
            let mut value = host.bool_value(id);
            if Switch::new(&mut value).label(label).show(ui) {
                host.values.insert(id.clone(), serde_json::Value::Bool(value));
                events.push(HostEvent::ValueChanged {
                    id: id.clone(),
                    value: serde_json::Value::Bool(value),
                });
            }
        }
        WidgetSpec::Select { id, label, options, .. } => {
            let mut selected = host.usize_value(id);
            let items: Vec<(usize, String)> = options.iter().cloned().enumerate().collect();
            if Select::new(label, &mut selected, &items).show(ui) {
                host.values.insert(id.clone(), serde_json::Value::from(selected));
                events.push(HostEvent::ValueChanged {
                    id: id.clone(),
                    value: serde_json::Value::from(selected),
                });
            }
        }
        WidgetSpec::Slider {
            id,
            label,
            min,
            max,
            step,
            ..
        } => {
            let mut value = host.f64_value(id);
            let response = ui.add(egui::Slider::new(&mut value, *min..=*max).text(label).step_by(*step));
            if response.changed() {
                host.values.insert(id.clone(), serde_json::Value::from(value));
                events.push(HostEvent::ValueChanged {
                    id: id.clone(),
                    value: serde_json::Value::from(value),
                });
            }
        }
        WidgetSpec::Separator => {
            ui.separator();
        }
        WidgetSpec::Spacer { height } => {
            ui.add_space(*height);
        }
        WidgetSpec::Row { children, gap } => {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = *gap;
                render_widgets(host, ui, children, events);
            });
        }
        WidgetSpec::Column { children, gap } => {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = *gap;
                render_widgets(host, ui, children, events);
            });
        }
        WidgetSpec::Card { title, children } => {
            Card::new()
                .title(title)
                .show(ui, |ui| render_widgets(host, ui, children, events));
        }
        WidgetSpec::Tabs { id, tabs, children, .. } => {
            let active = *host.tabs.get(id).unwrap_or(&0).min(&tabs.len().saturating_sub(1));
            ui.horizontal(|ui| {
                for (index, tab) in tabs.iter().enumerate() {
                    if ui.selectable_label(index == active, tab).clicked() {
                        host.tabs.insert(id.clone(), index);
                        events.push(HostEvent::TabChanged { id: id.clone(), index });
                    }
                }
            });
            if let Some(content) = children.get(active) {
                render_widgets(host, ui, content, events);
            }
        }
        WidgetSpec::Tree { id, nodes, .. } => {
            for node in nodes {
                tree(ui, node, events, id);
            }
        }
        WidgetSpec::DataGrid { id, dataset, height } => {
            if let Some(data) = host.dataset(dataset).cloned() {
                let state = host.grid_states.entry(id.clone()).or_default();
                if let Some(DataGridEvent::Copy { text, format }) = DataGrid::new(&data, state).height(*height).show(ui)
                {
                    events.push(HostEvent::Custom {
                        kind: format!("copy.{format}"),
                        payload: serde_json::Value::String(text),
                    });
                }
            } else {
                ui.label(
                    RichText::new(format!("dataset `{dataset}` is not registered")).color(ui.visuals().error_fg_color),
                );
            }
        }
        WidgetSpec::Plot {
            id,
            title,
            series,
            height,
            renderer,
        } => {
            render_plot(host, ui, id, title, series, *height, renderer);
        }
        WidgetSpec::Custom { widget, props } => {
            if let Some(custom_events) = host.custom.show(widget, ui, props) {
                events.extend(custom_events);
            } else {
                ui.label(RichText::new(format!("custom widget `{widget}` is not registered")).weak());
            }
        }
    }
}

#[cfg(feature = "plot")]
fn render_plot(
    host: &mut Host,
    ui: &mut Ui,
    id: &str,
    title: &str,
    series: &[SeriesSpec],
    height: f32,
    renderer: &str,
) {
    use egui_workbench::components::plot::{Plot, Renderer, Series};

    let mut plot = Plot::new(id)
        .title(title)
        .height(height)
        .renderer(if renderer == "gpu" {
            Renderer::Gpu
        } else {
            Renderer::Cpu
        });
    let mut owned: Vec<Vec<[f64; 2]>> = Vec::new();
    for spec in series {
        let points = if let Some(dataset) = &spec.dataset {
            host.dataset(dataset)
                .and_then(|data| data.xy_pairs(spec.x.as_deref().unwrap_or("x"), spec.y.as_deref().unwrap_or("y")))
                .unwrap_or_default()
        } else {
            spec.points.clone()
        };
        owned.push(points);
    }
    for (spec, points) in series.iter().zip(&owned) {
        let series = if spec.style == "scatter" {
            Series::scatter(spec.name.clone(), points.as_slice())
        } else {
            Series::line(spec.name.clone(), points.as_slice())
        };
        plot = plot.series(series);
    }
    let state = host.plot_states.entry(id.to_owned()).or_default();
    let _ = plot.show(ui, &mut host.icons, state);
}

#[cfg(not(feature = "plot"))]
fn render_plot(
    _host: &mut Host,
    ui: &mut Ui,
    _id: &str,
    title: &str,
    _series: &[SeriesSpec],
    _height: f32,
    _renderer: &str,
) {
    ui.label(RichText::new(format!("{title}: the host was built without the plot feature")).weak());
}

fn host_size(ui: &Ui, size: f32) -> f32 {
    if size > 0.0 {
        size
    } else {
        ui.style().text_styles[&egui::TextStyle::Body].size
    }
}

impl Host {
    fn string_value(&self, id: &str) -> String {
        self.values
            .get(id)
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned()
    }

    fn bool_value(&self, id: &str) -> bool {
        self.values
            .get(id)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    }

    fn usize_value(&self, id: &str) -> usize {
        self.values.get(id).and_then(serde_json::Value::as_u64).unwrap_or(0) as usize
    }

    fn f64_value(&self, id: &str) -> f64 {
        self.values.get(id).and_then(serde_json::Value::as_f64).unwrap_or(0.0)
    }
}
