//! The native window runner and spec seeding helpers.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::SystemTime;

use serde_json::Value;

#[cfg(target_arch = "wasm32")]
use super::HostError;
use super::{Host, HostEvent};
use crate::spec::WidgetSpec;

impl Host {
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
        let mut native = egui_workbench::native_options_sized(&title, [width, height], [640.0, 420.0]);
        // The spec's `resizable` was never applied.
        native.viewport = native.viewport.with_resizable(self.spec.window.resizable);
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
}

pub(super) fn seed_widgets(
    widgets: &[WidgetSpec],
    values: &mut BTreeMap<String, Value>,
    tabs: &mut BTreeMap<String, usize>,
) {
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

pub(super) fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|metadata| metadata.modified()).ok()
}
