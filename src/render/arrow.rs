//! The arrow that shows which way the layer turns.
//!
//! The path lies in the plane of a visible face (Front for `x` and `y`
//! axes, Right for `z`), along the middle of the turning layers, and follows
//! the velocity `sense * e_axis x p` of a point of that plane. It is drawn in
//! plane coordinates and projected with the stickers, so it shares their shear.

use super::action::{velocity, Action};
use super::font;
use super::geometry::{plane_coords, plane_point, Projection};
use super::scene::{disc, Part, Shape};
use super::style::{Rgb, ARROW_CYAN, ARROW_RED, WHITE};
use crate::puzzle::Face;

/// Which way the arrow points on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Clockwise,
    CounterClockwise,
}

impl Direction {
    pub(crate) fn glyph(self) -> char {
        match self {
            Direction::Up => '\u{2191}',
            Direction::Down => '\u{2193}',
            Direction::Left => '\u{2190}',
            Direction::Right => '\u{2192}',
            Direction::Clockwise => '\u{21bb}',
            Direction::CounterClockwise => '\u{21ba}',
        }
    }
}

pub(crate) struct Plan {
    pub face: Face,
    /// The centre line in plane coordinates, tail first.
    pub path: Vec<[f64; 2]>,
    pub direction: Direction,
    /// Whole-cube rotations get a curved cyan arrow.
    pub rotation: bool,
}

pub(crate) fn plan(action: &Action, n: usize) -> Plan {
    let nf = n as f64;
    let (axis, _) = action.axis_sign();
    let sense = action.sense();
    let face = if axis == 2 && !matches!(action, Action::Rotate { .. }) {
        Face::R
    } else {
        Face::F
    };
    let rotation = matches!(action, Action::Rotate { .. });
    if rotation {
        return rotation_plan(axis, sense, nf);
    }
    let mut p0 = [0.0; 3];
    p0[axis] = action.band_centre(n);
    let normal = if face == Face::R { 0 } else { 2 };
    p0[normal] = nf;
    let [cu, cv] = plane_coords(face, p0);
    let [du, dv] = plane_coords(face, velocity(axis, sense, p0));
    let len = du.hypot(dv);
    let (du, dv) = (du / len, dv / len);
    let reach = nf - 0.4;
    let path = vec![
        [cu - du * reach, cv - dv * reach],
        [cu + du * reach, cv + dv * reach],
    ];
    Plan {
        face,
        direction: straight_direction(du, dv),
        path,
        rotation: false,
    }
}

fn straight_direction(du: f64, dv: f64) -> Direction {
    if du.abs() > dv.abs() {
        if du > 0.0 {
            Direction::Right
        } else {
            Direction::Left
        }
    } else if dv > 0.0 {
        Direction::Up
    } else {
        Direction::Down
    }
}

/// A curved arrow over the Front face: a bow for `x` and `y`, a ring for `z`.
fn rotation_plan(axis: usize, sense: i32, n: f64) -> Plan {
    let steps = 32;
    let mut path = Vec::with_capacity(steps + 1);
    let direction = if axis == 2 {
        let clockwise = sense < 0;
        let (start, sweep) = if clockwise {
            (150.0_f64, -260.0)
        } else {
            (30.0, 260.0)
        };
        for i in 0..=steps {
            let angle = (start + sweep * i as f64 / steps as f64).to_radians();
            path.push([0.68 * n * angle.cos(), 0.68 * n * angle.sin()]);
        }
        if clockwise {
            Direction::Clockwise
        } else {
            Direction::CounterClockwise
        }
    } else {
        let p0 = [0.0, 0.0, n];
        let [du, dv] = plane_coords(Face::F, velocity(axis, sense, p0));
        let sign = if axis == 0 { dv.signum() } else { du.signum() };
        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let along = sign * 0.78 * n * (2.0 * t - 1.0);
            let bow = 0.3 * n * (std::f64::consts::PI * t).sin();
            path.push(if axis == 0 {
                [bow, along]
            } else {
                [along, -bow]
            });
        }
        straight_direction(du, dv)
    };
    Plan {
        face: Face::F,
        path,
        direction,
        rotation: true,
    }
}

fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn unit(v: [f64; 2]) -> [f64; 2] {
    let len = v[0].hypot(v[1]).max(1e-12);
    [v[0] / len, v[1] / len]
}

/// The path shortened by `cut` at its end, and the unit direction of the tip.
fn trim(path: &[[f64; 2]], cut: f64) -> (Vec<[f64; 2]>, [f64; 2], [f64; 2]) {
    let tip = *path.last().expect("a path has points");
    let mut left = cut;
    let mut kept = path.to_vec();
    while kept.len() > 2 {
        let last = kept[kept.len() - 1];
        let seg = sub(last, kept[kept.len() - 2]);
        let len = seg[0].hypot(seg[1]);
        if len > left {
            break;
        }
        left -= len;
        kept.pop();
    }
    let last = kept[kept.len() - 1];
    let back = kept[kept.len() - 2];
    let d = unit(sub(last, back));
    *kept.last_mut().expect("kept points") = [last[0] - d[0] * left, last[1] - d[1] * left];
    let base = *kept.last().expect("kept points");
    (kept, base, unit(sub(tip, base)))
}

/// An outline of the stroked `path`, `width` wide.
fn ribbon(path: &[[f64; 2]], width: f64) -> Vec<[f64; 2]> {
    let mut left = Vec::new();
    let mut right = Vec::new();
    for i in 0..path.len() {
        let a = path[i.saturating_sub(1)];
        let b = path[(i + 1).min(path.len() - 1)];
        let d = unit(sub(b, a));
        let normal = [-d[1] * width / 2.0, d[0] * width / 2.0];
        left.push([path[i][0] + normal[0], path[i][1] + normal[1]]);
        right.push([path[i][0] - normal[0], path[i][1] - normal[1]]);
    }
    right.reverse();
    left.extend(right);
    left
}

struct Metrics {
    width: f64,
    head_len: f64,
    head_half: f64,
}

/// One arrow (shaft and head) as plane-coordinate polygons; `grow` inflates it for the halo.
fn outline(path: &[[f64; 2]], m: &Metrics, grow: f64) -> Vec<Vec<[f64; 2]>> {
    let (shaft, base, dir) = trim(path, m.head_len);
    let tip = [base[0] + dir[0] * m.head_len, base[1] + dir[1] * m.head_len];
    let mut shaft = shaft;
    let first = shaft[0];
    let d0 = unit(sub(shaft[1], shaft[0]));
    shaft[0] = [first[0] - d0[0] * grow, first[1] - d0[1] * grow];
    let side = [-dir[1], dir[0]];
    let half = m.head_half + grow * 1.3;
    let b = [base[0] - dir[0] * grow * 0.3, base[1] - dir[1] * grow * 0.3];
    let t = [tip[0] + dir[0] * grow * 1.8, tip[1] + dir[1] * grow * 1.8];
    let head = vec![
        [b[0] + side[0] * half, b[1] + side[1] * half],
        t,
        [b[0] - side[0] * half, b[1] - side[1] * half],
    ];
    vec![ribbon(&shaft, m.width + 2.0 * grow), head]
}

/// The halo, the arrow and, for half turns, the badge.
///
/// `pixels_per_unit` is how many pixels one canvas unit becomes; it only
/// sets minimum sizes so that small icons keep a readable arrow.
pub(crate) fn shapes(
    plan: &Plan,
    action: &Action,
    n: usize,
    projection: &Projection,
    pixels_per_unit: f64,
) -> Vec<Shape> {
    let ppm = projection.scale * pixels_per_unit;
    let width = 0.95_f64.max(3.2 / ppm);
    // The Right face is foreshortened across its width (the `u` axis), so an
    // arrow running along it is widened to look as bold as one on the Front.
    let across = if plan.face == Face::R { 2.2 } else { 1.0 };
    let metrics = Metrics {
        width: width * across,
        head_len: 1.9 * width,
        head_half: (1.5 * width * across).min(2.2),
    };
    let grow = 0.28_f64.max(1.4 / ppm) * across.sqrt();
    let nf = n as f64;
    let flat = |poly: Vec<[f64; 2]>| -> Vec<[f64; 2]> {
        poly.into_iter()
            .map(|[u, v]| projection.project(plane_point(plan.face, nf, u, v)))
            .collect()
    };
    let colour: Rgb = if plan.rotation { ARROW_CYAN } else { ARROW_RED };
    let mut shapes = Vec::new();
    for poly in outline(&plan.path, &metrics, grow) {
        shapes.push(Shape::new(Part::Halo, flat(poly), WHITE));
    }
    for poly in outline(&plan.path, &metrics, 0.0) {
        shapes.push(Shape::new(Part::Arrow, flat(poly), colour));
    }
    if action.is_half() {
        let mid = plan.path[plan.path.len() / 2];
        let mid = if plan.path.len() % 2 == 0 {
            let a = plan.path[plan.path.len() / 2 - 1];
            [(a[0] + mid[0]) / 2.0, (a[1] + mid[1]) / 2.0]
        } else {
            mid
        };
        let centre = projection.project(plane_point(plan.face, nf, mid[0], mid[1]));
        let radius = (1.35 * projection.scale).max(5.5 / pixels_per_unit);
        let halo = radius + (grow * projection.scale).max(1.0 / pixels_per_unit);
        shapes.push(Shape::new(Part::Halo, disc(centre, halo), WHITE));
        shapes.push(Shape::new(Part::Badge, disc(centre, radius), ARROW_RED));
        let height = radius * 1.25;
        let glyph = height / font::GLYPH_HEIGHT;
        shapes.extend(font::text(
            "2",
            centre[0] - glyph * 0.5,
            centre[1] - height / 2.0,
            height,
            radius * 0.28,
            WHITE,
            Part::Badge,
        ));
    }
    shapes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{locate, Facelets, PuzzleKind};

    fn act(text: &str) -> Action {
        text.parse().unwrap()
    }

    fn centre(kind: PuzzleKind, index: usize) -> [f64; 3] {
        let (p, nrm) = locate(kind, index);
        [0, 1, 2].map(|i| f64::from(p[i] + nrm[i]))
    }

    fn kinds() -> [PuzzleKind; 3] {
        [PuzzleKind::Cube3, PuzzleKind::Cube4, PuzzleKind::Cube5]
    }

    /// The stickers that leave the arrow's face travel the way the arrow points.
    #[test]
    fn arrows_point_where_apply_moves_the_stickers() {
        let moves = [
            "R", "R'", "L", "L'", "U", "U'", "D", "D'", "F", "F'", "B", "B'", "Rw", "Rw'", "2R",
            "2U'", "2F", "3Rw", "x", "x'", "y", "y'",
        ];
        for kind in kinds() {
            for text in moves {
                let action = act(text);
                if !action.fits(kind) {
                    continue;
                }
                let plan = plan(&action, kind.size());
                let (start, end) = (plan.path[0], *plan.path.last().unwrap());
                let d = unit(sub(end, start));
                let d3 = plane_point(plan.face, 0.0, d[0], d[1]);
                let solved = Facelets::solved(kind);
                let after = action.apply(&solved).unwrap();
                let (mut old, mut new) = ([0.0; 3], [0.0; 3]);
                let (mut n_old, mut n_new) = (0.0, 0.0);
                for i in 0..kind.facelet_count() {
                    let c = centre(kind, i);
                    let on_plane = Face::ALL[i / kind.face_len()] == plan.face;
                    if on_plane && action.moves(kind, locate(kind, i).0) {
                        (0..3).for_each(|k| old[k] += c[k]);
                        n_old += 1.0;
                    }
                    if after.stickers()[i] == plan.face && !on_plane {
                        (0..3).for_each(|k| new[k] += c[k]);
                        n_new += 1.0;
                    }
                }
                assert!(n_old > 0.0 && n_new > 0.0, "{kind:?} {text}");
                let shift: f64 = (0..3)
                    .map(|k| (new[k] / n_new - old[k] / n_old) * d3[k])
                    .sum();
                assert!(shift > 0.5, "{kind:?} {text}: shift {shift} along {d:?}");
            }
        }
    }

    #[test]
    fn z_turns_the_right_face_down_when_the_arrow_is_clockwise() {
        for (text, clockwise, lands) in [("z", true, Face::D), ("z'", false, Face::U)] {
            let kind = PuzzleKind::Cube3;
            let action = act(text);
            let plan = plan(&action, 3);
            assert_eq!(plan.direction == Direction::Clockwise, clockwise);
            let after = action.apply(&Facelets::solved(kind)).unwrap();
            let landed = (0..kind.facelet_count())
                .any(|i| Face::ALL[i / kind.face_len()] == lands && after.stickers()[i] == Face::R);
            assert!(landed, "{text}");
        }
    }

    #[test]
    fn named_directions_follow_the_gan_sheet() {
        let want = [
            ("R", Direction::Up),
            ("R'", Direction::Down),
            ("L", Direction::Down),
            ("U", Direction::Left),
            ("U'", Direction::Right),
            ("D", Direction::Right),
            ("F", Direction::Down),
            ("B", Direction::Up),
            ("x", Direction::Up),
            ("y", Direction::Left),
            ("z", Direction::Clockwise),
        ];
        for (text, direction) in want {
            assert_eq!(plan(&act(text), 3).direction, direction, "{text}");
        }
    }

    #[test]
    fn half_turns_point_like_the_clockwise_turn() {
        for (half, quarter) in [("R2", "R"), ("U2", "U"), ("2F2", "2F"), ("Rw2", "Rw")] {
            let (a, b) = (plan(&act(half), 5), plan(&act(quarter), 5));
            assert_eq!(a.path, b.path);
        }
    }

    /// The arrow runs along a layer that really turns, over its whole length.
    #[test]
    fn the_arrow_lies_over_turning_stickers() {
        let kind = PuzzleKind::Cube5;
        for text in ["R", "L'", "U", "D", "F", "B'", "2R", "Rw", "3Rw", "2U"] {
            let action = act(text);
            let plan = plan(&action, 5);
            for t in [0.1, 0.3, 0.5, 0.7, 0.9] {
                let (a, b) = (plan.path[0], plan.path[1]);
                let point = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
                let hit = (0..kind.facelet_count())
                    .filter(|&i| Face::ALL[i / kind.face_len()] == plan.face)
                    .find(|&i| {
                        let [u, v] = plane_coords(plan.face, centre(kind, i));
                        (u - point[0]).abs() <= 1.0 && (v - point[1]).abs() <= 1.0
                    })
                    .expect("a sticker under the arrow");
                let (position, _) = locate(kind, hit);
                assert!(action.moves(kind, position), "{text} at {t}");
            }
        }
    }
}
