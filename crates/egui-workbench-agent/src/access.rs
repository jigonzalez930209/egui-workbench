//! A role/name tree derived from the host spec.
//!
//! This is the stable, serializable view an agent uses to find controls. It mirrors the
//! accessible names and roles the components expose through AccessKit, without requiring
//! a rendered frame or a platform accessibility bridge.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use egui_workbench_host::{HostSpec, SeriesSpec, TreeSpec, WidgetSpec};

/// One node in the accessibility tree.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccessNode {
    /// Accessible role, such as `button` or `text_field`.
    pub role: String,
    /// Accessible name.
    pub name: String,
    /// Stable widget id, when the widget has one.
    pub id: Option<String>,
    /// Child nodes.
    pub children: Vec<AccessNode>,
}

impl AccessNode {
    fn new(role: &str, name: impl Into<String>, id: Option<String>) -> Self {
        Self {
            role: role.to_owned(),
            name: name.into(),
            id,
            children: Vec::new(),
        }
    }
}

/// Build the accessibility tree of a spec.
pub fn accessibility_tree(spec: &HostSpec) -> Vec<AccessNode> {
    let mut nodes = Vec::new();
    for widget in &spec.widgets {
        nodes.push(widget_node(widget));
    }
    nodes
}

/// Depth-first search by accessible name and, optionally, role.
pub fn find_node<'a>(nodes: &'a [AccessNode], name: &str, role: Option<&str>) -> Option<&'a AccessNode> {
    for node in nodes {
        if node.name == name && role.is_none_or(|role| node.role == role) {
            return Some(node);
        }
        if let Some(found) = find_node(&node.children, name, role) {
            return Some(found);
        }
    }
    None
}

fn widget_node(widget: &WidgetSpec) -> AccessNode {
    match widget {
        WidgetSpec::Label { text, .. } => AccessNode::new("label", text, None),
        WidgetSpec::Button { id, label, enabled, .. } => {
            let mut node = AccessNode::new("button", label, Some(id.clone()));
            node.children.push(AccessNode::new(
                "state",
                if *enabled { "enabled" } else { "disabled" },
                None,
            ));
            node
        }
        WidgetSpec::TextField { id, label, value, .. } => AccessNode::new(
            "text_field",
            if label.is_empty() { id } else { label },
            Some(id.clone()),
        )
        .with_value(value),
        WidgetSpec::Checkbox { id, label, checked } => {
            AccessNode::new("checkbox", if label.is_empty() { id } else { label }, Some(id.clone()))
                .with_value(if *checked { "checked" } else { "unchecked" })
        }
        WidgetSpec::Switch { id, label, on } => {
            AccessNode::new("switch", if label.is_empty() { id } else { label }, Some(id.clone())).with_value(if *on {
                "on"
            } else {
                "off"
            })
        }
        WidgetSpec::Select {
            id,
            label,
            selected,
            options,
        } => AccessNode::new("combo_box", if label.is_empty() { id } else { label }, Some(id.clone()))
            .with_value(&options.get(*selected).cloned().unwrap_or_default()),
        WidgetSpec::Slider { id, label, value, .. } => {
            AccessNode::new("slider", if label.is_empty() { id } else { label }, Some(id.clone()))
                .with_value(&value.to_string())
        }
        WidgetSpec::Tabs {
            id,
            tabs,
            active,
            children,
        } => {
            let mut node = AccessNode::new("tab_list", id, Some(id.clone()));
            for (index, tab) in tabs.iter().enumerate() {
                let mut child = AccessNode::new("tab", tab, None);
                child.children.push(AccessNode::new(
                    "state",
                    if index == *active { "selected" } else { "unselected" },
                    None,
                ));
                if let Some(content) = children.get(index) {
                    child.children.extend(content.iter().map(widget_node));
                }
                node.children.push(child);
            }
            node
        }
        WidgetSpec::Tree { id, nodes, .. } => {
            let mut node = AccessNode::new("tree", id, Some(id.clone()));
            node.children.extend(nodes.iter().map(tree_node));
            node
        }
        WidgetSpec::DataGrid { id, dataset, .. } => AccessNode::new("table", dataset, Some(id.clone())),
        WidgetSpec::Plot { id, title, series, .. } => {
            AccessNode::new("image", title, Some(id.clone())).with_value(&format!("{} series", series.len()))
        }
        WidgetSpec::Custom { widget, .. } => AccessNode::new("group", widget, None),
        WidgetSpec::Row { children, .. } | WidgetSpec::Column { children, .. } | WidgetSpec::Card { children, .. } => {
            let role = match widget {
                WidgetSpec::Row { .. } => "row",
                WidgetSpec::Column { .. } => "column",
                _ => "group",
            };
            let mut node = AccessNode::new(role, "", None);
            node.children.extend(children.iter().map(widget_node));
            node
        }
        WidgetSpec::Separator | WidgetSpec::Spacer { .. } => AccessNode::new("separator", "", None),
        other => AccessNode::new("widget", format!("{other:?}"), None),
    }
}

impl AccessNode {
    fn with_value(mut self, value: &str) -> Self {
        if !value.is_empty() {
            self.children.push(AccessNode::new("value", value, None));
        }
        self
    }
}

fn tree_node(node: &TreeSpec) -> AccessNode {
    let mut access = AccessNode::new("tree_item", &node.label, Some(node.key.clone()));
    access.children.extend(node.children.iter().map(tree_node));
    access
}

/// A small JSON view used by the `get_accessibility_tree` MCP tool.
pub fn to_json(nodes: &[AccessNode]) -> Value {
    serde_json::to_value(nodes).unwrap_or(Value::Null)
}

/// The series array of a plot, if a caller wants to inspect it without the full spec.
pub fn series_summary(series: &[SeriesSpec]) -> Vec<String> {
    series
        .iter()
        .map(|series| format!("{} ({})", series.name, series.style))
        .collect()
}
