//! Ways of writing a cube down: Orbit64 tokens, colour schemes and input
//! detection. Everything here translates text into [`crate::puzzle`] types.

mod expr;
mod input;
pub mod orbit64;
mod scheme;

pub use expr::{parse_expression, ExprError, MAX_EXPANDED_MOVES};
pub use input::{parse_cube, InputError};
pub use scheme::{ColourScheme, SchemeError};
