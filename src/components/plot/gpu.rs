//! A small GPU line/marker renderer over `egui_wgpu` paint callbacks.
//!
//! The CPU scene remains the reference and the fallback. A plot built with
//! [`super::Renderer::Gpu`] collects its line and scatter series here; the callback
//! uploads a triangle list and draws it in the same render pass as egui.
//!
//! The output format must match the host's wgpu surface. Call
//! [`set_output_format`] once during startup (for example from an `eframe` wgpu
//! creation context). The default is `Bgra8UnormSrgb`, which is what winit/eframe use
//! on most desktops.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use egui::epaint::PaintCallback;
use egui::{Rect, Ui};
use egui_wgpu::wgpu;

/// Which wgpu surface format the line pipeline targets.
static OUTPUT_FORMAT: AtomicU8 = AtomicU8::new(0);

/// Set the host's wgpu surface format. Returns whether the format is supported.
///
/// Call before the first plot frame. Supported: `Bgra8UnormSrgb` (default),
/// `Rgba8UnormSrgb`, `Bgra8Unorm`, and `Rgba8Unorm`.
pub fn set_output_format(format: wgpu::TextureFormat) -> bool {
    let value = match format {
        wgpu::TextureFormat::Bgra8UnormSrgb => 0,
        wgpu::TextureFormat::Rgba8UnormSrgb => 1,
        wgpu::TextureFormat::Bgra8Unorm => 2,
        wgpu::TextureFormat::Rgba8Unorm => 3,
        _ => return false,
    };
    OUTPUT_FORMAT.store(value, Ordering::Relaxed);
    true
}

fn output_format() -> wgpu::TextureFormat {
    match OUTPUT_FORMAT.load(Ordering::Relaxed) {
        1 => wgpu::TextureFormat::Rgba8UnormSrgb,
        2 => wgpu::TextureFormat::Bgra8Unorm,
        3 => wgpu::TextureFormat::Rgba8Unorm,
        _ => wgpu::TextureFormat::Bgra8UnormSrgb,
    }
}

use super::gpu_line::GpuLine;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
    size: [f32; 2],
    _padding: [f32; 2],
}

struct LineBuffer {
    vertex: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertices: u32,
}

struct Resources {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    lines: HashMap<u64, LineBuffer>,
}

struct LineCallback {
    id: u64,
    vertices: Vec<Vertex>,
    size: [f32; 2],
}

impl egui_wgpu::CallbackTrait for LineCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if resources.get::<Resources>().is_none() {
            resources.insert(Resources::new(device));
        }
        let Some(resources) = resources.get_mut::<Resources>() else {
            return Vec::new();
        };
        let vertex_bytes: &[u8] = bytemuck::cast_slice(&self.vertices);
        let uniform = Uniform {
            size: self.size,
            _padding: [0.0; 2],
        };
        let vertex = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("workbench_gpu_plot_vertices"),
            size: vertex_bytes.len().max(4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertex, 0, vertex_bytes);
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("workbench_gpu_plot_uniform"),
            size: std::mem::size_of::<Uniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&uniform_buffer, 0, bytemuck::bytes_of(&uniform));
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("workbench_gpu_plot_bind_group"),
            layout: &resources.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });
        resources.lines.insert(
            self.id,
            LineBuffer {
                vertex,
                bind_group,
                vertices: self.vertices.len() as u32,
            },
        );
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::epaint::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let Some(resources) = resources.get::<Resources>() else {
            return;
        };
        let Some(line) = resources.lines.get(&self.id) else {
            return;
        };
        render_pass.set_pipeline(&resources.pipeline);
        render_pass.set_bind_group(0, &line.bind_group, &[]);
        render_pass.set_vertex_buffer(0, line.vertex.slice(..));
        render_pass.draw(0..line.vertices, 0..1);
    }
}

impl Resources {
    fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("workbench_gpu_plot_shader"),
            source: wgpu::ShaderSource::Wgsl(WGSL.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("workbench_gpu_plot_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: false,
                    min_binding_size: None,
                    ty: wgpu::BufferBindingType::Uniform,
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("workbench_gpu_plot_pipeline_layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("workbench_gpu_plot_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                entry_point: Some("vs_main"),
                module: &shader,
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                entry_point: Some("fs_main"),
                module: &shader,
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format(),
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            layout,
            lines: HashMap::new(),
        }
    }
}

const WGSL: &str = r#"
struct Uniform { size: vec2<f32>, padding: vec2<f32> };
@group(0) @binding(0) var<uniform> frame: Uniform;
struct VertexIn { @location(0) position: vec2<f32>, @location(1) color: vec4<f32> };
struct VertexOut { @builtin(position) position: vec4<f32>, @location(0) color: vec4<f32> };
@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    let ndc = vec2<f32>(in.position.x / frame.size.x * 2.0 - 1.0, 1.0 - in.position.y / frame.size.y * 2.0);
    out.position = vec4<f32>(ndc, 0.0, 1.0);
    out.color = in.color;
    return out;
}
@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

/// Upload and draw line runs in `frame`. Call once per plot frame when the GPU renderer
/// is selected.
pub fn paint_lines(ui: &Ui, frame: Rect, lines: &[GpuLine], plot_id: egui::Id) {
    let mut vertices = Vec::new();
    for line in lines {
        let color = line.color.to_normalized_gamma_f32();
        let points = &line.points;
        if points.len() < 2 {
            continue;
        }
        let segments = if line.closed { points.len() } else { points.len() - 1 };
        for index in 0..segments {
            let a = points[index] - frame.min.to_vec2();
            let b = points[(index + 1) % points.len()] - frame.min.to_vec2();
            let delta = b - a;
            let length = delta.length();
            if length <= f32::EPSILON {
                continue;
            }
            let normal = egui::vec2(-delta.y, delta.x) / length * line.width.max(0.5);
            let quad = [a + normal, a - normal, b + normal, b + normal, a - normal, b - normal];
            for point in quad {
                vertices.push(Vertex {
                    position: [point.x, point.y],
                    color,
                });
            }
        }
    }
    if vertices.is_empty() {
        return;
    }
    let id = plot_id.value() ^ (u64::from(frame.left().to_bits()) << 32) ^ u64::from(frame.top().to_bits());
    let callback = LineCallback {
        id,
        vertices,
        size: [frame.width().max(1.0), frame.height().max(1.0)],
    };
    let callback = PaintCallback {
        rect: frame,
        callback: Arc::new(egui_wgpu::Callback::new_paint_callback(frame, callback)),
    };
    ui.painter().add(egui::Shape::Callback(callback));
}

/// Collect line and scatter series in screen space for the GPU callback.
pub(super) fn collect(
    plot: &super::Plot<'_>,
    transform: &super::axis::Transform,
    state: &super::PlotState,
    theme: &crate::theme::Theme,
) -> Vec<GpuLine> {
    let mut lines = Vec::new();
    for (index, series) in plot.series.iter().enumerate() {
        if state.is_hidden(&series.name) {
            continue;
        }
        let look = super::overlay::look(plot, index, theme);
        match series.kind {
            super::series::Kind::Line { .. } => {
                let indices = super::paint::visible_indices(transform, series, series.sorted_x(), look.radius);
                let points: Vec<egui::Pos2> = indices
                    .iter()
                    .map(|index| transform.to_screen(series.data()[*index]))
                    .filter(|point| point.is_finite())
                    .collect();
                if points.len() >= 2 {
                    lines.push(GpuLine::new(points, look.color, look.width));
                }
            }
            super::series::Kind::Scatter { .. } => {
                let indices = super::paint::visible_indices(transform, series, series.sorted_x(), look.radius);
                for index in indices {
                    let point = transform.to_screen(series.data()[index]);
                    if !point.is_finite() || !transform.frame.expand(look.radius * 2.0).contains(point) {
                        continue;
                    }
                    let radius = look.radius;
                    let diamond = vec![
                        egui::pos2(point.x, point.y - radius),
                        egui::pos2(point.x + radius, point.y),
                        egui::pos2(point.x, point.y + radius),
                        egui::pos2(point.x - radius, point.y),
                    ];
                    lines.push(GpuLine::new(diamond, look.color, 1.5).closed());
                }
            }
            _ => {}
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::plot::axis::Transform;
    use crate::components::plot::{Bounds, Plot, PlotState, Scale, Series};
    use egui::{Rect, pos2};

    #[test]
    fn supported_formats_are_accepted() {
        assert!(set_output_format(wgpu::TextureFormat::Bgra8UnormSrgb));
        assert!(set_output_format(wgpu::TextureFormat::Rgba8Unorm));
        assert!(!set_output_format(wgpu::TextureFormat::R32Float));
        assert!(set_output_format(wgpu::TextureFormat::Bgra8UnormSrgb));
    }

    #[test]
    fn a_line_series_collects_screen_points() {
        let data = [[0.0, 0.0], [1.0, 1.0], [2.0, 0.5]];
        let plot = Plot::new("gpu-test").series(Series::line("v", &data));
        let transform = Transform {
            frame: Rect::from_min_max(pos2(0.0, 0.0), pos2(200.0, 100.0)),
            view: Bounds::from_corners([0.0, 0.0], [2.0, 1.0]),
            x_scale: Scale::Linear,
            y_scale: Scale::Linear,
            y2: None,
        };
        let lines = collect(&plot, &transform, &PlotState::default(), &crate::theme::Theme::dark());
        assert_eq!(lines.len(), 1);
        assert!(lines[0].points.len() >= 2);
    }
}
