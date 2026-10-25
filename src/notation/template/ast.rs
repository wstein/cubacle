//! The guard after its names are resolved: variables are indices into the
//! schema's variable list, and every `*` is already expanded.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Cmp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Cmp {
    pub(crate) fn holds(self, left: u32, right: u32) -> bool {
        match self {
            Cmp::Eq => left == right,
            Cmp::Ne => left != right,
            Cmp::Lt => left < right,
            Cmp::Le => left <= right,
            Cmp::Gt => left > right,
            Cmp::Ge => left >= right,
        }
    }
}

/// One side of a comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Operand {
    Number(u32),
    /// The color of a variable, as its face number 0..6.
    Variable(usize),
    /// The number of distinct colors among the variables.
    Count(Vec<usize>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Node {
    /// The empty guard.
    True,
    Compare(Operand, Cmp, Operand),
    /// All variables have different colors.
    Unique(Vec<usize>),
    /// All variables have the same color.
    Equal(Vec<usize>),
    Not(Box<Node>),
    And(Vec<Node>),
    Or(Vec<Node>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_operators() {
        assert!(Cmp::Eq.holds(2, 2) && !Cmp::Eq.holds(2, 3));
        assert!(Cmp::Ne.holds(2, 3) && !Cmp::Ne.holds(2, 2));
        assert!(Cmp::Lt.holds(2, 3) && !Cmp::Lt.holds(3, 3));
        assert!(Cmp::Le.holds(3, 3) && !Cmp::Le.holds(4, 3));
        assert!(Cmp::Gt.holds(4, 3) && !Cmp::Gt.holds(3, 3));
        assert!(Cmp::Ge.holds(3, 3) && !Cmp::Ge.holds(2, 3));
    }
}
