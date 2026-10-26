//! The oblique projection, the three visible face planes and the stickers.
//!
//! Model coordinates are those of `puzzle::locate`: `x` right, `y` up, `z`
//! towards the viewer, one cubie is 2 units wide and the cube spans `-n..n`.

use super::action::Action;
use super::scene::{Part, Shape};
use super::style::{face_light, Rgb, Shading, Style, BODY, GREY};
use crate::puzzle::{locate, Face, Facelets};

/// The color that marks a differing sticker (amber, as the quarter-turn frame).
pub(crate) const MARK: Rgb = Rgb(255, 176, 0);

/// How far one unit of depth shifts a point right and up on screen: a deep
/// oblique view that makes the Top and Right faces large.
pub(crate) const DEPTH: (f64, f64) = (0.5, 0.42);

pub(crate) struct Projection {
    pub scale: f64,
    ox: f64,
    oy: f64,
    dx: f64,
    dy: f64,
}

/// Where the camera is: above the cube, so that its Up face shows over the front
/// face, or below it, so that its Down face shows under the front face. Both see
/// the Front and the Right face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Camera {
    Above,
    Below,
}

impl Camera {
    /// The face of the top (above) or bottom (below) slot of the picture.
    pub(crate) fn cap(self) -> Face {
        match self {
            Camera::Above => Face::U,
            Camera::Below => Face::D,
        }
    }
}

impl Projection {
    /// Fits the cube of size `n` into the box `x, y, w, h`, centered.
    pub(crate) fn new(n: usize, x: f64, y: f64, w: f64, h: f64) -> Projection {
        Projection::seen_from(Camera::Above, n, x, y, w, h)
    }

    /// Like [`Projection::new`]; from below the depth runs down instead of up.
    pub(crate) fn seen_from(
        camera: Camera,
        n: usize,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    ) -> Projection {
        let (dx, dy) = DEPTH;
        let dy = if camera == Camera::Below { -dy } else { dy };
        let size = 2.0 * n as f64;
        Projection {
            scale: (w / (size * (1.0 + dx))).min(h / (size * (1.0 + dy.abs()))),
            ox: x + w / 2.0,
            oy: y + h / 2.0,
            dx,
            dy,
        }
    }

    pub(crate) fn project(&self, p: [f64; 3]) -> [f64; 2] {
        [
            self.ox + self.scale * (p[0] - self.dx * p[2]),
            self.oy - self.scale * (p[1] - self.dy * p[2]),
        ]
    }
}

/// Whether a face is one of the three drawn.
pub(crate) fn visible(camera: Camera, face: Face) -> bool {
    matches!(face, Face::R | Face::F) || face == camera.cap()
}

/// The model point of plane coordinates `(u, v)` on a visible face of a cube
/// of size `n`. `u` runs right and `v` up on the screen.
pub(crate) fn plane_point(face: Face, n: f64, u: f64, v: f64) -> [f64; 3] {
    match face {
        Face::F => [u, v, n],
        Face::R => [n, v, -u],
        Face::D => [u, -n, v],
        _ => [u, n, -v],
    }
}

/// The inverse of [`plane_point`] for the in-plane part of a point or vector.
pub(crate) fn plane_coords(face: Face, p: [f64; 3]) -> [f64; 2] {
    match face {
        Face::F => [p[0], p[1]],
        Face::R => [-p[2], p[1]],
        Face::D => [p[0], p[2]],
        _ => [p[0], -p[2]],
    }
}

/// A square of half-width `h` with rounded corners, counter-clockwise.
pub(crate) fn rounded_square(center: [f64; 2], h: f64, radius: f64) -> Vec<[f64; 2]> {
    let inner = h - radius;
    let mut points = Vec::with_capacity(12);
    for (corner, (sx, sy)) in [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)]
        .into_iter()
        .enumerate()
    {
        for step in 0..3 {
            let angle = (corner as f64 * 90.0 + f64::from(step) * 45.0).to_radians();
            points.push([
                center[0] + inner * sx + radius * angle.cos(),
                center[1] + inner * sy + radius * angle.sin(),
            ]);
        }
    }
    points
}

/// The dark body behind the stickers and the flat stickers.
pub(crate) fn cube_shapes(
    state: &Facelets,
    action: Option<&Action>,
    style: &Style,
    camera: Camera,
    projection: &Projection,
    pixels_per_model: f64,
) -> Vec<Shape> {
    let kind = state.kind();
    let n = kind.size() as f64;
    let flat = |face: Face, points: Vec<[f64; 2]>| -> Vec<[f64; 2]> {
        points
            .into_iter()
            .map(|[u, v]| projection.project(plane_point(face, n, u, v)))
            .collect()
    };
    let mut shapes = Vec::new();
    let rim = n + 0.1;
    for face in [Face::F, camera.cap(), Face::R] {
        let corners = vec![[-rim, -rim], [rim, -rim], [rim, rim], [-rim, rim]];
        shapes.push(Shape::new(Part::Body, flat(face, corners), BODY));
    }
    // A gap of at least 1.6 pixels, however small the icon.
    let gap = 0.2_f64.max(1.6 / pixels_per_model);
    let half = 1.0 - gap / 2.0;
    let radius = (0.3_f64).min(half * 0.6);
    for index in 0..kind.facelet_count() {
        let face = Face::ALL[index / kind.face_len()];
        if !visible(camera, face) {
            continue;
        }
        let (position, _) = locate(kind, index);
        let [u, v] = plane_coords(face, position.map(f64::from));
        let color = sticker_color(state, action, style, index, face, position);
        let square = rounded_square([u, v], half, radius);
        shapes.push(Shape::new(Part::Sticker, flat(face, square), color));
    }
    shapes
}

/// A small amber square in the middle of every visible sticker of `state`
/// whose color differs from the same place on `reference`.
pub(crate) fn difference_marks(
    state: &Facelets,
    reference: &Facelets,
    camera: Camera,
    projection: &Projection,
) -> Vec<Shape> {
    let kind = state.kind();
    let n = kind.size() as f64;
    let mut shapes = Vec::new();
    for index in 0..kind.facelet_count() {
        let face = Face::ALL[index / kind.face_len()];
        if !visible(camera, face) || state.stickers()[index] == reference.stickers()[index] {
            continue;
        }
        let (position, _) = locate(kind, index);
        let [u, v] = plane_coords(face, position.map(f64::from));
        let square = rounded_square([u, v], 0.32, 0.08)
            .into_iter()
            .map(|[a, b]| projection.project(plane_point(face, n, a, b)))
            .collect();
        shapes.push(Shape::new(Part::Mark, square, MARK));
    }
    shapes
}

/// With no action nothing is dimmed and nothing is grey: a plain picture of the state.
fn sticker_color(
    state: &Facelets,
    action: Option<&Action>,
    style: &Style,
    index: usize,
    face: Face,
    position: [i32; 3],
) -> Rgb {
    let base = style.palette.color(state.stickers()[index]);
    let moving = action.is_none_or(|a| a.moves(state.kind(), position));
    let whole = matches!(action, Some(Action::Rotate { .. }));
    let shown = match style.shading {
        Shading::GreyMoving if moving && !whole => GREY,
        _ => base,
    };
    let dim = if style.shading == Shading::DimRest && !moving {
        style.backdrop.dimming()
    } else {
        1.0
    };
    shown.scale(face_light(face) * dim)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::PuzzleKind;
    use crate::render::style::Palette;

    fn corners(shape: &Shape) -> &[[f64; 2]] {
        &shape.points
    }

    fn scene(kind: PuzzleKind, text: &str) -> (Vec<Shape>, Projection) {
        let state = Facelets::solved(kind);
        let projection = Projection::new(kind.size(), 10.0, 9.0, 80.0, 76.0);
        let action: Action = text.parse().unwrap();
        let ppm = projection.scale * 0.96;
        (
            cube_shapes(
                &state,
                Some(&action),
                &Style::default(),
                Camera::Above,
                &projection,
                ppm,
            ),
            projection,
        )
    }

    #[test]
    fn plane_maps_invert() {
        for face in [Face::F, Face::U, Face::R] {
            let p = plane_point(face, 5.0, 1.5, -2.5);
            assert_eq!(plane_coords(face, p), [1.5, -2.5]);
        }
    }

    #[test]
    fn counts_and_bounding_box_for_every_size() {
        for kind in [
            PuzzleKind::Cube2,
            PuzzleKind::Cube3,
            PuzzleKind::Cube4,
            PuzzleKind::Cube5,
            PuzzleKind::Cube6,
            PuzzleKind::Cube7,
        ] {
            let n = kind.size();
            let (shapes, projection) = scene(kind, "R");
            let count = |part| shapes.iter().filter(|s| s.part == part).count();
            assert_eq!(count(Part::Body), 3);
            assert_eq!(count(Part::Sticker), 3 * n * n, "{kind:?}");
            let all: Vec<[f64; 2]> = shapes
                .iter()
                .filter(|s| s.part == Part::Body)
                .flat_map(|s| s.points.clone())
                .collect();
            let min_x = all.iter().map(|p| p[0]).fold(f64::MAX, f64::min);
            let max_x = all.iter().map(|p| p[0]).fold(f64::MIN, f64::max);
            let half = (n as f64 + 0.1) * (1.0 + DEPTH.0) * projection.scale;
            assert!((min_x - (50.0 - half)).abs() < 0.05, "{kind:?} {min_x}");
            assert!((max_x - (50.0 + half)).abs() < 0.05);
        }
    }

    /// Separating-axis test: convex polygons overlap only if no edge normal separates them.
    fn overlap(a: &[[f64; 2]], b: &[[f64; 2]]) -> bool {
        for poly in [a, b] {
            for i in 0..poly.len() {
                let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
                let axis = [q[1] - p[1], p[0] - q[0]];
                let project = |s: &[[f64; 2]]| {
                    s.iter().fold((f64::MAX, f64::MIN), |(lo, hi), pt| {
                        let d = pt[0] * axis[0] + pt[1] * axis[1];
                        (lo.min(d), hi.max(d))
                    })
                };
                let ((alo, ahi), (blo, bhi)) = (project(a), project(b));
                let eps = 1e-9 * (axis[0].abs() + axis[1].abs());
                if ahi <= blo + eps || bhi <= alo + eps {
                    return false;
                }
            }
        }
        true
    }

    #[test]
    fn stickers_never_overlap() {
        for kind in [
            PuzzleKind::Cube2,
            PuzzleKind::Cube3,
            PuzzleKind::Cube4,
            PuzzleKind::Cube5,
        ] {
            let (shapes, _) = scene(kind, "U");
            let stickers: Vec<_> = shapes.iter().filter(|s| s.part == Part::Sticker).collect();
            for (i, a) in stickers.iter().enumerate() {
                for b in &stickers[i + 1..] {
                    assert!(!overlap(corners(a), corners(b)), "{kind:?}");
                }
            }
        }
    }

    #[test]
    fn moving_layer_is_bright_and_the_rest_dim() {
        let (shapes, _) = scene(PuzzleKind::Cube3, "R");
        let palette = Palette::default();
        let dimmed: Vec<Rgb> = [Face::U, Face::R, Face::F]
            .into_iter()
            .map(|f| {
                palette
                    .color(f)
                    .scale(face_light(f) * Style::default().backdrop.dimming())
            })
            .collect();
        let stickers = shapes.iter().filter(|s| s.part == Part::Sticker);
        // 27 visible stickers; R turns 9 + 3 + 3 of them, so 12 are dimmed.
        assert_eq!(stickers.filter(|s| dimmed.contains(&s.fill)).count(), 12);
    }
}
