//! Ways of writing a cube down: Orbit64 tokens, color schemes and input
//! detection. Everything here translates text into [`crate::puzzle`] types.

mod alphabet;
mod axis;
mod expr;
mod impossible;
mod input;
mod named;
pub mod orbit64;
mod partial;
mod range;
mod scheme;
mod sign;
mod sse;
mod sse_write;
mod subset;
mod template;

pub use alphabet::{parse_alphabet, AlphabetError, NAMED_SETS};
pub use axis::axis_units;
pub use expr::{
    parse_expression, parse_grouped, ExprError, Group, Item, MAX_DEPTH, MAX_EXPANDED_MOVES,
};
pub use impossible::{check, Impossible, PieceKind};
pub use input::{color_scheme_of, parse_cube, InputError};
pub use named::{named_target, NAMED_TARGETS};
pub use partial::parse_partial;
pub use scheme::{ColorScheme, SchemeError};
pub use sign::{parse_move, parse_moves, to_wca_token, MovesError, Notation};
pub(crate) use sse_write::write_layer_move as write_sse_layer_move;
pub use subset::{named_subset, NAMED_SUBSETS};
pub use template::{GenFile, Template, TemplateError};
