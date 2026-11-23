//! Automatic bounds, draw order, view state helpers, and one frame of primitives.

use egui::Rect;

use super::super::axis::Scale;
use super::super::layout::Colors;
use super::super::scene::{Measure, Prim, Scene};
use super::camera::{Camera3D, Projector};
use super::layout::Layout3D;
use super::paint::HoverPoint;
use super::{Bounds3D, Plot3D, Plot3DState, Series3D, axes, legend, paint};

impl<'a> Plot3D<'a> {
    pub(super) fn visible<'s>(&'s self, state: &'s Plot3DState) -> impl Iterator<Item = &'s Series3D<'a>> {
        self.series.iter().filter(|series| !state.is_hidden(&series.name))
    }

    /// Whether the direction arrow draws for series `index` after the overrides.
    pub(super) fn draws_direction(&self, index: usize) -> bool {
        self.config
            .series_style(&self.series[index].name)
            .and_then(|style| style.direction_arrow)
            .unwrap_or(self.config.direction.enabled)
    }

    fn cached_bounds(&self, state: &Plot3DState, series: &Series3D<'_>) -> Bounds3D {
        let key = series.fingerprint(self.config.x.scale, self.config.y.scale, self.config.z.scale);
        state.extents.iter().find(|(cached, _)| *cached == key).map_or_else(
            || series.bounds(self.config.x.scale, self.config.y.scale, self.config.z.scale),
            |(_, bounds)| *bounds,
        )
    }

    pub(super) fn refresh_extents(&self, state: &mut Plot3DState) {
        state.extents = self
            .series
            .iter()
            .map(|series| {
                let key = series.fingerprint(self.config.x.scale, self.config.y.scale, self.config.z.scale);
                let bounds = state.extents.iter().find(|(cached, _)| *cached == key).map_or_else(
                    || series.bounds(self.config.x.scale, self.config.y.scale, self.config.z.scale),
                    |(_, bounds)| *bounds,
                );
                (key, bounds)
            })
            .collect();
    }

    /// The automatic view: every visible series plus margins, in scaled space, with the
    /// fixed axis limits applied on top.
    pub(super) fn auto_bounds(&self, state: &Plot3DState) -> Bounds3D {
        let mut bounds = Bounds3D::NOTHING;
        for series in self.visible(state) {
            bounds.union(&self.cached_bounds(state, series));
        }
        if self.include.is_valid() {
            bounds.extend([self.include.min_x, self.include.min_y, self.include.min_z]);
            bounds.extend([self.include.max_x, self.include.max_y, self.include.max_z]);
        }
        if !bounds.is_valid() {
            bounds = Bounds3D::from_corners([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        }
        pad(&mut bounds.min_x, &mut bounds.max_x);
        pad(&mut bounds.min_y, &mut bounds.max_y);
        pad(&mut bounds.min_z, &mut bounds.max_z);
        if let Some(min) = limit(self.config.x.min, self.config.x.scale) {
            bounds.min_x = min;
        }
        if let Some(max) = limit(self.config.x.max, self.config.x.scale) {
            bounds.max_x = max;
        }
        if let Some(min) = limit(self.config.y.min, self.config.y.scale) {
            bounds.min_y = min;
        }
        if let Some(max) = limit(self.config.y.max, self.config.y.scale) {
            bounds.max_y = max;
        }
        if let Some(min) = limit(self.config.z.min, self.config.z.scale) {
            bounds.min_z = min;
        }
        if let Some(max) = limit(self.config.z.max, self.config.z.scale) {
            bounds.max_z = max;
        }
        if bounds.max_x <= bounds.min_x {
            bounds.max_x = bounds.min_x + 1.0;
        }
        if bounds.max_y <= bounds.min_y {
            bounds.max_y = bounds.min_y + 1.0;
        }
        if bounds.max_z <= bounds.min_z {
            bounds.max_z = bounds.min_z + 1.0;
        }
        bounds
    }

    /// Series indices in draw order, with the hovered legend row last.
    pub(super) fn draw_order(&self, state: &Plot3DState) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.series.len()).collect();
        if let Some(hovered) = state.legend_hover()
            && self
                .series
                .get(hovered)
                .is_some_and(|series| !state.is_hidden(&series.name))
            && let Some(position) = order.iter().position(|index| *index == hovered)
        {
            let index = order.remove(position);
            order.push(index);
        }
        order
    }

    /// The camera for this frame: the state's orbit, with the config's projection and
    /// field of view applied, or the automatic fit.
    pub(super) fn camera(&self, state: &Plot3DState, bounds: Bounds3D, frame: Rect) -> Camera3D {
        state.camera().map_or_else(
            || Camera3D::auto(bounds, &self.config, frame),
            |camera| Camera3D {
                fov: self.config.fov,
                projection: self.config.projection,
                ..camera
            },
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_scene(
    measure: &Measure,
    plot: &Plot3D<'_>,
    layout: &Layout3D,
    projector: &Projector,
    bounds: Bounds3D,
    state: &Plot3DState,
    colors: &Colors,
    theme: &crate::theme::Theme,
    exporting: bool,
    now: f64,
    motion: bool,
) -> (Scene, Vec<(usize, Vec<HoverPoint>)>) {
    let mut scene = Scene {
        size: layout.rect.size(),
        frame: layout.frame,
        under: Vec::new(),
        data: Vec::new(),
        over: Vec::new(),
        drawn: vec![None; plot.series.len()],
    };
    if let Some(canvas) = colors.canvas {
        scene.under.push(super::scene::Prim::Rect {
            rect: layout.rect,
            radius: 0.0,
            fill: canvas,
            stroke: None,
        });
    }
    scene.under.push(super::scene::Prim::Rect {
        rect: layout.frame,
        radius: 0.0,
        fill: colors.background,
        stroke: None,
    });
    axes::build(&mut scene, measure, &plot.config, layout, projector, bounds, colors);
    let mut drawn = Vec::new();
    for index in plot.draw_order(state) {
        let series = &plot.series[index];
        if state.is_hidden(&series.name) {
            continue;
        }
        let look = paint::look(plot, index, theme);
        let effect = super::super::animation::Effect::for_legend(
            &plot.config.legend_hover,
            state.legend_hover(),
            state.legend_hover_start(),
            index,
            now,
            theme,
            motion && !exporting,
        );
        let points = paint::series(
            &mut scene,
            projector,
            series,
            &look,
            effect,
            &plot.config,
            theme,
            bounds,
            plot.draws_direction(index),
        );
        drawn.push((index, points));
    }
    let hovered_effect = state.legend_hover().and_then(|index| {
        super::super::animation::Effect::for_legend(
            &plot.config.legend_hover,
            state.legend_hover(),
            state.legend_hover_start(),
            index,
            now,
            theme,
            motion && !exporting,
        )
    });
    if let Some(legend) = legend::layout(measure, plot, layout.frame, state, !exporting) {
        legend::prims(
            &mut scene,
            measure,
            plot,
            &legend,
            colors,
            theme,
            state.legend_hover(),
            hovered_effect.filter(|_| !exporting),
        );
    }
    (scene, drawn)
}

/// The span every automatic range is padded by on both sides.
fn pad(min: &mut f64, max: &mut f64) {
    let span = *max - *min;
    let span = if span > 0.0 { span } else { min.abs().max(1.0) * 0.2 };
    let margin = span * 0.05;
    *min -= margin;
    *max += margin;
}

fn limit(value: Option<f64>, scale: Scale) -> Option<f64> {
    value
        .map(|value| scale.forward(value))
        .filter(|value| value.is_finite())
}
