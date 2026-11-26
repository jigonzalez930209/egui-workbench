//! Offscreen software rendering for publication figures.
//!
//! The interactive viewport remains `egui`; this module provides deterministic
//! headless frames (4K/8K, transparent background, clipping, SSAO, and toon edges)
//! and is also the reference used by the regression tests.

use egui::{Rect, pos2};
use image::{ImageFormat, RgbaImage};

use crate::color::ColorScheme;
use crate::math::Vec3;
use crate::measure::centroid;
use crate::representation::{Representation, RepresentationSet, vdw_radius};
use crate::scene::Mol3dScene;
use crate::view::{Mol3dStyle, Mol3dView};

/// A half-space clip: keep points with `normal · p <= offset`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipPlane {
    /// Plane normal; does not need to be unit length.
    pub normal: Vec3,
    /// Plane offset along the normalized normal.
    pub offset: f64,
}

/// Offscreen rendering options.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderOptions {
    /// Output width in pixels.
    pub width: u32,
    /// Output height in pixels.
    pub height: u32,
    /// Supersampling factor (`1` disables it).
    pub supersample: u32,
    /// Background color.
    pub background: [u8; 4],
    /// Keep the background transparent.
    pub transparent: bool,
    /// Dim distant atoms.
    pub depth_cue: bool,
    /// Screen-space ambient occlusion.
    pub ambient_occlusion: bool,
    /// Quantized toon shading and dark silhouettes.
    pub toon: bool,
    /// Explicit silhouettes on depth discontinuities.
    pub silhouette: bool,
    /// Display radius multiplier.
    pub atom_scale: f64,
    /// Bond radius multiplier.
    pub bond_scale: f64,
    /// Color scheme.
    pub color_scheme: ColorScheme,
    /// Active clipping half-spaces.
    pub clip_planes: Vec<ClipPlane>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 1600,
            height: 1200,
            supersample: 1,
            background: [255, 255, 255, 255],
            transparent: false,
            depth_cue: true,
            ambient_occlusion: true,
            toon: false,
            silhouette: false,
            atom_scale: 1.0,
            bond_scale: 1.0,
            color_scheme: ColorScheme::Cpk,
            clip_planes: Vec::new(),
        }
    }
}

impl RenderOptions {
    /// A 4K figure with a transparent background.
    pub fn figure_4k() -> Self {
        Self {
            width: 3840,
            height: 2160,
            supersample: 1,
            transparent: true,
            ..Self::default()
        }
    }

    /// An 8K figure with a transparent background.
    pub fn figure_8k() -> Self {
        Self {
            width: 7680,
            height: 4320,
            supersample: 1,
            transparent: true,
            ..Self::default()
        }
    }
}

/// Largest edge of a rendered image, in pixels.
const MAX_EDGE: u32 = 8_192;

/// Most pixels (before supersampling) of a rendered image.
const MAX_PIXELS: u64 = 36_000_000;

/// Most samples per pixel edge.
const MAX_SUPERSAMPLE: u32 = 4;

/// The output size and supersampling factor that fit the limits, keeping the aspect ratio.
fn bounded_size(options: &RenderOptions) -> (u32, u32, u32) {
    let mut width = options.width.clamp(1, MAX_EDGE);
    let mut height = options.height.clamp(1, MAX_EDGE);
    let mut scale = options.supersample.clamp(1, MAX_SUPERSAMPLE);
    // Supersampling multiplies both edges; shed it first, then shrink the image.
    while scale > 1 && u64::from(width * scale) * u64::from(height * scale) > MAX_PIXELS {
        scale -= 1;
    }
    while u64::from(width) * u64::from(height) * u64::from(scale * scale) > MAX_PIXELS {
        width = (width * 9 / 10).max(1);
        height = (height * 9 / 10).max(1);
    }
    (width, height, scale)
}

/// Render the scene to an RGBA image.
///
/// The image is at most 8192 pixels on a side and 36 million pixels in all; a larger
/// request is rendered at the largest size that fits.
pub fn render_rgba(scene: &Mol3dScene, camera: crate::camera::Camera3D, options: &RenderOptions) -> RgbaImage {
    // `width * scale` overflowed u32 and the pixel buffer was sized from it unchecked. A
    // request beyond the limits is rendered smaller rather than allocated.
    let (out_width, out_height, scale) = bounded_size(options);
    let width = out_width * scale;
    let height = out_height * scale;
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(width as f32, height as f32));
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    let mut depth = vec![f64::INFINITY; (width * height) as usize];
    let representations = RepresentationSet::default();
    let bounds = scene.bounds();
    let diagonal = bounds.map_or(1.0, |bounds| {
        Vec3::new(bounds[3] - bounds[0], bounds[4] - bounds[1], bounds[5] - bounds[2]).length()
    });
    let light = (camera.up() * 0.6 + camera.right() * 0.35 - camera.forward() * 0.7).normalized();
    for y in 0..height {
        for x in 0..width {
            let pointer = pos2(x as f32 + 0.5, y as f32 + 0.5);
            let ray = camera.ray(pointer, viewport);
            let mut nearest = f64::INFINITY;
            let mut color = [0u8; 3];
            let mut alpha = 0u8;
            for atom in &scene.atoms {
                let representation = representations.for_atom(atom.id);
                let radius = if representation == Representation::SpaceFilling {
                    vdw_radius(atom.element)
                } else {
                    atom.radius.max(0.25)
                } * options.atom_scale;
                if let Some(distance) = sphere_hit(ray.origin, ray.direction, atom.position, radius)
                    && distance < nearest
                    && unclipped(atom.position, options)
                {
                    nearest = distance;
                    let hit = ray.origin + ray.direction * distance;
                    let normal = (hit - atom.position).normalized();
                    let [r, g, b] = options.color_scheme.color(atom);
                    color = shade([r, g, b], normal, light, options);
                    alpha = 255;
                }
            }
            if representations.for_atom(
                scene
                    .atoms
                    .first()
                    .map_or(egui_workbench_chem_core::AtomId::new(0, 1), |atom| atom.id),
            ) != Representation::SpaceFilling
            {
                for bond in &scene.bonds {
                    let (Some(a), Some(b)) = (scene.atom(bond.a), scene.atom(bond.b)) else {
                        continue;
                    };
                    let radius = 0.13 * options.bond_scale;
                    if let Some((distance, normal, point)) =
                        capsule_hit(ray.origin, ray.direction, a.position, b.position, radius)
                        && distance < nearest
                        && unclipped(point, options)
                    {
                        nearest = distance;
                        let [r, g, b] = options.color_scheme.color(a);
                        color = shade([r, g, b], normal, light, options);
                        alpha = 255;
                    }
                }
            }
            if alpha == 255 && options.depth_cue {
                let normalized = ((nearest - 1.0) / diagonal.max(1.0)).clamp(0.0, 1.0);
                let fade = 1.0 - 0.25 * normalized;
                for channel in &mut color {
                    *channel = (*channel as f64 * fade) as u8;
                }
            }
            let index = (y * width + x) as usize;
            depth[index] = nearest;
            pixels[index * 4..index * 4 + 4].copy_from_slice(&[color[0], color[1], color[2], alpha]);
        }
    }
    if options.ambient_occlusion || options.silhouette || options.toon {
        post_process(&mut pixels, &depth, width, height, options);
    }
    fill_background(&mut pixels, options);
    if scale > 1 {
        pixels = downsample(&pixels, width, height, scale);
    }
    RgbaImage::from_raw(out_width, out_height, pixels).expect("image size")
}

/// Render directly to PNG bytes.
///
/// # Errors
/// When the PNG encoder fails.
pub fn render_png(
    scene: &Mol3dScene,
    camera: crate::camera::Camera3D,
    options: &RenderOptions,
) -> Result<Vec<u8>, String> {
    let image = render_rgba(scene, camera, options);
    let mut bytes = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut bytes, ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    Ok(bytes.into_inner())
}

impl Mol3dView {
    /// Render the current scene and camera to PNG bytes.
    ///
    /// # Errors
    /// When the PNG encoder fails.
    pub fn capture_png(&self, width: u32, height: u32, transparent: bool) -> Result<Vec<u8>, String> {
        let options = RenderOptions {
            width,
            height,
            transparent,
            depth_cue: self.style.depth_cue,
            atom_scale: self.style.atom_scale,
            color_scheme: self.color_scheme,
            ..RenderOptions::default()
        };
        render_png(&self.scene, self.camera, &options)
    }

    /// Render a publication figure using the view style.
    ///
    /// # Errors
    /// When the PNG encoder fails.
    pub fn capture_figure(&self, options: &RenderOptions) -> Result<Vec<u8>, String> {
        render_png(&self.scene, self.camera, options)
    }
}

/// The default representation-aware style used by the offscreen renderer.
pub fn render_style(style: &Mol3dStyle) -> RenderOptions {
    RenderOptions {
        depth_cue: style.depth_cue,
        atom_scale: style.atom_scale,
        ..RenderOptions::default()
    }
}

fn shade(albedo: [u8; 3], normal: Vec3, light: Vec3, options: &RenderOptions) -> [u8; 3] {
    let diffuse = normal.dot(light).max(0.0);
    let rim = (1.0 - normal.dot(-light).abs()).powi(3) * 0.15;
    let mut intensity = 0.32 + 0.68 * diffuse + rim;
    if options.toon {
        intensity = (intensity * 3.0).round() / 3.0;
    }
    [
        (albedo[0] as f64 * intensity).clamp(0.0, 255.0) as u8,
        (albedo[1] as f64 * intensity).clamp(0.0, 255.0) as u8,
        (albedo[2] as f64 * intensity).clamp(0.0, 255.0) as u8,
    ]
}

fn post_process(pixels: &mut [u8], depth: &[f64], width: u32, height: u32, options: &RenderOptions) {
    let original = pixels.to_vec();
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) as usize;
            if !depth[index].is_finite() {
                continue;
            }
            if options.silhouette {
                for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                        continue;
                    }
                    let neighbor = depth[(ny as u32 * width + nx as u32) as usize];
                    if neighbor - depth[index] > 0.15 {
                        pixels[index * 4..index * 4 + 2].copy_from_slice(&[20, 20]);
                        break;
                    }
                }
            }
            if options.ambient_occlusion {
                let mut blocked = 0.0;
                let mut samples = 0.0;
                for radius in [3i32, 7] {
                    for (dx, dy) in [(-radius, 0), (radius, 0), (0, -radius), (0, radius)] {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                            continue;
                        }
                        let neighbor = depth[(ny as u32 * width + nx as u32) as usize];
                        samples += 1.0;
                        if neighbor < depth[index] - 0.08 {
                            blocked += 1.0;
                        }
                    }
                }
                if samples > 0.0 {
                    let occlusion = 1.0 - 0.45 * blocked / samples;
                    for channel in 0..3 {
                        pixels[index * 4 + channel] = (f64::from(original[index * 4 + channel]) * occlusion) as u8;
                    }
                }
            }
        }
    }
}

fn fill_background(pixels: &mut [u8], options: &RenderOptions) {
    for chunk in pixels.chunks_exact_mut(4) {
        if chunk[3] == 0 {
            if options.transparent {
                chunk.copy_from_slice(&[0, 0, 0, 0]);
            } else {
                chunk.copy_from_slice(&options.background);
            }
        }
    }
}

fn downsample(pixels: &[u8], width: u32, height: u32, scale: u32) -> Vec<u8> {
    let out_width = width / scale;
    let out_height = height / scale;
    let mut output = vec![0u8; (out_width * out_height * 4) as usize];
    for y in 0..out_height {
        for x in 0..out_width {
            let mut sums = [0u32; 4];
            for dy in 0..scale {
                for dx in 0..scale {
                    let index = (((y * scale + dy) * width + (x * scale + dx)) * 4) as usize;
                    for channel in 0..4 {
                        sums[channel] += u32::from(pixels[index + channel]);
                    }
                }
            }
            let count = scale * scale;
            let index = ((y * out_width + x) * 4) as usize;
            for channel in 0..4 {
                output[index + channel] = (sums[channel] / count) as u8;
            }
        }
    }
    output
}

fn unclipped(point: Vec3, options: &RenderOptions) -> bool {
    options.clip_planes.iter().all(|plane| {
        let normal = plane.normal.normalized();
        normal.dot(point) <= plane.offset
    })
}

fn sphere_hit(origin: Vec3, direction: Vec3, center: Vec3, radius: f64) -> Option<f64> {
    let offset = origin - center;
    let b = offset.dot(direction);
    let c = offset.dot(offset) - radius * radius;
    let discriminant = b * b - c;
    if discriminant < 0.0 {
        return None;
    }
    let root = discriminant.sqrt();
    let near = -b - root;
    let far = -b + root;
    if near > 1.0e-4 {
        Some(near)
    } else if far > 1.0e-4 {
        Some(far)
    } else {
        None
    }
}

fn capsule_hit(origin: Vec3, direction: Vec3, a: Vec3, b: Vec3, radius: f64) -> Option<(f64, Vec3, Vec3)> {
    let axis = b - a;
    let length_squared = axis.dot(axis);
    let steps = 24;
    let mut best: Option<(f64, Vec3, Vec3)> = None;
    for step in 0..=steps {
        let t = step as f64 / steps as f64;
        let sphere_center = a + axis * t;
        if let Some(distance) = sphere_hit(origin, direction, sphere_center, radius * 1.35)
            && best.is_none_or(|(current, _, _)| distance < current)
        {
            let point = origin + direction * distance;
            let projected = a + axis * ((point - a).dot(axis) / length_squared).clamp(0.0, 1.0);
            let normal = (point - projected).normalized();
            best = Some((distance, normal, point));
        }
    }
    best
}

/// The centroid helper exposed for figure annotations.
pub fn figure_center(scene: &Mol3dScene) -> Option<Vec3> {
    let points = scene.atoms.iter().map(|atom| atom.position).collect::<Vec<_>>();
    centroid(&points)
}

#[cfg(test)]
mod tests;
