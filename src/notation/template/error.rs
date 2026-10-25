use std::fmt;

/// Why a schema, a guard or a `.cubacle` file was rejected.
///
/// Line and column count from 1; the column counts characters. A guard or
/// schema parsed on its own reports positions inside its own text, a `.cubacle`
/// file reports positions in the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateError {
    line: usize,
    column: usize,
    message: String,
}

impl TemplateError {
    pub(crate) fn new(line: usize, column: usize, message: impl Into<String>) -> TemplateError {
        TemplateError {
            line,
            column,
            message: message.into(),
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    /// What was wrong or expected, without the position.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The same error for text that starts `lines` lines further down.
    pub(crate) fn shifted(mut self, lines: usize) -> TemplateError {
        self.line += lines;
        self
    }
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for TemplateError {}

/// The characters of a text, so that errors can name a line and a column.
pub(crate) struct Source {
    chars: Vec<char>,
}

impl Source {
    pub(crate) fn new(text: &str) -> Source {
        Source {
            chars: text.chars().collect(),
        }
    }

    pub(crate) fn chars(&self) -> &[char] {
        &self.chars
    }

    /// An error at the character `index` (the end of the text is allowed).
    pub(crate) fn error(&self, index: usize, message: impl Into<String>) -> TemplateError {
        let before = &self.chars[..index.min(self.chars.len())];
        let line = before.iter().filter(|&&c| c == '\n').count() + 1;
        let column = before.iter().rev().take_while(|&&c| c != '\n').count() + 1;
        TemplateError::new(line, column, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_counts_lines_and_columns_from_one() {
        let source = Source::new("ab\ncd");
        assert_eq!(source.error(0, "x").to_string(), "line 1, column 1: x");
        assert_eq!(source.error(4, "x").to_string(), "line 2, column 2: x");
        assert_eq!(source.error(5, "x").to_string(), "line 2, column 3: x");
        assert_eq!(source.error(99, "x").column(), 3);
    }

    #[test]
    fn shifting_moves_only_the_line() {
        let error = TemplateError::new(2, 5, "boom").shifted(10);
        assert_eq!((error.line(), error.column()), (12, 5));
        assert_eq!(error.message(), "boom");
    }
}
