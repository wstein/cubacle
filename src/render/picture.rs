// SPDX-License-Identifier: AGPL-3.0-or-later
//! Fixed-center picture drawing using the existing color geometry.

use super::geometry::Camera;
use super::scene::{Part, Shape};
use super::{Action, Icon, RenderError, Rgb, Style};
use crate::puzzle::{locate, CenterTwists, Face, Facelets};

/// Renderer input for the six fixed-center Z4 orientations of an odd cube.
/// Colors and twists share a size. Other stickers have no orientation here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PictureState {
    colors: Facelets,
    centers: CenterTwists,
}

impl PictureState {
    /// Joins color facelets and fixed-center twists; mismatched sizes fail.
    pub fn new(colors: Facelets, centers: CenterTwists) -> Option<Self> {
        (colors.kind() == centers.kind()).then_some(Self { colors, centers })
    }

    pub fn colors(&self) -> &Facelets {
        &self.colors
    }

    pub fn centers(&self) -> &CenterTwists {
        &self.centers
    }

    /// The move and the picture state before it.
    pub fn icon(&self, action: &Action, style: &Style) -> Result<Icon, RenderError> {
        Ok(self.decorate(
            super::icon(&self.colors, action, style)?,
            &[Face::U, Face::R, Face::F],
            false,
        ))
    }

    pub fn state_icon(&self, style: &Style) -> Icon {
        self.decorate(
            super::state_icon(&self.colors, style),
            &[Face::U, Face::R, Face::F],
            false,
        )
    }

    pub fn folded(&self, style: &Style) -> Icon {
        self.decorate(
            super::state_icon_folded(&self.colors, style),
            &Face::ALL,
            false,
        )
    }

    pub fn net(&self, style: &Style) -> Icon {
        self.decorate(super::state_icon_net(&self.colors, style), &Face::ALL, true)
    }

    pub fn pair(&self, style: &Style) -> [Icon; 2] {
        let opposite = self
            .apply(&"y2".parse().expect("rotation"))
            .expect("rotation fits");
        let mut pair = [
            self.state_icon(style),
            opposite.decorate(
                super::state_icon_seen(&opposite.colors, style, Camera::Below),
                &[Face::R, Face::F, Face::D],
                false,
            ),
        ];
        super::pair::label(&mut pair);
        pair
    }

    /// Applies the same layer selection and rotation to colors and center
    /// tangents. A center tangent is transported in 3D before it is expressed
    /// in the destination face's clockwise quarter turns.
    pub fn apply(&self, action: &Action) -> Result<Self, RenderError> {
        let colors = action.apply(&self.colors)?;
        let kind = colors.kind();
        let mut turns = [0; 6];
        for face in Face::ALL {
            let index = face as usize * kind.face_len() + kind.face_len() / 2;
            let (position, normal) = locate(kind, index);
            let mut normal = normal;
            let mut tangent = tangent(face, self.centers.twist(face));
            if action.moves(kind, position) {
                let (axis, _) = action.axis_sign();
                let count = if action.is_half() {
                    2
                } else if action.sense() > 0 {
                    1
                } else {
                    3
                };
                for _ in 0..count {
                    normal = quarter(axis, normal);
                    tangent = quarter(axis, tangent);
                }
            }
            let destination = Face::ALL
                .into_iter()
                .find(|&f| {
                    locate(kind, f as usize * kind.face_len() + kind.face_len() / 2).1 == normal
                })
                .expect("a rotated normal is a face");
            turns[destination as usize] = (0..4)
                .find(|&r| self::tangent(destination, r) == tangent)
                .expect("a tangent has four orientations");
        }
        Ok(Self {
            colors,
            centers: CenterTwists::new(kind, turns).expect("same odd size, quarter turns"),
        })
    }

    fn decorate(&self, mut icon: Icon, faces: &[Face], down: bool) -> Icon {
        let n = self.colors.kind().size();
        // All three geometry builders emit stickers in URFDLB/row-major order.
        // Their rounded squares have 12 vertices; opposite vertices recover
        // the affine in-plane axes without losing perspective or face lighting.
        let stickers: Vec<_> = icon
            .shapes
            .iter()
            .filter(|s| s.part == Part::Sticker)
            .collect();
        let mut marks = Vec::new();
        for (at, &face) in faces.iter().enumerate() {
            let p = &stickers[at * n * n + n * n / 2].points;
            let c = [(p[0][0] + p[6][0]) / 2.0, (p[0][1] + p[6][1]) / 2.0];
            let right = [(p[0][0] - p[5][0]) / 2.0, (p[0][1] - p[5][1]) / 2.0];
            let sign = if down { -1.0 } else { 1.0 };
            let up = [
                sign * (p[2][0] - p[8][0]) / 2.0,
                sign * (p[2][1] - p[8][1]) / 2.0,
            ];
            let points = [
                [0.0, 0.65],
                [-0.32, 0.12],
                [-0.12, 0.12],
                [-0.12, -0.5],
                [0.12, -0.5],
                [0.12, 0.12],
                [0.32, 0.12],
            ]
            .map(|mut q| {
                for _ in 0..self.centers.twist(face) {
                    q = [q[1], -q[0]];
                }
                [
                    c[0] + q[0] * right[0] + q[1] * up[0],
                    c[1] + q[0] * right[1] + q[1] * up[1],
                ]
            });
            marks.push(Shape::new(
                Part::Mark,
                points.to_vec(),
                Rgb(0x18, 0x18, 0x18),
            ));
        }
        icon.shapes.extend(marks);
        let grid = &mut icon.grid;
        if let Some(net) = &mut grid.net {
            for (cells, face) in
                net.iter_mut()
                    .zip([Face::U, Face::L, Face::F, Face::R, Face::B, Face::D])
            {
                cells[n * n / 2].letter = glyph(self.centers.twist(face));
            }
        } else {
            for (cells, face) in grid.faces.iter_mut().zip([
                if grid.below { Face::D } else { Face::U },
                Face::F,
                Face::R,
            ]) {
                cells[n * n / 2].letter = glyph(self.centers.twist(face));
            }
        }
        icon
    }
}

fn glyph(turn: u8) -> char {
    ['↑', '→', '↓', '←'][turn as usize]
}

fn quarter(axis: usize, mut v: [i32; 3]) -> [i32; 3] {
    let (b, c) = ((axis + 1) % 3, (axis + 2) % 3);
    let old = v[b];
    v[b] = -v[c];
    v[c] = old;
    v
}

fn tangent(face: Face, turn: u8) -> [i32; 3] {
    let (up, right) = match face {
        Face::U => ([0, 0, -1], [1, 0, 0]),
        Face::R => ([0, 1, 0], [0, 0, -1]),
        Face::F => ([0, 1, 0], [1, 0, 0]),
        Face::D => ([0, 0, 1], [1, 0, 0]),
        Face::L => ([0, 1, 0], [0, 0, 1]),
        Face::B => ([0, 1, 0], [-1, 0, 0]),
    };
    match turn {
        0 => up,
        1 => right,
        2 => up.map(|v| -v),
        _ => right.map(|v| -v),
    }
}
