//! Offscreen scene pass with an exact depth buffer, composited into egui.

use egui_wgpu::wgpu;

use super::Resources;

/// The result of rendering one viewport offscreen.
pub(super) struct OffscreenScene {
    /// Command buffer that renders the scene.
    pub command: wgpu::CommandBuffer,
    /// Bind group sampling the rendered color texture.
    pub bind_group: wgpu::BindGroup,
    /// Rendered color texture.
    #[allow(dead_code)]
    pub color: wgpu::Texture,
    /// Exact depth texture.
    #[allow(dead_code)]
    pub depth: wgpu::Texture,
}

/// Render the uploaded instances into a transparent color target with depth testing.
pub(super) fn render(
    device: &wgpu::Device,
    resources: &Resources,
    id: u64,
    width: u32,
    height: u32,
) -> Option<OffscreenScene> {
    let buffers = resources.buffers.get(&id)?;
    // A zero or oversized texture is a wgpu validation error, which aborts the frame. A
    // collapsed panel or a very large window can ask for either.
    let limit = device.limits().max_texture_dimension_2d;
    if width == 0 || height == 0 || width > limit || height > limit {
        return None;
    }
    let color = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("workbench_mol3d_scene_color"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("workbench_mol3d_scene_depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let color_view = color.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("workbench_mol3d_scene_encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("workbench_mol3d_scene_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_bind_group(0, &buffers.bind_group, &[]);
        if buffers.bond_count > 0 {
            pass.set_pipeline(&resources.bond_pipeline);
            pass.set_vertex_buffer(0, buffers.bonds.slice(..));
            pass.draw(0..6, 0..buffers.bond_count);
        }
        if buffers.sphere_count > 0 {
            pass.set_pipeline(&resources.sphere_pipeline);
            pass.set_vertex_buffer(0, buffers.spheres.slice(..));
            pass.draw(0..6, 0..buffers.sphere_count);
        }
    }
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("workbench_mol3d_quad_bind_group"),
        layout: &resources.quad_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&color_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&resources.sampler),
            },
        ],
    });
    Some(OffscreenScene {
        command: encoder.finish(),
        bind_group,
        color,
        depth,
    })
}
