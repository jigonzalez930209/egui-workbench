//! The direction arrow beside the newest samples of streaming line series.

use egui::{Color32, Pos2, vec2};

use super::axis::Transform;
use super::chart_config::DirectionArrowConfig;
use super::scene::{Dash, Prim};
use super::series::Series;

/// Draw the direction arrow beside the newest sample of a line-like series. It uses the
/// configured number of trailing finite samples, so the head follows buffer growth.
pub(super) fn direction_arrow(
    out: &mut Vec<Prim>,
    transform: &Transform,
    series: &Series<'_>,
    config: &DirectionArrowConfig,
    color: Color32,
) {
    if !series.has_direction() || series.data().len() < 2 {
        return;
    }
    let mut samples: Vec<Pos2> = series
        .data()
        .iter()
        .filter(|point| point[0].is_finite() && point[1].is_finite())
        .rev()
        .take(config.points.max(2))
        .map(|point| transform.to_screen(*point))
        .collect();
    samples.reverse();
    arrow(out, transform.frame, &samples, config, color);
}

/// The arrow beside the newest point of a projected path. The shaft sits beside the
/// path (toward the frame center) so both the shaft and the head stay visible.
pub(super) fn arrow(
    out: &mut Vec<Prim>,
    frame: egui::Rect,
    samples: &[Pos2],
    config: &DirectionArrowConfig,
    color: Color32,
) {
    let Some((first, last)) = samples.first().zip(samples.last()) else {
        return;
    };
    let (start, end) = (*first, *last);
    if !(start.is_finite() && end.is_finite()) || !frame.expand(18.0).contains(end) {
        return;
    }
    let direction = end - start;
    if direction.length() < 6.0 {
        return;
    }
    let unit = direction.normalized();
    let normal = vec2(-unit.y, unit.x);
    let head = config.head.max(2.0);
    let length = config.length.max(4.0);
    // The series line already occupies the exact path of the newest samples, so an arrow
    // drawn on that path only shows its head. Shift it beside the line (toward the
    // frame center) and the shaft becomes visible.
    let gap = (head * 0.9 + config.width * 2.0).max(6.0);
    let towards_center = normal
        * if normal.dot(frame.center() - end) >= 0.0 {
            1.0
        } else {
            -1.0
        };
    let candidate = end + towards_center * gap;
    let fits = |at: egui::Pos2| {
        let tip = at - unit * 1.5;
        let tail = tip - unit * length;
        frame.shrink(2.0).contains(tip) && frame.shrink(2.0).contains(tail)
    };
    let anchor = if fits(candidate) {
        candidate
    } else if fits(end - towards_center * gap) {
        end - towards_center * gap
    } else {
        end
    };
    let tip = anchor - unit * 1.5;
    let tail = tip - unit * length;
    let base = tip - unit * head;
    out.push(Prim::Line {
        points: vec![tail, base],
        width: config.width,
        color,
        dash: Dash::Solid,
    });
    out.push(Prim::Polygon {
        points: vec![tip, base + normal * head * 0.55, base - normal * head * 0.55],
        color,
    });
}
