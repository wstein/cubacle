mod error;
mod glued;
mod grouped;

pub use error::ExprError;
pub use grouped::{Group, Item};
use grouped::{Mark, MarkKind, Run};

use super::sign::is_separator;
use crate::puzzle::{Face, Move, MoveSequence, NotationError, PuzzleKind, Turn};

/// The most moves an expression may expand to.
pub const MAX_EXPANDED_MOVES: usize = 100_000;

/// The deepest nesting of brackets an expression may have.
pub const MAX_DEPTH: usize = 100;

/// Reads a move expression for a cube of the given kind.
///
/// Moves are separated by whitespace, in WCA, SiGN or SSE spelling (`T3R`,
/// `N2-4R`, `WR`, `SR`, `CR`: see [`super::sse`]), plus layer ranges (`2-3r`, `2-3Rw`) which expand into plain moves. Around them:
///
/// - `( ... )` groups moves. After a group, `*N` or a bare `N` repeats it N
///   times, `'` inverts it (the moves reversed, each inverted), and the two
///   combine: `(R U)*3`, `(R U)3`, `(R U)'`, `(R U)2'`. After a single move
///   only `*N` is a repeat (`R2` is a half turn).
/// - `[A, B]` is the commutator `A B A' B'` and `[A: B]` the conjugate
///   `A B A'`. A and B are any expression; both may be nested and repeated like
///   a group.
///
/// Parentheses and brackets need no spaces around them, but `*N`, `N` and `'`
/// must follow their group directly. The result is the plain list of layer
/// turns; every move must exist on the cube, and at most
/// [`MAX_EXPANDED_MOVES`] come out.
///
/// Besides layer turns the words `M E S` and `x y z` are read. `M`, `E` and `S`
/// turn the middle layer of an odd cube in the direction of `L`, `D` and `F`
/// (on a 3x3x3 `M` is `2L`); an even cube has no middle layer and refuses them.
/// `x`, `y` and `z` turn the whole cube like `R`, `U` and `F`. All take the
/// suffixes `'`, `2` and the aliases of ordinary moves, and a following `*N`.
pub fn parse_expression(text: &str, kind: PuzzleKind) -> Result<MoveSequence, ExprError> {
    Ok(MoveSequence::new(read(text, kind, false)?.moves))
}

/// Reads an expression like [`parse_expression`] and keeps the hand units: a
/// `·` closes a group, a `( ... )` block is a group of groups (two levels are
/// kept, deeper parentheses are flattened). Repetition expands the groups
/// along with the moves; flattening the result gives exactly the moves of
/// [`parse_expression`].
pub fn parse_grouped(text: &str, kind: PuzzleKind) -> Result<Group, ExprError> {
    let run = read(text, kind, true)?;
    Ok(Group::build(&run.moves, &run.marks))
}

fn read(text: &str, kind: PuzzleKind, track: bool) -> Result<Run, ExprError> {
    let mut reader = Reader {
        chars: text.chars().collect(),
        at: 0,
        kind,
        track,
    };
    let (run, stop) = reader.sequence(0)?;
    match stop {
        Stop::End => Ok(run),
        Stop::Close(_) => Err(ExprError::Unbalanced {
            position: reader.at,
        }),
        Stop::Separator(_) => Err(reader.stray_separator()),
    }
}

/// What ended a run of items; the character is not consumed.
enum Stop {
    End,
    Close(char),
    Separator(char),
}

struct Reader {
    chars: Vec<char>,
    at: usize,
    kind: PuzzleKind,
    /// Whether group marks are recorded (off: only the moves).
    track: bool,
}

impl Reader {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn stray_separator(&self) -> ExprError {
        ExprError::Syntax {
            position: self.at,
            reason: "',' and ':' only separate the two parts of a [ ]",
        }
    }

    /// Items up to the end of the text or an unconsumed `)` `]` `,` `:`.
    fn sequence(&mut self, depth: usize) -> Result<(Run, Stop), ExprError> {
        let mut out = Run::default();
        loop {
            while let Some(c) = self.peek().filter(|c| is_separator(*c)) {
                if c == '·' && self.track {
                    out.marks.push(Mark {
                        at: out.moves.len(),
                        kind: MarkKind::Dot,
                    });
                }
                self.at += 1;
            }
            let item = match self.peek() {
                None => return Ok((out, Stop::End)),
                Some(c @ (')' | ']')) => return Ok((out, Stop::Close(c))),
                Some(c @ (',' | ':')) => return Ok((out, Stop::Separator(c))),
                Some('(') => {
                    let group = self.group(depth)?;
                    self.suffixes(group, true)?
                }
                Some('[') => {
                    let bracket = self.bracket(depth)?;
                    self.suffixes(bracket, true)?
                }
                Some('*') => return Err(ExprError::BadRepeat { position: self.at }),
                Some(_) => {
                    let word = Run::plain(self.word()?);
                    self.suffixes(word, false)?
                }
            };
            if out.moves.len() + item.moves.len() > MAX_EXPANDED_MOVES
                || out.marks.len() + item.marks.len() > grouped::MAX_MARKS
            {
                return Err(ExprError::TooLong);
            }
            out.append(item);
        }
    }

    /// `( ... )`, the moves inside.
    fn group(&mut self, depth: usize) -> Result<Run, ExprError> {
        if depth >= MAX_DEPTH {
            return Err(ExprError::TooDeep);
        }
        let open = self.at;
        self.at += 1;
        let (inner, stop) = self.sequence(depth + 1)?;
        match stop {
            Stop::Close(')') => {
                self.at += 1;
                Ok(if self.track { inner.enclosed() } else { inner })
            }
            Stop::End => Err(ExprError::Unbalanced { position: open }),
            Stop::Close(_) => Err(ExprError::Unbalanced { position: self.at }),
            Stop::Separator(_) => Err(self.stray_separator()),
        }
    }

    /// `[A, B]` or `[A: B]`, expanded.
    fn bracket(&mut self, depth: usize) -> Result<Run, ExprError> {
        if depth >= MAX_DEPTH {
            return Err(ExprError::TooDeep);
        }
        let open = self.at;
        self.at += 1;
        let (first, stop) = self.sequence(depth + 1)?;
        let (separator, at) = match stop {
            Stop::Separator(c) => (c, self.at),
            Stop::End => return Err(ExprError::Unbalanced { position: open }),
            Stop::Close(']') => {
                return Err(ExprError::Syntax {
                    position: open,
                    reason: "[ ] needs a comma or a colon: [A, B] or [A: B]",
                })
            }
            Stop::Close(_) => return Err(ExprError::Unbalanced { position: self.at }),
        };
        self.at += 1;
        let (second, stop) = self.sequence(depth + 1)?;
        match stop {
            Stop::Close(']') => self.at += 1,
            Stop::End => return Err(ExprError::Unbalanced { position: open }),
            Stop::Separator(_) => {
                return Err(ExprError::Syntax {
                    position: self.at,
                    reason: "a [ ] has one comma or colon; nest another [ ] for more",
                })
            }
            Stop::Close(_) => return Err(ExprError::Unbalanced { position: self.at }),
        }
        if first.moves.is_empty() || second.moves.is_empty() {
            return Err(ExprError::Syntax {
                position: at,
                reason: "a [ ] needs moves on both sides of the comma or colon",
            });
        }
        let size = first.moves.len() + second.moves.len();
        if size.saturating_mul(2) > MAX_EXPANDED_MOVES
            || (first.marks.len() + second.marks.len()).saturating_mul(2) > grouped::MAX_MARKS
        {
            return Err(ExprError::TooLong);
        }
        let mut out = first.clone();
        out.append(second.clone());
        out.append(first.inverse());
        if separator == ',' {
            out.append(second.inverse());
        }
        Ok(out)
    }

    /// One move word: everything up to whitespace, `·` or a bracket, `*`, `,` `:`.
    /// A layer range is several moves.
    fn word(&mut self) -> Result<Vec<Move>, ExprError> {
        let start = self.at;
        while self
            .peek()
            .is_some_and(|c| !is_separator(c) && !"()[]*,:".contains(c))
        {
            self.at += 1;
        }
        let word: String = self.chars[start..self.at].iter().collect();
        match self.one(&word, start) {
            Err(error) => match self.glued(&word, start) {
                Some(moves) => Ok(moves),
                None => Err(error),
            },
            done => done,
        }
    }

    /// A word of several moves written without spaces (see [`glued`]): every
    /// piece must parse, else the word keeps its original error.
    fn glued(&self, word: &str, start: usize) -> Option<Vec<Move>> {
        let mut out = Vec::new();
        for (offset, piece) in glued::pieces(word)? {
            out.extend(
                self.one(piece, start + word[..offset].chars().count())
                    .ok()?,
            );
        }
        Some(out)
    }

    /// One move word as a single move or range.
    fn one(&self, word: &str, start: usize) -> Result<Vec<Move>, ExprError> {
        let word = word.to_owned();
        if let Some(range) = super::range::expand(&word) {
            let moves = range.map_err(|error| ExprError::Sign {
                position: start,
                error,
            })?;
            for mv in &moves {
                self.check_fit(*mv, start)?;
            }
            return Ok(moves);
        }
        if let Some(sse) = super::sse::expand(&word, Some(self.kind.size())) {
            let moves = sse.map_err(|error| ExprError::Sign {
                position: start,
                error,
            })?;
            for mv in &moves {
                self.check_fit(*mv, start)?;
            }
            return Ok(moves);
        }
        if let Some(moves) = self.special(&word, start)? {
            return Ok(moves);
        }
        let wca = super::sign::to_wca_token(&word).map_err(|error| ExprError::Sign {
            position: start,
            error,
        })?;
        let mv: Move = wca.parse().map_err(|error| ExprError::Move {
            position: start,
            error,
        })?;
        self.check_fit(mv, start)?;
        Ok(vec![mv])
    }

    fn check_fit(&self, mv: Move, position: usize) -> Result<(), ExprError> {
        if mv.fits(self.kind) {
            Ok(())
        } else {
            Err(ExprError::DoesNotFit {
                position,
                mv,
                kind: self.kind,
            })
        }
    }

    /// `M E S` (middle slice) and `x y z` (whole-cube rotation), as plain moves.
    fn special(&self, word: &str, position: usize) -> Result<Option<Vec<Move>>, ExprError> {
        let mut chars = word.chars();
        let letter = chars.next().unwrap_or(' ');
        if !"MESxyz".contains(letter) {
            return Ok(None);
        }
        let turn = Turn::from_suffix(chars.as_str()).ok_or_else(|| ExprError::Move {
            position,
            error: NotationError {
                token: word.to_owned(),
            },
        })?;
        let n = self.kind.size();
        let layers = u8::try_from(n).expect("sizes are small");
        let moves = match letter {
            'M' | 'E' | 'S' => {
                if n.is_multiple_of(2) {
                    return Err(ExprError::NoMiddleLayer {
                        position,
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

    /// Applies what directly follows an item: `*N`, and for a group also a bare
    /// `N` and one `'` (before or after the count).
    fn suffixes(&mut self, item: Run, group: bool) -> Result<Run, ExprError> {
        let mut inverted = false;
        if group && self.peek() == Some('\'') {
            self.at += 1;
            inverted = true;
        }
        let count = match self.peek() {
            Some('*') => {
                let star = self.at;
                self.at += 1;
                self.count(star)?
            }
            Some(c) if group && c.is_ascii_digit() => self.count(self.at)?,
            _ => 1,
        };
        if group && self.peek() == Some('\'') {
            if inverted {
                return Err(ExprError::Syntax {
                    position: self.at,
                    reason: "a group is inverted by one ' only",
                });
            }
            self.at += 1;
            inverted = true;
        }
        if item.moves.len().saturating_mul(count) > MAX_EXPANDED_MOVES
            || item.marks.len().saturating_mul(count) > grouped::MAX_MARKS
        {
            return Err(ExprError::TooLong);
        }
        let repeated = item.repeat(count);
        Ok(if inverted {
            repeated.inverse()
        } else {
            repeated
        })
    }

    /// The digits at the cursor as a count of at least 1; `blame` is where a
    /// missing or zero count is reported.
    fn count(&mut self, blame: usize) -> Result<usize, ExprError> {
        let digits = self.at;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.at += 1;
        }
        if digits == self.at {
            return Err(ExprError::BadRepeat { position: blame });
        }
        let count: usize = self.chars[digits..self.at]
            .iter()
            .collect::<String>()
            .parse()
            // more digits than a usize holds: certainly too long
            .map_err(|_| ExprError::TooLong)?;
        if count == 0 {
            return Err(ExprError::BadRepeat { position: blame });
        }
        Ok(count)
    }
}

/// The moves undone: reversed, each turned the other way.
pub(super) fn inverse(moves: &[Move]) -> Vec<Move> {
    moves
        .iter()
        .rev()
        .map(|mv| Move::new(mv.face, mv.depth, mv.wide, opposite(mv.turn)))
        .collect()
}

fn opposite(turn: Turn) -> Turn {
    match turn {
        Turn::Clockwise => Turn::CounterClockwise,
        Turn::CounterClockwise => Turn::Clockwise,
        Turn::Half => Turn::Half,
    }
}

#[cfg(test)]
mod bracket_tests;
#[cfg(test)]
mod tests;
