//! Rounded primitives in plane coordinates for the soft arrow: discs and
//! triangles with round corners, so outlines have round joins and no spikes.

use std::f64::consts::TAU;

type P = [f64; 2];

/// A polygon approximating the disc at `c` with radius `r`.
pub(crate) fn disc(c: P, r: f64) -> Vec<P> {
    const STEPS: usize = 28;
    (0..STEPS)
        .map(|i| {
            let a = TAU * i as f64 / STEPS as f64;
            [c[0] + r * a.cos(), c[1] + r * a.sin()]
        })
        .collect()
}

fn dist(a: P, b: P) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// The triangle with its corners rounded by `radius` and grown by `grow` all
/// around: its inset by `radius`, dilated by `radius + grow`, so the outline
/// has round joins at every distance.
pub(crate) fn rounded_triangle(tri: [P; 3], radius: f64, grow: f64) -> Vec<P> {
    let [a, b, c] = tri;
    let (la, lb, lc) = (dist(b, c), dist(a, c), dist(a, b));
    let perimeter = la + lb + lc;
    let incentre = [
        (la * a[0] + lb * b[0] + lc * c[0]) / perimeter,
        (la * a[1] + lb * b[1] + lc * c[1]) / perimeter,
    ];
    let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    let inradius = cross.abs() / perimeter;
    let radius = radius.min(0.6 * inradius);
    let k = 1.0 - radius / inradius;
    let inset = tri.map(|v| {
        [
            incentre[0] + (v[0] - incentre[0]) * k,
            incentre[1] + (v[1] - incentre[1]) * k,
        ]
    });
    let sgn = if cross > 0.0 { 1.0 } else { -1.0 };
    let normal_angle = |p: P, q: P| {
        let d = [q[0] - p[0], q[1] - p[1]];
        (sgn * -d[0]).atan2(sgn * d[1])
    };
    let r = radius + grow;
    let mut out = Vec::new();
    for i in 0..3 {
        let (prev, cur, next) = (inset[(i + 2) % 3], inset[i], inset[(i + 1) % 3]);
        let a1 = normal_angle(prev, cur);
        let mut delta = normal_angle(cur, next) - a1;
        while delta * sgn < 0.0 {
            delta += sgn * TAU;
        }
        const STEPS: usize = 8;
        for s in 0..=STEPS {
            let t = a1 + delta * s as f64 / STEPS as f64;
            out.push([cur[0] + r * t.cos(), cur[1] + r * t.sin()]);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rounded_triangle_stays_within_its_grown_bounds() {
        let tri = [[0.0, 0.0], [4.0, 0.0], [2.0, 5.0]];
        let poly = rounded_triangle(tri, 0.5, 0.0);
        for p in &poly {
            assert!(
                p[0] > -0.01 && p[0] < 4.01 && p[1] > -0.01 && p[1] < 5.01,
                "{p:?}"
            );
        }
        let grown = rounded_triangle(tri, 0.5, 1.0);
        let top = |p: &[P]| p.iter().map(|q| q[1]).fold(f64::MIN, f64::max);
        assert!(top(&grown) > top(&poly) + 0.9);
    }
}
