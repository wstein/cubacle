//! Curves in model coordinates and what the viewer sees of them.
//!
//! The cube spans `-n..n` on every axis, `x` points right, `y` up and `z`
//! towards the viewer. A model point `p` appears on the screen where every
//! point `p + t * VIEW` appears.

use crate::render::geometry::{Projection, DEPTH};

pub(super) type P3 = [f64; 3];
pub(super) type P2 = [f64; 2];

/// The direction towards the viewer along which a point keeps its screen position.
pub(super) const VIEW: P3 = [DEPTH.0, DEPTH.1, 1.0];

/// A point of the circle of radius `r` around `axis` through the cube center.
/// Growing `angle` is the positive turn of the whole cube: for `x` the Front
/// moves up, for `y` it moves left, for `z` the turn is clockwise seen from the
/// front.
pub(super) fn ring_point(axis: usize, r: f64, angle: f64) -> P3 {
    let (e1, e2): (P3, P3) = match axis {
        0 => ([0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
        1 => ([0.0, 0.0, 1.0], [-1.0, 0.0, 0.0]),
        _ => ([1.0, 0.0, 0.0], [0.0, -1.0, 0.0]),
    };
    let (cos, sin) = (angle.cos(), angle.sin());
    [0, 1, 2].map(|k| r * (cos * e1[k] + sin * e2[k]))
}

/// The point of the axis line at signed distance `s` from the cube center.
pub(super) fn axis_point(axis: usize, s: f64) -> P3 {
    let mut p = [0.0; 3];
    p[axis] = s;
    p
}

/// Does the ray from `p` towards the viewer hit the cube `[-m, m]^3`? Then the
/// cube hides `p`.
pub(super) fn occluded(p: P3, m: f64) -> bool {
    let (mut near, mut far) = (f64::NEG_INFINITY, f64::INFINITY);
    for k in 0..3 {
        let (a, b) = ((-m - p[k]) / VIEW[k], (m - p[k]) / VIEW[k]);
        near = near.max(a.min(b));
        far = far.min(a.max(b));
    }
    far >= near.max(0.0)
}

/// Is a point of a circle around `axis` (through the cube center) nearer to the
/// viewer than the axis line? The half of the circle on the viewer's side of the
/// plane through the axis is in front of the line, the other half behind it.
pub(super) fn in_front_of_axis(axis: usize, p: P3) -> bool {
    let (mut across, mut view) = (p, VIEW);
    across[axis] = 0.0;
    view[axis] = 0.0;
    (0..3).map(|k| across[k] * view[k]).sum::<f64>() > 0.0
}

pub(super) fn project_all(points: &[P3], proj: &Projection) -> Vec<P2> {
    points.iter().map(|&p| proj.project(p)).collect()
}

/// The length of a polyline.
pub(super) fn length(points: &[P2]) -> f64 {
    points
        .windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .sum()
}

/// `points` without the last `cut` canvas units (measured on the screen); the
/// new end lies on the old path.
pub(super) fn cut_end(points: &[P3], proj: &Projection, cut: f64) -> Vec<P3> {
    let flat = project_all(points, proj);
    let mut taken = 0.0;
    for i in (1..points.len()).rev() {
        let segment = length(&flat[i - 1..=i]);
        if taken + segment >= cut {
            let f = (cut - taken) / segment.max(1e-12);
            let end = [0, 1, 2].map(|k| points[i][k] + (points[i - 1][k] - points[i][k]) * f);
            let mut kept = points[..i].to_vec();
            kept.push(end);
            return kept;
        }
        taken += segment;
    }
    points[..2].to_vec()
}

pub(super) fn reversed(points: &[P3]) -> Vec<P3> {
    points.iter().rev().copied().collect()
}
