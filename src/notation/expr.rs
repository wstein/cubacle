use std::fmt;

use crate::puzzle::{Face, Move, MoveSequence, NotationError, PuzzleKind, Turn};

/// The most moves an expression may expand to.
pub const MAX_EXPANDED_MOVES: usize = 100_000;

/// Why a move expression could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprError {
    /// A word is not a move.
    Move(NotationError),
    /// A `(` or `)` has no partner.
    Unbalanced { position: usize },
    /// `*` is not followed by a repeat count of at least 1, or follows nothing.
    BadRepeat { position: usize },
    /// A move turns a layer the cube does not have.
    DoesNotFit { mv: Move, kind: PuzzleKind },
    /// A middle-slice move (`M`, `E`, `S`) on a cube without a middle layer.
    NoMiddleLayer { word: String, kind: PuzzleKind },
    /// The expression expands to more than [`MAX_EXPANDED_MOVES`] moves.
    TooLong,
}

impl fmt::Display for ExprError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprError::Move(error) => error.fmt(f),
            ExprError::Unbalanced { position } => {
                write!(f, "unbalanced parenthesis at character {position}")
            }
            ExprError::BadRepeat { position } => write!(
                f,
                "character {position}: '*' needs a group or move before it and a count of 1 or more after it"
            ),
            ExprError::DoesNotFit { mv, kind } => {
                let size = kind.size();
                write!(f, "move {mv} does not exist on a {size}x{size}x{size}")
            }
            ExprError::NoMiddleLayer { word, kind } => {
                let size = kind.size();
                write!(
                    f,
                    "{word} turns the middle layer, which a {size}x{size}x{size} does not have"
                )
            }
            ExprError::TooLong => write!(
                f,
                "the expression expands to more than {MAX_EXPANDED_MOVES} moves"
            ),
        }
    }
}

impl std::error::Error for ExprError {}

impl From<NotationError> for ExprError {
    fn from(error: NotationError) -> ExprError {
        ExprError::Move(error)
    }
}

/// Reads a move expression for a cube of the given kind.
///
/// Moves are separated by whitespace. `( ... )` groups moves and `*N` repeats
/// the group or move right before it N times; groups nest, so
/// `((R U)*2 F)*3` is valid. Parentheses need no spaces around them, but `*N`
/// must follow its group or move directly. The result is the plain list of
/// layer turns; every move must exist on the cube.
///
/// Besides layer turns the words `M E S` and `x y z` are read. `M`, `E` and `S`
/// turn the middle layer of an odd cube in the direction of `L`, `D` and `F`
/// (on a 3x3x3 `M` is `2L`); an even cube has no middle layer and refuses them.
/// `x`, `y` and `z` turn the whole cube like `R`, `U` and `F`. All take the
/// suffixes `'`, `2` and the aliases of ordinary moves, and a following `*N`.
pub fn parse_expression(text: &str, kind: PuzzleKind) -> Result<MoveSequence, ExprError> {
    let mut reader = Reader {
        chars: text.chars().collect(),
        at: 0,
        kind,
    };
    let moves = reader.sequence(0)?;
    Ok(MoveSequence::new(moves))
}

struct Reader {
    chars: Vec<char>,
    at: usize,
    kind: PuzzleKind,
}

impl Reader {
    /// Items up to the end of the text or an unconsumed `)`.
    fn sequence(&mut self, depth: usize) -> Result<Vec<Move>, ExprError> {
        let mut out = Vec::new();
        loop {
            while self.chars.get(self.at).is_some_and(|c| c.is_whitespace()) {
                self.at += 1;
            }
            let Some(&c) = self.chars.get(self.at) else {
                return if depth == 0 {
                    Ok(out)
                } else {
                    Err(ExprError::Unbalanced { position: self.at })
                };
            };
            let item = match c {
                ')' if depth > 0 => return Ok(out),
                ')' => return Err(ExprError::Unbalanced { position: self.at }),
                '(' => {
                    let open = self.at;
                    self.at += 1;
                    let inner = self.sequence(depth + 1)?;
                    if self.chars.get(self.at) != Some(&')') {
                        return Err(ExprError::Unbalanced { position: open });
                    }
                    self.at += 1;
                    inner
                }
                '*' => return Err(ExprError::BadRepeat { position: self.at }),
                _ => self.word()?,
            };
            let item = self.repeated(item)?;
            if out.len() + item.len() > MAX_EXPANDED_MOVES {
                return Err(ExprError::TooLong);
            }
            out.extend(item);
        }
    }

    /// One move word: everything up to whitespace, a parenthesis or `*`.
    fn word(&mut self) -> Result<Vec<Move>, ExprError> {
        let start = self.at;
        while self
            .chars
            .get(self.at)
            .is_some_and(|c| !c.is_whitespace() && !"()*".contains(*c))
        {
            self.at += 1;
        }
        let word: String = self.chars[start..self.at].iter().collect();
        if let Some(moves) = self.special(&word)? {
            return Ok(moves);
        }
        let mv: Move = word.parse()?;
        if !mv.fits(self.kind) {
            return Err(ExprError::DoesNotFit {
                mv,
                kind: self.kind,
            });
        }
        Ok(vec![mv])
    }

    /// `M E S` (middle slice) and `x y z` (whole-cube rotation), as plain moves.
    fn special(&self, word: &str) -> Result<Option<Vec<Move>>, ExprError> {
        let mut chars = word.chars();
        let letter = chars.next().unwrap_or(' ');
        if !"MESxyz".contains(letter) {
            return Ok(None);
        }
        let turn = format!("R{}", chars.as_str())
            .parse::<Move>()
            .ok()
            .filter(|mv| !mv.wide && mv.depth == 1)
            .map(|mv| mv.turn)
            .ok_or_else(|| NotationError {
                token: word.to_owned(),
            })?;
        let n = self.kind.size();
        let layers = u8::try_from(n).expect("sizes are small");
        let moves = match letter {
            'M' | 'E' | 'S' => {
                if n % 2 == 0 {
                    return Err(ExprError::NoMiddleLayer {
                        word: word.to_owned(),
                        kind: self.kind,
                    });
                }
                let face = match letter {
                    'M' => Face::L,
                    'E' => Face::D,
                    _ => Face::F,
                };
                vec![Move::new(face, layers / 2 + 1, false, turn)]
            }
            _ => {
                let (front, back) = match letter {
                    'x' => (Face::R, Face::L),
                    'y' => (Face::U, Face::D),
                    _ => (Face::F, Face::B),
                };
                // All layers but the last as one wide turn, the last layer
                // the opposite face turns the same way round the axis.
                let wide = layers - 1;
                vec![
                    Move::new(front, wide, wide >= 2, turn),
                    Move::new(back, 1, false, opposite(turn)),
                ]
            }
        };
        Ok(Some(moves))
    }

    /// Applies a directly following `*N` to `item`.
    fn repeated(&mut self, item: Vec<Move>) -> Result<Vec<Move>, ExprError> {
        if self.chars.get(self.at) != Some(&'*') {
            return Ok(item);
        }
        let star = self.at;
        self.at += 1;
        let digits = self.at;
        while self.chars.get(self.at).is_some_and(char::is_ascii_digit) {
            self.at += 1;
        }
        let count: usize = self.chars[digits..self.at]
            .iter()
            .collect::<String>()
            .parse()
            .ok()
            .filter(|&n| n >= 1)
            .ok_or(ExprError::BadRepeat { position: star })?;
        if item.len().saturating_mul(count) > MAX_EXPANDED_MOVES {
            return Err(ExprError::TooLong);
        }
        Ok(item.repeat(count))
    }
}

fn opposite(turn: Turn) -> Turn {
    match turn {
        Turn::Clockwise => Turn::CounterClockwise,
        Turn::CounterClockwise => Turn::Clockwise,
        Turn::Half => Turn::Half,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::Facelets;

    fn expand(text: &str, kind: PuzzleKind) -> Result<String, ExprError> {
        parse_expression(text, kind).map(|moves| moves.to_string())
    }

    #[test]
    fn plain_moves_read_as_before() {
        assert_eq!(expand("R U R' U'", PuzzleKind::Cube3).unwrap(), "R U R' U'");
        assert_eq!(expand("", PuzzleKind::Cube3).unwrap(), "");
    }

    #[test]
    fn a_group_repeats() {
        assert_eq!(expand("(R U)*3", PuzzleKind::Cube3).unwrap(), "R U R U R U");
        assert_eq!(expand("R*2 U", PuzzleKind::Cube3).unwrap(), "R R U");
        assert_eq!(expand("(R U)*1", PuzzleKind::Cube3).unwrap(), "R U");
    }

    #[test]
    fn groups_nest_and_need_no_spaces() {
        assert_eq!(
            expand("((R U')*2 F)*2 D", PuzzleKind::Cube3).unwrap(),
            "R U' R U' F R U' R U' F D"
        );
        assert_eq!(
            expand("(R(U F)*2)*2", PuzzleKind::Cube3).unwrap(),
            "R U F U F R U F U F"
        );
    }

    #[test]
    fn six_sexy_moves_are_the_identity() {
        let moves = parse_expression("(R U R' U')*6", PuzzleKind::Cube3).unwrap();
        let cube = crate::puzzle::Facelets::solved(PuzzleKind::Cube3)
            .apply(&moves)
            .unwrap();
        assert!(cube.is_solved());
    }

    #[test]
    fn malformed_expressions_are_rejected() {
        for text in [
            "(R U", "R U)", "((R)", "*2", "R*", "R*0", "(R)*x", "R *2", "R**2",
        ] {
            assert!(expand(text, PuzzleKind::Cube3).is_err(), "{text:?}");
        }
        assert_eq!(
            expand("(R U", PuzzleKind::Cube3),
            Err(ExprError::Unbalanced { position: 4 })
        );
    }

    #[test]
    fn moves_must_fit_the_cube() {
        assert!(matches!(
            expand("(3R)*2", PuzzleKind::Cube3),
            Err(ExprError::DoesNotFit { .. })
        ));
        assert!(expand("(3R)*2", PuzzleKind::Cube4).is_ok());
    }

    #[test]
    fn runaway_repeats_are_refused() {
        assert_eq!(
            expand("((R)*1000)*1000", PuzzleKind::Cube3),
            Err(ExprError::TooLong)
        );
    }

    fn cube(text: &str, kind: PuzzleKind) -> Facelets {
        let moves = parse_expression(text, kind).unwrap_or_else(|e| panic!("{text}: {e}"));
        Facelets::solved(kind).apply(&moves).unwrap()
    }

    fn sticker(cube: &Facelets, face: Face, index: usize) -> char {
        cube.to_string()
            .chars()
            .nth(face.index() * 9 + index)
            .unwrap()
    }

    #[test]
    fn middle_slices_on_a_3x3_are_the_inner_layer_of_l_d_f() {
        let kind = PuzzleKind::Cube3;
        for (slice, layer) in [("M", "2L"), ("E", "2D"), ("S", "2F")] {
            for suffix in ["", "'", "2"] {
                assert_eq!(
                    cube(&format!("{slice}{suffix}"), kind),
                    cube(&format!("{layer}{suffix}"), kind),
                    "{slice}{suffix}"
                );
            }
        }
    }

    #[test]
    fn slices_follow_l_d_and_f() {
        let kind = PuzzleKind::Cube3;
        // L carries the top to the front, D the left to the front, F the top to the right.
        assert_eq!(sticker(&cube("M", kind), Face::F, 4), 'U');
        assert_eq!(sticker(&cube("E", kind), Face::F, 4), 'L');
        assert_eq!(sticker(&cube("S", kind), Face::R, 4), 'U');
    }

    #[test]
    fn rotations_follow_r_u_and_f() {
        let kind = PuzzleKind::Cube3;
        assert_eq!(sticker(&cube("x", kind), Face::U, 4), 'F');
        assert_eq!(sticker(&cube("y", kind), Face::L, 4), 'F');
        assert_eq!(sticker(&cube("z", kind), Face::R, 4), 'U');
    }

    #[test]
    fn rotations_are_a_face_and_slices() {
        let kind = PuzzleKind::Cube3;
        assert_eq!(cube("x", kind), cube("R M' L'", kind));
        assert_eq!(cube("y", kind), cube("U E' D'", kind));
        assert_eq!(cube("z", kind), cube("F S B'", kind));
        for size in [
            PuzzleKind::Cube2,
            kind,
            PuzzleKind::Cube4,
            PuzzleKind::Cube5,
        ] {
            assert_eq!(cube("x2", size), cube("x x", size));
            assert_eq!(cube("y'", size), cube("y y y", size));
            assert_eq!(cube("z2'", size), cube("z2", size));
        }
    }

    #[test]
    fn a_rotation_and_its_inverse_cancel_on_every_size() {
        for kind in [
            PuzzleKind::Cube2,
            PuzzleKind::Cube3,
            PuzzleKind::Cube4,
            PuzzleKind::Cube5,
            PuzzleKind::Cube6,
            PuzzleKind::Cube7,
        ] {
            assert!(cube("x y z z' y' x'", kind).is_solved(), "{kind:?}");
            assert!(cube("(x y z)*4 (z' y' x')*4", kind).is_solved());
            assert!(cube("x*4 y2*2 z'*4", kind).is_solved());
        }
    }

    #[test]
    fn slice_turns_cancel_on_odd_cubes() {
        for kind in [PuzzleKind::Cube3, PuzzleKind::Cube5, PuzzleKind::Cube7] {
            assert!(cube("M*4 E' E S2 S2", kind).is_solved(), "{kind:?}");
        }
        // The middle layer of a 7x7x7 is its fourth.
        assert_eq!(cube("M", PuzzleKind::Cube7), cube("4L", PuzzleKind::Cube7));
    }

    #[test]
    fn even_cubes_refuse_slices() {
        for kind in [PuzzleKind::Cube2, PuzzleKind::Cube4, PuzzleKind::Cube6] {
            for word in ["M", "E'", "S2"] {
                assert!(matches!(
                    expand(word, kind),
                    Err(ExprError::NoMiddleLayer { .. })
                ));
            }
        }
    }

    #[test]
    fn bad_slice_and_rotation_words_are_rejected() {
        for text in ["Mw", "M3'", "xw", "2M", "x4", "M''", "m", "X"] {
            assert!(expand(text, PuzzleKind::Cube5).is_err(), "{text:?}");
        }
    }

    #[test]
    fn a_nested_expression_with_slices_and_rotations_is_a_valid_cube() {
        let text = "((M' U)*4 y' z')*3";
        let kind = PuzzleKind::Cube7;
        let moves = parse_expression(text, kind).unwrap();
        // Each rotation is two turns: 3 * (4 * 2 + 2 * 2) moves.
        assert_eq!(moves.len(), 3 * (4 * 2 + 2 * 2));
        let state = cube(text, kind);
        assert!(!state.is_solved());
        let token = crate::notation::orbit64::encode(&state).unwrap();
        assert_eq!(crate::notation::orbit64::decode(&token).unwrap(), state);
        assert_eq!(
            crate::notation::parse_cube(&state.to_string(), None).unwrap(),
            state
        );
    }
}
