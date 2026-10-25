//! Splits a guard into tokens.

use super::ast::Cmp;
use super::error::{Source, TemplateError};

/// A guard longer than this is refused before it is read.
pub(crate) const MAX_GUARD_CHARS: usize = 100_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Tok {
    LParen,
    RParen,
    Comma,
    Compare(Cmp),
    And,
    Or,
    Not,
    Number(u32),
    /// A name, with `true` when it ends in `*`.
    Name(String, bool),
    End,
}

impl Tok {
    /// How the token is named in a message.
    pub(crate) fn describe(&self) -> String {
        match self {
            Tok::LParen => "\"(\"".into(),
            Tok::RParen => "\")\"".into(),
            Tok::Comma => "\",\"".into(),
            Tok::Compare(_) => "a comparison".into(),
            Tok::And => "\"&&\"".into(),
            Tok::Or => "\"||\"".into(),
            Tok::Not => "\"!\"".into(),
            Tok::Number(n) => format!("the number {n}"),
            Tok::Name(name, star) => format!("\"{name}{}\"", if *star { "*" } else { "" }),
            Tok::End => "the end of the guard".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Token {
    pub(crate) tok: Tok,
    /// Index of the first character.
    pub(crate) pos: usize,
}

/// All tokens of the guard, ending with [`Tok::End`].
pub(crate) fn lex(source: &Source) -> Result<Vec<Token>, TemplateError> {
    let chars = source.chars();
    if chars.len() > MAX_GUARD_CHARS {
        return Err(source.error(
            MAX_GUARD_CHARS,
            format!("the guard is longer than {MAX_GUARD_CHARS} characters"),
        ));
    }
    let at = |i: usize| chars.get(i).copied();
    let mut tokens = Vec::new();
    let mut i = 0;
    while let Some(c) = at(i) {
        let start = i;
        i += 1;
        let tok = match c {
            c if c.is_whitespace() => continue,
            '(' => Tok::LParen,
            ')' => Tok::RParen,
            ',' => Tok::Comma,
            '=' if at(i) == Some('=') => {
                i += 1;
                Tok::Compare(Cmp::Eq)
            }
            '=' => return Err(source.error(start, "expected \"==\", found \"=\"")),
            '!' if at(i) == Some('=') => {
                i += 1;
                Tok::Compare(Cmp::Ne)
            }
            '!' => Tok::Not,
            '<' if at(i) == Some('=') => {
                i += 1;
                Tok::Compare(Cmp::Le)
            }
            '<' => Tok::Compare(Cmp::Lt),
            '>' if at(i) == Some('=') => {
                i += 1;
                Tok::Compare(Cmp::Ge)
            }
            '>' => Tok::Compare(Cmp::Gt),
            '&' if at(i) == Some('&') => {
                i += 1;
                Tok::And
            }
            '&' => return Err(source.error(start, "expected \"&&\", found \"&\"")),
            '|' if at(i) == Some('|') => {
                i += 1;
                Tok::Or
            }
            '|' => return Err(source.error(start, "expected \"||\", found \"|\"")),
            c if c.is_ascii_digit() => {
                while at(i).is_some_and(|d| d.is_ascii_digit()) {
                    i += 1;
                }
                let digits: String = chars[start..i].iter().collect();
                match digits.parse::<u32>() {
                    Ok(n) => Tok::Number(n),
                    Err(_) => return Err(source.error(start, "the number is too large")),
                }
            }
            c if c.is_ascii_alphabetic() => {
                while at(i).is_some_and(|d| d.is_ascii_alphanumeric()) {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                if at(i) == Some('*') {
                    i += 1;
                    Tok::Name(word, true)
                } else {
                    match word.as_str() {
                        "and" | "AND" => Tok::And,
                        "or" | "OR" => Tok::Or,
                        "not" | "NOT" => Tok::Not,
                        _ => Tok::Name(word, false),
                    }
                }
            }
            other => {
                return Err(source.error(start, format!("unexpected character {other:?}")));
            }
        };
        tokens.push(Token { tok, pos: start });
    }
    tokens.push(Token {
        tok: Tok::End,
        pos: chars.len(),
    });
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(text: &str) -> Vec<Tok> {
        lex(&Source::new(text))
            .expect("lexes")
            .into_iter()
            .map(|t| t.tok)
            .collect()
    }

    #[test]
    fn operators_and_spellings() {
        assert_eq!(
            toks("a1 != b1 && !c"),
            vec![
                Tok::Name("a1".into(), false),
                Tok::Compare(Cmp::Ne),
                Tok::Name("b1".into(), false),
                Tok::And,
                Tok::Not,
                Tok::Name("c".into(), false),
                Tok::End
            ]
        );
        for (word, tok) in [
            ("and", Tok::And),
            ("AND", Tok::And),
            ("or", Tok::Or),
            ("OR", Tok::Or),
            ("not", Tok::Not),
            ("NOT", Tok::Not),
        ] {
            assert_eq!(toks(word), vec![tok, Tok::End]);
        }
        assert_eq!(toks("a*")[0], Tok::Name("a".into(), true));
        assert_eq!(toks("<= >= < >").len(), 5);
    }

    #[test]
    fn bad_characters_are_errors_not_panics() {
        let message = |text: &str| lex(&Source::new(text)).unwrap_err().to_string();
        assert_eq!(
            message("a1 & b1"),
            "line 1, column 4: expected \"&&\", found \"&\""
        );
        assert_eq!(
            message("a1 = b1"),
            "line 1, column 4: expected \"==\", found \"=\""
        );
        assert_eq!(
            message("a1 $"),
            "line 1, column 4: unexpected character '$'"
        );
        assert_eq!(
            message("99999999999"),
            "line 1, column 1: the number is too large"
        );
    }
}
