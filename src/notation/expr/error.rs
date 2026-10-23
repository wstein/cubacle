use std::fmt;

use super::{MAX_DEPTH, MAX_EXPANDED_MOVES};
use crate::notation::MovesError;
use crate::puzzle::{Move, NotationError, PuzzleKind};

/// Why a move expression could not be read. Positions count characters of the
/// text from 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprError {
    /// A word is not a move.
    Move {
        position: usize,
        error: NotationError,
    },
    /// A word is not a move in SiGN notation (or a layer range).
    Sign { position: usize, error: MovesError },
    /// A `(`, `)`, `[` or `]` has no partner.
    Unbalanced { position: usize },
    /// `*` is not followed by a repeat count of at least 1, or follows nothing,
    /// or a bare count after a group is 0.
    BadRepeat { position: usize },
    /// A `,` `:` `'` or part of a `[ ]` is where it cannot be.
    Syntax {
        position: usize,
        reason: &'static str,
    },
    /// A move turns a layer the cube does not have.
    DoesNotFit {
        position: usize,
        mv: Move,
        kind: PuzzleKind,
    },
    /// A middle-slice move (`M`, `E`, `S`) on a cube without a middle layer.
    NoMiddleLayer {
        position: usize,
        word: String,
        kind: PuzzleKind,
    },
    /// Parentheses are nested deeper than [`MAX_DEPTH`].
    TooDeep,
    /// The expression expands to more than [`MAX_EXPANDED_MOVES`] moves.
    TooLong,
}

impl fmt::Display for ExprError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprError::Move { position, error } => write!(f, "character {position}: {error}"),
            ExprError::Sign { position, error } => write!(f, "character {position}: {error}"),
            ExprError::Unbalanced { position } => {
                write!(f, "unbalanced bracket at character {position}")
            }
            ExprError::BadRepeat { position } => write!(
                f,
                "character {position}: '*' needs a group or move before it and a count of 1 or more after it (a group also takes a bare count)"
            ),
            ExprError::Syntax { position, reason } => write!(f, "character {position}: {reason}"),
            ExprError::DoesNotFit { position, mv, kind } => {
                let size = kind.size();
                write!(
                    f,
                    "character {position}: move {mv} does not exist on a {size}x{size}x{size}"
                )
            }
            ExprError::NoMiddleLayer {
                position,
                word,
                kind,
            } => {
                let size = kind.size();
                write!(
                    f,
                    "character {position}: {word} turns the middle layer, which a {size}x{size}x{size} does not have"
                )
            }
            ExprError::TooDeep => {
                write!(f, "brackets are nested deeper than {MAX_DEPTH}")
            }
            ExprError::TooLong => write!(
                f,
                "the expression expands to more than {MAX_EXPANDED_MOVES} moves"
            ),
        }
    }
}

impl std::error::Error for ExprError {}
