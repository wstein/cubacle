//! The flat net of a whole cube: all six faces equally large and undistorted,
//! each seen from outside with U up, in the classic cross
//!
//! ```text
//!       U
//!    L  F  R  B
//!       D
//! ```
//!
//! U lies above F and D below F; L, F, R and B form one row. Every sticker is
//! drawn exactly once. There is no projection, so patterns (rings, point
//! symmetry) read directly.

use super::geometry::{rounded_square, MARK};
use super::scene::{Part, Shape, HEIGHT};
use super::style::{Style, BODY};
use super::text::{Cell, Grid};
use super::Icon;
use crate::puzzle::{locate, Face, Facelets};

/// Space between two faces, in model units (a sticker is 2 wide).
const FACE_GAP: f64 = 0.6;
/// Canvas margin around the net, in canvas units; the frame lies inside it.
const MARGIN: f64 = 3.0;
/// Width of the frame line and its opacity, as for the other state icons.
const LINE: f64 = 1.6;
const OPACITY: f64 = 0.7;

pub(super) const HEADER: &str = "state, flat view: U above F, L F R B in a row, D below";

/// The cross: face, column and row in the grid of faces.
pub(super) const LAYOUT: [(Face, usize, usize); 6] = [
    (Face::U, 1, 0),
    (Face::L, 0, 1),
    (Face::F, 1, 1),
    (Face::R, 2, 1),
    (Face::B, 3, 1),
    (Face::D, 1, 2),
];

/// The model vectors of the screen directions "right" and "up" when a face is
/// looked at from outside. U is seen with F at its bottom edge and D with F at
/// its top edge, as in the net.
fn axes(face: Face) -> ([i32; 3], [i32; 3]) {
    match face {
        Face::F => ([1, 0, 0], [0, 1, 0]),
        Face::R => ([0, 0, -1], [0, 1, 0]),
        Face::B => ([-1, 0, 0], [0, 1, 0]),
        Face::L => ([0, 0, 1], [0, 1, 0]),
        Face::U => ([1, 0, 0], [0, 0, -1]),
        Face::D => ([1, 0, 0], [0, 0, 1]),
    }
}

fn dot(a: [i32; 3], b: [i32; 3]) -> i32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The row (0 at the top) and column (0 at the left) of a sticker inside its
/// face, as drawn.
pub(super) fn row_col(face: Face, position: [i32; 3], n: usize) -> (usize, usize) {
    let (right, up) = axes(face);
    let m = n as i32 - 1;
    (
        ((m - dot(position, up)) / 2) as usize,
        ((m + dot(position, right)) / 2) as usize,
    )
}

/// Layout of one cube size, in model units of 2 per sticker.
struct Net {
    n: usize,
    scale: f64,
    width: f64,
}

impl Net {
    fn new(n: usize) -> Net {
        let model_h = 6.0 * n as f64 + 2.0 * FACE_GAP;
        let scale = (HEIGHT - 2.0 * MARGIN) / model_h;
        let model_w = 8.0 * n as f64 + 3.0 * FACE_GAP;
        Net {
            n,
            scale,
            width: model_w * scale + 2.0 * MARGIN,
        }
    }

    fn pitch(&self) -> f64 {
        2.0 * self.n as f64 + FACE_GAP
    }

    /// Canvas position of the model point `(x, y)` measured from the top left
    /// of the net.
    fn at(&self, x: f64, y: f64) -> [f64; 2] {
        [MARGIN + x * self.scale, MARGIN + y * self.scale]
    }

    /// Model coordinates of the center of a sticker of `face`.
    fn center(&self, face: Face, position: [i32; 3]) -> [f64; 2] {
        let (_, col, row) = LAYOUT.iter().copied().find(|l| l.0 == face).unwrap();
        let (r, c) = row_col(face, position, self.n);
        [
            col as f64 * self.pitch() + 2.0 * c as f64 + 1.0,
            row as f64 * self.pitch() + 2.0 * r as f64 + 1.0,
        ]
    }

    fn square(&self, center: [f64; 2], half: f64, radius: f64) -> Vec<[f64; 2]> {
        rounded_square(center, half, radius)
            .into_iter()
            .map(|[x, y]| self.at(x, y))
            .collect()
    }
}

fn shapes(state: &Facelets, reference: Option<&Facelets>, style: &Style) -> (Vec<Shape>, f64) {
    let kind = state.kind();
    let net = Net::new(kind.size());
    let side = 2.0 * kind.size() as f64;
    let mut out = Vec::new();
    for (_, col, row) in LAYOUT {
        let (x, y) = (col as f64 * net.pitch(), row as f64 * net.pitch());
        let rim = 0.05;
        let corners = [
            [x - rim, y - rim],
            [x + side + rim, y - rim],
            [x + side + rim, y + side + rim],
            [x - rim, y + side + rim],
        ];
        let points = corners.iter().map(|c| net.at(c[0], c[1])).collect();
        out.push(Shape::new(Part::Body, points, BODY));
    }
    // A gap of at least 1.6 pixels, however small the icon.
    let pixels_per_model = net.scale * f64::from(style.size.max(16)) / HEIGHT;
    let gap = 0.2_f64.max(1.6 / pixels_per_model);
    let half = 1.0 - gap / 2.0;
    let radius = 0.3_f64.min(half * 0.6);
    let mut marks = Vec::new();
    for index in 0..kind.facelet_count() {
        let face = Face::ALL[index / kind.face_len()];
        let (position, _) = locate(kind, index);
        let center = net.center(face, position);
        let color = style.palette.color(state.stickers()[index]);
        let square = net.square(center, half, radius);
        out.push(Shape::new(Part::Sticker, square, color));
        if reference.is_some_and(|r| r.stickers()[index] != state.stickers()[index]) {
            marks.push(Shape::new(Part::Mark, net.square(center, 0.32, 0.08), MARK));
        }
    }
    out.extend(marks);
    (out, net.width)
}

pub(super) fn frame(width: f64, style: &Style) -> Vec<Shape> {
    let ink = style.backdrop.ink();
    let strip = |x0: f64, y0: f64, x1: f64, y1: f64| {
        let points = vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]];
        let mut shape = Shape::new(Part::Frame, points, ink);
        shape.opacity = OPACITY;
        shape
    };
    vec![
        strip(0.0, 0.0, width, LINE),
        strip(0.0, HEIGHT - LINE, width, HEIGHT),
        strip(0.0, LINE, LINE, HEIGHT - LINE),
        strip(width - LINE, LINE, width, HEIGHT - LINE),
    ]
}

fn grid(state: &Facelets, style: &Style) -> Grid {
    let kind = state.kind();
    let faces = LAYOUT.map(|(face, _, _)| {
        let start = face as usize * kind.face_len();
        (start..start + kind.face_len())
            .map(|index| {
                let sticker = state.stickers()[index];
                Cell {
                    letter: style.palette.letter(sticker),
                    moving: true,
                    color: style.palette.color(sticker),
                }
            })
            .collect()
    });
    Grid {
        n: kind.size(),
        header: HEADER.to_owned(),
        below: false,
        faces: Default::default(),
        net: Some(faces.to_vec()),
        folded: false,
    }
}

/// The flat net of `state` (see the module text). [`Icon::label`] is `"state"`.
pub fn state_icon_net(state: &Facelets, style: &Style) -> Icon {
    build(state, None, style)
}

/// [`state_icon_net`] with an amber square on every sticker that differs from
/// `reference`. The net shows every sticker, so none is missed.
pub fn state_icon_net_against(state: &Facelets, reference: &Facelets, style: &Style) -> Icon {
    build(state, Some(reference), style)
}

fn build(state: &Facelets, reference: Option<&Facelets>, style: &Style) -> Icon {
    let (mut drawn, width) = shapes(state, reference, style);
    drawn.extend(frame(width, style));
    Icon {
        grid: grid(state, style),
        direction: None,
        shapes: drawn,
        label: "state".to_owned(),
        size: style.size,
        background: style.background,
        canvas: (width, HEIGHT),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{MoveSequence, PuzzleKind};

    const SIZES: [PuzzleKind; 6] = [
        PuzzleKind::Cube2,
        PuzzleKind::Cube3,
        PuzzleKind::Cube4,
        PuzzleKind::Cube5,
        PuzzleKind::Cube6,
        PuzzleKind::Cube7,
    ];

    fn scrambled(kind: PuzzleKind) -> Facelets {
        let moves: MoveSequence = match kind.size() {
            2 => "R U2 F' R U' F2",
            3 => "R U2 F' D L2 B' R' U F2",
            _ => "R U2 F' D L2 B' 2R 2U' 2F Rw Uw' 3Fw",
        }
        .parse()
        .unwrap();
        Facelets::solved(kind).apply(&moves).unwrap()
    }

    fn center(points: &[[f64; 2]]) -> [f64; 2] {
        let n = points.len() as f64;
        [
            points.iter().map(|p| p[0]).sum::<f64>() / n,
            points.iter().map(|p| p[1]).sum::<f64>() / n,
        ]
    }

    fn stickers(icon: &Icon, part: Part) -> Vec<&Shape> {
        icon.shapes.iter().filter(|s| s.part == part).collect()
    }

    fn near(a: [f64; 2], b: [f64; 2]) -> bool {
        (a[0] - b[0]).hypot(a[1] - b[1]) < 1e-6
    }

    /// The drawn stickers come in facelet order, so sticker `i` is shape `i`.
    #[test]
    fn every_sticker_is_drawn_once_in_its_color_and_place() {
        let style = Style::default();
        for kind in SIZES {
            let cube = scrambled(kind);
            let icon = state_icon_net(&cube, &style);
            let shapes = stickers(&icon, Part::Sticker);
            let n = kind.size();
            assert_eq!(shapes.len(), 6 * n * n, "{kind:?}");
            let net = Net::new(n);
            let mut seen: Vec<[f64; 2]> = Vec::new();
            for (i, shape) in shapes.iter().enumerate() {
                let face = Face::ALL[i / kind.face_len()];
                assert_eq!(shape.fill, style.palette.color(cube.stickers()[i]));
                let (position, _) = locate(kind, i);
                let [x, y] = net.center(face, position);
                let got = center(&shape.points);
                assert!(near(got, net.at(x, y)), "{kind:?} sticker {i}");
                assert!(seen.iter().all(|s| !near(*s, got)), "{kind:?} {i} twice");
                seen.push(got);
            }
            assert!(seen.iter().all(|p| p[0] < net.width && p[1] < HEIGHT));
            // a solved cube: every face is one color
            let solved = state_icon_net(&Facelets::solved(kind), &style);
            for (i, shape) in stickers(&solved, Part::Sticker).iter().enumerate() {
                let face = Face::ALL[i / kind.face_len()];
                assert_eq!(shape.fill, style.palette.color(face));
            }
        }
    }

    /// Two stickers of one cubie on faces that touch in the net must sit
    /// across the gap from each other: that fixes the orientation of both faces.
    #[test]
    fn faces_are_oriented_so_that_shared_edges_meet() {
        let style = Style::default();
        for kind in SIZES {
            let n = kind.size();
            let net = Net::new(n);
            let icon = state_icon_net(&Facelets::solved(kind), &style);
            let shapes = stickers(&icon, Part::Sticker);
            let at = |i: usize| center(&shapes[i].points);
            let range = |f: Face| f as usize * kind.face_len()..(f as usize + 1) * kind.face_len();
            let neighbors = [
                (Face::U, Face::F),
                (Face::F, Face::D),
                (Face::L, Face::F),
                (Face::F, Face::R),
                (Face::R, Face::B),
            ];
            let apart = (2.0 + FACE_GAP) * net.scale;
            for (a, b) in neighbors {
                let mut pairs = 0;
                for i in range(a) {
                    for j in range(b) {
                        if locate(kind, i).0 == locate(kind, j).0 {
                            let (p, q) = (at(i), at(j));
                            let d = (p[0] - q[0]).hypot(p[1] - q[1]);
                            assert!((d - apart).abs() < 1e-6, "{kind:?} {a:?}-{b:?}");
                            pairs += 1;
                        }
                    }
                }
                // one sticker of each face per cubie along the shared edge
                assert_eq!(pairs, n, "{kind:?} {a:?}-{b:?}");
            }
            // the U edge of every side face is its top row
            for face in [Face::L, Face::F, Face::R, Face::B] {
                for i in range(face) {
                    let (p, _) = locate(kind, i);
                    let (row, _) = row_col(face, p, n);
                    assert_eq!(row == 0, p[1] == n as i32 - 1, "{kind:?} {face:?}");
                }
            }
        }
    }

    #[test]
    fn every_differing_sticker_is_marked_exactly_once() {
        let style = Style::default();
        for kind in SIZES {
            let solved = Facelets::solved(kind);
            let cube = scrambled(kind);
            let icon = state_icon_net_against(&cube, &solved, &style);
            let plain = state_icon_net(&cube, &style);
            let marks: Vec<[f64; 2]> = stickers(&icon, Part::Mark)
                .iter()
                .map(|s| center(&s.points))
                .collect();
            let differ = crate::render::differing_stickers(&cube, &solved);
            assert_eq!(marks.len(), differ);
            for (i, shape) in stickers(&plain, Part::Sticker).iter().enumerate() {
                let on = marks.iter().filter(|m| near(**m, center(&shape.points)));
                let differs = cube.stickers()[i] != solved.stickers()[i];
                assert_eq!(on.count(), usize::from(differs), "{kind:?} sticker {i}");
            }
            let same = state_icon_net_against(&cube, &cube, &style);
            assert!(stickers(&same, Part::Mark).is_empty());
        }
    }

    #[test]
    fn the_text_picture_of_a_solved_3x3_is_the_cross() {
        let cube = Facelets::solved(PuzzleKind::Cube3);
        let text = state_icon_net(&cube, &Style::default()).to_text(false);
        let top = "       W W W\n";
        let middle = "O O O  G G G  R R R  B B B\n";
        let bottom = "       Y Y Y\n";
        let want = format!(
            "{HEADER}\n{}{}{}",
            top.repeat(3),
            middle.repeat(3),
            bottom.repeat(3)
        );
        assert_eq!(text, want);
    }

    #[test]
    fn the_text_picture_has_six_n_squared_letters_for_every_size() {
        for kind in SIZES {
            let text = state_icon_net(&scrambled(kind), &Style::default()).to_text(false);
            let letters = text.lines().skip(1).flat_map(str::split_whitespace).count();
            assert_eq!(letters, 6 * kind.size() * kind.size(), "{kind:?}");
            assert_eq!(text.lines().count(), 1 + 3 * kind.size());
        }
    }
}
