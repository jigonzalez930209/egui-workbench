//! The widget inspector: widget tree on the left, selected JSON on the right.

use egui::{RichText, Ui};
use serde_json::Value;

use super::show_state_tree;

/// Draw the widget tree with its accessible names. Returns the selected path.
pub fn show_inspector(ui: &mut Ui, snapshot: &Value, selected: &mut Option<String>) {
    let widgets = snapshot.get("widgets").and_then(Value::as_array);
    let Some(widgets) = widgets else {
        ui.label(RichText::new("The snapshot has no widgets.").weak());
        return;
    };
    ui.columns(2, |columns| {
        egui::ScrollArea::vertical().show(&mut columns[0], |ui| {
            for (index, widget) in widgets.iter().enumerate() {
                inspector_row(ui, &format!("/widgets/{index}"), widget, selected);
            }
        });
        egui::ScrollArea::vertical().show(&mut columns[1], |ui| {
            if let Some(path) = selected {
                if let Some(value) = pointer(snapshot, path) {
                    show_state_tree(ui, value, "");
                }
            } else {
                ui.label(RichText::new("Select a widget.").weak());
            }
        });
    });
}

fn inspector_row(ui: &mut Ui, path: &str, widget: &Value, selected: &mut Option<String>) {
    let kind = widget.get("kind").and_then(Value::as_str).unwrap_or("widget");
    let name = widget
        .get("label")
        .or_else(|| widget.get("text"))
        .or_else(|| widget.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let label = if name.is_empty() {
        format!("{kind}  {path}")
    } else {
        format!("{kind}  {name}")
    };
    if ui.selectable_label(selected.as_deref() == Some(path), label).clicked() {
        *selected = Some(path.to_owned());
    }
    for key in ["children", "nodes"] {
        if let Some(children) = widget.get(key).and_then(Value::as_array) {
            ui.indent(path, |ui| {
                for (index, child) in children.iter().enumerate() {
                    inspector_row(ui, &format!("{path}/{key}/{index}"), child, selected);
                }
            });
        }
    }
}

fn pointer<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('/').skip(1).try_fold(value, |current, token| {
        if let Ok(index) = token.parse::<usize>() {
            current.get(index)
        } else {
            current.get(token)
        }
    })
}
