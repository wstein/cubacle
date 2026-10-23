use std::fmt;

use crate::puzzle::{Move, MoveSequence, NotationError, PuzzleKind};

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
        let mv: Move = word.parse()?;
        if !mv.fits(self.kind) {
            return Err(ExprError::DoesNotFit {
                mv,
                kind: self.kind,
            });
        }
        Ok(vec![mv])
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
