use std::cell::RefCell;
use std::rc::Rc;

use egui_workbench::app::ThemeState;
use egui_workbench::{ThemeId, eframe};
use egui_workbench_host::{Host, HostSpec, Patch};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

/// A native spec rendered in a browser canvas.
#[wasm_bindgen]
pub struct WebWorkbench {
    host: Rc<RefCell<Host>>,
    theme: Rc<RefCell<Option<ThemeState>>>,
    context: Rc<RefCell<Option<egui::Context>>>,
    choice: Rc<RefCell<(String, bool, bool)>>,
    runner: RefCell<Option<eframe::WebRunner>>,
}

#[wasm_bindgen]
impl WebWorkbench {
    /// Create a workbench from a versioned `HostSpec` JSON document.
    #[wasm_bindgen(constructor)]
    pub fn new(spec_json: &str) -> Result<Self, JsValue> {
        console_error_panic_hook::set_once();
        let spec = HostSpec::from_json(spec_json).map_err(|error| JsValue::from_str(&error))?;
        let theme = (spec.theme.id.clone(), spec.theme.follow_system, spec.theme.reduced_motion);
        let host = Host::new(spec).map_err(|error| JsValue::from_str(&error.to_string()))?;
        Ok(Self {
            host: Rc::new(RefCell::new(host)),
            theme: Rc::new(RefCell::new(None)),
            context: Rc::new(RefCell::new(None)),
            choice: Rc::new(RefCell::new(theme)),
            runner: RefCell::new(None),
        })
    }

    /// Mount the workbench on a canvas. Call once.
    pub async fn start(&self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        if self.runner.borrow().is_some() {
            return Err(JsValue::from_str("the workbench is already running"));
        }
        let runner = eframe::WebRunner::new();
        let host = Rc::clone(&self.host);
        let theme = Rc::clone(&self.theme);
        let context = Rc::clone(&self.context);
        let choice = Rc::clone(&self.choice);
        runner
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(move |cc| {
                    let (id, follow, reduced) = choice.borrow().clone();
                    let mut state = ThemeState::new(&cc.egui_ctx, ThemeId::from_setting(&id), follow);
                    state.set_reduced_motion(&cc.egui_ctx, reduced);
                    *theme.borrow_mut() = Some(state);
                    *context.borrow_mut() = Some(cc.egui_ctx.clone());
                    Ok(Box::new(WebApp {
                        host,
                        theme: Rc::clone(&theme),
                        choice,
                    }))
                }),
            )
            .await?;
        self.runner.replace(Some(runner));
        Ok(())
    }

    /// Apply a JSON-pointer patch to the spec.
    pub fn apply_patch(&self, patch_json: &str) -> Result<(), JsValue> {
        let patch: Patch = serde_json::from_str(patch_json).map_err(|error| JsValue::from_str(&error.to_string()))?;
        self.host
            .borrow_mut()
            .apply(&patch)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        self.request_repaint();
        Ok(())
    }

    /// Replace the spec.
    pub fn set_spec(&self, spec_json: &str) -> Result<(), JsValue> {
        let spec = HostSpec::from_json(spec_json).map_err(|error| JsValue::from_str(&error))?;
        self.host
            .borrow_mut()
            .set_spec(spec)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        self.request_repaint();
        Ok(())
    }

    /// The current spec as JSON, including queued actions that were applied.
    pub fn spec_json(&self) -> String {
        serde_json::to_string(self.host.borrow().json()).unwrap_or_default()
    }

    /// Drain host events as a JSON array.
    pub fn poll_events_json(&self) -> String {
        let events = self.host.borrow_mut().poll_events();
        serde_json::to_string(&events).unwrap_or_else(|_| "[]".to_owned())
    }

    /// Change the theme id without rebuilding the window.
    pub fn set_theme(&self, id: &str) {
        self.choice.borrow_mut().0 = id.to_owned();
        if let (Some(theme), Some(context)) = (self.theme.borrow_mut().as_mut(), self.context.borrow().as_ref()) {
            theme.set_requested(context, ThemeId::from_setting(id));
        }
        self.request_repaint();
    }

    /// The `HostSpec` JSON Schema.
    pub fn schema() -> String {
        serde_json::to_string_pretty(&HostSpec::schema()).unwrap_or_default()
    }

    /// The host crate version.
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }

    /// Stop eframe and release its listeners.
    pub fn destroy(&self) {
        if let Some(runner) = self.runner.borrow_mut().take() {
            runner.destroy();
        }
    }

    fn request_repaint(&self) {
        if let Some(context) = self.context.borrow().as_ref() {
            context.request_repaint();
        }
    }
}

struct WebApp {
    host: Rc<RefCell<Host>>,
    theme: Rc<RefCell<Option<ThemeState>>>,
    choice: Rc<RefCell<(String, bool, bool)>>,
}

impl eframe::App for WebApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (id, reduced) = {
            let choice = self.choice.borrow();
            (choice.0.clone(), choice.2)
        };
        if let Some(theme) = self.theme.borrow_mut().as_mut() {
            if theme.requested() != ThemeId::from_setting(&id) {
                theme.set_requested(ctx, ThemeId::from_setting(&id));
            }
            theme.set_reduced_motion(ctx, reduced);
            theme.sync_system(ctx);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut host = self.host.borrow_mut();
        if host.apply_actions() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        host.show(ui);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.theme
            .borrow()
            .as_ref()
            .map(ThemeState::clear_color)
            .unwrap_or([0.0, 0.0, 0.0, 0.0])
    }
}
