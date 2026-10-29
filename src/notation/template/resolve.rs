//! Turns the names in one term of a guard into variable indices and expands
//! `*`: the same index in all arguments of the term, for every index that
//! occurs, as a conjunction. The index may be empty: `a*` also names `a`.

use super::ast::{Cmp, Node, Operand};
use super::error::{Source, TemplateError};

/// How many variable references all expansions of one guard may produce.
pub(crate) const MAX_EXPANSION: usize = 100_000;

/// A variable as written in the guard.
#[derive(Debug, Clone)]
pub(crate) struct VarRef {
    pub(crate) name: String,
    /// Written with a trailing `*`.
    pub(crate) star: bool,
    pub(crate) pos: usize,
}

/// An operand as written, before names are resolved.
#[derive(Debug, Clone)]
pub(crate) enum Raw {
    Number(u32),
    Var(VarRef),
    Count(Vec<VarRef>),
}

impl Raw {
    fn refs(&self) -> &[VarRef] {
        match self {
            Raw::Number(_) => &[],
            Raw::Var(var) => std::slice::from_ref(var),
            Raw::Count(vars) => vars,
        }
    }
}

pub(crate) struct Resolver<'a> {
    pub(crate) source: &'a Source,
    /// The variable names of the schema, by index.
    pub(crate) names: &'a [String],
    /// Variable references expanded so far.
    pub(crate) spent: usize,
}

impl Resolver<'_> {
    /// For each index that occurs, the variable of every reference at that
    /// index, `None` where a `*` reference has no variable with the index.
    fn combos(&mut self, refs: &[&VarRef]) -> Result<Vec<Vec<Option<usize>>>, TemplateError> {
        let mut suffixes: Vec<&str> = Vec::new();
        for var in refs.iter().filter(|var| var.star) {
            let before = suffixes.len();
            for name in self.names {
                if let Some(rest) = name.strip_prefix(var.name.as_str()) {
                    if rest.chars().all(|c| c.is_ascii_digit()) && !suffixes.contains(&rest) {
                        suffixes.push(rest);
                    }
                }
            }
            if suffixes.len() == before && !self.any_match(var) {
                return Err(self.source.error(
                    var.pos,
                    format!("no variable of the schema matches \"{}*\"", var.name),
                ));
            }
        }
        let indices: Vec<&str> = if suffixes.is_empty() {
            vec![""]
        } else {
            suffixes
        };
        self.spent += indices.len() * refs.len();
        if self.spent > MAX_EXPANSION {
            return Err(self.source.error(
                refs.first().map_or(0, |var| var.pos),
                format!("the guard expands to more than {MAX_EXPANSION} variable references"),
            ));
        }
        let mut out = Vec::with_capacity(indices.len());
        for index in indices {
            let mut row = Vec::with_capacity(refs.len());
            for var in refs {
                let wanted = if var.star {
                    format!("{}{index}", var.name)
                } else {
                    var.name.clone()
                };
                let found = self.names.iter().position(|name| *name == wanted);
                if found.is_none() && !var.star {
                    return Err(self.source.error(
                        var.pos,
                        format!("variable \"{}\" does not occur in the schema", var.name),
                    ));
                }
                row.push(found);
            }
            out.push(row);
        }
        Ok(out)
    }

    fn any_match(&self, var: &VarRef) -> bool {
        self.names.iter().any(|name| {
            name.strip_prefix(var.name.as_str())
                .is_some_and(|rest| rest.chars().all(|c| c.is_ascii_digit()))
        })
    }

    /// `unique(...)` or `equal(...)`, one condition per index.
    pub(crate) fn call(&mut self, unique: bool, args: &[VarRef]) -> Result<Node, TemplateError> {
        let refs: Vec<&VarRef> = args.iter().collect();
        let mut nodes: Vec<Node> = self
            .combos(&refs)?
            .into_iter()
            .map(|row| {
                let ids: Vec<usize> = row.into_iter().flatten().collect();
                if unique {
                    Node::Unique(ids)
                } else {
                    Node::Equal(ids)
                }
            })
            .collect();
        Ok(if nodes.len() == 1 {
            nodes.remove(0)
        } else {
            Node::And(nodes)
        })
    }

    /// `left op right`, one comparison per index of the references in it.
    pub(crate) fn compare(
        &mut self,
        left: &Raw,
        op: Cmp,
        right: &Raw,
    ) -> Result<Node, TemplateError> {
        let refs: Vec<&VarRef> = left.refs().iter().chain(right.refs()).collect();
        let mut nodes = Vec::new();
        'rows: for row in self.combos(&refs)? {
            let mut cursor = row.into_iter();
            let mut build = |raw: &Raw| -> Option<Operand> {
                Some(match raw {
                    Raw::Number(n) => Operand::Number(*n),
                    Raw::Var(_) => Operand::Variable(cursor.next()??),
                    Raw::Count(vars) => Operand::Count(
                        vars.iter()
                            .filter_map(|_| cursor.next().expect("one entry per reference"))
                            .collect(),
                    ),
                })
            };
            let Some(l) = build(left) else { continue 'rows };
            let Some(r) = build(right) else {
                continue 'rows;
            };
            nodes.push(Node::Compare(l, op, r));
        }
        Ok(if nodes.len() == 1 {
            nodes.remove(0)
        } else {
            Node::And(nodes)
        })
    }
}
