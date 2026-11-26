//! GPU impostors and ID-buffer picking through an `egui_wgpu` paint callback.
//!
//! Atoms are analytic sphere impostors and bonds are analytic capsule/cylinder
//! impostors. The ID pass renders the same instanced geometry into an `R32Uint`
//! target with an exact fragment depth and reads back one pixel on click. The painter
//! viewport remains the reference and the fallback.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use egui::epaint::PaintCallback;
use egui::{Color32, Context, Pos2, Rect, Ui};
use egui_wgpu::wgpu;
use egui_workbench_chem_core::SelectionMode;

use crate::camera::{Camera3D, Projection3D};
use crate::math::Vec3;
use crate::scene::Mol3dScene;
use egui_workbench_chem_core::{AtomId, BondId};

mod instances;
mod offscreen;
mod pick;
mod pipelines;
mod shader;
#[cfg(test)]
mod tests;

/// Mask that marks a bond id in the GPU ID buffer.
const BOND_BIT: u32 = 0x8000_0000;

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

/// What a GPU pick found under the pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickTarget {
    /// An atom.
    Atom(AtomId),
    /// A bond.
    Bond(BondId),
}

/// One pending GPU pick.
#[derive(Clone)]
pub struct PickRequest {
    /// Pointer position in points.
    pub pointer: Pos2,
    /// Result slot; `None` means the pass has not completed.
    pub result: Arc<Mutex<Option<Option<PickTarget>>>>,
}

/// Deferred GPU picking state kept by the viewport across frames.
#[derive(Default)]
pub struct PickState {
    pending: Option<PendingPick>,
}

struct PendingPick {
    pointer: Pos2,
    viewport: Rect,
    mode: SelectionMode,
    result: Arc<Mutex<Option<Option<PickTarget>>>>,
    age: u8,
}

impl PickState {
    /// Schedule a pick for the next paint callback.
    pub fn schedule(&mut self, pointer: Pos2, viewport: Rect, mode: SelectionMode, context: &Context) {
        self.pending = Some(PendingPick {
            pointer,
            viewport,
            mode,
            result: Arc::new(Mutex::new(None)),
            age: 0,
        });
        context.request_repaint();
    }

    /// The request to hand to the paint callback, while one is pending.
    pub fn request(&self) -> Option<PickRequest> {
        self.pending.as_ref().map(|pending| PickRequest {
            pointer: pending.pointer,
            result: Arc::clone(&pending.result),
        })
    }

    /// The completed pick, if the GPU has written its result.
    pub fn take_result(&mut self) -> Option<(SelectionMode, Option<PickTarget>)> {
        let pending = self.pending.as_ref()?;
        let result = pending.result.lock().ok()?.take()?;
        let mode = pending.mode;
        self.pending = None;
        Some((mode, result))
    }

    /// Give up on a pick that never completed, returning the pointer for a CPU pick.
    pub fn take_fallback(&mut self) -> Option<(SelectionMode, Pos2, Rect)> {
        let pending = self.pending.as_mut()?;
        pending.age = pending.age.saturating_add(1);
        if pending.age < 3 || pending.result.lock().ok()?.is_some() {
            return None;
        }
        let pending = self.pending.take()?;
        Some((pending.mode, pending.pointer, pending.viewport))
    }
}

/// Whether synchronous GPU readback is available on this target.
pub const fn supports_gpu_picking() -> bool {
    !cfg!(target_arch = "wasm32")
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    center: [f32; 3],
    radius: f32,
    color: [f32; 4],
    pick: u32,
    _pad: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BondInstance {
    a: [f32; 3],
    _pad0: f32,
    b: [f32; 3],
    radius: f32,
    color: [f32; 4],
    pick: u32,
    _pad1: [u32; 3],
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
    spheres: wgpu::Buffer,
    bonds: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    quad_bind_group: Option<wgpu::BindGroup>,
    sphere_count: u32,
    bond_count: u32,
}

pub(crate) struct Resources {
    sphere_pipeline: wgpu::RenderPipeline,
    bond_pipeline: wgpu::RenderPipeline,
    sphere_pick_pipeline: wgpu::RenderPipeline,
    bond_pick_pipeline: wgpu::RenderPipeline,
    quad_pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    quad_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    buffers: HashMap<u64, Buffers>,
}

struct SceneCallback {
    id: u64,
    rect: Rect,
    spheres: Vec<Instance>,
    bonds: Vec<BondInstance>,
    atom_ids: Vec<AtomId>,
    bond_ids: Vec<BondId>,
    uniform: Uniform,
    pick: Option<PickRequest>,
}

impl egui_wgpu::CallbackTrait for SceneCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if resources.get::<Resources>().is_none() {
            resources.insert(Resources::new(device));
        }
        let Some(resources) = resources.get_mut::<Resources>() else {
            return Vec::new();
        };
        resources.upload(device, queue, self.id, &self.spheres, &self.bonds, &self.uniform);
        let width = ((self.rect.width().max(1.0) * screen_descriptor.pixels_per_point).ceil() as u32).clamp(1, 8192);
        let height = ((self.rect.height().max(1.0) * screen_descriptor.pixels_per_point).ceil() as u32).clamp(1, 8192);
        let scene = offscreen::render(device, resources, self.id, width, height);
        if let Some(scene) = &scene
            && let Some(buffers) = resources.buffers.get_mut(&self.id)
        {
            buffers.quad_bind_group = Some(scene.bind_group.clone());
        }
        if let Some(request) = &self.pick
            && supports_gpu_picking()
        {
            pick::read_pick(
                device,
                queue,
                resources,
                request,
                self.id,
                self.rect,
                screen_descriptor.pixels_per_point,
                &self.atom_ids,
                &self.bond_ids,
            );
        }
        scene.map(|scene| scene.command).into_iter().collect()
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
        let Some(bind_group) = &buffers.quad_bind_group else {
            return;
        };
        render_pass.set_pipeline(&resources.quad_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

/// Paint atoms and bonds through the GPU callback and optionally run an ID pick.
///
/// Returns false when the camera is orthographic or the scene is empty, so the caller
/// can use the painter fallback.
#[allow(clippy::too_many_arguments)]
pub fn paint_scene(
    ui: &Ui,
    viewport: Rect,
    camera: Camera3D,
    scene: &Mol3dScene,
    scale: f64,
    representations: &crate::representation::RepresentationSet,
    colors: crate::color::ColorScheme,
    style: &crate::view::Mol3dStyle,
    pick: Option<PickRequest>,
) -> bool {
    let Projection3D::Perspective { fov_degrees } = camera.projection else {
        return false;
    };
    let (spheres, bonds, atom_ids, bond_ids) =
        instances::build(scene, camera, viewport, scale, representations, colors, style);
    if spheres.is_empty() && bonds.is_empty() {
        return false;
    }
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
        callback: Arc::new(SceneCallback {
            id: id.value(),
            rect: viewport,
            spheres,
            bonds,
            atom_ids,
            bond_ids,
            uniform,
            pick,
        }),
    });
    true
}

fn vec4(value: Vec3) -> [f32; 4] {
    [value.x as f32, value.y as f32, value.z as f32, 0.0]
}

/// Keep the color type available for the API documentation.
pub type AtomColor = Color32;
