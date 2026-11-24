//! The host runtime: spec, patches, events, data, protocol, and the native runner.

use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use egui_workbench::Icons;
use egui_workbench::components::{DataGridState, Toasts};
use egui_workbench::data::Dataset;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::custom::CustomWidgetRegistry;
use crate::data::{Datasets, dataset_from_columns};
use crate::patch::Patch;
use crate::protocol::{Message, Request, Response};
use crate::spec::{HostSpec, SPEC_VERSION, ToastSpec, WidgetSpec};

/// Something the host reports to the embedding application.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case")]
#[non_exhaustive]
pub enum HostEvent {
    /// A button or command ran.
    Command {
        /// Command id.
        id: String,
    },
    /// A value changed.
    ValueChanged {
        /// Widget id.
        id: String,
        /// New value.
        value: Value,
    },
    /// A tab strip changed.
    TabChanged {
        /// Strip id.
        id: String,
        /// Active index.
        index: usize,
    },
    /// A tree selection changed.
    TreeSelected {
        /// Tree id.
        id: String,
        /// Node key.
        key: String,
    },
    /// A custom widget emitted an event.
    Custom {
        /// Widget kind.
        kind: String,
        /// Opaque payload.
        payload: Value,
    },
    /// The host window was closed.
    Closed,
}

/// An action queued by an event handler and applied on the next frame.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum HostAction {
    /// Apply a patch to the spec.
    Patch(Patch),
    /// Show a toast.
    Toast(ToastSpec),
    /// Change the window title.
    SetTitle(String),
    /// Request another frame.
    Repaint,
    /// Close the window.
    Close,
}

/// A cloneable handle to the action queue, safe from any thread.
///
/// The queue also holds the current egui context, so a push from a worker thread wakes
/// the window instead of waiting for the next input event.
#[derive(Clone, Default)]
pub struct ActionQueue {
    actions: Arc<Mutex<VecDeque<HostAction>>>,
    waker: Arc<Mutex<Option<egui::Context>>>,
}

impl ActionQueue {
    /// Queue an action and wake the window, when it is running.
    pub fn push(&self, action: HostAction) {
        {
            let mut actions = self.actions.lock().unwrap_or_else(|error| error.into_inner());
            actions.push_back(action);
        }
        if let Some(ctx) = self.waker.lock().unwrap_or_else(|error| error.into_inner()).as_ref() {
            ctx.request_repaint();
        }
    }

    /// Remember the context used to wake the window.
    pub fn set_waker(&self, ctx: &egui::Context) {
        *self.waker.lock().unwrap_or_else(|error| error.into_inner()) = Some(ctx.clone());
    }

    fn drain(&self) -> Vec<HostAction> {
        let mut actions = self.actions.lock().unwrap_or_else(|error| error.into_inner());
        actions.drain(..).collect()
    }
}

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
        host.modified = modified(path);
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
                        self.events.push_back(HostEvent::Custom {
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
        let spec: HostSpec =
            serde_json::from_value(candidate.clone()).map_err(|error| HostError::Spec(error.to_string()))?;
        if spec.version > SPEC_VERSION {
            return Err(HostError::UnsupportedVersion(spec.version));
        }
        self.json = candidate;
        self.spec = spec;
        self.values.clear();
        self.tabs.clear();
        self.seed_values();
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
        let Some(path) = &self.source else {
            return false;
        };
        let current = modified(path);
        if current == self.modified {
            return false;
        }
        self.modified = current;
        match Self::load(path) {
            Ok(mut fresh) => {
                fresh.source = self.source.clone();
                fresh.modified = self.modified;
                fresh.datasets = std::mem::take(&mut self.datasets);
                *self = fresh;
                true
            }
            Err(error) => {
                self.events.push_back(HostEvent::Custom {
                    kind: "host.error".to_owned(),
                    payload: Value::String(error.to_string()),
                });
                false
            }
        }
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
                        self.events.push_back(HostEvent::Command { id });
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

    /// Run a native window. This owns the event loop and must run on the main thread.
    ///
    /// # Errors
    /// When eframe cannot create the window.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run(self) -> eframe::Result {
        self.run_with(|_| {})
    }

    /// Run a native window and call `on_event` for each host event.
    ///
    /// The callback runs on the UI thread with the event loop, so keep it short. This is
    /// the hook a Python binding or an embedding application uses to dispatch commands.
    ///
    /// # Errors
    /// When eframe cannot create the window.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_with<F>(self, on_event: F) -> eframe::Result
    where
        F: FnMut(&HostEvent) + 'static,
    {
        self.run_with_frame(on_event, || {})
    }

    /// Run a native window, call `on_event` for each host event, and call `on_frame` once
    /// per frame after the UI is drawn.
    ///
    /// Both callbacks run on the UI thread. `on_frame` is where an embedding turns posted
    /// work from another thread into spec patches.
    ///
    /// # Errors
    /// When eframe cannot create the window.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_with_frame<E, F>(self, on_event: E, on_frame: F) -> eframe::Result
    where
        E: FnMut(&HostEvent) + 'static,
        F: FnMut() + 'static,
    {
        let title = self.spec.window.title.clone();
        let width = self.spec.window.width;
        let height = self.spec.window.height;
        let native = egui_workbench::native_options_sized(&title, [width, height], [640.0, 420.0]);
        let mut on_event = Some(Box::new(on_event) as crate::view::EventCallback);
        let mut on_frame = Some(Box::new(on_frame) as crate::view::FrameCallback);
        eframe::run_native(
            &title,
            native,
            Box::new(move |cc| {
                self.actions.set_waker(&cc.egui_ctx);
                let mut theme = egui_workbench::app::ThemeState::new(
                    &cc.egui_ctx,
                    egui_workbench::ThemeId::from_setting(&self.spec.theme.id),
                    self.spec.theme.follow_system,
                );
                theme.set_reduced_motion(&cc.egui_ctx, self.spec.theme.reduced_motion);
                Ok(Box::new(crate::view::HostApp {
                    host: self,
                    theme,
                    on_event: on_event.take(),
                    on_frame: on_frame.take(),
                }))
            }),
        )
    }

    /// Run a native window, or return an error in a browser build.
    ///
    /// # Errors
    /// In a browser build.
    #[cfg(target_arch = "wasm32")]
    pub fn run(self) -> Result<(), HostError> {
        Err(HostError::Io("the native host cannot run in a browser".to_owned()))
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
        seed_widgets(&self.spec.widgets, &mut self.values, &mut self.tabs);
    }
}

fn seed_widgets(widgets: &[WidgetSpec], values: &mut BTreeMap<String, Value>, tabs: &mut BTreeMap<String, usize>) {
    for widget in widgets {
        match widget {
            WidgetSpec::TextField { id, value, .. } => {
                values.insert(id.clone(), Value::String(value.clone()));
            }
            WidgetSpec::Checkbox { id, checked, .. } => {
                values.insert(id.clone(), Value::Bool(*checked));
            }
            WidgetSpec::Switch { id, on, .. } => {
                values.insert(id.clone(), Value::Bool(*on));
            }
            WidgetSpec::Select { id, selected, .. } => {
                values.insert(id.clone(), Value::from(*selected));
            }
            WidgetSpec::Slider { id, value, .. } => {
                values.insert(id.clone(), Value::from(*value));
            }
            WidgetSpec::Tabs { id, active, .. } => {
                tabs.insert(id.clone(), *active);
                if let WidgetSpec::Tabs { children, .. } = widget {
                    for child in children.iter().flatten() {
                        seed_widgets(std::slice::from_ref(child), values, tabs);
                    }
                }
            }
            WidgetSpec::Row { children, .. }
            | WidgetSpec::Column { children, .. }
            | WidgetSpec::Card { children, .. } => seed_widgets(children, values, tabs),
            _ => {}
        }
    }
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|metadata| metadata.modified()).ok()
}
