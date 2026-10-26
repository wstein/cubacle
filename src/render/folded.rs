//! The folded view of a whole cube: the corner U, F, R in perspective as in
//! every state icon, and the three hidden faces unfolded flat around it.
//!
//! ```text
//!         U U U          B B B
//!        U U U           B B B
//!     L  F F F R         B B B
//!        F F F R
//!        D D D
//! ```
//!
//! L lies directly left of F, D directly below F (both as large as F), and B
//! right of the R parallelogram with its top aligned with the top of the
//! drawing. The flat faces are oriented as in the flat net, so every sticker
//! appears exactly once, the visible faces keep their 3D look and the others
//! are undistorted.

use super::geometry::{plane_coords, plane_point, rounded_square, DEPTH, MARK};
use super::net::{frame, row_col};
use super::scene::{Part, Shape, HEIGHT};
use super::style::{face_light, Style, BODY};
use super::text::{Cell, Grid};
use super::Icon;
use crate::puzzle::{locate, Face, Facelets};

/// Space between a flat face and the corner, in model units (a sticker is 2 wide).
const GAP: f64 = 0.3;
/// Canvas margin around the drawing, in canvas units; the frame lies inside it.
const MARGIN: f64 = 3.0;

pub(super) const HEADER: &str = "state, folded view: corner U F R, with L, D and B flat around it";

/// Layout of one cube size. Model coordinates: `x` right, `y` up, F is the
/// square `[-n(1+dx), n(1-dx)]` by `[-n(1+dy), n(1-dy)]`.
struct Folded {
    n: f64,
    scale: f64,
    width: f64,
    left: f64,
    top: f64,
}

impl Folded {
    fn new(size: usize) -> Folded {
        let n = size as f64;
        let (dx, dy) = DEPTH;
        let model_w = n * (6.0 + 2.0 * dx) + 2.0 * GAP;
        let model_h = 2.0 * n * (1.0 + dy) + GAP + 2.0 * n;
        let scale = (HEIGHT - 2.0 * MARGIN) / model_h;
        Folded {
            n,
            scale,
            width: model_w * scale + 2.0 * MARGIN,
            left: -n * (3.0 + dx) - GAP,
            top: n * (1.0 + dy),
        }
    }

    /// Canvas position of the model point `(x, y)`, `y` up.
    fn at(&self, x: f64, y: f64) -> [f64; 2] {
        [
            MARGIN + (x - self.left) * self.scale,
            MARGIN + (self.top - y) * self.scale,
        ]
    }

    /// The perspective image of a point of the cube.
    fn project(&self, p: [f64; 3]) -> [f64; 2] {
        let (dx, dy) = DEPTH;
        self.at(p[0] - dx * p[2], p[1] - dy * p[2])
    }

    /// Left edge and top of a flat face in model coordinates.
    fn origin(&self, face: Face) -> [f64; 2] {
        let (n, (dx, dy)) = (self.n, DEPTH);
        match face {
            Face::L => [-n * (3.0 + dx) - GAP, n * (1.0 - dy)],
            Face::D => [-n * (1.0 + dx), -n * (1.0 + dy) - GAP],
            _ => [n * (1.0 + dx) + GAP, n * (1.0 + dy)],
        }
    }

    fn corner(face: Face) -> bool {
        matches!(face, Face::U | Face::F | Face::R)
    }

    /// Model coordinates of the center of a sticker of a flat face.
    fn flat_center(&self, face: Face, position: [i32; 3]) -> [f64; 2] {
        let (r, c) = row_col(face, position, self.n as usize);
        let [x0, y0] = self.origin(face);
        [x0 + 2.0 * c as f64 + 1.0, y0 - 2.0 * r as f64 - 1.0]
    }

    /// A rounded square of half-width `half` on the sticker at `position`.
    fn square(&self, face: Face, position: [i32; 3], half: f64, radius: f64) -> Vec<[f64; 2]> {
        if Folded::corner(face) {
            let [u, v] = plane_coords(face, position.map(f64::from));
            rounded_square([u, v], half, radius)
                .into_iter()
                .map(|[a, b]| self.project(plane_point(face, self.n, a, b)))
                .collect()
        } else {
            rounded_square(self.flat_center(face, position), half, radius)
                .into_iter()
                .map(|[x, y]| self.at(x, y))
                .collect()
        }
    }
}

fn shapes(state: &Facelets, reference: Option<&Facelets>, style: &Style) -> (Vec<Shape>, f64) {
    let kind = state.kind();
    let layout = Folded::new(kind.size());
    let n = layout.n;
    let mut out = Vec::new();
    let rim = n + 0.1;
    for face in [Face::F, Face::U, Face::R] {
        let plate = [[-rim, -rim], [rim, -rim], [rim, rim], [-rim, rim]]
            .iter()
            .map(|c| layout.project(plane_point(face, n, c[0], c[1])))
            .collect();
        out.push(Shape::new(Part::Body, plate, BODY));
    }
    for face in [Face::L, Face::D, Face::B] {
        let [x, y] = layout.origin(face);
        let (lo, hi) = (-0.05, 2.0 * n + 0.05);
        let plate = [[lo, -lo], [hi, -lo], [hi, -hi], [lo, -hi]]
            .iter()
            .map(|c| layout.at(x + c[0], y + c[1]))
            .collect();
        out.push(Shape::new(Part::Body, plate, BODY));
    }
    // A gap of at least 1.6 pixels, however small the icon.
    let pixels_per_model = layout.scale * f64::from(style.size.max(16)) / HEIGHT;
    let gap = 0.2_f64.max(1.6 / pixels_per_model);
    let half = 1.0 - gap / 2.0;
    let radius = 0.3_f64.min(half * 0.6);
    let mut marks = Vec::new();
    for index in 0..kind.facelet_count() {
        let face = Face::ALL[index / kind.face_len()];
        let (position, _) = locate(kind, index);
        let base = style.palette.color(state.stickers()[index]);
        let color = if Folded::corner(face) {
            base.scale(face_light(face))
        } else {
            base
        };
        out.push(Shape::new(
            Part::Sticker,
            layout.square(face, position, half, radius),
            color,
        ));
        if reference.is_some_and(|r| r.stickers()[index] != state.stickers()[index]) {
            marks.push(Shape::new(
                Part::Mark,
                layout.square(face, position, 0.32, 0.08),
                MARK,
            ));
        }
    }
    out.extend(marks);
    (out, layout.width)
}

fn grid(state: &Facelets, style: &Style) -> Grid {
    let kind = state.kind();
    let faces = [Face::U, Face::L, Face::F, Face::R, Face::B, Face::D].map(|face| {
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
        folded: true,
    }
}

/// The folded view of `state` (see the module text). [`Icon::label`] is `"state"`.
pub fn state_icon_folded(state: &Facelets, style: &Style) -> Icon {
    build(state, None, style)
}

/// [`state_icon_folded`] with an amber square on every sticker that differs
/// from `reference`, on the perspective faces and on the flat ones. Every
/// sticker is shown, so none is missed.
pub fn state_icon_folded_against(state: &Facelets, reference: &Facelets, style: &Style) -> Icon {
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

    fn parts(icon: &Icon, part: Part) -> Vec<&Shape> {
        icon.shapes.iter().filter(|s| s.part == part).collect()
    }

    fn near(a: [f64; 2], b: [f64; 2]) -> bool {
        (a[0] - b[0]).hypot(a[1] - b[1]) < 1e-6
    }

    fn range(kind: PuzzleKind, f: Face) -> std::ops::Range<usize> {
        f as usize * kind.face_len()..(f as usize + 1) * kind.face_len()
    }

    #[test]
    fn every_sticker_is_drawn_once_in_its_color_and_place() {
        let style = Style::default();
        for kind in SIZES {
            let cube = scrambled(kind);
            let icon = state_icon_folded(&cube, &style);
            let shapes = parts(&icon, Part::Sticker);
            let n = kind.size();
            assert_eq!(shapes.len(), 6 * n * n, "{kind:?}");
            assert_eq!(parts(&icon, Part::Body).len(), 6);
            let layout = Folded::new(n);
            let mut seen: Vec<[f64; 2]> = Vec::new();
            for (i, shape) in shapes.iter().enumerate() {
                let face = Face::ALL[i / kind.face_len()];
                let light = if Folded::corner(face) {
                    face_light(face)
                } else {
                    1.0
                };
                let want = style.palette.color(cube.stickers()[i]).scale(light);
                assert_eq!(shape.fill, want, "{kind:?} {i}");
                let (position, _) = locate(kind, i);
                let got = center(&shape.points);
                if !Folded::corner(face) {
                    let [x, y] = layout.flat_center(face, position);
                    assert!(near(got, layout.at(x, y)), "{kind:?} flat sticker {i}");
                }
                assert!(seen.iter().all(|s| !near(*s, got)), "{kind:?} {i} twice");
                assert!(got[0] > 0.0 && got[0] < icon.canvas.0 && got[1] < HEIGHT);
                seen.push(got);
            }
        }
    }

    /// The flat faces are as large as F and placed around it.
    #[test]
    fn flat_faces_are_as_large_as_f_and_placed_around_it() {
        let style = Style::default();
        for kind in SIZES {
            let icon = state_icon_folded(&Facelets::solved(kind), &style);
            let bodies = parts(&icon, Part::Body);
            let layout = Folded::new(kind.size());
            let span = |s: &Shape, axis: usize| {
                let v: Vec<f64> = s.points.iter().map(|p| p[axis]).collect();
                (
                    v.iter().cloned().fold(f64::MAX, f64::min),
                    v.iter().cloned().fold(f64::MIN, f64::max),
                )
            };
            // bodies: F, U, R, L, D, B
            let (f, l, d, b) = (bodies[0], bodies[3], bodies[4], bodies[5]);
            let (fx, fy) = (span(f, 0), span(f, 1));
            let side = fx.1 - fx.0;
            for flat in [l, d, b] {
                let (x, y) = (span(flat, 0), span(flat, 1));
                let tol = 0.2 * layout.scale;
                assert!((x.1 - x.0 - side).abs() < tol && (y.1 - y.0 - side).abs() < tol);
            }
            let gap = GAP * layout.scale;
            assert!((span(l, 0).1 + gap - fx.0).abs() < 0.2 * layout.scale);
            assert!((span(l, 1).0 - fy.0).abs() < 0.2 * layout.scale);
            assert!((span(d, 1).0 - fy.1 - gap).abs() < 0.2 * layout.scale + 1e-6);
            // B is right of everything else and its top is the top of the drawing
            let top = parts(&icon, Part::Body)
                .iter()
                .map(|s| span(s, 1).0)
                .fold(f64::MAX, f64::min);
            assert!((span(b, 1).0 - top).abs() < 0.2 * layout.scale, "{kind:?}");
            assert!(span(b, 0).0 > span(bodies[2], 0).1);
        }
    }

    /// Stickers of one cubie on faces that touch lie next to each other.
    #[test]
    fn flat_faces_meet_the_corner_across_the_gaps() {
        let style = Style::default();
        for kind in SIZES {
            let n = kind.size();
            let layout = Folded::new(n);
            let icon = state_icon_folded(&Facelets::solved(kind), &style);
            let shapes = parts(&icon, Part::Sticker);
            let at = |i: usize| center(&shapes[i].points);
            let flat = (2.0 + GAP) * layout.scale;
            for (a, b) in [(Face::F, Face::L), (Face::F, Face::D)] {
                let mut pairs = 0;
                for i in range(kind, a) {
                    for j in range(kind, b) {
                        if locate(kind, i).0 == locate(kind, j).0 {
                            let (p, q) = (at(i), at(j));
                            let d = (p[0] - q[0]).hypot(p[1] - q[1]);
                            assert!((d - flat).abs() < 1e-6, "{kind:?} {a:?}-{b:?}");
                            pairs += 1;
                        }
                    }
                }
                assert_eq!(pairs, n, "{kind:?} {a:?}-{b:?}");
            }
            // R's far column and B's first column share a cubie on the same row
            let mut pairs = 0;
            for i in range(kind, Face::R) {
                for j in range(kind, Face::B) {
                    if locate(kind, i).0 == locate(kind, j).0 {
                        let (p, q) = (at(i), at(j));
                        assert!(q[0] > p[0], "{kind:?} B is right of R");
                        // same row of the cube: the heights differ only by the lean
                        let lean = 2.0 * n as f64 * DEPTH.1 * layout.scale;
                        assert!(p[1] - q[1] <= lean && q[1] - p[1] <= lean);
                        pairs += 1;
                    }
                }
            }
            assert_eq!(pairs, n, "{kind:?} R-B");
        }
    }

    #[test]
    fn every_differing_sticker_is_marked_exactly_once() {
        let style = Style::default();
        for kind in SIZES {
            let solved = Facelets::solved(kind);
            let cube = scrambled(kind);
            let icon = state_icon_folded_against(&cube, &solved, &style);
            let plain = state_icon_folded(&cube, &style);
            let marks: Vec<[f64; 2]> = parts(&icon, Part::Mark)
                .iter()
                .map(|s| center(&s.points))
                .collect();
            let differ = crate::render::differing_stickers(&cube, &solved);
            assert_eq!(marks.len(), differ);
            for (i, shape) in parts(&plain, Part::Sticker).iter().enumerate() {
                let on = marks.iter().filter(|m| near(**m, center(&shape.points)));
                let differs = cube.stickers()[i] != solved.stickers()[i];
                assert_eq!(on.count(), usize::from(differs), "{kind:?} sticker {i}");
            }
            let same = state_icon_folded_against(&cube, &cube, &style);
            assert!(parts(&same, Part::Mark).is_empty());
        }
    }

    #[test]
    fn the_text_picture_of_a_solved_3x3_is_the_folded_view() {
        let cube = Facelets::solved(PuzzleKind::Cube3);
        let text = state_icon_folded(&cube, &Style::default()).to_text(false);
        let lines = [
            "          W W W     B B B",
            "         W W W      B B B",
            "        W W W       B B B",
            "O O O  G G G R R R",
            "O O O  G G G R R R",
            "O O O  G G G R R R",
            "       Y Y Y",
            "       Y Y Y",
            "       Y Y Y",
        ];
        let want = format!("{HEADER}\n{}", lines.map(|l| format!("{l}\n")).concat());
        assert_eq!(text, want);
    }

    #[test]
    fn the_text_picture_has_six_n_squared_letters_for_every_size() {
        for kind in SIZES {
            let text = state_icon_folded(&scrambled(kind), &Style::default()).to_text(false);
            let letters = text.lines().skip(1).flat_map(str::split_whitespace).count();
            assert_eq!(letters, 6 * kind.size() * kind.size(), "{kind:?}");
            assert_eq!(text.lines().count(), 1 + 3 * kind.size());
        }
    }
}
