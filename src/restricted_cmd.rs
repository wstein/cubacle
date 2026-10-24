//! `solve --moves SET`: the solver for a restricted move set.

use std::sync::Arc;

use cubacle::engines::restricted::{SubsetSolver, DEFAULT_TABLE_ENTRIES};
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::notation::parse_alphabet;
use cubacle::puzzle::{Alphabet, Failure, PuzzleKind, Solver};

/// The move set of `--moves`, read for a cube of `kind`; `None` for the sets
/// that restrict nothing (`htm`).
pub fn read_set(text: &str, kind: PuzzleKind) -> Result<Option<Alphabet>, String> {
    let alphabet = parse_alphabet(text, kind).map_err(|error| format!("--moves: {error}"))?;
    Ok((!alphabet.is_unrestricted()).then_some(alphabet))
}

/// The engine that solves with `alphabet`; its tables stay within
/// `max_memory_mb` megabytes.
pub fn solver_for(alphabet: Alphabet, max_memory_mb: usize) -> Result<Box<dyn Solver>, String> {
    match alphabet.kind() {
        PuzzleKind::Cube3 => {
            // About 70 bytes a state while the table is built.
            let entries = (max_memory_mb.saturating_mul(1 << 20) / 70).min(DEFAULT_TABLE_ENTRIES);
            let solver = SubsetSolver::new(alphabet)
                .map_err(|error| format!("--moves: {error}"))?
                .with_table_entries(entries)
                .with_helper(Arc::new(Min2PhaseSolver::new()));
            Ok(Box::new(solver))
        }
        kind => Err(format!(
            "--moves is available for the 3x3x3 here, not for the {0}x{0}x{0}",
            kind.size()
        )),
    }
}

/// The text of a failed restricted solve: the engines write a sentence.
pub fn failure_text(failure: Failure, message: Option<String>) -> String {
    let fallback = match failure {
        Failure::InvalidInput => "the cube state is impossible",
        Failure::BudgetExhausted => "the time budget ran out",
        Failure::Exhausted => "no solution within the move limit",
        Failure::Internal => "internal error",
    };
    match (failure, message) {
        (Failure::Exhausted | Failure::BudgetExhausted, Some(message)) => message,
        (_, Some(message)) => format!("{fallback} ({message})"),
        (_, None) => fallback.to_owned(),
    }
}
