//! The guard: a condition on the colors of the schema's variables.
//!
//! ```text
//! guard      = or-expr
//! or-expr    = and-expr { ("||" | "OR" | "or") and-expr }
//! and-expr   = not-expr { ("&&" | "AND" | "and") not-expr }
//! not-expr   = ("!" | "NOT" | "not") not-expr | comparison
//! comparison = term [ cmp-op term ]
//! term       = call | variable | number | "(" guard ")"
//! call       = ("unique" | "equal" | "count") "(" variable { "," variable } ")"
//! ```

use super::ast::Node;
use super::error::{Source, TemplateError};
use super::eval;
use super::lexer::{lex, Tok, Token};
use super::resolve::{Raw, Resolver, VarRef};

/// How deep parentheses and negations may nest.
pub(crate) const MAX_DEPTH: usize = 64;

#[derive(Debug, Clone)]
pub(crate) struct Guard {
    root: Node,
}

impl Guard {
    /// Parses `text` against the variable names of the schema. Blank text is
    /// the guard that always holds.
    pub(crate) fn parse(text: &str, names: &[String]) -> Result<Guard, TemplateError> {
        let source = Source::new(text);
        let tokens = lex(&source)?;
        let mut parser = Parser {
            tokens: &tokens,
            at: 0,
            depth: 0,
            resolver: Resolver {
                source: &source,
                names,
                spent: 0,
            },
        };
        let root = if tokens.len() == 1 {
            Node::True
        } else {
            let root = parser.or()?;
            parser.expect_end()?;
            root
        };
        Ok(Guard { root })
    }

    /// Whether the guard holds when variable `i` has the color `colors[i]`.
    pub(crate) fn holds(&self, colors: &[u8]) -> bool {
        eval::eval(&self.root, colors)
    }
}

/// A term is a condition or something to compare.
enum Term {
    Condition(Node, usize),
    Value(Raw),
}

struct Parser<'a> {
    tokens: &'a [Token],
    at: usize,
    depth: usize,
    resolver: Resolver<'a>,
}

impl Parser<'_> {
    fn peek(&self) -> &Token {
        &self.tokens[self.at.min(self.tokens.len() - 1)]
    }

    fn bump(&mut self) -> Token {
        let token = self.peek().clone();
        if self.at < self.tokens.len() - 1 {
            self.at += 1;
        }
        token
    }

    fn fail<T>(&self, expected: &str) -> Result<T, TemplateError> {
        let token = self.peek();
        Err(self.resolver.source.error(
            token.pos,
            format!("expected {expected}, found {}", token.tok.describe()),
        ))
    }

    fn expect(&mut self, tok: Tok) -> Result<(), TemplateError> {
        if self.peek().tok == tok {
            self.bump();
            Ok(())
        } else {
            self.fail(&tok.describe())
        }
    }

    fn expect_end(&self) -> Result<(), TemplateError> {
        if self.peek().tok == Tok::End {
            Ok(())
        } else {
            self.fail("\"&&\", \"||\" or the end of the guard")
        }
    }

    fn enter(&mut self) -> Result<(), TemplateError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(self.resolver.source.error(
                self.peek().pos,
                format!("the guard nests deeper than {MAX_DEPTH} levels"),
            ));
        }
        Ok(())
    }

    fn or(&mut self) -> Result<Node, TemplateError> {
        let mut parts = vec![self.and()?];
        while self.peek().tok == Tok::Or {
            self.bump();
            parts.push(self.and()?);
        }
        Ok(one_or(parts, Node::Or))
    }

    fn and(&mut self) -> Result<Node, TemplateError> {
        let mut parts = vec![self.not()?];
        while self.peek().tok == Tok::And {
            self.bump();
            parts.push(self.not()?);
        }
        Ok(one_or(parts, Node::And))
    }

    fn not(&mut self) -> Result<Node, TemplateError> {
        if self.peek().tok == Tok::Not {
            self.bump();
            self.enter()?;
            let inner = self.not();
            self.depth -= 1;
            return Ok(Node::Not(Box::new(inner?)));
        }
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Node, TemplateError> {
        let left = self.term()?;
        if let Tok::Compare(op) = self.peek().tok {
            let (Term::Value(left), op) = (left, op) else {
                return Err(self.condition_compared());
            };
            self.bump();
            return match self.term()? {
                Term::Value(right) => self.resolver.compare(&left, op, &right),
                Term::Condition(_, pos) => Err(self.resolver.source.error(
                    pos,
                    "a condition cannot be compared, expected a variable, count(...) or a number",
                )),
            };
        }
        match left {
            Term::Condition(node, _) => Ok(node),
            Term::Value(_) => {
                self.fail("a comparison (\"==\", \"!=\", \"<\", \"<=\", \">\" or \">=\")")
            }
        }
    }

    fn condition_compared(&self) -> TemplateError {
        self.resolver.source.error(
            self.peek().pos,
            "unique(...), equal(...) and parenthesised conditions cannot be compared",
        )
    }

    fn term(&mut self) -> Result<Term, TemplateError> {
        let Token { tok, pos } = self.peek().clone();
        match tok {
            Tok::LParen => {
                self.bump();
                self.enter()?;
                let inner = self.or();
                self.depth -= 1;
                let inner = inner?;
                self.expect(Tok::RParen)?;
                Ok(Term::Condition(inner, pos))
            }
            Tok::Number(n) => {
                self.bump();
                Ok(Term::Value(Raw::Number(n)))
            }
            Tok::Name(name, false) if matches!(name.as_str(), "unique" | "equal" | "count") => {
                self.bump();
                self.expect(Tok::LParen)?;
                let args = self.arguments()?;
                match name.as_str() {
                    "count" => Ok(Term::Value(Raw::Count(args))),
                    other => {
                        let node = self.resolver.call(other == "unique", &args)?;
                        Ok(Term::Condition(node, pos))
                    }
                }
            }
            Tok::Name(name, star) => {
                self.bump();
                Ok(Term::Value(Raw::Var(VarRef { name, star, pos })))
            }
            _ => self.fail("a variable, a number, a call or \"(\""),
        }
    }

    /// `variable { "," variable } ")"`, the opening parenthesis is consumed.
    fn arguments(&mut self) -> Result<Vec<VarRef>, TemplateError> {
        let mut args = Vec::new();
        loop {
            let Token { tok, pos } = self.peek().clone();
            let Tok::Name(name, star) = tok else {
                return self.fail("a variable");
            };
            self.bump();
            args.push(VarRef { name, star, pos });
            match self.peek().tok {
                Tok::Comma => {
                    self.bump();
                }
                Tok::RParen => {
                    self.bump();
                    return Ok(args);
                }
                _ => return self.fail("\")\" or \",\""),
            }
        }
    }
}

fn one_or(mut parts: Vec<Node>, join: fn(Vec<Node>) -> Node) -> Node {
    if parts.len() == 1 {
        parts.remove(0)
    } else {
        join(parts)
    }
}

#[cfg(test)]
mod tests;
