//! What an icon shows: a layer move or a whole-cube rotation, and the
//! rotation maths shared by the layer selection and the arrows.

use std::fmt;
use std::str::FromStr;

use crate::puzzle::{
    Face, Facelets, LayerMove, Move, MoveSequence, NotationError, PuzzleKind, Turn,
};

/// The axis of a whole-cube rotation: `x` turns like `R`, `y` like `U`, `z` like `F`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// A layer move (`R`, `2R`, `Rw`, `3Rw2`), an inner block of layers (`2-3Rw`) or
/// a whole-cube rotation (`x`, `y'`, `z2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Layer(Move),
    /// Layers `from..=to` turned together, neither one layer nor an outer
    /// block; build it with [`Action::block`].
    Block(LayerMove),
    Rotate {
        axis: Axis,
        turn: Turn,
    },
}

/// Why an icon could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderError {
    pub action: Action,
    pub kind: PuzzleKind,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = self.kind.size();
        write!(f, "move {} does not exist on a {n}x{n}x{n}", self.action)
    }
}

impl std::error::Error for RenderError {}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Layer(mv) => write!(f, "{mv}"),
            Action::Block(mv) => write!(f, "{mv}"),
            Action::Rotate { axis, turn } => {
                let letter = match axis {
                    Axis::X => 'x',
                    Axis::Y => 'y',
                    Axis::Z => 'z',
                };
                let suffix = match turn {
                    Turn::Clockwise => "",
                    Turn::Half => "2",
                    Turn::CounterClockwise => "'",
                };
                write!(f, "{letter}{suffix}")
            }
        }
    }
}

impl FromStr for Action {
    type Err = NotationError;

    fn from_str(token: &str) -> Result<Action, NotationError> {
        let mut chars = token.chars();
        let axis = match chars.next() {
            Some('x') => Some(Axis::X),
            Some('y') => Some(Axis::Y),
            Some('z') => Some(Axis::Z),
            _ => None,
        };
        let Some(axis) = axis else {
            return token.parse().map(Action::Layer);
        };
        // The same suffixes as layer moves, but icons keep their stricter
        // spelling of rotations: only "", "2" and "'" (no x3, x2').
        let suffix = chars.as_str();
        let turn = Turn::from_suffix(suffix)
            .filter(|_| matches!(suffix, "" | "2" | "'"))
            .ok_or_else(|| NotationError {
                token: token.to_owned(),
            })?;
        Ok(Action::Rotate { axis, turn })
    }
}

/// The axis index (0 x, 1 y, 2 z) and outward sign of a face.
fn face_axis(face: Face) -> (usize, i32) {
    match face {
        Face::U => (1, 1),
        Face::D => (1, -1),
        Face::R => (0, 1),
        Face::L => (0, -1),
        Face::F => (2, 1),
        Face::B => (2, -1),
    }
}

fn dir(turn: Turn) -> i32 {
    match turn {
        Turn::Clockwise | Turn::Half => 1,
        Turn::CounterClockwise => -1,
    }
}

impl Action {
    /// The action of a compressed move: a plain layer move when one exists,
    /// else the block.
    pub fn block(mv: LayerMove) -> Action {
        match mv.to_move() {
            Some(plain) => Action::Layer(plain),
            None => Action::Block(mv),
        }
    }

    pub(crate) fn fits(&self, kind: PuzzleKind) -> bool {
        match self {
            Action::Layer(mv) => mv.fits(kind),
            Action::Block(mv) => mv.fits(kind),
            Action::Rotate { .. } => true,
        }
    }

    pub(crate) fn is_half(&self) -> bool {
        matches!(
            self,
            Action::Layer(Move {
                turn: Turn::Half,
                ..
            }) | Action::Block(LayerMove {
                turn: Turn::Half,
                ..
            }) | Action::Rotate {
                turn: Turn::Half,
                ..
            }
        )
    }

    /// The axis index and the outward sign of the turning face.
    pub(crate) fn axis_sign(&self) -> (usize, i32) {
        match self {
            Action::Layer(mv) => face_axis(mv.face),
            Action::Block(mv) => face_axis(mv.face),
            Action::Rotate { axis, .. } => (*axis as usize, 1),
        }
    }

    /// The sense of the turn about the positive axis: `+1` is counter-clockwise
    /// seen from the positive side (the right-hand rule), as in `Facelets::apply`.
    /// A clockwise turn of a face is a negative turn about its outward axis.
    pub(crate) fn sense(&self) -> i32 {
        let (_, sign) = self.axis_sign();
        let turn = match self {
            Action::Layer(mv) => mv.turn,
            Action::Block(mv) => mv.turn,
            Action::Rotate { turn, .. } => *turn,
        };
        -sign * dir(turn)
    }

    /// Whether the cubie at a doubled-grid `position` is turned.
    pub(crate) fn moves(&self, kind: PuzzleKind, position: [i32; 3]) -> bool {
        if let Action::Block(mv) = self {
            let (axis, sign) = face_axis(mv.face);
            let n = kind.size() as i32;
            let height = sign * position[axis];
            let outer = n - 1 - 2 * (i32::from(mv.from) - 1);
            let inner = n - 1 - 2 * (i32::from(mv.to) - 1);
            return (inner..=outer).contains(&height);
        }
        let Action::Layer(mv) = self else {
            return true;
        };
        let (axis, sign) = face_axis(mv.face);
        let edge = kind.size() as i32 - 1 - 2 * (i32::from(mv.depth) - 1);
        let height = sign * position[axis];
        if mv.wide {
            height >= edge
        } else {
            height == edge
        }
    }

    /// The signed position, along the axis, of the middle of the turning layers.
    pub(crate) fn band_center(&self, n: usize) -> f64 {
        if let Action::Block(mv) = self {
            let (_, sign) = face_axis(mv.face);
            let spread = i32::from(mv.from) + i32::from(mv.to) - 2;
            return f64::from(sign) * (n as f64 - 1.0 - f64::from(spread));
        }
        let Action::Layer(mv) = self else {
            return 0.0;
        };
        let (_, sign) = face_axis(mv.face);
        let first = if mv.wide { 1 } else { i32::from(mv.depth) };
        let last = i32::from(mv.depth);
        f64::from(sign) * (n as f64 - 1.0 - f64::from(first + last - 2))
    }

    /// Performs the action on a state; a rotation is the equivalent layer moves.
    pub fn apply(&self, state: &Facelets) -> Result<Facelets, RenderError> {
        let kind = state.kind();
        let error = RenderError {
            action: *self,
            kind,
        };
        if !self.fits(kind) {
            return Err(error);
        }
        let moves = match self {
            Action::Layer(mv) => vec![*mv],
            Action::Block(mv) => mv.to_moves(),
            Action::Rotate { axis, turn } => {
                let (front, back) = match axis {
                    Axis::X => (Face::R, Face::L),
                    Axis::Y => (Face::U, Face::D),
                    Axis::Z => (Face::F, Face::B),
                };
                let inverse = match turn {
                    Turn::Clockwise => Turn::CounterClockwise,
                    Turn::CounterClockwise => Turn::Clockwise,
                    Turn::Half => Turn::Half,
                };
                let rest = (kind.size() - 1) as u8;
                vec![
                    Move::new(front, rest, rest >= 2, *turn),
                    Move::new(back, 1, false, inverse),
                ]
            }
        };
        state.apply(&MoveSequence::new(moves)).map_err(|_| error)
    }
}

/// A quarter turn about the positive `axis` (the right-hand rule).
#[cfg(test)]
pub(crate) fn quarter(axis: usize, v: [i32; 3]) -> [i32; 3] {
    let (b, c) = ((axis + 1) % 3, (axis + 2) % 3);
    let mut out = v;
    out[b] = -v[c];
    out[c] = v[b];
    out
}

/// `sense * e_axis x p`: how a point moves while the action turns.
pub(crate) fn velocity(axis: usize, sense: i32, p: [f64; 3]) -> [f64; 3] {
    let (b, c) = ((axis + 1) % 3, (axis + 2) % 3);
    let mut out = [0.0; 3];
    out[b] = -f64::from(sense) * p[c];
    out[c] = f64::from(sense) * p[b];
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::locate;

    fn act(text: &str) -> Action {
        text.parse().unwrap()
    }

    #[test]
    fn parses_layer_moves_and_rotations() {
        assert_eq!(act("x'").to_string(), "x'");
        assert_eq!(act("z2").to_string(), "z2");
        assert_eq!(act("3Rw2").to_string(), "3Rw2");
        assert!("x3".parse::<Action>().is_err());
        assert!("q".parse::<Action>().is_err());
    }

    /// The moving stickers a state shows differ from the solved cube exactly
    /// where the layer predicate says, except on the turning face itself.
    #[test]
    fn layer_selection_matches_apply() {
        for kind in [
            PuzzleKind::Cube2,
            PuzzleKind::Cube3,
            PuzzleKind::Cube4,
            PuzzleKind::Cube5,
        ] {
            let solved = Facelets::solved(kind);
            let n = kind.size();
            let mut moves = vec![];
            for face in Face::ALL {
                for suffix in ["", "'", "2"] {
                    for depth in 1..n {
                        moves.push(format!("{depth}{}{suffix}", face.letter()));
                        if depth >= 2 {
                            moves.push(format!("{depth}{}w{suffix}", face.letter()));
                        }
                    }
                }
            }
            for text in moves {
                let text = text.strip_prefix('1').unwrap_or(&text).to_owned();
                let action = act(&text);
                let after = action.apply(&solved).unwrap();
                let (axis, _) = action.axis_sign();
                for index in 0..kind.facelet_count() {
                    let (position, normal) = locate(kind, index);
                    let on_axis = normal[axis] != 0;
                    let changed = after.stickers()[index] != solved.stickers()[index];
                    assert_eq!(
                        changed,
                        action.moves(kind, position) && !on_axis,
                        "{kind:?} {text} sticker {index}"
                    );
                }
            }
        }
    }

    /// A block of inner layers lights exactly the stickers its plain moves turn.
    #[test]
    fn block_selection_matches_apply() {
        for kind in [PuzzleKind::Cube4, PuzzleKind::Cube5, PuzzleKind::Cube7] {
            let solved = Facelets::solved(kind);
            let n = kind.size() as u8;
            for face in Face::ALL {
                for from in 2..n {
                    for to in from + 1..n {
                        let action = Action::block(LayerMove::new(face, from, to, Turn::Clockwise));
                        assert!(matches!(action, Action::Block(_)));
                        let after = action.apply(&solved).unwrap();
                        let (axis, _) = action.axis_sign();
                        for index in 0..kind.facelet_count() {
                            let (position, normal) = locate(kind, index);
                            let changed = after.stickers()[index] != solved.stickers()[index];
                            let lit = action.moves(kind, position) && normal[axis] == 0;
                            assert_eq!(changed, lit, "{kind:?} {action} sticker {index}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn a_block_with_a_plain_spelling_is_a_layer_move() {
        let outer = Action::block(LayerMove::new(Face::R, 1, 3, Turn::Half));
        assert_eq!(outer, act("3Rw2"));
        let inner = Action::block(LayerMove::new(Face::R, 2, 3, Turn::CounterClockwise));
        assert_eq!(inner.to_string(), "2-3Rw'");
    }

    #[test]
    fn quarter_turns_agree_with_apply() {
        // Where does the color of a side sticker go? Predicted by `quarter`
        // and the sense, checked on the face that receives it.
        let kind = PuzzleKind::Cube4;
        let solved = Facelets::solved(kind);
        for text in [
            "R", "R'", "U", "U'", "F", "F'", "L", "D'", "B", "2R", "Rw'", "3Fw",
        ] {
            let action = act(text);
            let after = action.apply(&solved).unwrap();
            let (axis, _) = action.axis_sign();
            for index in 0..kind.facelet_count() {
                let (position, normal) = locate(kind, index);
                if normal[axis] != 0 || !action.moves(kind, position) {
                    continue;
                }
                let sense = action.sense();
                let turned = if sense > 0 {
                    quarter(axis, normal)
                } else {
                    quarter(axis, quarter(axis, quarter(axis, normal)))
                };
                let target = Face::ALL
                    .into_iter()
                    .find(|&face| locate(kind, face as usize * kind.face_len()).1 == turned)
                    .unwrap();
                let from = solved.stickers()[index];
                assert_eq!(from, Face::ALL[index / kind.face_len()]);
                let landed = (0..kind.facelet_count()).any(|i| {
                    let (p, nrm) = locate(kind, i);
                    nrm == turned && action.moves(kind, p) && after.stickers()[i] == from
                });
                assert!(landed, "{text}: {from:?} should reach {target:?}");
            }
        }
    }
}
