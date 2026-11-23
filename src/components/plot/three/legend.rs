//! The 3D legend: the shared row geometry with 3D swatches.

use egui::{Align2, Color32, Pos2, Rect, pos2, vec2};

use super::super::animation::{Effect, mix};
use super::super::layout::Colors;
use super::super::legend::{LegendEntry, LegendLayout, elide, layout_entries};
use super::super::scene::{Dash, Measure, Prim, Scene};
use super::super::series::{LineStyle, Marker};
use super::paint::{self, Look3D};
use super::series::{Kind3D, Series3D};
use super::{Plot3D, Plot3DState};

const SWATCH: f32 = 18.0;
const LEGEND_PAD: f32 = 8.0;

pub(super) fn layout(
    measure: &Measure,
    plot: &Plot3D<'_>,
    frame: Rect,
    state: &Plot3DState,
    include_hidden: bool,
) -> Option<LegendLayout> {
    let entries: Vec<LegendEntry<'_>> = plot
        .series
        .iter()
        .enumerate()
        .filter(|(_, series)| series.legend && !series.name.is_empty())
        .map(|(index, series)| LegendEntry {
            index,
            name: &series.name,
            hidden: state.is_hidden(&series.name),
        })
        .collect();
    layout_entries(
        measure,
        plot.config.legend,
        plot.config.legend_size,
        frame,
        state.legend_position(),
        super::widgets::toolbar_rect(plot, frame),
        &entries,
        include_hidden,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prims(
    scene: &mut Scene,
    measure: &Measure,
    plot: &Plot3D<'_>,
    legend: &LegendLayout,
    colors: &Colors,
    theme: &crate::theme::Theme,
    hovered: Option<usize>,
    effect: Option<Effect>,
) {
    let config = &plot.config;
    if config.legend_frame {
        scene.over.push(Prim::Rect {
            rect: legend.rect,
            radius: 6.0,
            fill: colors.legend_fill.gamma_multiply(0.94),
            stroke: Some((1.0, colors.legend_stroke)),
        });
    }
    let max_text = legend.rect.right() - LEGEND_PAD - legend.rows.first().map_or(0.0, |row| row.text.x) + 0.5;
    for row in &legend.rows {
        let series = &plot.series[row.series];
        let mut look = paint::look(plot, row.series, theme);
        let row_effect = (hovered == Some(row.series)).then_some(effect).flatten();
        if let Some(row_effect) = row_effect {
            look.color = row_effect.color(look.color);
            look.width = row_effect.width(look.width);
            scene.over.push(Prim::Rect {
                rect: row.rect.shrink2(vec2(1.0, 0.5)),
                radius: 4.0,
                fill: row_effect.highlight.gamma_multiply(0.10 + 0.10 * row_effect.wave()),
                stroke: Some((1.0, row_effect.highlight.gamma_multiply(0.22))),
            });
        }
        let color = if row.hidden {
            look.color.gamma_multiply(0.3)
        } else {
            look.color
        };
        swatch(&mut scene.over, series, &look, row.swatch, color, row_effect);
        let name = elide(measure, &series.name, config.legend_size, max_text);
        let text_color = if row.hidden {
            colors.tick_text.gamma_multiply(0.6)
        } else if let Some(effect) = row_effect {
            effect.color(colors.text)
        } else {
            colors.text
        };
        scene
            .over
            .push(measure.label(row.text, Align2::LEFT_CENTER, &name, config.legend_size, text_color));
    }
}

fn swatch(
    out: &mut Vec<Prim>,
    series: &Series3D<'_>,
    look: &Look3D,
    center: Pos2,
    color: Color32,
    effect: Option<Effect>,
) {
    let half = SWATCH * 0.5;
    let width = effect.map_or(look.width, |effect| effect.width(look.width));
    let dash = match look.style {
        LineStyle::Solid => Dash::Solid,
        LineStyle::Dashed => Dash::Dashed(5.0, 3.0),
        LineStyle::Dotted => Dash::Dotted(4.0),
    };
    let dash = effect.map_or(dash, |effect| effect.dash(dash));
    match series.kind {
        Kind3D::Line => {
            out.push(Prim::Line {
                points: vec![pos2(center.x - half, center.y), pos2(center.x + half, center.y)],
                width: width.clamp(1.0, 4.0),
                color,
                dash,
            });
            if let Some(marker) = look.marker {
                swatch_marker(out, center, marker, look.radius.min(4.0), color);
            }
        }
        Kind3D::Scatter => super::super::paint::marker_prims(
            out,
            center,
            look.marker.unwrap_or(super::super::series::Marker::Circle),
            look.radius.min(5.0),
            color,
        ),
        Kind3D::Bars => swatch_cube(out, center, color),
        Kind3D::Surface => swatch_surface(out, center, color),
    }
}

/// A marker at a fixed screen position, for legend swatches.
pub(super) fn swatch_marker(out: &mut Vec<Prim>, center: Pos2, marker: Marker, radius: f32, color: Color32) {
    super::super::paint::marker_prims(out, center, marker, radius, color);
}

/// A small shaded cube for the bar legend swatch.
pub(super) fn swatch_cube(out: &mut Vec<Prim>, center: Pos2, color: Color32) {
    let half = 5.0;
    let top = vec![
        center + vec2(0.0, -half),
        center + vec2(half, -half * 0.55),
        center + vec2(0.0, -half * 0.1),
        center + vec2(-half, -half * 0.55),
    ];
    out.push(Prim::Polygon { points: top, color });
    out.push(Prim::Polygon {
        points: vec![
            center + vec2(-half, -half * 0.55),
            center + vec2(0.0, -half * 0.1),
            center + vec2(0.0, half),
            center + vec2(-half, half * 0.45),
        ],
        color: color.gamma_multiply(0.7),
    });
    out.push(Prim::Polygon {
        points: vec![
            center + vec2(0.0, -half * 0.1),
            center + vec2(half, -half * 0.55),
            center + vec2(half, half * 0.45),
            center + vec2(0.0, half),
        ],
        color: color.gamma_multiply(0.84),
    });
}

/// A 3×3 shaded patch for the surface legend swatch.
pub(super) fn swatch_surface(out: &mut Vec<Prim>, center: Pos2, color: Color32) {
    let cell = 4.0;
    for row in 0..3 {
        for column in 0..3 {
            let tint = 0.45 + 0.55 * ((row + column) as f32 / 4.0);
            out.push(Prim::Rect {
                rect: egui::Rect::from_min_size(
                    pos2(
                        center.x - cell * 1.5 + column as f32 * cell,
                        center.y - cell * 1.5 + row as f32 * cell,
                    ),
                    vec2(cell - 0.5, cell - 0.5),
                ),
                radius: 0.5,
                fill: mix(color, Color32::WHITE, tint * 0.12).gamma_multiply(tint),
                stroke: None,
            });
        }
    }
}
