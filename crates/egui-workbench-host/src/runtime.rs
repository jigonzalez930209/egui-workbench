//! The host runtime: spec, patches, events, data, protocol, and the native runner.

use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use egui_workbench::Icons;
use egui_workbench::components::{DataGridState, Toasts};
use egui_workbench::data::Dataset;
use serde_json::Value;

use crate::custom::CustomWidgetRegistry;
use crate::data::{Datasets, dataset_from_columns};
use crate::patch::Patch;
use crate::protocol::{Message, Request, Response};
use crate::spec::{HostSpec, SPEC_VERSION, ToastSpec};

mod events;
mod native;

pub use events::{ActionQueue, HostAction, HostEvent};

/// Why a host operation failed.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum HostError {
    /// The spec JSON is invalid.
    Spec(String),
    /// The spec version is newer than [`SPEC_VERSION`].
    UnsupportedVersion(u32),
    /// A file could not be read or written.
    Io(String),
    /// A patch could not be applied.
    Patch(String),
}

impl std::fmt::Display for HostError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spec(error) => write!(formatter, "invalid host spec: {error}"),
            Self::UnsupportedVersion(version) => write!(formatter, "unsupported host spec version {version}"),
            Self::Io(error) => formatter.write_str(error),
            Self::Patch(error) => write!(formatter, "patch failed: {error}"),
        }
    }
}

impl std::error::Error for HostError {}

/// Deepest JSON nesting a patched spec may reach.
const MAX_SPEC_DEPTH: usize = 128;

/// Most unpolled events kept. An embedding that never polls would otherwise grow the
/// queue without bound.
const MAX_PENDING_EVENTS: usize = 10_000;

/// The nesting depth of a JSON value, counted without recursion.
fn json_depth(root: &Value) -> usize {
    let mut deepest = 0;
    let mut stack = vec![(root, 1usize)];
    while let Some((value, depth)) = stack.pop() {
        deepest = deepest.max(depth);
        match value {
            Value::Array(items) => stack.extend(items.iter().map(|item| (item, depth + 1))),
            Value::Object(map) => stack.extend(map.values().map(|item| (item, depth + 1))),
            _ => {}
        }
    }
    deepest
}

/// A running host.
pub struct Host {
    pub(crate) spec: HostSpec,
    pub(crate) json: Value,
    source: Option<PathBuf>,
    modified: Option<SystemTime>,
    pub(crate) events: VecDeque<HostEvent>,
    pub(crate) values: BTreeMap<String, Value>,
    pub(crate) tabs: BTreeMap<String, usize>,
    pub(crate) datasets: Datasets,
    pub(crate) custom: CustomWidgetRegistry,
    pub(crate) grid_states: BTreeMap<String, DataGridState>,
    #[cfg(feature = "plot")]
    pub(crate) plot_states: BTreeMap<String, egui_workbench::components::plot::PlotState>,
    pub(crate) icons: Icons,
    pub(crate) toasts: Toasts,
    #[cfg(not(target_arch = "wasm32"))]
    headless_context: Option<egui::Context>,
    actions: ActionQueue,
    revision: u64,
}

impl Host {
    /// A host for a spec.
    ///
    /// # Errors
    /// When the spec version is newer than this host.
    pub fn new(spec: HostSpec) -> Result<Self, HostError> {
        Self::with_actions(spec, ActionQueue::default())
    }

    /// A host that applies queued actions from an existing queue.
    ///
    /// # Errors
    /// When the spec version is newer than this host.
    pub fn with_actions(spec: HostSpec, actions: ActionQueue) -> Result<Self, HostError> {
        if spec.version > SPEC_VERSION {
            return Err(HostError::UnsupportedVersion(spec.version));
        }
        let json = serde_json::to_value(&spec).map_err(|error| HostError::Spec(error.to_string()))?;
        let mut host = Self {
            spec,
            json,
            source: None,
            modified: None,
            events: VecDeque::new(),
            values: BTreeMap::new(),
            tabs: BTreeMap::new(),
            datasets: BTreeMap::new(),
            custom: CustomWidgetRegistry::new(),
            grid_states: BTreeMap::new(),
            #[cfg(feature = "plot")]
            plot_states: BTreeMap::new(),
            icons: Icons::new(),
            toasts: Toasts::new(),
            #[cfg(not(target_arch = "wasm32"))]
            headless_context: None,
            actions,
            revision: 0,
        };
        host.seed_values();
        host.seed_toasts();
        Ok(host)
    }

    /// Load a spec from a JSON file; the path is watched by `reload_if_changed` in the
    /// native runner.
    ///
    /// # Errors
    /// When the file cannot be read or parsed.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, HostError> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|error| HostError::Io(error.to_string()))?;
        let spec = HostSpec::from_json(&text).map_err(HostError::Spec)?;
        let mut host = Self::new(spec)?;
        host.source = Some(path.to_path_buf());
        host.modified = native::modified(path);
        Ok(host)
    }

    /// The current spec.
    pub fn spec(&self) -> &HostSpec {
        &self.spec
    }

    /// The current spec as JSON (the patch target).
    pub fn json(&self) -> &Value {
        &self.json
    }

    /// How many toasts are queued or visible.
    pub fn toast_count(&self) -> usize {
        self.toasts.len()
    }

    /// The action queue, shared with event handlers and other threads.
    pub fn actions(&self) -> ActionQueue {
        self.actions.clone()
    }

    /// Drain and apply queued actions. Returns `true` when the window should close.
    pub fn apply_actions(&mut self) -> bool {
        let queued = self.actions.drain();
        let mut close = false;
        for action in queued {
            match action {
                HostAction::Patch(patch) => {
                    if let Err(error) = self.apply(&patch) {
                        self.push_event(HostEvent::Custom {
                            kind: "host.error".to_owned(),
                            payload: Value::String(error.to_string()),
                        });
                    }
                }
                HostAction::Toast(toast) => self.push_toast(&toast),
                HostAction::SetTitle(title) => {
                    self.spec.window.title = title;
                    self.json["window"]["title"] = Value::String(self.spec.window.title.clone());
                }
                HostAction::Repaint => {}
                HostAction::Close => close = true,
            }
        }
        close
    }

    /// The source path, when loaded from a file.
    pub fn source(&self) -> Option<&Path> {
        self.source.as_deref()
    }

    /// A revision that changes with every patch or reload.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Apply a patch to the spec, validating the result before replacing it.
    ///
    /// # Errors
    /// When the patch or the resulting spec is invalid.
    pub fn apply(&mut self, patch: &Patch) -> Result<(), HostError> {
        let mut candidate = self.json.clone();
        patch
            .apply(&mut candidate)
            .map_err(|error| HostError::Patch(error.to_string()))?;
        // `serde_json` bounds the depth of text it parses, but successive patches can nest
        // a value one level deeper each time; rendering and dropping recurse on it.
        if json_depth(&candidate) > MAX_SPEC_DEPTH {
            return Err(HostError::Patch(format!(
                "the spec is nested deeper than {MAX_SPEC_DEPTH} levels"
            )));
        }
        let spec: HostSpec =
            serde_json::from_value(candidate.clone()).map_err(|error| HostError::Spec(error.to_string()))?;
        if spec.version > SPEC_VERSION {
            return Err(HostError::UnsupportedVersion(spec.version));
        }
        // Widgets the patch did not touch keep what the user typed or picked; a widget
        // whose seeded value changed takes the new one.
        let (mut old_values, mut old_tabs) = (BTreeMap::new(), BTreeMap::new());
        native::seed_widgets(&self.spec.widgets, &mut old_values, &mut old_tabs);
        let (user_values, user_tabs) = (std::mem::take(&mut self.values), std::mem::take(&mut self.tabs));
        self.json = candidate;
        self.spec = spec;
        self.seed_values();
        for (id, value) in user_values {
            if old_values.get(&id) == Some(&value) {
                continue;
            }
            if old_values.contains_key(&id) && old_values.get(&id) == self.values.get(&id) {
                self.values.insert(id, value);
            }
        }
        for (id, index) in user_tabs {
            if old_tabs.get(&id) == self.tabs.get(&id) && self.tabs.contains_key(&id) {
                self.tabs.insert(id, index);
            }
        }
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }

    /// Replace the spec.
    ///
    /// # Errors
    /// When the spec version is newer.
    pub fn set_spec(&mut self, spec: HostSpec) -> Result<(), HostError> {
        if spec.version > SPEC_VERSION {
            return Err(HostError::UnsupportedVersion(spec.version));
        }
        self.json = serde_json::to_value(&spec).map_err(|error| HostError::Spec(error.to_string()))?;
        self.spec = spec;
        self.values.clear();
        self.tabs.clear();
        self.seed_values();
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }

    /// Reload the source file when its modification time changed. Returns whether it
    /// reloaded.
    pub fn reload_if_changed(&mut self) -> bool {
        let Some(path) = self.source.clone() else {
            return false;
        };
        let current = native::modified(&path);
        if current == self.modified {
            return false;
        }
        self.modified = current;
        match Self::load(&path) {
            Ok(mut fresh) => {
                fresh.source = self.source.clone();
                fresh.modified = self.modified;
                fresh.datasets = std::mem::take(&mut self.datasets);
                // The queue handle was shared with other threads, the custom widgets were
                // registered by the embedding application, and events are not yet polled:
                // a reload replaces the spec, not those.
                fresh.actions = self.actions.clone();
                fresh.custom = std::mem::take(&mut self.custom);
                fresh.events = std::mem::take(&mut self.events);
                fresh.revision = self.revision.wrapping_add(1);
                *self = fresh;
                true
            }
            Err(error) => {
                self.push_event(HostEvent::Custom {
                    kind: "host.error".to_owned(),
                    payload: Value::String(error.to_string()),
                });
                false
            }
        }
    }

    pub(crate) fn push_event(&mut self, event: HostEvent) {
        if self.events.len() >= MAX_PENDING_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// The events produced since the last poll, without draining them.
    pub fn peek_events(&self) -> Vec<HostEvent> {
        self.events.iter().cloned().collect()
    }

    /// Take the events produced since the last poll.
    pub fn poll_events(&mut self) -> Vec<HostEvent> {
        self.events.drain(..).collect()
    }

    /// Register a named dataset.
    pub fn set_dataset(&mut self, name: impl Into<String>, dataset: Dataset) {
        self.datasets.insert(name.into(), Arc::new(dataset));
        self.revision = self.revision.wrapping_add(1);
    }

    /// A named dataset.
    pub fn dataset(&self, name: &str) -> Option<&Dataset> {
        self.datasets.get(name).map(|dataset| dataset.as_ref())
    }

    /// Register a native custom widget.
    pub fn register_custom(&mut self, kind: impl Into<String>, widget: impl crate::custom::CustomWidget + 'static) {
        self.custom.register(kind, widget);
    }

    /// Handle one protocol message and return the response message.
    pub fn handle(&mut self, message: Message) -> Message {
        match message {
            Message::Hello { version, .. } => {
                if version > crate::protocol::PROTOCOL_VERSION {
                    Message::Error {
                        message: format!("unsupported protocol version {version}"),
                    }
                } else {
                    Message::Response {
                        id: 0,
                        response: Response::Ok,
                    }
                }
            }
            Message::Request { id, request } => {
                let response = match request {
                    Request::ApplyPatch { patch } => match self.apply(&patch) {
                        Ok(()) => Response::Ok,
                        Err(error) => Response::Error {
                            message: error.to_string(),
                        },
                    },
                    Request::GetSpec => Response::Spec {
                        spec: self.json.clone(),
                    },
                    Request::RunCommand { id } => {
                        self.push_event(HostEvent::Command { id });
                        Response::Ok
                    }
                    Request::SetData { name, columns } => {
                        self.set_dataset(name, dataset_from_columns(columns));
                        Response::Ok
                    }
                    Request::Reload => {
                        if self.source.is_some() && self.reload_if_changed() {
                            Response::Ok
                        } else {
                            Response::Error {
                                message: "no changed source to reload".to_owned(),
                            }
                        }
                    }
                };
                Message::Response { id, response }
            }
            Message::Patch { patch } => match self.apply(&patch) {
                Ok(()) => Message::Response {
                    id: 0,
                    response: Response::Ok,
                },
                Err(error) => Message::Error {
                    message: error.to_string(),
                },
            },
            Message::Data { name, columns } => {
                self.set_dataset(name, dataset_from_columns(columns));
                Message::Response {
                    id: 0,
                    response: Response::Ok,
                }
            }
            Message::Response { .. } | Message::Event { .. } | Message::Error { .. } => Message::Error {
                message: "the host cannot consume this message".to_owned(),
            },
        }
    }

    fn seed_toasts(&mut self) {
        let toasts = self.spec.toasts.clone();
        for toast in &toasts {
            self.push_toast(toast);
        }
    }

    fn push_toast(&mut self, spec: &ToastSpec) {
        use egui_workbench::components::{Toast, ToastKind};
        let kind = match spec.kind.as_str() {
            "success" => ToastKind::Success,
            "warning" => ToastKind::Warning,
            "error" => ToastKind::Error,
            "loading" => ToastKind::Loading,
            _ => ToastKind::Info,
        };
        let mut toast = Toast::new(kind, spec.title.clone());
        if let Some(description) = &spec.description {
            toast = toast.description(description.clone());
        }
        self.toasts.add(toast);
    }

    /// Draw the host in a root `Ui` and collect its events.
    ///
    /// The native runner and the WebAssembly host both call this; the event queue is
    /// drained with [`Self::poll_events`].
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.ui(ui);
    }

    /// Run one headless frame and return the events it produced.
    ///
    /// This renders with a default `egui::Context` and applies the action queue first,
    /// so a test or a Python `step` sees the same tree and events as a window.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn frame(&mut self) -> Vec<HostEvent> {
        let _ = self.apply_actions();
        let ctx = self.headless_context.get_or_insert_with(egui::Context::default).clone();
        let size = egui::vec2(self.spec.window.width, self.spec.window.height);
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| self.ui(ui));
        output.textures_delta.clear();
        self.poll_events()
    }

    fn seed_values(&mut self) {
        native::seed_widgets(&self.spec.widgets, &mut self.values, &mut self.tabs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_patch_keeps_what_the_user_typed_in_untouched_widgets() {
        let spec = r#"{"widgets":[{"kind":"text_field","id":"a","label":"A","hint":"","value":"x"},{"kind":"text_field","id":"b","label":"B","hint":"","value":"y"}]}"#;
        let mut host = Host::new(HostSpec::from_json(spec).expect("spec")).expect("host");
        host.values.insert("a".to_owned(), json!("typed"));
        host.values.insert("b".to_owned(), json!("typed"));
        host.apply(&Patch::Set {
            path: "/widgets/1/value".to_owned(),
            value: json!("patched"),
        })
        .expect("patch");
        assert_eq!(host.values["a"], json!("typed"));
        assert_eq!(host.values["b"], json!("patched"));
    }
}
