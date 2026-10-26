//! The arrow that shows which way the layer turns.
//!
//! The path lies in the plane of a visible face (Front for `x` and `y`
//! axes, Right for `z`), along the middle of the turning layers, and follows
//! the velocity `sense * e_axis x p` of a point of that plane. It is drawn in
//! plane coordinates and projected with the stickers, so it shares their shear.
//!
//! The arrow is soft: a shaft with a round tail and a head, about twice as
//! long as wide, with rounded corners, all outlined in white with round joins.
//! Its size is constant on the canvas, graded by cube size, and a margin of 12 %
//! of the face width stays free at both ends.

use super::action::{velocity, Action};
use super::font;
use super::geometry::{plane_coords, plane_point, Projection};
use super::label::LABEL_SCALE;
use super::round::{disc, rounded_triangle};
use super::scene::{Part, Shape, HEIGHT, WIDTH};
use super::style::{ARROW_VIOLET, WHITE};
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
    /// The center line in plane coordinates, tail first.
    pub path: Vec<[f64; 2]>,
    pub direction: Direction,
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
    p0[axis] = action.band_center(n);
    let normal = if face == Face::R { 0 } else { 2 };
    p0[normal] = nf;
    let [cu, cv] = plane_coords(face, p0);
    let [du, dv] = plane_coords(face, velocity(axis, sense, p0));
    let len = du.hypot(dv);
    let (du, dv) = (du / len, dv / len);
    // 12 % of the face width stays free at both ends.
    let reach = nf * 0.76;
    let path = vec![
        [cu - du * reach, cv - dv * reach],
        [cu + du * reach, cv + dv * reach],
    ];
    Plan {
        face,
        direction: straight_direction(du, dv),
        path,
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

/// Sizes in plane units.
struct Metrics {
    width: f64,
    head_len: f64,
    head_half: f64,
    /// Radius of the rounded head corners.
    round: f64,
}

/// The soft arrow: a shaft with a round tail and a head with round corners,
/// all grown by `grow` with round joins, so the outline has no spikes.
fn soft_outline(path: &[[f64; 2]], m: &Metrics, grow: f64) -> Vec<Vec<[f64; 2]>> {
    let (shaft, base, dir) = trim(path, m.head_len);
    let tip = [base[0] + dir[0] * m.head_len, base[1] + dir[1] * m.head_len];
    let side = [-dir[1], dir[0]];
    let head = rounded_triangle(
        [
            [
                base[0] + side[0] * m.head_half,
                base[1] + side[1] * m.head_half,
            ],
            tip,
            [
                base[0] - side[0] * m.head_half,
                base[1] - side[1] * m.head_half,
            ],
        ],
        m.round,
        grow,
    );
    let tail = disc(shaft[0], (m.width + 2.0 * grow) / 2.0);
    vec![ribbon(&shaft, m.width + 2.0 * grow), tail, head]
}

/// How much a unit circle of a face plane shrinks across its narrow axis on the
/// screen, relative to the projection scale: 1 on the Front, less on the
/// skewed Up and Right faces.
fn foreshortening(face: Face, n: f64, projection: &Projection) -> f64 {
    let at = |u: f64, v: f64| projection.project(plane_point(face, n, u, v));
    let o = at(0.0, 0.0);
    let (eu, ev) = (at(1.0, 0.0), at(0.0, 1.0));
    let (a, b) = (eu[0] - o[0], eu[1] - o[1]);
    let (c, d) = (ev[0] - o[0], ev[1] - o[1]);
    let sum = a * a + b * b + c * c + d * d;
    let det = (a * d - b * c).abs();
    let smin = ((sum - (sum * sum - 4.0 * det * det).max(0.0).sqrt()) / 2.0).sqrt();
    (smin / projection.scale).clamp(0.0, 1.0)
}

/// The halo, the arrow and, for half turns, a `2x`.
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
    // The shaft goes from 4.0 canvas units on the 2x2 to 2.8 on the 7x7.
    let grade = (n as f64 - 2.0).clamp(0.0, 5.0) / 5.0;
    // Canvas units per model unit: the arrow is sized in canvas units.
    let unit = 1.0 / projection.scale;
    let width = (4.0 + (2.8 - 4.0) * grade).max(3.2 / pixels_per_unit) * unit;
    // The arrow is drawn in the face plane and projected with it, so it is
    // foreshortened like the face, but only down to 70 % of the Front width
    // so that it stays readable.
    let squeeze = foreshortening(plan.face, n as f64, projection);
    let across = (0.7 / squeeze).clamp(1.0, 2.2);
    // The head is twice as long as wide.
    let metrics = Metrics {
        width: width * across,
        head_len: 4.4 * width,
        head_half: 1.1 * width * across,
        round: 0.12 * width * across,
    };
    let grow = 1.1_f64.max(1.4 / pixels_per_unit) * unit * across.sqrt();
    let nf = n as f64;
    let flat = |poly: Vec<[f64; 2]>| -> Vec<[f64; 2]> {
        poly.into_iter()
            .map(|[u, v]| projection.project(plane_point(plan.face, nf, u, v)))
            .collect()
    };
    let mut shapes = Vec::new();
    for poly in soft_outline(&plan.path, &metrics, grow) {
        shapes.push(Shape::new(Part::Halo, flat(poly), WHITE));
    }
    for poly in soft_outline(&plan.path, &metrics, 0.0) {
        shapes.push(Shape::new(Part::Arrow, flat(poly), ARROW_VIOLET));
    }
    if action.is_half() {
        shapes.extend(half_text(plan, nf, projection, width * across));
    }
    shapes
}

/// The `2x` mark relative to the notation: its digit is this much taller than
/// the capitals of the label (cap heights 14.6 against 11.2 units).
const HALF_MARK_SCALE: f64 = 1.3;
/// The white outline of the mark, in canvas units.
const HALO: f64 = 1.2;
/// Distance kept between any drawing and the canvas edge.
const EDGE: f64 = 2.0;

/// An upright `2x`, not in perspective, in the arrow color with a white
/// outline: right of a vertical arrow, above a horizontal one (right of it
/// when above does not fit).
fn half_text(plan: &Plan, nf: f64, projection: &Projection, width: f64) -> Vec<Shape> {
    let at = |p: [f64; 2]| projection.project(plane_point(plan.face, nf, p[0], p[1]));
    let (tail, tip) = (at(plan.path[0]), at(*plan.path.last().expect("a path")));
    let dir = sub(tip, tail);
    let vertical = dir[1].abs() > dir[0].abs();
    let point = [(tail[0] + tip[0]) / 2.0, (tail[1] + tip[1]) / 2.0];
    let scale = LABEL_SCALE * HALF_MARK_SCALE;
    let (w, h) = (font::measure("2x") * scale, font::GLYPH_HEIGHT * scale);
    let shaft = width * projection.scale * foreshortening(plan.face, nf, projection).max(0.7) / 2.0;
    let gap = 3.0;
    let right = [point[0] + shaft + gap, point[1] - h / 2.0];
    let above = [point[0] - w / 2.0, point[1] - shaft - gap - h];
    let (x, y) = if vertical || above[1] < EDGE + HALO {
        (right[0], right[1])
    } else {
        (above[0], above[1])
    };
    // Every drawing stays `EDGE` units inside the canvas, halo included.
    let x = x.clamp(EDGE + HALO, WIDTH - EDGE - HALO - w);
    let y = y.clamp(EDGE + HALO, HEIGHT - EDGE - HALO - h);
    let mut shapes = font::halo("2x", (x, y), h, HALO, WHITE, Part::Halo);
    shapes.extend(font::text("2x", (x, y), h, ARROW_VIOLET, Part::Mark));
    shapes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{locate, Facelets, PuzzleKind};

    fn act(text: &str) -> Action {
        text.parse().unwrap()
    }

    fn center(kind: PuzzleKind, index: usize) -> [f64; 3] {
        let (p, nrm) = locate(kind, index);
        [0, 1, 2].map(|i| f64::from(p[i] + nrm[i]))
    }

    fn bounds(shapes: &[Shape], part: Part) -> [f64; 4] {
        let mut b = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
        for p in shapes
            .iter()
            .filter(|s| s.part == part)
            .flat_map(|s| &s.points)
        {
            b = [
                b[0].min(p[0]),
                b[1].min(p[1]),
                b[2].max(p[0]),
                b[3].max(p[1]),
            ];
        }
        b
    }

    fn drawn(kind: PuzzleKind, text: &str) -> Vec<Shape> {
        let state = Facelets::solved(kind);
        let style = crate::render::Style::default();
        crate::render::icon(&state, &act(text), &style)
            .unwrap()
            .shapes
    }

    /// The `2x` is at least as tall as the capitals of the notation.
    #[test]
    fn the_mark_is_not_smaller_than_the_label() {
        for kind in [PuzzleKind::Cube3, PuzzleKind::Cube5, PuzzleKind::Cube7] {
            let shapes = drawn(kind, "R2");
            let (m, l) = (bounds(&shapes, Part::Mark), bounds(&shapes, Part::Label));
            assert!(m[3] - m[1] >= l[3] - l[1], "{kind:?}: {m:?} {l:?}");
        }
    }

    /// The mark, with its halo, stays 2 units inside the canvas and clear of the label.
    #[test]
    fn the_mark_is_never_clipped() {
        let moves = [
            "R2", "L2", "U2", "D2", "F2", "B2", "2R2", "3Rw2", "Rw2", "x2", "2U2",
        ];
        for kind in [
            PuzzleKind::Cube2,
            PuzzleKind::Cube3,
            PuzzleKind::Cube5,
            PuzzleKind::Cube7,
        ] {
            for text in moves {
                if !act(text).fits(kind) {
                    continue;
                }
                let shapes = drawn(kind, text);
                let all: Vec<Shape> = shapes
                    .iter()
                    .filter(|s| s.part == Part::Halo)
                    .cloned()
                    .collect();
                let m = bounds(&shapes, Part::Mark);
                let h = bounds(&all, Part::Halo);
                assert!(
                    m[0] >= EDGE + HALO - 0.01 && m[1] >= EDGE + HALO - 0.01,
                    "{kind:?} {text} {m:?}"
                );
                assert!(
                    m[2] <= WIDTH - EDGE - HALO + 0.01 && m[3] <= HEIGHT - EDGE - HALO + 0.01,
                    "{kind:?} {text} {m:?}"
                );
                assert!(h[0] >= 0.0 && h[2] <= WIDTH, "{kind:?} {text}");
            }
        }
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
                    let c = center(kind, i);
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
                        let [u, v] = plane_coords(plan.face, center(kind, i));
                        (u - point[0]).abs() <= 1.0 && (v - point[1]).abs() <= 1.0
                    })
                    .expect("a sticker under the arrow");
                let (position, _) = locate(kind, hit);
                assert!(action.moves(kind, position), "{text} at {t}");
            }
        }
    }
}
