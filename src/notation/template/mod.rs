//! Templates: a schema grid says which stickers share a color, a guard
//! restricts the colors the placeholders may take.
//!
//! A schema lists all six faces, `N*N` tokens each, in `URFDLB` order, row by
//! row. A lowercase letter with an optional number (`a`, `b12`) is a
//! placeholder named by the whole token, an uppercase letter is a fixed color
//! read through the color scheme, `.` and `?` are don't care. `,` and space
//! separate tokens, `/` or a newline separate faces.
//!
//! The guard is a condition on the colors of the placeholders:
//! `unique(...)`, `equal(...)`, `count(...)` compared with `==`, `!=`, `<`,
//! `<=`, `>`, `>=`, combined with `&&`/`AND`/`and`, `||`/`OR`/`or` and
//! `!`/`NOT`/`not`. A `*` at the end of a name (`a*`) stands for the same
//! index in all arguments of one term, for every index that occurs.

mod ast;
mod error;
mod eval;
mod genfile;
mod guard;
mod lexer;
mod matching;
mod resolve;
mod schema;

use crate::notation::ColourScheme;
use crate::puzzle::PuzzleKind;

use guard::Guard;
use schema::{Schema, Slot};

pub use error::TemplateError;
pub use genfile::GenFile;

/// A parsed schema and guard for one cube size.
#[derive(Debug, Clone)]
pub struct Template {
    kind: PuzzleKind,
    slots: Vec<Slot>,
    names: Vec<String>,
    guard: Guard,
}

impl Template {
    /// Parses a schema grid and a guard (blank for none) for a cube of `kind`.
    /// Positions in errors are inside the text that is wrong.
    pub fn parse(
        schema: &str,
        guard: &str,
        kind: PuzzleKind,
        scheme: &ColourScheme,
    ) -> Result<Template, TemplateError> {
        let schema = schema::parse(schema, kind, scheme)?;
        let guard = Guard::parse(guard, &schema.names)?;
        Ok(Template::from_parts(kind, schema, guard))
    }

    pub(crate) fn from_parts(kind: PuzzleKind, schema: Schema, guard: Guard) -> Template {
        Template {
            kind,
            slots: schema.slots,
            names: schema.names,
            guard,
        }
    }

    pub fn kind(&self) -> PuzzleKind {
        self.kind
    }

    /// The placeholder names in order of first appearance.
    pub fn variables(&self) -> &[String] {
        &self.names
    }
}

#[cfg(test)]
mod tests;
