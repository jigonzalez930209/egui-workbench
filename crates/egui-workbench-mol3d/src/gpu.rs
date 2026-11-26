//! GPU sphere impostors through an `egui_wgpu` paint callback.
//!
//! Instances are sorted far-to-near and rendered as billboards with an analytic
//! ray-sphere test in the fragment shader. The painter viewport remains the reference
//! and the fallback.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use egui::epaint::PaintCallback;
use egui::{Color32, Rect, Ui};
use egui_wgpu::wgpu;

use crate::camera::{Camera3D, Projection3D};
use crate::math::Vec3;
use crate::scene::Mol3dScene;

static OUTPUT_FORMAT: AtomicU8 = AtomicU8::new(0);

/// Set the host's wgpu surface format.
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

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    center: [f32; 3],
    radius: f32,
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
    eye: [f32; 4],
    right: [f32; 4],
    up: [f32; 4],
    forward: [f32; 4],
    params: [f32; 4],
}

struct Buffers {
    vertices: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    instances: u32,
}

struct Resources {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    buffers: HashMap<u64, Buffers>,
}

struct AtomCallback {
    id: u64,
    instances: Vec<Instance>,
    uniform: Uniform,
}

impl egui_wgpu::CallbackTrait for AtomCallback {
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
        let instance_bytes: &[u8] = bytemuck::cast_slice(&self.instances);
        let vertex = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("workbench_mol3d_instances"),
            size: instance_bytes.len().max(4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertex, 0, instance_bytes);
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("workbench_mol3d_uniform"),
            size: std::mem::size_of::<Uniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&uniform, 0, bytemuck::bytes_of(&self.uniform));
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("workbench_mol3d_bind_group"),
            layout: &resources.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform,
                    offset: 0,
                    size: None,
                }),
            }],
        });
        resources.buffers.insert(
            self.id,
            Buffers {
                vertices: vertex,
                bind_group,
                instances: self.instances.len() as u32,
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
        let Some(buffers) = resources.buffers.get(&self.id) else {
            return;
        };
        render_pass.set_pipeline(&resources.pipeline);
        render_pass.set_bind_group(0, &buffers.bind_group, &[]);
        render_pass.set_vertex_buffer(0, buffers.vertices.slice(..));
        render_pass.draw(0..6, 0..buffers.instances);
    }
}

impl Resources {
    fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("workbench_mol3d_shader"),
            source: wgpu::ShaderSource::Wgsl(WGSL.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("workbench_mol3d_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: false,
                    min_binding_size: None,
                    ty: wgpu::BufferBindingType::Uniform,
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("workbench_mol3d_pipeline_layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("workbench_mol3d_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                entry_point: Some("vs_main"),
                module: &shader,
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Instance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: 12,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32,
                        },
                        wgpu::VertexAttribute {
                            offset: 16,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                    ],
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
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
            buffers: HashMap::new(),
        }
    }
}

/// Paint the scene atoms through the GPU callback. Returns false when the camera is
/// orthographic or the scene is empty, so the caller can use the painter fallback.
pub fn paint_atoms(
    ui: &Ui,
    viewport: Rect,
    camera: Camera3D,
    scene: &Mol3dScene,
    scale: f64,
    representations: &crate::representation::RepresentationSet,
    colors: crate::color::ColorScheme,
) -> bool {
    let Projection3D::Perspective { fov_degrees } = camera.projection else {
        return false;
    };
    let mut instances = scene
        .atoms
        .iter()
        .filter_map(|atom| {
            let representation = representations.for_atom(atom.id);
            if !representation.shows_atoms() {
                return None;
            }
            let (_, depth) = camera.project(atom.position, viewport)?;
            let [r, g, b] = colors.color(atom);
            Some((
                depth,
                Instance {
                    center: [atom.position.x as f32, atom.position.y as f32, atom.position.z as f32],
                    radius: (representation.atom_radius(atom) * scale) as f32,
                    color: [f32::from(r) / 255.0, f32::from(g) / 255.0, f32::from(b) / 255.0, 1.0],
                },
            ))
        })
        .collect::<Vec<_>>();
    if instances.is_empty() {
        return false;
    }
    instances.sort_by(|a, b| b.0.total_cmp(&a.0));
    let (right, up, back) = camera.rotation.basis();
    let uniform = Uniform {
        eye: vec4(camera.eye()),
        right: vec4(right),
        up: vec4(up),
        forward: vec4(-back),
        params: [
            (fov_degrees.to_radians() * 0.5).tan() as f32,
            viewport.width() / viewport.height().max(1.0),
            0.0,
            0.0,
        ],
    };
    let id = ui.scope_id().with("mol3d_gpu");
    ui.painter().add(PaintCallback {
        rect: viewport,
        callback: Arc::new(AtomCallback {
            id: id.value(),
            instances: instances.into_iter().map(|(_, instance)| instance).collect(),
            uniform,
        }),
    });
    true
}

fn vec4(value: Vec3) -> [f32; 4] {
    [value.x as f32, value.y as f32, value.z as f32, 0.0]
}

/// Keep the color type available for the API documentation.
pub type AtomColor = Color32;

const WGSL: &str = r#"
struct Uniform {
    eye: vec4<f32>,
    right: vec4<f32>,
    up: vec4<f32>,
    forward: vec4<f32>,
    params: vec4<f32>,
};
@group(0) @binding(0) var<uniform> camera: Uniform;
struct Instance {
    @location(0) center: vec3<f32>,
    @location(1) radius: f32,
    @location(2) color: vec4<f32>,
};
struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) center: vec3<f32>,
    @location(2) radius: f32,
    @location(3) color: vec4<f32>,
};
const CORNERS = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
);
@vertex
fn vs_main(@builtin(vertex_index) vertex: u32, instance: Instance) -> VertexOut {
    var out: VertexOut;
    let corner = CORNERS[vertex] * instance.radius * 1.35;
    let world = instance.center + camera.right.xyz * corner.x + camera.up.xyz * corner.y;
    let relative = world - camera.eye.xyz;
    let depth = dot(relative, camera.forward.xyz);
    let tan_half = camera.params.x;
    let aspect = camera.params.y;
    let x = dot(relative, camera.right.xyz) / (depth * tan_half * aspect);
    let y = dot(relative, camera.up.xyz) / (depth * tan_half);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.world = world;
    out.center = instance.center;
    out.radius = instance.radius;
    out.color = instance.color;
    return out;
}
@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let ray = normalize(in.world - camera.eye.xyz);
    let oc = camera.eye.xyz - in.center;
    let b = dot(oc, ray);
    let c = dot(oc, oc) - in.radius * in.radius;
    let discriminant = b * b - c;
    if discriminant < 0.0 { discard; }
    let t = -b - sqrt(discriminant);
    if t < 0.0 { discard; }
    let hit = camera.eye.xyz + ray * t;
    let normal = normalize(hit - in.center);
    let light = normalize(vec3<f32>(0.35, 0.7, 0.9));
    let diffuse = max(dot(normal, light), 0.18);
    let color = in.color.rgb * (0.42 + 0.58 * diffuse);
    return vec4<f32>(color, in.color.a);
}
"#;
