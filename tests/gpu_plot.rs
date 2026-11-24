//! The GPU data layer renders through an actual wgpu callback.

#![cfg(feature = "gpu")]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use egui_kittest::Harness;
use egui_kittest::wgpu::{WgpuTestRenderer, create_render_state, default_wgpu_setup};
use egui_workbench::components::plot::{Plot, PlotState, Renderer, Series, set_output_format};
use egui_workbench::egui;

#[test]
fn the_gpu_renderer_draws_a_line_plot() {
    let render_state = create_render_state(default_wgpu_setup(), egui_wgpu::RendererOptions::PREDICTABLE);
    assert!(set_output_format(render_state.target_format));
    let renderer = WgpuTestRenderer::from_render_state(render_state);
    let data: Vec<[f64; 2]> = (0..1_000)
        .map(|index| {
            let t = index as f64 * 0.01;
            [t, (t * 2.0).sin()]
        })
        .collect();
    let mut icons = egui_workbench::Icons::new();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(420.0, 260.0))
        .with_max_steps(4)
        .renderer(renderer)
        .build_ui_state(
            move |ui, state: &mut PlotState| {
                let _ = Plot::new("gpu")
                    .renderer(Renderer::Gpu)
                    .height(240.0)
                    .series(Series::line("gpu-line", data.as_slice()))
                    .show(ui, &mut icons, state);
            },
            PlotState::default(),
        );
    harness.run();
    let image = harness.render().expect("GPU render");
    assert!(image.width() > 0 && image.height() > 0);
}
