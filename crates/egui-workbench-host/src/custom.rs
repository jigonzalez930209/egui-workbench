//! The escape hatch for native Rust widgets.

use std::collections::BTreeMap;

use egui::Ui;
use serde_json::Value;

use crate::HostEvent;

/// A native widget registered by a Rust extension.
pub trait CustomWidget: Send {
    /// Draw the widget and return any events.
    fn show(&mut self, ui: &mut Ui, props: &Value) -> Vec<HostEvent>;
}

/// Registered custom widgets by kind.
#[derive(Default)]
pub struct CustomWidgetRegistry {
    widgets: BTreeMap<String, Box<dyn CustomWidget + Send>>,
}

impl CustomWidgetRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register or replace a widget kind.
    pub fn register(&mut self, kind: impl Into<String>, widget: impl CustomWidget + 'static) {
        self.widgets.insert(kind.into(), Box::new(widget));
    }

    /// Whether a kind is registered.
    pub fn contains(&self, kind: &str) -> bool {
        self.widgets.contains_key(kind)
    }

    /// Draw a registered widget, or return `None` when the kind is unknown.
    pub fn show(&mut self, kind: &str, ui: &mut Ui, props: &Value) -> Option<Vec<HostEvent>> {
        self.widgets.get_mut(kind).map(|widget| widget.show(ui, props))
    }
}
