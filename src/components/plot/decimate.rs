//! Fewer points with the same picture: thinning of unsorted lines, one marker per
//! screen cell for dense scatter, and Ramer–Douglas–Peucker simplification of the
//! screen polylines. The tolerances are below what a display can show.

use egui::Pos2;

use super::axis::Transform;

/// Points of an unsorted line closer than this to the last drawn point are skipped.
pub(super) const LINE_SPACING: f32 = 0.5;
/// Scatter markers keep one per square cell this fraction of their radius wide: a
/// marker in an occupied cell overlaps the first one almost entirely, and the outline of
/// a dense band of markers moves by at most 7 % of the radius.
pub(super) const MARKER_SPACING: f32 = 0.75;

/// Vertices of a line closer than this (in points) to the straight path between their
/// neighbors are dropped: invisible, and a smooth curve of thousands of samples
/// tessellates as a few hundred vertices.
pub(super) const SIMPLIFY_TOLERANCE: f32 = 0.3;

/// Ramer–Douglas–Peucker on screen positions: keep the ends and every vertex farther
/// than `tolerance` from the chord of the stretch it lies in.
pub(super) fn simplify(points: Vec<Pos2>, tolerance: f32) -> Vec<Pos2> {
    if points.len() < 3 {
        return points;
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut stack = vec![(0, points.len() - 1)];
    let tolerance_sq = tolerance * tolerance;
    while let Some((first, last)) = stack.pop() {
        if last <= first + 1 {
            continue;
        }
        let (a, b) = (points[first], points[last]);
        let chord = b - a;
        let length_sq = chord.length_sq();
        let mut farthest = (0.0, first);
        for (index, point) in points.iter().enumerate().take(last).skip(first + 1) {
            let offset = *point - a;
            // Distance to the segment (to the end point when the chord is degenerate).
            let distance_sq = if length_sq <= f32::EPSILON {
                offset.length_sq()
            } else {
                let t = (offset.dot(chord) / length_sq).clamp(0.0, 1.0);
                (offset - chord * t).length_sq()
            };
            if distance_sq > farthest.0 {
                farthest = (distance_sq, index);
            }
        }
        if farthest.0 > tolerance_sq {
            keep[farthest.1] = true;
            stack.push((first, farthest.1));
            stack.push((farthest.1, last));
        }
    }
    points
        .into_iter()
        .zip(keep)
        .filter_map(|(point, keep)| keep.then_some(point))
        .collect()
}

/// One marker per `cell`-sized square of the frame: the first point that lands in it.
/// Unlike [`thin`], a curve that retraces itself (the cycles of a voltammogram) draws
/// its markers once. Points outside the frame (plus a cell) are skipped.
pub(super) fn one_per_cell(transform: &Transform, data: &[[f64; 2]], cell: f32) -> Vec<usize> {
    let frame = transform.frame.expand(cell);
    let columns = (frame.width() / cell).ceil() as usize + 1;
    let rows = (frame.height() / cell).ceil() as usize + 1;
    let mut seen = vec![0u64; (columns * rows).div_ceil(64)];
    let mut indices = Vec::new();
    for (index, point) in data.iter().enumerate() {
        let position = transform.to_screen(*point);
        if !position.is_finite() || !frame.contains(position) {
            continue;
        }
        let column = ((position.x - frame.left()) / cell) as usize;
        let row = ((position.y - frame.top()) / cell) as usize;
        let bit = row.min(rows - 1) * columns + column.min(columns - 1);
        let (word, mask) = (bit / 64, 1u64 << (bit % 64));
        if seen[word] & mask == 0 {
            seen[word] |= mask;
            indices.push(index);
        }
    }
    indices
}

/// Every point at least `spacing` points (on screen) from the last one kept, plus the
/// last point. Non-finite points are kept, so they still break the line.
pub(super) fn thin(transform: &Transform, data: &[[f64; 2]], spacing: f32) -> Vec<usize> {
    let mut indices = Vec::new();
    let mut last: Option<Pos2> = None;
    let spacing_sq = spacing * spacing;
    for (index, point) in data.iter().enumerate() {
        let position = transform.to_screen(*point);
        if !position.is_finite() {
            indices.push(index);
            last = None;
            continue;
        }
        if let Some(previous) = last
            && (position - previous).length_sq() < spacing_sq
            && index + 1 != data.len()
        {
            continue;
        }
        indices.push(index);
        last = Some(position);
    }
    indices
}

#[cfg(test)]
mod tests {
    use super::super::paint::nearest;
    use super::super::series::Series;
    use super::*;
    use crate::components::plot::{Bounds, Scale};
    use egui::{Rect, pos2, vec2};

    fn transform() -> Transform {
        Transform {
            frame: Rect::from_min_size(Pos2::ZERO, vec2(400.0, 300.0)),
            view: Bounds::from_corners([-1.0, -1.0], [1.0, 1.0]),
            x_scale: Scale::Linear,
            y_scale: Scale::Linear,
            y2: None,
        }
    }

    #[test]
    fn unsorted_loops_keep_what_the_screen_can_show() {
        // A circle traced ten times with 100 000 points: x goes back and forth.
        let data: Vec<[f64; 2]> = (0..100_000)
            .map(|index| {
                let angle = index as f64 * std::f64::consts::TAU * 10.0 / 100_000.0;
                [angle.cos() * 0.8, angle.sin() * 0.8]
            })
            .collect();
        let kept = thin(&transform(), &data, LINE_SPACING);
        assert!(kept.len() < 20_000, "{} points kept", kept.len());
        assert_eq!(kept.first(), Some(&0));
        assert_eq!(
            kept.last(),
            Some(&(data.len() - 1)),
            "the line still ends at its last point"
        );
        let screen: Vec<Pos2> = kept.iter().map(|&index| transform().to_screen(data[index])).collect();
        assert!(
            screen.windows(2).all(|pair| pair[0].distance(pair[1]) < 2.0),
            "no visible jump between kept points"
        );
    }

    #[test]
    fn simplification_keeps_corners_and_drops_collinear_points() {
        let line: Vec<Pos2> = (0..=100).map(|x| pos2(x as f32, 10.0)).collect();
        assert_eq!(simplify(line, 0.3), [pos2(0.0, 10.0), pos2(100.0, 10.0)]);
        let corner = vec![pos2(0.0, 0.0), pos2(5.0, 0.0), pos2(10.0, 0.0), pos2(10.0, 10.0)];
        assert_eq!(
            simplify(corner, 0.3),
            [pos2(0.0, 0.0), pos2(10.0, 0.0), pos2(10.0, 10.0)]
        );
        // A smooth arc keeps every vertex within the tolerance of the original.
        let arc: Vec<Pos2> = (0..2000)
            .map(|index| {
                let angle = index as f32 / 2000.0 * std::f32::consts::PI;
                pos2(200.0 * angle.cos(), 200.0 * angle.sin())
            })
            .collect();
        let simple = simplify(arc.clone(), 0.3);
        assert!(simple.len() < 200, "{} vertices", simple.len());
        for point in &arc {
            let nearest = simple
                .windows(2)
                .map(|pair| {
                    let chord = pair[1] - pair[0];
                    let t = ((*point - pair[0]).dot(chord) / chord.length_sq()).clamp(0.0, 1.0);
                    (*point - pair[0] - chord * t).length()
                })
                .fold(f32::INFINITY, f32::min);
            assert!(nearest <= 0.31, "{nearest}");
        }
    }

    #[test]
    fn retraced_markers_are_drawn_once() {
        let cycle: Vec<[f64; 2]> = (0..1000)
            .map(|index| {
                let angle = index as f64 / 1000.0 * std::f64::consts::TAU;
                [angle.cos() * 0.8, angle.sin() * 0.8]
            })
            .collect();
        let once = one_per_cell(&transform(), &cycle, 2.0).len();
        let ten: Vec<[f64; 2]> = cycle.iter().copied().cycle().take(10_000).collect();
        assert_eq!(
            one_per_cell(&transform(), &ten, 2.0).len(),
            once,
            "ten laps draw like one"
        );
        assert!(once > 100, "the circle is still drawn: {once}");
    }

    #[test]
    fn hover_on_a_thinned_line_finds_the_exact_point() {
        let data: Vec<[f64; 2]> = (0..20_000)
            .map(|index| {
                let angle = index as f64 * std::f64::consts::TAU * 3.0 / 20_000.0;
                [angle.cos() * 0.8, angle.sin() * 0.8]
            })
            .collect();
        let series = Series::line("loop", data.as_slice());
        let drawn = thin(&transform(), &data, LINE_SPACING);
        for pointer in [pos2(360.0, 150.0), pos2(200.0, 30.0), pos2(81.0, 170.0)] {
            let fast = nearest(&transform(), &series, pointer, 12.0, false, Some(&drawn));
            let full = nearest(&transform(), &series, pointer, 12.0, false, None);
            assert_eq!(
                fast.map(|(_, _, distance)| distance),
                full.map(|(_, _, distance)| distance),
                "{pointer:?}"
            );
        }
    }

    #[test]
    fn breaks_survive_thinning() {
        let data = [[0.0, 0.0], [0.0001, 0.0], [0.0, f64::NAN], [0.5, 0.5], [0.5001, 0.5]];
        let kept = thin(&transform(), &data, LINE_SPACING);
        assert_eq!(kept, [0, 2, 3, 4]);
    }
}
