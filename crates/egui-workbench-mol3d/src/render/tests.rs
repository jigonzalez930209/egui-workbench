//! Unit tests.

use super::*;
use egui_workbench_chem_core::Element;

fn scene() -> Mol3dScene {
    let mut scene = Mol3dScene::default();
    scene.push_atom(Element::from_symbol("O").expect("O"), [0.0, 0.0, 0.0]);
    scene.push_atom(Element::from_symbol("H").expect("H"), [0.96, 0.0, 0.0]);
    scene
}

#[test]
fn a_small_render_has_the_requested_size_and_png_signature() {
    let camera = crate::camera::Camera3D {
        distance: 6.0,
        ..crate::camera::Camera3D::default()
    };
    let options = RenderOptions {
        width: 64,
        height: 48,
        ambient_occlusion: false,
        ..RenderOptions::default()
    };
    let image = render_rgba(&scene(), camera, &options);
    assert_eq!(image.dimensions(), (64, 48));
    assert!(image.pixels().any(|pixel| pixel[3] == 255));
    let png = render_png(&scene(), camera, &options).expect("png");
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn transparent_renders_keep_an_alpha_zero_background() {
    let options = RenderOptions {
        width: 32,
        height: 32,
        transparent: true,
        ambient_occlusion: false,
        ..RenderOptions::default()
    };
    let image = render_rgba(&scene(), crate::camera::Camera3D::default(), &options);
    assert!(image.pixels().any(|pixel| pixel[3] == 0));
}

#[test]
fn clipping_removes_every_hit() {
    let options = RenderOptions {
        width: 32,
        height: 32,
        ambient_occlusion: false,
        clip_planes: vec![ClipPlane {
            normal: Vec3::new(1.0, 0.0, 0.0),
            offset: -100.0,
        }],
        ..RenderOptions::default()
    };
    let image = render_rgba(&scene(), crate::camera::Camera3D::default(), &options);
    assert!(image.pixels().all(|pixel| pixel[3] == 0 || pixel[3] == 255));
    assert!(image.pixels().all(|pixel| pixel[3] == 255));
}

#[test]
fn oversized_requests_are_bounded_instead_of_overflowing() {
    let options = RenderOptions {
        width: u32::MAX,
        height: u32::MAX,
        supersample: u32::MAX,
        ..RenderOptions::default()
    };
    let (width, height, scale) = bounded_size(&options);
    assert!(width <= MAX_EDGE && height <= MAX_EDGE && scale <= MAX_SUPERSAMPLE);
    assert!(u64::from(width) * u64::from(height) * u64::from(scale * scale) <= MAX_PIXELS);
    let zero = bounded_size(&RenderOptions {
        width: 0,
        height: 0,
        ..RenderOptions::default()
    });
    assert_eq!((zero.0, zero.1), (1, 1));
}
