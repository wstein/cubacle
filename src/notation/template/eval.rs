//! Evaluates a resolved guard for a complete assignment of colors.

use super::ast::{Node, Operand};

/// `colors[i]` is the face number 0..6 that variable `i` shows.
pub(crate) fn eval(node: &Node, colors: &[u8]) -> bool {
    match node {
        Node::True => true,
        Node::Compare(left, op, right) => op.holds(value(left, colors), value(right, colors)),
        Node::Unique(vars) => {
            let mut seen = 0u32;
            vars.iter().all(|&var| {
                let bit = 1u32 << colors[var];
                let fresh = seen & bit == 0;
                seen |= bit;
                fresh
            })
        }
        Node::Equal(vars) => vars.windows(2).all(|w| colors[w[0]] == colors[w[1]]),
        Node::Not(inner) => !eval(inner, colors),
        Node::And(parts) => parts.iter().all(|part| eval(part, colors)),
        Node::Or(parts) => parts.iter().any(|part| eval(part, colors)),
    }
}

fn value(operand: &Operand, colors: &[u8]) -> u32 {
    match operand {
        Operand::Number(n) => *n,
        Operand::Variable(var) => u32::from(colors[*var]),
        Operand::Count(vars) => vars
            .iter()
            .fold(0u32, |mask, &var| mask | (1 << colors[var]))
            .count_ones(),
    }
}
