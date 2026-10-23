use std::fmt;

use crate::puzzle::{Face, Move, MoveSequence, NotationError, PuzzleKind, Turn};

/// The most moves an expression may expand to.
pub const MAX_EXPANDED_MOVES: usize = 100_000;

/// The deepest nesting of parentheses an expression may have.
pub const MAX_DEPTH: usize = 100;

/// Why a move expression could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprError {
    /// A word is not a move.
    Move(NotationError),
    /// A word is not a move in SiGN notation.
    Sign(super::sign::MovesError),
    /// A `(` or `)` has no partner.
    Unbalanced { position: usize },
    /// `*` is not followed by a repeat count of at least 1, or follows nothing.
    BadRepeat { position: usize },
    /// A move turns a layer the cube does not have.
    DoesNotFit { mv: Move, kind: PuzzleKind },
    /// A middle-slice move (`M`, `E`, `S`) on a cube without a middle layer.
    NoMiddleLayer { word: String, kind: PuzzleKind },
    /// Parentheses are nested deeper than [`MAX_DEPTH`].
    TooDeep,
    /// The expression expands to more than [`MAX_EXPANDED_MOVES`] moves.
    TooLong,
}

impl fmt::Display for ExprError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprError::Move(error) => error.fmt(f),
            ExprError::Sign(error) => error.fmt(f),
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
            ExprError::TooDeep => {
                write!(f, "parentheses are nested deeper than {MAX_DEPTH}")
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
    let moves = reader.sequence(0, 0)?;
    Ok(MoveSequence::new(moves))
}

struct Reader {
    chars: Vec<char>,
    at: usize,
    kind: PuzzleKind,
}

impl Reader {
    /// Items up to the end of the text or an unconsumed `)`.
    fn sequence(&mut self, depth: usize, open: usize) -> Result<Vec<Move>, ExprError> {
        let mut out = Vec::new();
        loop {
            while self.chars.get(self.at).is_some_and(|c| c.is_whitespace()) {
                self.at += 1;
            }
            let Some(&c) = self.chars.get(self.at) else {
                return if depth == 0 {
                    Ok(out)
                } else {
                    Err(ExprError::Unbalanced { position: open })
                };
            };
            let item = match c {
                ')' if depth > 0 => return Ok(out),
                ')' => return Err(ExprError::Unbalanced { position: self.at }),
                '(' => {
                    if depth >= MAX_DEPTH {
                        return Err(ExprError::TooDeep);
                    }
                    let open = self.at;
                    self.at += 1;
                    // Returns only at the matching ')', or with an error.
                    let inner = self.sequence(depth + 1, open)?;
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
        let wca = super::sign::to_wca_token(&word).map_err(ExprError::Sign)?;
        let mv: Move = wca.parse()?;
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
        let turn = Turn::from_suffix(chars.as_str()).ok_or_else(|| NotationError {
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
        if digits == self.at {
            return Err(ExprError::BadRepeat { position: star });
        }
        let count: usize = self.chars[digits..self.at]
            .iter()
            .collect::<String>()
            .parse()
            // more digits than a usize holds: certainly too long
            .map_err(|_| ExprError::TooLong)?;
        if count == 0 {
            return Err(ExprError::BadRepeat { position: star });
        }
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
mod tests;
