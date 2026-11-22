//! Spline smoothing of screen-space polylines: centripetal Catmull–Rom and monotone
//! cubic (Fritsch–Carlson), sampled as cubic Béziers.

use egui::{Pos2, pos2};

use super::series::Smoothing;

/// `run` as a smooth curve in screen space (so it works on log axes too and the
/// export matches the screen). Runs denser than one point per 2 pt stay straight.
pub(super) fn smooth(run: Vec<Pos2>, smoothing: Smoothing, frame_width: f32) -> Vec<Pos2> {
    if smoothing == Smoothing::None || run.len() < 3 || run.len() as f32 * 2.0 > frame_width.max(1.0) {
        return run;
    }
    let increasing = run.windows(2).all(|pair| pair[1].x > pair[0].x);
    let controls = if smoothing == Smoothing::Monotone && increasing {
        monotone_controls(&run)
    } else {
        catmull_rom_controls(&run)
    };
    let mut out = Vec::with_capacity(run.len() * 8);
    out.push(run[0]);
    for (index, [c1, c2]) in controls.into_iter().enumerate() {
        let (p0, p3) = (run[index], run[index + 1]);
        // About one vertex per 3 pt of segment: smooth at any zoom, cheap when far.
        let steps = (p0.distance(p3) / 3.0).ceil().clamp(1.0, 32.0) as usize;
        for step in 1..=steps {
            let t = step as f32 / steps as f32;
            let u = 1.0 - t;
            let point = p0.to_vec2() * (u * u * u)
                + c1.to_vec2() * (3.0 * u * u * t)
                + c2.to_vec2() * (3.0 * u * t * t)
                + p3.to_vec2() * (t * t * t);
            out.push(point.to_pos2());
        }
    }
    out
}

/// The two Bézier control points of every segment of a centripetal Catmull–Rom
/// spline (α = ½: no cusps or self-intersections inside a segment). The ends are
/// extended by mirroring their neighbor.
fn catmull_rom_controls(points: &[Pos2]) -> Vec<[Pos2; 2]> {
    let n = points.len();
    let at = |index: isize| -> Pos2 {
        if index < 0 {
            points[0] + (points[0] - points[1])
        } else if index as usize >= n {
            points[n - 1] + (points[n - 1] - points[n - 2])
        } else {
            points[index as usize]
        }
    };
    (0..n - 1)
        .map(|segment| {
            let index = segment as isize;
            let (p0, p1, p2, p3) = (at(index - 1), at(index), at(index + 1), at(index + 2));
            // d = |Δ|^α and d² = |Δ|^(2α) with α = ½.
            let (s1, s2, s3) = (p0.distance(p1), p1.distance(p2), p2.distance(p3));
            let (d1, d2, d3) = (s1.sqrt(), s2.sqrt(), s3.sqrt());
            let c1 = if d1 < 1e-4 {
                p1
            } else {
                let v = (p2.to_vec2() * s1 - p0.to_vec2() * s2 + p1.to_vec2() * (2.0 * s1 + 3.0 * d1 * d2 + s2))
                    / (3.0 * d1 * (d1 + d2));
                v.to_pos2()
            };
            let c2 = if d3 < 1e-4 {
                p2
            } else {
                let v = (p1.to_vec2() * s3 - p3.to_vec2() * s2 + p2.to_vec2() * (2.0 * s3 + 3.0 * d3 * d2 + s2))
                    / (3.0 * d3 * (d3 + d2));
                v.to_pos2()
            };
            [c1, c2]
        })
        .collect()
}

/// Control points of a monotone cubic Hermite spline (Fritsch–Carlson) for points
/// with strictly increasing x: the curve never leaves the range of its two ends.
fn monotone_controls(points: &[Pos2]) -> Vec<[Pos2; 2]> {
    let n = points.len();
    let slopes: Vec<f32> = points
        .windows(2)
        .map(|pair| (pair[1].y - pair[0].y) / (pair[1].x - pair[0].x))
        .collect();
    let mut tangents = vec![0.0f32; n];
    tangents[0] = slopes[0];
    tangents[n - 1] = slopes[n - 2];
    for index in 1..n - 1 {
        let (a, b) = (slopes[index - 1], slopes[index]);
        tangents[index] = if a * b <= 0.0 { 0.0 } else { (a + b) * 0.5 };
    }
    for index in 0..n - 1 {
        let slope = slopes[index];
        if slope == 0.0 {
            tangents[index] = 0.0;
            tangents[index + 1] = 0.0;
            continue;
        }
        let (a, b) = (tangents[index] / slope, tangents[index + 1] / slope);
        let length = a.hypot(b);
        if length > 3.0 {
            let scale = 3.0 / length;
            tangents[index] = scale * a * slope;
            tangents[index + 1] = scale * b * slope;
        }
    }
    points
        .windows(2)
        .enumerate()
        .map(|(index, pair)| {
            let third = (pair[1].x - pair[0].x) / 3.0;
            [
                pos2(pair[0].x + third, pair[0].y + tangents[index] * third),
                pos2(pair[1].x - third, pair[1].y - tangents[index + 1] * third),
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zigzag() -> Vec<Pos2> {
        vec![pos2(0.0, 50.0), pos2(40.0, 10.0), pos2(80.0, 50.0), pos2(120.0, 30.0)]
    }

    #[test]
    fn splines_pass_through_every_point() {
        for smoothing in [Smoothing::Spline, Smoothing::Monotone] {
            let input = zigzag();
            let curve = smooth(input.clone(), smoothing, 1000.0);
            assert!(curve.len() > input.len() * 4, "{smoothing:?} adds vertices");
            for point in &input {
                assert!(
                    curve.iter().any(|vertex| vertex.distance(*point) < 1e-3),
                    "{smoothing:?} passes through {point:?}"
                );
            }
        }
    }

    #[test]
    fn monotone_never_overshoots() {
        let curve = smooth(zigzag(), Smoothing::Monotone, 1000.0);
        assert!(
            curve
                .iter()
                .all(|vertex| (10.0 - 1e-3..=50.0 + 1e-3).contains(&vertex.y))
        );
        // Between the two lows at y = 50 the curve stays inside [10, 50] too, and the
        // x order is kept.
        assert!(curve.windows(2).all(|pair| pair[1].x >= pair[0].x));
    }

    #[test]
    fn dense_or_unsmoothed_runs_stay_straight() {
        assert_eq!(smooth(zigzag(), Smoothing::None, 1000.0), zigzag());
        assert_eq!(smooth(zigzag(), Smoothing::Spline, 6.0), zigzag());
    }

    #[test]
    fn unsorted_runs_fall_back_to_catmull_rom() {
        let loop_points = vec![pos2(0.0, 0.0), pos2(50.0, 0.0), pos2(50.0, 50.0), pos2(0.0, 50.0)];
        let curve = smooth(loop_points.clone(), Smoothing::Monotone, 1000.0);
        assert!(curve.len() > loop_points.len());
        assert!(curve.iter().all(|vertex| vertex.is_finite()));
    }
}
