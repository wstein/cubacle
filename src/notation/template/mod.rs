//! Templates: a schema grid says which stickers share a colour, a guard
//! restricts the colours the placeholders may take.

// Nothing builds a template yet; the next commits add the schema and `Template`.
#[allow(dead_code)]
mod ast;
mod error;
#[allow(dead_code)]
mod eval;
#[allow(dead_code)]
mod guard;
#[allow(dead_code)]
mod lexer;
#[allow(dead_code)]
mod resolve;
#[allow(dead_code)]
mod schema;

pub use error::TemplateError;
