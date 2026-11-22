//! The legend: its layout (shared by painting and interaction) and its primitives.

use egui::{Align2, Color32, Pos2, Rect, pos2, vec2};
use std::f32::consts::PI;

use super::animation::Effect;
use super::overlay::{Colors, look};
use super::paint::{self, Look};
use super::scene::{Dash, Measure, Prim, Scene};
use super::series::{Kind, LineStyle};
use super::{LegendPosition, Plot, PlotState};

const LEGEND_PAD: f32 = 8.0;
const SWATCH: f32 = 18.0;
/// From the legend box's left edge to its text: inset, padding, swatch, gap.
const TEXT_OFFSET: f32 = 2.0 + LEGEND_PAD + SWATCH + 6.0 - 2.0;

/// One legend row: which series, where it is, and where its swatch and text go.
pub(super) struct LegendRow {
    pub series: usize,
    pub rect: Rect,
    pub swatch: Pos2,
    pub text: Pos2,
    pub hidden: bool,
}

pub(super) struct LegendLayout {
    pub rect: Rect,
    pub rows: Vec<LegendRow>,
}

/// Where the legend and its rows go. Hidden series stay listed (dimmed) on screen so
/// they can be shown again; exports leave them out.
pub(super) fn layout(
    measure: &Measure,
    plot: &Plot<'_>,
    frame: Rect,
    state: &PlotState,
    include_hidden: bool,
) -> Option<LegendLayout> {
    // `include_hidden` is true on screen (hidden series stay listed so they can be
    // shown again, and the toolbar is there); exports pass false.
    let config = &plot.config;
    if config.legend == LegendPosition::Hidden {
        return None;
    }
    let entries: Vec<(usize, bool)> = plot
        .series
        .iter()
        .enumerate()
        .filter(|(_, series)| series.legend && !series.name.is_empty())
        .map(|(index, series)| (index, state.is_hidden(&series.name)))
        .filter(|(_, hidden)| include_hidden || !hidden)
        .collect();
    if entries.is_empty() {
        return None;
    }
    let (_, text_height, _) = measure.text("Ag", config.legend_size);
    let row_height = (text_height + 6.0).max(SWATCH).round();
    let max_text = (frame.width() * 0.6 - SWATCH - 3.0 * LEGEND_PAD).max(40.0);
    let text_width = entries
        .iter()
        .map(|(index, _)| measure.text(&plot.series[*index].name, config.legend_size).0)
        .fold(0.0, f32::max)
        .min(max_text);
    // Text starts TEXT_OFFSET after the box's left edge (see the rows below).
    let size = vec2(
        (TEXT_OFFSET + text_width + LEGEND_PAD).ceil(),
        (entries.len() as f32 * row_height + LEGEND_PAD).round(),
    );
    let min = match config.legend {
        LegendPosition::BottomLeft => pos2(frame.left() + LEGEND_PAD, frame.bottom() - LEGEND_PAD - size.y),
        LegendPosition::BottomRight => pos2(
            frame.right() - LEGEND_PAD - size.x,
            frame.bottom() - LEGEND_PAD - size.y,
        ),
        _ => pos2(frame.left() + LEGEND_PAD, frame.top() + LEGEND_PAD),
    };
    let mut size = size;
    let mut min = min;
    if let Some([fx, fy]) = state.legend_position() {
        // Dragged: the same fraction of the free room, so it stays put across resizes
        // and lands in the same place in an export.
        let room = (frame.size() - size).max(vec2(0.0, 0.0));
        min = frame.min + vec2(fx * room.x, fy * room.y);
    }
    // On screen, keep clear of the toolbar: elide names when enough room is left beside
    // it, otherwise move the legend below it. A dragged legend goes where it was put.
    if include_hidden
        && state.legend_position().is_none()
        && let Some(toolbar) = super::widgets::toolbar_rect(plot, frame)
        && Rect::from_min_size(min, size).intersects(toolbar.expand(4.0))
    {
        let room = toolbar.left() - 6.0 - min.x;
        if room - TEXT_OFFSET - LEGEND_PAD >= 90.0 {
            size.x = room.floor();
        } else if config.legend == LegendPosition::TopLeft {
            min.y = toolbar.bottom() + 6.0;
        }
    }
    let rect = Rect::from_min_size(pos2(measure.snap(min.x), measure.snap(min.y)), size);
    let rows = entries
        .into_iter()
        .enumerate()
        .map(|(row, (series, hidden))| {
            let rect = Rect::from_min_size(
                pos2(
                    rect.left() + 2.0,
                    rect.top() + LEGEND_PAD * 0.5 + row as f32 * row_height,
                ),
                vec2(rect.width() - 4.0, row_height),
            );
            // One pixel-centered midline for the line, the marker, and the text.
            let middle = measure.center(rect.center().y);
            let swatch = pos2(measure.snap(rect.left() + LEGEND_PAD + SWATCH * 0.5 - 2.0), middle);
            let text = pos2(rect.left() - 2.0 + TEXT_OFFSET, middle);
            LegendRow {
                series,
                rect,
                swatch,
                text,
                hidden,
            }
        })
        .collect();
    Some(LegendLayout { rect, rows })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prims(
    scene: &mut Scene,
    measure: &Measure,
    plot: &Plot<'_>,
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
    // Half a point of slack absorbs rounding; only names wider than the cap elide.
    let max_text = legend.rect.right() - LEGEND_PAD - legend.rows.first().map_or(0.0, |row| row.text.x) + 0.5;
    for row in &legend.rows {
        let series = &plot.series[row.series];
        let mut look = look(plot, row.series, theme);
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

/// The legend sample of a series, centered on `center`: the line and its marker share
/// the same pixel-centered midline.
fn swatch(
    out: &mut Vec<Prim>,
    series: &super::series::Series<'_>,
    look: &Look,
    center: Pos2,
    color: Color32,
    effect: Option<Effect>,
) {
    let half = SWATCH * 0.5;
    let width = effect.map_or(look.width, |effect| effect.width(look.width));
    let style = match look.style {
        LineStyle::Solid => Dash::Solid,
        LineStyle::Dashed => Dash::Dashed(5.0, 3.0),
        LineStyle::Dotted => Dash::Dotted(4.0),
    };
    let dash = effect.map_or(style, |effect| effect.dash(style));
    let line = |width: f32, dash: Dash| Prim::Line {
        points: vec![pos2(center.x - half, center.y), pos2(center.x + half, center.y)],
        width,
        color,
        dash,
    };
    match series.kind {
        Kind::Scatter { marker, .. } => {
            paint::marker_prims(out, center, look.marker.unwrap_or(marker), look.radius.min(5.0), color);
        }
        Kind::Bars { .. } | Kind::Pie { .. } | Kind::Waterfall { .. } => out.push(Prim::Rect {
            rect: Rect::from_center_size(center, vec2(10.0, 10.0)),
            radius: 2.0,
            fill: color.gamma_multiply(look.fill_opacity.max(0.75)),
            stroke: None,
        }),
        Kind::Candles { .. } => {
            out.push(line(1.0, Dash::Solid));
            out.push(Prim::Rect {
                rect: Rect::from_center_size(center, vec2(6.0, 9.0)),
                radius: 0.0,
                fill: color,
                stroke: None,
            });
        }
        Kind::Heatmap => {
            for (index, tint) in [0.35, 0.6, 0.9].into_iter().enumerate() {
                out.push(Prim::Rect {
                    rect: Rect::from_min_size(
                        pos2(center.x - half + index as f32 * (SWATCH / 3.0), center.y - 5.0),
                        vec2(SWATCH / 3.0, 10.0),
                    ),
                    radius: 0.0,
                    fill: color.gamma_multiply(tint),
                    stroke: None,
                });
            }
        }
        Kind::Radar => {
            let points = vec![
                center + vec2(0.0, -5.0),
                center + vec2(5.0, 0.0),
                center + vec2(0.0, 5.0),
                center + vec2(-5.0, 0.0),
                center + vec2(0.0, -5.0),
            ];
            out.push(Prim::Line {
                points,
                width: width.clamp(1.0, 3.0),
                color,
                dash,
            });
        }
        Kind::Gauge { .. } => {
            let points: Vec<Pos2> = (0..=8)
                .map(|step| {
                    let angle = PI * 1.15 + PI * 0.7 * step as f32 / 8.0;
                    center + vec2(angle.cos(), angle.sin()) * 6.0
                })
                .collect();
            out.push(Prim::Line {
                points,
                width: width.clamp(1.0, 4.0),
                color,
                dash: Dash::Solid,
            });
        }
        Kind::Line { fill_to, .. } => {
            if fill_to.is_some() {
                out.push(Prim::Rect {
                    rect: Rect::from_min_max(pos2(center.x - half, center.y), pos2(center.x + half, center.y + 5.0)),
                    radius: 0.0,
                    fill: color.gamma_multiply(look.fill_opacity.max(0.25)),
                    stroke: None,
                });
            }
            out.push(line(width.clamp(1.0, 4.0), dash));
            if let Some(marker) = look.marker {
                paint::marker_prims(out, center, marker, look.radius.min(4.0), color);
            }
        }
        Kind::HLine(_) | Kind::VLine(_) => out.push(line(width.clamp(1.0, 3.0), Dash::Solid)),
    }
}

/// `text` cut with "…" to fit `width`.
fn elide(measure: &Measure, text: &str, size: f32, width: f32) -> String {
    if measure.text(text, size).0 <= width {
        return text.to_owned();
    }
    let mut chars: Vec<char> = text.chars().collect();
    while !chars.is_empty() {
        chars.pop();
        let candidate: String = chars.iter().collect::<String>() + "…";
        if measure.text(&candidate, size).0 <= width {
            return candidate;
        }
    }
    "…".to_owned()
}
