//! Web entry point for the documentation scenes and the component catalog.
//!
//! Build with `npm run build:demo` in `docs-site/` (see `docs-site/scripts/build-web-demo.sh`).
//! `docs-site/public/demo/index.html` loads the module, calls `start` with the page's
//! query string (`?scene=plot&theme=github-light`), and forwards the docs site's
//! light/dark switch to `set_theme`. Without a `scene`, the full catalog runs.

use egui_workbench::{ThemeId, eframe};
use wasm_bindgen::prelude::*;

#[path = "../../examples/scenes/scenes.rs"]
mod scenes;

/// The JavaScript handle to the running scene.
#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
    inbox: std::cell::RefCell<Option<scenes::SharedInbox>>,
}

impl Default for WebHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WebHandle {
    /// Install a panic hook and create the runner.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            runner: eframe::WebRunner::new(),
            inbox: std::cell::RefCell::new(None),
        }
    }

    /// Start the scene described by `query` on `canvas`. Called once from JavaScript.
    #[wasm_bindgen]
    pub async fn start(&self, canvas: web_sys::HtmlCanvasElement, query: String) -> Result<(), JsValue> {
        let params = scenes::Params::parse(&query);
        let embedded = params.embedded;
        let app = scenes::SceneApp::new(params).with_saver(Box::new(download));
        self.inbox.replace(Some(app.inbox()));
        let mut options = eframe::WebOptions::default();
        if embedded {
            // Without focus, let the wheel scroll the docs page (the app drops it too).
            options.should_prevent_default = Box::new(|event| match event {
                egui_workbench::egui::Event::MouseWheel { .. } | egui_workbench::egui::Event::Zoom(_) => has_focus(),
                _ => true,
            });
        }
        self.runner
            .start(canvas, options, Box::new(move |_| Ok(Box::new(app))))
            .await
    }

    /// Switch the theme (a setting string such as `github-dark`) without losing state.
    #[wasm_bindgen]
    pub fn set_theme(&self, theme: &str) {
        let Some(inbox) = self.inbox.borrow().clone() else {
            return;
        };
        if let Ok(mut inbox) = inbox.lock() {
            inbox.theme = Some(ThemeId::from_setting(theme));
            if let Some(ctx) = &inbox.ctx {
                ctx.request_repaint();
            }
        }
    }

    /// Stop eframe and release its listeners.
    #[wasm_bindgen]
    pub fn destroy(&self) {
        self.runner.destroy();
    }

    /// Whether the app panicked.
    #[wasm_bindgen]
    pub fn has_panicked(&self) -> bool {
        self.runner.has_panicked()
    }

    /// The panic message, when the app panicked.
    #[wasm_bindgen]
    pub fn panic_message(&self) -> Option<String> {
        self.runner.panic_summary().map(|summary| summary.message())
    }

    /// The panic callstack, when the app panicked.
    #[wasm_bindgen]
    pub fn panic_callstack(&self) -> Option<String> {
        self.runner.panic_summary().map(|summary| summary.callstack())
    }
}

/// Whether the scene has keyboard focus: an element other than `<body>` (the canvas or
/// egui's text agent) is active, which is egui's own notion of focus.
fn has_focus() -> bool {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return true;
    };
    match (document.active_element(), document.body()) {
        (Some(active), Some(body)) => active != *body.unchecked_ref::<web_sys::Element>(),
        (Some(_), None) => true,
        (None, _) => false,
    }
}

/// Save an exported file as a browser download.
fn download(name: &str, mime: &str, bytes: Vec<u8>) -> String {
    match try_download(name, mime, &bytes) {
        Ok(()) => format!("Downloaded {name}"),
        Err(error) => format!("Download failed: {error:?}"),
    }
}

fn try_download(name: &str, mime: &str, bytes: &[u8]) -> Result<(), JsValue> {
    let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(bytes));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type(mime);
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| JsValue::from_str("no document"))?;
    let anchor: web_sys::HtmlAnchorElement = document.create_element("a")?.dyn_into()?;
    anchor.set_href(&url);
    anchor.set_download(name);
    anchor.click();
    web_sys::Url::revoke_object_url(&url)?;
    Ok(())
}
