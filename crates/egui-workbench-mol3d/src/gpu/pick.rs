//! One-pixel ID-buffer readback for GPU picking.

use std::sync::mpsc;
use std::time::Duration;

use egui::Rect;
use egui_wgpu::wgpu;
use egui_workbench_chem_core::{AtomId, BondId};

use super::{BOND_BIT, PickRequest, PickTarget, Resources};

/// Render the ID pass and read the texel under the pointer.
#[allow(clippy::too_many_arguments)]
pub(super) fn read_pick(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    resources: &Resources,
    request: &PickRequest,
    id: u64,
    rect: Rect,
    pixels_per_point: f32,
    atom_ids: &[AtomId],
    bond_ids: &[BondId],
) {
    let Some(buffers) = resources.buffers.get(&id) else {
        set_result(request, None);
        return;
    };
    let width = ((rect.width().max(1.0) * pixels_per_point).ceil() as u32).max(1);
    let height = ((rect.height().max(1.0) * pixels_per_point).ceil() as u32).max(1);
    let local = request.pointer - rect.min;
    let x = (local.x * pixels_per_point).floor();
    let y = (local.y * pixels_per_point).floor();
    if !(0.0..width as f32).contains(&x) || !(0.0..height as f32).contains(&y) {
        set_result(request, None);
        return;
    }
    let (x, y) = (x as u32, y as u32);
    let color = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("workbench_mol3d_pick_ids"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Uint,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("workbench_mol3d_pick_depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let color_view = color.create_view(&wgpu::TextureViewDescriptor::default());
    let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("workbench_mol3d_pick_readback"),
        size: 256,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("workbench_mol3d_pick_encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("workbench_mol3d_pick_pass"),
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
            pass.set_pipeline(&resources.bond_pick_pipeline);
            pass.set_vertex_buffer(0, buffers.bonds.slice(..));
            pass.draw(0..6, 0..buffers.bond_count);
        }
        if buffers.sphere_count > 0 {
            pass.set_pipeline(&resources.sphere_pick_pipeline);
            pass.set_vertex_buffer(0, buffers.spheres.slice(..));
            pass.draw(0..6, 0..buffers.sphere_count);
        }
    }
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &color,
            mip_level: 0,
            origin: wgpu::Origin3d { x, y, z: 0 },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: Some(1),
            },
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));
    let slice = readback.slice(..4);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    let mapped = receiver
        .recv_timeout(Duration::from_secs(2))
        .ok()
        .and_then(Result::ok)
        .is_some();
    if !mapped {
        let _ = device.poll(wgpu::PollType::Poll);
        set_result(request, None);
        return;
    }
    let id = match slice.get_mapped_range() {
        Ok(data) => data
            .get(..4)
            .map(|bytes| u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            .unwrap_or(0),
        Err(_) => 0,
    };
    readback.unmap();
    let target = decode(id, atom_ids, bond_ids);
    set_result(request, target);
}

fn decode(id: u32, atom_ids: &[AtomId], bond_ids: &[BondId]) -> Option<PickTarget> {
    if id == 0 {
        return None;
    }
    if id & BOND_BIT != 0 {
        bond_ids
            .get((id & !BOND_BIT).wrapping_sub(1) as usize)
            .copied()
            .map(PickTarget::Bond)
    } else {
        atom_ids.get(id.wrapping_sub(1) as usize).copied().map(PickTarget::Atom)
    }
}

fn set_result(request: &PickRequest, target: Option<PickTarget>) {
    if let Ok(mut slot) = request.result.lock() {
        *slot = Some(target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_decode_atoms_and_bonds_and_background() {
        let atoms = [AtomId::new(3, 1), AtomId::new(4, 1)];
        let bonds = [BondId::new(8, 1)];
        assert_eq!(decode(0, &atoms, &bonds), None);
        assert_eq!(decode(1, &atoms, &bonds), Some(PickTarget::Atom(atoms[0])));
        assert_eq!(decode(2, &atoms, &bonds), Some(PickTarget::Atom(atoms[1])));
        assert_eq!(decode(3, &atoms, &bonds), None);
        assert_eq!(decode(BOND_BIT | 1, &atoms, &bonds), Some(PickTarget::Bond(bonds[0])));
    }
}
