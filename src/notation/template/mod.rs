//! Templates: a schema grid says which stickers share a colour, a guard
//! restricts the colours the placeholders may take.

#[allow(dead_code)] // the parser and the evaluator use it from the next commit on
mod ast;
#[allow(dead_code)] // the parsers use it from the next commit on
mod error;

pub use error::TemplateError;
