//! GPU shader, pipeline, and ID-buffer tests.

use super::*;

/// Build the headless GPU pipelines to validate the shaders and layouts.
///
/// Returns the adapter name, or an error when no adapter is available.
#[cfg(test)]
fn headless_smoke_test() -> Result<String, String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
        .map_err(|error| error.to_string())?;
    let (device, _queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("workbench_mol3d_smoke"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::default(),
        trace: wgpu::Trace::Off,
    }))
    .map_err(|error| error.to_string())?;
    let _resources = Resources::new(&device);
    Ok(adapter.get_info().name)
}
fn test_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("workbench_mol3d_pick_test"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::default(),
        trace: wgpu::Trace::Off,
    }))
    .ok()
}

fn test_uniform(camera: Camera3D, viewport: Rect) -> Uniform {
    let (right, up, back) = camera.rotation.basis();
    let Projection3D::Perspective { fov_degrees } = camera.projection else {
        unreachable!();
    };
    Uniform {
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
    }
}

#[test]
fn the_id_pass_picks_an_atom_under_the_pointer() {
    let Some((device, queue)) = test_device() else {
        return;
    };
    let mut resources = Resources::new(&device);
    let atom = AtomId::new(1, 1);
    let camera = Camera3D::default();
    let viewport = Rect::from_min_size(Pos2::ZERO, egui::vec2(200.0, 150.0));
    let uniform = test_uniform(camera, viewport);
    resources.upload(
        &device,
        &queue,
        1,
        &[Instance {
            center: [0.0, 0.0, 0.0],
            radius: 1.0,
            color: [1.0, 0.5, 0.0, 1.0],
            pick: 1,
            _pad: [0; 3],
        }],
        &[],
        &uniform,
    );
    let request = PickRequest {
        pointer: viewport.center(),
        result: Arc::new(Mutex::new(None)),
    };
    pick::read_pick(&device, &queue, &resources, &request, 1, viewport, 1.0, &[atom], &[]);
    assert_eq!(
        request.result.lock().expect("lock").as_ref().copied(),
        Some(Some(PickTarget::Atom(atom)))
    );
}

#[test]
fn the_id_pass_picks_a_bond_and_reports_background() {
    let Some((device, queue)) = test_device() else {
        return;
    };
    let mut resources = Resources::new(&device);
    let bond = BondId::new(2, 1);
    let camera = Camera3D::default();
    let viewport = Rect::from_min_size(Pos2::ZERO, egui::vec2(200.0, 150.0));
    let uniform = test_uniform(camera, viewport);
    resources.upload(
        &device,
        &queue,
        2,
        &[],
        &[BondInstance {
            a: [-1.0, 0.0, 0.0],
            _pad0: 0.0,
            b: [1.0, 0.0, 0.0],
            radius: 0.3,
            color: [0.0, 1.0, 0.0, 1.0],
            pick: BOND_BIT | 1,
            _pad1: [0; 3],
        }],
        &uniform,
    );
    let request = PickRequest {
        pointer: viewport.center(),
        result: Arc::new(Mutex::new(None)),
    };
    pick::read_pick(&device, &queue, &resources, &request, 2, viewport, 1.0, &[], &[bond]);
    assert_eq!(
        request.result.lock().expect("lock").as_ref().copied(),
        Some(Some(PickTarget::Bond(bond)))
    );
    let empty = PickRequest {
        pointer: Pos2::new(3.0, 3.0),
        result: Arc::new(Mutex::new(None)),
    };
    pick::read_pick(&device, &queue, &resources, &empty, 2, viewport, 1.0, &[], &[bond]);
    assert_eq!(empty.result.lock().expect("lock").as_ref().copied(), Some(None));
}

#[test]
fn the_wgsl_parses_and_validates() {
    let module = naga::front::wgsl::parse_str(shader::WGSL).expect("wgsl parses");
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::default(),
    );
    validator.validate(&module).expect("wgsl validates");
}

#[test]
fn the_headless_gpu_smoke_test_builds_pipelines_when_an_adapter_exists() {
    match headless_smoke_test() {
        Ok(adapter) => eprintln!("GPU impostor pipelines validated on {adapter}"),
        Err(error) => eprintln!("GPU smoke test skipped: {error}"),
    }
}

#[test]
fn pick_state_starts_and_finishes_deferred() {
    let mut state = PickState::default();
    let context = Context::default();
    state.schedule(
        Pos2::new(10.0, 20.0),
        Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 100.0)),
        SelectionMode::Replace,
        &context,
    );
    let request = state.request().expect("request");
    assert!(state.take_result().is_none());
    let atom = AtomId::new(7, 1);
    *request.result.lock().expect("lock") = Some(Some(PickTarget::Atom(atom)));
    let (mode, target) = state.take_result().expect("result");
    assert_eq!(mode, SelectionMode::Replace);
    assert_eq!(target, Some(PickTarget::Atom(atom)));
    assert!(state.request().is_none());
}

#[test]
fn the_offscreen_display_pass_writes_color_and_exact_depth() {
    let Some((device, queue)) = test_device() else {
        return;
    };
    let mut resources = Resources::new(&device);
    let camera = Camera3D::default();
    let viewport = Rect::from_min_size(Pos2::ZERO, egui::vec2(200.0, 150.0));
    let uniform = test_uniform(camera, viewport);
    resources.upload(
        &device,
        &queue,
        3,
        &[Instance {
            center: [0.0, 0.0, 0.0],
            radius: 1.0,
            color: [1.0, 0.5, 0.0, 1.0],
            pick: 1,
            _pad: [0; 3],
        }],
        &[],
        &uniform,
    );
    let scene = offscreen::render(&device, &resources, 3, 200, 150).expect("scene");
    queue.submit([scene.command]);
    let color = read_back(&device, &queue, &scene.color, 200, 150, 4);
    let color_offset = (75 * 1024 + 100 * 4) as usize;
    assert_eq!(color[color_offset + 3], 255);
    assert!(color[color_offset] > 0 || color[color_offset + 1] > 0 || color[color_offset + 2] > 0);
    let depth = read_back(&device, &queue, &scene.depth, 200, 150, 4);
    let depth_offset = (75 * 1024 + 100 * 4) as usize;
    let depth = f32::from_ne_bytes([
        depth[depth_offset],
        depth[depth_offset + 1],
        depth[depth_offset + 2],
        depth[depth_offset + 3],
    ]);
    assert!(depth > 0.0 && depth < 1.0, "depth was {depth}");
}

fn read_back(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
    bytes_per_pixel: u32,
) -> Vec<u8> {
    let bytes_per_row = ((width * bytes_per_pixel).div_ceil(256) * 256) as usize;
    let size = (bytes_per_row * height as usize) as u64;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("workbench_mol3d_test_readback"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("workbench_mol3d_test_readback_encoder"),
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row as u32),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let slice = buffer.slice(..size);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    assert!(receiver.recv().expect("map callback").is_ok());
    let data = slice.get_mapped_range().expect("mapped").to_vec();
    buffer.unmap();
    data
}

#[test]
fn a_stalled_gpu_pick_falls_back_to_the_cpu() {
    let mut state = PickState::default();
    let context = Context::default();
    let viewport = Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 100.0));
    state.schedule(Pos2::new(10.0, 20.0), viewport, SelectionMode::Replace, &context);
    assert!(state.take_fallback().is_none());
    assert!(state.take_fallback().is_none());
    let (mode, pointer, rect) = state.take_fallback().expect("fallback");
    assert_eq!(mode, SelectionMode::Replace);
    assert_eq!(pointer, Pos2::new(10.0, 20.0));
    assert_eq!(rect, viewport);
}

#[test]
fn instance_layouts_match_the_shader_offsets() {
    assert_eq!(std::mem::size_of::<Instance>(), 48);
    assert_eq!(std::mem::size_of::<BondInstance>(), 64);
    assert_eq!(std::mem::offset_of!(Instance, pick), 32);
    assert_eq!(std::mem::offset_of!(BondInstance, radius), 28);
    assert_eq!(std::mem::offset_of!(BondInstance, color), 32);
    assert_eq!(std::mem::offset_of!(BondInstance, pick), 48);
}
