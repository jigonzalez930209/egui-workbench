//! A spec-driven WebAssembly host.
//!
//! `WebWorkbench` owns a `Host`, mounts it on a canvas with eframe's WebRunner, and
//! exposes patches, events, and the generated schema to JavaScript. The JavaScript
//! wrapper in `js/index.js` adds the npm package's ESM surface and the remote client.

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::*;

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    use crate::WebWorkbench;

    #[wasm_bindgen_test]
    fn patches_round_trip_in_the_wasm_host() {
        let spec = r#"{"version":1,"widgets":[{"kind":"label","text":"before","size":13.0,"tone":"default","strong":false}]}"#;
        let workbench = WebWorkbench::new(spec).expect("host");
        workbench
            .apply_patch(r#"{"op":"set","path":"/widgets/0/text","value":"after"}"#)
            .expect("patch");
        let json = workbench.spec_json();
        assert!(json.contains("after"));
        assert!(!json.contains("before"));
        assert!(!WebWorkbench::schema().is_empty());
    }

    #[wasm_bindgen_test]
    async fn starts_on_a_canvas() {
        use wasm_bindgen::JsCast;

        let document = web_sys::window().expect("window").document().expect("document");
        let canvas = document
            .create_element("canvas")
            .expect("canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("canvas element");
        canvas.set_width(320);
        canvas.set_height(200);
        document.body().expect("body").append_child(&canvas).expect("append");
        let spec = r#"{"version":1,"widgets":[{"kind":"label","text":"browser","size":13.0,"tone":"default","strong":false}]}"#;
        let workbench = WebWorkbench::new(spec).expect("host");
        workbench.start(canvas).await.expect("start");
    }
}
