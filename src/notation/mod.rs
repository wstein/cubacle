//! Ways of writing a cube down: Orbit64 tokens, colour schemes and input
//! detection. Everything here translates text into [`crate::puzzle`] types.

mod expr;
mod input;
pub mod orbit64;
mod scheme;
mod sign;

pub use expr::{parse_expression, ExprError, MAX_DEPTH, MAX_EXPANDED_MOVES};
pub use input::{colour_scheme_of, parse_cube, InputError};
pub use scheme::{ColourScheme, SchemeError};
pub use sign::{parse_move, parse_moves, to_wca_token, MovesError, Notation};
