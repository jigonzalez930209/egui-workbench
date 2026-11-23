//! The 3D box, back-wall grid, axes, ticks, titles, and labels.

use egui::{Align2, Color32, Pos2, pos2};

use super::super::axis::{TickRules, ticks};
use super::super::layout::Colors;
use super::super::scene::{Dash, Measure, Prim, Scene};
use super::camera::{Projector, corners};
use super::config::Plot3DConfig;
use super::layout::Layout3D;
use super::types::Bounds3D;

/// Paint the box, grid, and labels for one camera.
#[allow(clippy::too_many_arguments)]
pub(super) fn build(
    scene: &mut Scene,
    measure: &Measure,
    config: &Plot3DConfig,
    layout: &Layout3D,
    projector: &Projector,
    bounds: Bounds3D,
    colors: &Colors,
) {
    title(scene, measure, config, layout, colors);
    if !bounds.is_valid() {
        return;
    }
    let frame = layout.frame;
    let eye = projector.eye();
    let center = bounds.center();
    let front_x = if eye[0] < center[0] { bounds.min_x } else { bounds.max_x };
    let front_y = if eye[1] < center[1] { bounds.min_y } else { bounds.max_y };
    let far_x = if eye[0] < center[0] { bounds.max_x } else { bounds.min_x };
    let far_y = if eye[1] < center[0] { bounds.max_y } else { bounds.min_y };

    let x_ticks = ticks(
        config.x.scale,
        bounds.min_x,
        bounds.max_x,
        frame.width(),
        52.0,
        TickRules::of(&config.x),
    );
    let y_ticks = ticks(
        config.y.scale,
        bounds.min_y,
        bounds.max_y,
        frame.width() * 0.72,
        52.0,
        TickRules::of(&config.y),
    );
    let z_ticks = ticks(
        config.z.scale,
        bounds.min_z,
        bounds.max_z,
        frame.height(),
        44.0,
        TickRules::of(&config.z),
    );

    // Grid on the floor and on the two far walls, so it reads as a room.
    for tick in &x_ticks {
        let color = grid_color(config, colors, tick.major);
        segment(
            scene,
            projector,
            [tick.value, bounds.min_y, bounds.min_z],
            [tick.value, bounds.max_y, bounds.min_z],
            color,
        );
        if tick.major {
            segment(
                scene,
                projector,
                [tick.value, far_y, bounds.min_z],
                [tick.value, far_y, bounds.max_z],
                color,
            );
        }
    }
    for tick in &y_ticks {
        let color = grid_color(config, colors, tick.major);
        segment(
            scene,
            projector,
            [bounds.min_x, tick.value, bounds.min_z],
            [bounds.max_x, tick.value, bounds.min_z],
            color,
        );
        if tick.major {
            segment(
                scene,
                projector,
                [far_x, tick.value, bounds.min_z],
                [far_x, tick.value, bounds.max_z],
                color,
            );
        }
    }
    for tick in &z_ticks {
        if tick.major {
            let color = grid_color(config, colors, true);
            segment(
                scene,
                projector,
                [bounds.min_x, far_y, tick.value],
                [bounds.max_x, far_y, tick.value],
                color,
            );
            segment(
                scene,
                projector,
                [far_x, bounds.min_y, tick.value],
                [far_x, bounds.max_y, tick.value],
                color,
            );
        }
    }

    box_edges(scene, projector, bounds, colors);

    // The three labelled axes: two along the near floor edges, one up the left corner.
    let x_axis = [
        (bounds.min_x, front_y, bounds.min_z),
        (bounds.max_x, front_y, bounds.min_z),
    ];
    let y_axis = [
        (front_x, bounds.min_y, bounds.min_z),
        (front_x, bounds.max_y, bounds.min_z),
    ];
    let z_corner = left_corner(bounds, projector);
    let z_axis = [
        (z_corner.0, z_corner.1, bounds.min_z),
        (z_corner.0, z_corner.1, bounds.max_z),
    ];
    axis_line(scene, projector, x_axis, config, colors);
    axis_line(scene, projector, y_axis, config, colors);
    axis_line(scene, projector, z_axis, config, colors);

    let text = colors.tick_text;
    let size = config.tick_size;
    let box_center = projector
        .project(bounds.center())
        .map_or(frame.center(), |(pos, _)| pos);
    let y_normal = outward_normal(
        projector,
        [front_x, bounds.min_y, bounds.min_z],
        [front_x, bounds.max_y, bounds.min_z],
        box_center,
    );
    for tick in x_ticks.iter().filter(|tick| tick.major && !tick.label.is_empty()) {
        let Some((pos, _)) = projector.project([tick.value, front_y, bounds.min_z]) else {
            continue;
        };
        tick_mark(scene, pos, egui::vec2(0.0, 3.0), colors.frame);
        scene.over.push(measure.label(
            pos + egui::vec2(0.0, config.tick_length + 3.0),
            Align2::CENTER_TOP,
            &tick.label,
            size,
            config.x.tick_color.unwrap_or(text),
        ));
    }
    for tick in y_ticks.iter().filter(|tick| tick.major && !tick.label.is_empty()) {
        let Some((pos, _)) = projector.project([front_x, tick.value, bounds.min_z]) else {
            continue;
        };
        tick_mark(scene, pos, y_normal * 3.0, colors.frame);
        scene.over.push(measure.label(
            pos + y_normal * 7.0,
            normal_anchor(y_normal),
            &tick.label,
            size,
            config.y.tick_color.unwrap_or(text),
        ));
    }
    for tick in z_ticks.iter().filter(|tick| tick.major && !tick.label.is_empty()) {
        let Some((pos, _)) = projector.project([z_corner.0, z_corner.1, tick.value]) else {
            continue;
        };
        tick_mark(scene, pos, egui::vec2(-3.0, 0.0), colors.frame);
        scene.over.push(measure.label(
            pos + egui::vec2(-config.tick_length - 3.0, 0.0),
            Align2::RIGHT_CENTER,
            &tick.label,
            size,
            config.z.tick_color.unwrap_or(text),
        ));
    }

    axis_titles(
        scene,
        measure,
        config,
        projector,
        bounds,
        colors,
        (front_x, front_y),
        z_corner,
        &z_ticks,
        y_normal,
    );
}

fn title(scene: &mut Scene, measure: &Measure, config: &Plot3DConfig, layout: &Layout3D, colors: &Colors) {
    if let Some(title) = &config.title {
        let color = config.title_color.unwrap_or(colors.text);
        let y = layout.rect.top() + 6.0;
        let (at, anchor) = match config.title_align {
            super::super::config::TitleAlign::Left => (pos2(layout.frame.left(), y), Align2::LEFT_TOP),
            super::super::config::TitleAlign::Center => (pos2(layout.frame.center().x, y), Align2::CENTER_TOP),
            super::super::config::TitleAlign::Right => (pos2(layout.frame.right(), y), Align2::RIGHT_TOP),
        };
        scene
            .over
            .push(measure.label(at, anchor, title, config.title_size, color));
    }
}

#[allow(clippy::too_many_arguments)]
fn axis_titles(
    scene: &mut Scene,
    measure: &Measure,
    config: &Plot3DConfig,
    projector: &Projector,
    bounds: Bounds3D,
    colors: &Colors,
    front: (f64, f64),
    z_corner: (f64, f64),
    z_ticks: &[super::super::axis::Tick],
    y_normal: egui::Vec2,
) {
    let size = config.label_size;
    if let Some(label) = &config.x.label
        && let Some((at, _)) = projector.project([bounds.center()[0], front.1, bounds.min_z])
    {
        scene.over.push(measure.label(
            at + egui::vec2(0.0, 34.0),
            Align2::CENTER_TOP,
            label,
            size,
            config.x.label_color.unwrap_or(colors.text),
        ));
    }
    if let Some(label) = &config.y.label
        && let Some((at, _)) = projector.project([front.0, bounds.center()[1], bounds.min_z])
    {
        scene.over.push(measure.label(
            at + y_normal * 24.0,
            normal_anchor(y_normal),
            label,
            size,
            config.y.label_color.unwrap_or(colors.text),
        ));
    }
    if let Some(label) = &config.z.label
        && let Some((at, _)) = projector.project([z_corner.0, z_corner.1, bounds.center()[2]])
    {
        let widest = z_ticks
            .iter()
            .filter(|tick| tick.major)
            .map(|tick| measure.text(&tick.label, config.tick_size).0)
            .fold(0.0, f32::max);
        scene.over.push(measure.label(
            at + egui::vec2(-widest - 18.0, 0.0),
            Align2::RIGHT_CENTER,
            label,
            size,
            config.z.label_color.unwrap_or(colors.text),
        ));
    }
}

fn box_edges(scene: &mut Scene, projector: &Projector, bounds: Bounds3D, colors: &Colors) {
    let points = corners(bounds);
    let color = colors.frame.gamma_multiply(0.7);
    for (a, b) in [
        (0, 1),
        (0, 2),
        (1, 3),
        (2, 3),
        (4, 5),
        (4, 6),
        (5, 7),
        (6, 7),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ] {
        segment(scene, projector, points[a], points[b], color);
    }
}

fn axis_line(
    scene: &mut Scene,
    projector: &Projector,
    axis: [(f64, f64, f64); 2],
    config: &Plot3DConfig,
    colors: &Colors,
) {
    segment3(
        scene,
        projector,
        [axis[0].0, axis[0].1, axis[0].2],
        [axis[1].0, axis[1].1, axis[1].2],
        config.frame_width.max(1.0),
        colors.frame,
    );
}

fn segment(scene: &mut Scene, projector: &Projector, a: [f64; 3], b: [f64; 3], color: Color32) {
    segment3(scene, projector, a, b, 1.0, color);
}

fn segment3(scene: &mut Scene, projector: &Projector, a: [f64; 3], b: [f64; 3], width: f32, color: Color32) {
    let (Some((a, _)), Some((b, _))) = (projector.project(a), projector.project(b)) else {
        return;
    };
    scene.data.push(Prim::Line {
        points: vec![a, b],
        width,
        color,
        dash: Dash::Solid,
    });
}

fn tick_mark(scene: &mut Scene, at: Pos2, offset: egui::Vec2, color: Color32) {
    scene.data.push(Prim::Line {
        points: vec![at, at + offset],
        width: 1.0,
        color,
        dash: Dash::Solid,
    });
}

fn grid_color(config: &Plot3DConfig, colors: &Colors, major: bool) -> Color32 {
    let opacity = if major {
        config.grid_major_opacity
    } else {
        config.grid_minor_opacity
    };
    colors.grid.gamma_multiply(opacity)
}

/// A screen-space normal on the outside of an axis edge.
fn outward_normal(projector: &Projector, a: [f64; 3], b: [f64; 3], center: Pos2) -> egui::Vec2 {
    let (Some((a, _)), Some((b, _))) = (projector.project(a), projector.project(b)) else {
        return egui::vec2(0.0, 1.0);
    };
    let direction = (b - a).normalized();
    let mut normal = egui::vec2(-direction.y, direction.x);
    let midpoint = a + (b - a) * 0.5;
    if normal.dot(midpoint - center) < 0.0 {
        normal = -normal;
    }
    normal
}

fn normal_anchor(normal: egui::Vec2) -> Align2 {
    if normal.x > 0.35 {
        Align2::LEFT_CENTER
    } else if normal.x < -0.35 {
        Align2::RIGHT_CENTER
    } else if normal.y > 0.0 {
        Align2::CENTER_TOP
    } else {
        Align2::CENTER_BOTTOM
    }
}

/// The bottom corner drawn furthest left: the z axis labels get the outside room.
fn left_corner(bounds: Bounds3D, projector: &Projector) -> (f64, f64) {
    let candidates = [
        (bounds.min_x, bounds.min_y),
        (bounds.max_x, bounds.min_y),
        (bounds.min_x, bounds.max_y),
        (bounds.max_x, bounds.max_y),
    ];
    candidates
        .into_iter()
        .min_by(|a, b| {
            let ax = projector
                .project([a.0, a.1, bounds.min_z])
                .map_or(f32::MAX, |(pos, _)| pos.x);
            let bx = projector
                .project([b.0, b.1, bounds.min_z])
                .map_or(f32::MAX, |(pos, _)| pos.x);
            ax.total_cmp(&bx)
        })
        .unwrap_or((bounds.min_x, bounds.min_y))
}
