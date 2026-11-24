//! Spec-level accessibility audit.

use egui::{RichText, Ui};
use serde_json::Value;

pub fn show_accessibility(ui: &mut Ui, snapshot: &Value) {
    let issues = accessibility_issues(snapshot);
    if issues.is_empty() {
        ui.label(RichText::new("No accessibility issues found in the spec.").weak());
        return;
    }
    for issue in issues {
        ui.label(RichText::new(issue).color(ui.visuals().warn_fg_color));
    }
}

/// The issues found in a spec snapshot, as human-readable strings.
pub fn accessibility_issues(snapshot: &Value) -> Vec<String> {
    let mut issues = Vec::new();
    if let Some(widgets) = snapshot.get("widgets").and_then(Value::as_array) {
        walk_accessibility(widgets, "", &mut issues);
    }
    issues
}

fn walk_accessibility(widgets: &[Value], path: &str, issues: &mut Vec<String>) {
    for (index, widget) in widgets.iter().enumerate() {
        let here = format!("{path}/widgets/{index}");
        let kind = widget.get("kind").and_then(Value::as_str).unwrap_or("widget");
        let label = widget.get("label").and_then(Value::as_str).unwrap_or("");
        let text = widget.get("text").and_then(Value::as_str).unwrap_or("");
        let id = widget.get("id").and_then(Value::as_str).unwrap_or("");
        if matches!(
            kind,
            "button" | "text_field" | "checkbox" | "switch" | "select" | "slider"
        ) && label.is_empty()
            && id.is_empty()
        {
            issues.push(format!("{here}: {kind} has no label or id"));
        }
        if kind == "label" && text.is_empty() {
            issues.push(format!("{here}: label is empty"));
        }
        if matches!(kind, "plot" | "data_grid") && widget.get("title").and_then(Value::as_str).unwrap_or("").is_empty()
        {
            issues.push(format!("{here}: {kind} has no title"));
        }
        for key in ["children", "nodes"] {
            if let Some(children) = widget.get(key).and_then(Value::as_array) {
                walk_accessibility(children, &format!("{here}/{key}"), issues);
            }
        }
    }
}
