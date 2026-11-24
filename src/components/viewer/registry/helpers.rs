//! Detection and tree helpers for the viewer registry.

use egui::{RichText, Ui};

use super::{LogLevel, ViewerKind};

#[derive(Clone, Copy)]
pub(super) enum TreeFormat {
    Json,
    Yaml,
    Toml,
}

pub(super) fn sniff(bytes: &[u8]) -> ViewerKind {
    let Some(text) = std::str::from_utf8(bytes).ok() else {
        return ViewerKind::Hex;
    };
    let trimmed = text.trim_start();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return ViewerKind::Json;
    }
    if trimmed.starts_with("<svg") || trimmed.starts_with("<?xml") {
        return ViewerKind::Svg;
    }
    if text.contains('\u{0}') || bytes.iter().any(|byte| *byte < 0x09) {
        return ViewerKind::Hex;
    }
    ViewerKind::Text
}

pub(super) fn line_level(line: &str) -> Option<LogLevel> {
    let upper = line.to_ascii_uppercase();
    if upper.contains("ERROR") || upper.contains("[E]") || upper.contains("FATAL") {
        Some(LogLevel::Error)
    } else if upper.contains("WARN") || upper.contains("[W]") {
        Some(LogLevel::Warning)
    } else {
        None
    }
}

pub(super) fn show_value(ui: &mut Ui, key: &str, value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            egui::CollapsingHeader::new(format!("{key} {{ {} }}", map.len()))
                .default_open(key.is_empty())
                .show(ui, |ui| {
                    for (key, value) in map {
                        show_value(ui, key, value);
                    }
                });
        }
        serde_json::Value::Array(items) => {
            egui::CollapsingHeader::new(format!("{key} [ {} ]", items.len()))
                .default_open(key.is_empty())
                .show(ui, |ui| {
                    for (index, value) in items.iter().enumerate() {
                        show_value(ui, &format!("[{index}]"), value);
                    }
                });
        }
        serde_json::Value::String(text) => {
            ui.label(format!("{key}: \"{text}\""));
        }
        serde_json::Value::Number(number) => {
            ui.label(format!("{key}: {number}"));
        }
        serde_json::Value::Bool(value) => {
            ui.label(format!("{key}: {value}"));
        }
        serde_json::Value::Null => {
            ui.label(RichText::new(format!("{key}: null")).weak());
        }
    }
}
