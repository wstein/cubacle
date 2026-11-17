// SPDX-License-Identifier: AGPL-3.0-or-later
//! Fixed-center picture drawing using the existing color geometry.

use super::geometry::Camera;
use super::{Action, Icon, RenderError, Style};
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
        super::icon(&self.colors, action, style)
    }

    pub fn state_icon(&self, style: &Style) -> Icon {
        super::state_icon(&self.colors, style)
    }

    pub fn folded(&self, style: &Style) -> Icon {
        super::state_icon_folded(&self.colors, style)
    }

    pub fn net(&self, style: &Style) -> Icon {
        super::state_icon_net(&self.colors, style)
    }

    pub fn pair(&self, style: &Style) -> [Icon; 2] {
        let opposite = self
            .apply(&"y2".parse().expect("rotation"))
            .expect("rotation fits");
        let mut pair = [
            self.state_icon(style),
            super::state_icon_seen(&opposite.colors, style, Camera::Below),
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
