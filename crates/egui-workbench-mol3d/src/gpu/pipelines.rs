//! Pipeline creation for the GPU impostors and the ID pass.

use std::collections::HashMap;

use egui_wgpu::wgpu;

use super::{BondInstance, Buffers, Instance, Resources, Uniform, output_format, shader};

impl Resources {
    pub(super) fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: u64,
        spheres: &[Instance],
        bonds: &[BondInstance],
        uniform: &Uniform,
    ) {
        let sphere_bytes: &[u8] = bytemuck::cast_slice(spheres);
        let sphere_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("workbench_mol3d_spheres"),
            size: sphere_bytes.len().max(4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&sphere_buffer, 0, sphere_bytes);
        let bond_bytes: &[u8] = bytemuck::cast_slice(bonds);
        let bond_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("workbench_mol3d_bonds"),
            size: bond_bytes.len().max(4) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&bond_buffer, 0, bond_bytes);
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("workbench_mol3d_uniform"),
            size: std::mem::size_of::<Uniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&uniform_buffer, 0, bytemuck::bytes_of(uniform));
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("workbench_mol3d_bind_group"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });
        self.buffers.insert(
            id,
            Buffers {
                spheres: sphere_buffer,
                bonds: bond_buffer,
                bind_group,
                quad_bind_group: None,
                sphere_count: spheres.len() as u32,
                bond_count: bonds.len() as u32,
            },
        );
    }

    pub(super) fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("workbench_mol3d_shader"),
            source: wgpu::ShaderSource::Wgsl(shader::WGSL.into()),
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
        let sphere_layout = wgpu::VertexBufferLayout {
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
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        };
        let bond_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BondInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 28,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        };
        let sphere_pipeline = display_pipeline(
            device,
            &pipeline_layout,
            &shader,
            "workbench_mol3d_spheres",
            "vs_sphere",
            "fs_sphere",
            &sphere_layout,
        );
        let bond_pipeline = display_pipeline(
            device,
            &pipeline_layout,
            &shader,
            "workbench_mol3d_bonds",
            "vs_bond",
            "fs_bond",
            &bond_layout,
        );
        let sphere_pick_pipeline = pick_pipeline(
            device,
            &pipeline_layout,
            &shader,
            "workbench_mol3d_sphere_ids",
            "vs_sphere",
            "fs_sphere_id",
            &sphere_layout,
        );
        let bond_pick_pipeline = pick_pipeline(
            device,
            &pipeline_layout,
            &shader,
            "workbench_mol3d_bond_ids",
            "vs_bond",
            "fs_bond_id",
            &bond_layout,
        );
        let quad_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("workbench_mol3d_quad_shader"),
            source: wgpu::ShaderSource::Wgsl(super::shader::QUAD_WGSL.into()),
        });
        let quad_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("workbench_mol3d_quad_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let quad_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("workbench_mol3d_quad_pipeline_layout"),
            bind_group_layouts: &[Some(&quad_layout)],
            immediate_size: 0,
        });
        let quad_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("workbench_mol3d_quad"),
            layout: Some(&quad_pipeline_layout),
            vertex: wgpu::VertexState {
                entry_point: Some("vs_quad"),
                module: &quad_shader,
                buffers: &[],
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
                entry_point: Some("fs_quad"),
                module: &quad_shader,
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
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("workbench_mol3d_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            sphere_pipeline,
            bond_pipeline,
            sphere_pick_pipeline,
            bond_pick_pipeline,
            quad_pipeline,
            layout,
            quad_layout,
            sampler,
            buffers: HashMap::new(),
        }
    }
}

fn display_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    label: &str,
    vertex: &str,
    fragment: &str,
    buffer_layout: &wgpu::VertexBufferLayout<'_>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            entry_point: Some(vertex),
            module: shader,
            buffers: &[Some(buffer_layout.clone())],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            entry_point: Some(fragment),
            module: shader,
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn pick_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    label: &str,
    vertex: &str,
    fragment: &str,
    buffer_layout: &wgpu::VertexBufferLayout<'_>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            entry_point: Some(vertex),
            module: shader,
            buffers: &[Some(buffer_layout.clone())],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            entry_point: Some(fragment),
            module: shader,
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::R32Uint,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}
