//! Structural definitions: a pattern is a color for every sticker, computed
//! from the cube size and the sticker's place, never from a move list.

use crate::puzzle::{locate, Face, Facelets, PuzzleKind};

/// A relabelling of the six colors. `Cw` sends U to F, F to R and R to U
/// (and D to B, B to L, L to D); `Opp` sends a color to its opposite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Map {
    Id,
    Opp,
    Cw,
    Ccw,
    OppCw,
    OppCcw,
}

impl Map {
    pub(crate) fn parse(text: &str) -> Option<Map> {
        Some(match text {
            "id" => Map::Id,
            "opp" => Map::Opp,
            "cw" => Map::Cw,
            "ccw" => Map::Ccw,
            "oppcw" => Map::OppCw,
            "oppccw" => Map::OppCcw,
            _ => return None,
        })
    }

    fn apply(self, face: Face) -> Face {
        const CW: [usize; 6] = [2, 0, 1, 5, 3, 4];
        const CCW: [usize; 6] = [1, 2, 0, 4, 5, 3];
        let i = face.index();
        let idx = match self {
            Map::Id => i,
            Map::Opp => (i + 3) % 6,
            Map::Cw => CW[i],
            Map::Ccw => CCW[i],
            Map::OppCw => (CW[i] + 3) % 6,
            Map::OppCcw => (CCW[i] + 3) % 6,
        };
        Face::ALL[idx]
    }
}

/// How a pattern colors the stickers of one cube size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Def {
    /// A sticker keeps its face color on the cubies whose distances from
    /// the nearest outside face sum to an even number, and shows the
    /// opposite color on the others (corners always keep theirs).
    Parity,
    /// Every edge piece flipped in place, everything else solved.
    Superflip,
    /// The color map depends on the ring of the sticker on its face (ring 0
    /// is the border); the last map repeats for deeper rings. Faces outside
    /// `faces` keep their own color.
    Rings { maps: Vec<Map>, faces: Vec<Face> },
    /// Two opposite corners each carry a block of the face color: on the
    /// faces U, R and F the stickers whose in-plane cubie coordinates are
    /// both at least `e` (and on D, L and B both at most `n - 1 - e`) use
    /// `maps[e]`, the last map repeating. The rest of the face uses the
    /// shallower maps, so `cw id` is a cube in a cube.
    Blocks { maps: Vec<Map>, faces: Vec<Face> },
    /// The maps for the corner, edge (border, not corner) and inner stickers.
    Classes {
        corner: Map,
        edge: Map,
        inner: Map,
        faces: Vec<Face>,
    },
}

impl Def {
    /// The state on a cube of `kind`.
    pub(crate) fn state(&self, kind: PuzzleKind) -> Facelets {
        let stickers = (0..kind.facelet_count())
            .map(|index| self.color(kind, index))
            .collect();
        Facelets::from_stickers(stickers).expect("a definition keeps every color count")
    }

    fn color(&self, kind: PuzzleKind, index: usize) -> Face {
        let n = kind.size();
        let face = Face::ALL[index / kind.face_len()];
        let (row, col) = ((index % kind.face_len()) / n, index % n);
        let ring = row.min(col).min(n - 1 - row).min(n - 1 - col);
        let at_end = |v: usize| v == 0 || v == n - 1;
        let corner = at_end(row) && at_end(col);
        match self {
            Def::Parity => {
                let (pos, _) = locate(kind, index);
                let m = n as i32 - 1;
                // Folded coordinates, so that a cubie and its mirror image
                // across the cube agree and the corners stay solved.
                let sum: i32 = pos.iter().map(|p| (m - p.abs()) / 2).sum();
                if sum % 2 == 0 {
                    face
                } else {
                    Map::Opp.apply(face)
                }
            }
            Def::Superflip => {
                let (pos, normal) = locate(kind, index);
                let m = n as i32 - 1;
                let extreme: Vec<usize> = (0..3)
                    .filter(|&a| normal[a] == 0 && pos[a].abs() == m)
                    .collect();
                match extreme.as_slice() {
                    // Axes x, y, z have the positive faces R, U, F.
                    [axis] => {
                        let positive = [1, 0, 2][*axis];
                        Face::ALL[positive + if pos[*axis] > 0 { 0 } else { 3 }]
                    }
                    _ => face,
                }
            }
            Def::Rings { maps, faces } => {
                if !faces.contains(&face) {
                    return face;
                }
                maps[ring.min(maps.len() - 1)].apply(face)
            }
            Def::Blocks { maps, faces } => {
                if !faces.contains(&face) {
                    return face;
                }
                let (pos, normal) = locate(kind, index);
                let m = n as i32 - 1;
                let depth = (0..3)
                    .filter(|&a| normal[a] == 0)
                    .map(|a| {
                        let q = (pos[a] + m) / 2;
                        if face.index() < 3 {
                            q
                        } else {
                            m - q
                        }
                    })
                    .min()
                    .unwrap_or(0) as usize;
                maps[depth.min(maps.len() - 1)].apply(face)
            }
            Def::Classes {
                corner: c,
                edge,
                inner,
                faces,
            } => {
                if !faces.contains(&face) {
                    return face;
                }
                let map = if corner {
                    c
                } else if ring == 0 {
                    edge
                } else {
                    inner
                };
                map.apply(face)
            }
        }
    }
}
