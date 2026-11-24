//! A collapsible JSON tree.

use egui::{RichText, Ui};
use serde_json::Value;

/// Draw a JSON value as a collapsible tree. Keys that contain `filter` (case-insensitive)
/// are highlighted; an empty filter shows everything.
pub fn show_state_tree(ui: &mut Ui, value: &Value, filter: &str) {
    tree(ui, "", value, filter);
}

fn tree(ui: &mut Ui, key: &str, value: &Value, filter: &str) {
    let matches = filter.is_empty() || key.to_lowercase().contains(&filter.to_lowercase());
    match value {
        Value::Object(map) => {
            let label = if key.is_empty() {
                format!("object ({})", map.len())
            } else {
                format!("{key}  {{ {} }}", map.len())
            };
            let header = egui::CollapsingHeader::new(label).default_open(key.is_empty() || matches);
            header.show(ui, |ui| {
                for (child_key, child) in map {
                    tree(ui, child_key, child, filter);
                }
            });
        }
        Value::Array(items) => {
            let label = if key.is_empty() {
                format!("array ({})", items.len())
            } else {
                format!("{key}  [ {} ]", items.len())
            };
            egui::CollapsingHeader::new(label)
                .default_open(key.is_empty() || matches)
                .show(ui, |ui| {
                    for (index, item) in items.iter().enumerate() {
                        tree(ui, &format!("[{index}]"), item, filter);
                    }
                });
        }
        Value::String(text) => {
            ui.label(RichText::new(format!("{key}: \"{text}\"")).color(ui.visuals().hyperlink_color));
        }
        Value::Number(number) => {
            ui.label(format!("{key}: {number}"));
        }
        Value::Bool(boolean) => {
            ui.label(format!("{key}: {boolean}"));
        }
        Value::Null => {
            ui.label(RichText::new(format!("{key}: null")).weak());
        }
    }
}
