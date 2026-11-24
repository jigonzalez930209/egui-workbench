//! The default conformance spec renders through a headless wgpu harness.

use egui_kittest::Harness;
use egui_kittest::wgpu::{WgpuTestRenderer, create_render_state, default_wgpu_setup};
use egui_workbench_host::{Host, HostSpec};

#[test]
fn the_default_spec_renders() {
    let render_state = create_render_state(default_wgpu_setup(), egui_wgpu::RendererOptions::PREDICTABLE);
    let renderer = WgpuTestRenderer::from_render_state(render_state);
    let host = Host::new(HostSpec::default()).expect("host");
    let mut harness = Harness::builder()
        .with_size(egui::vec2(640.0, 480.0))
        .with_max_steps(8)
        .renderer(renderer)
        .build_ui_state(
            move |ui, host: &mut Host| {
                host.ui(ui);
            },
            host,
        );
    harness.run();
    let image = harness.render().expect("render");
    assert!(image.width() > 0 && image.height() > 0);
}
