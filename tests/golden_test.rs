//! Golden record of single-thread solver output.
//!
//! `tests/fixtures/golden.txt` holds, per line, a cube, the options it was
//! solved with and the exact solution and probe count the engines produced.
//! The solvers are deterministic without a time budget, so any structural
//! change that alters search behaviour shows up here. Never edit the fixture
//! to make a refactor pass.

use cubacle::engines::three::Min2PhaseSolver;
use cubacle::engines::two::PocketSolver;
use cubacle::puzzle::{Facelets, Failure, SolveOptions, Solver};

const GOLDEN: &str = include_str!("fixtures/golden.txt");

fn describe(solver: &dyn Solver, facelets: &Facelets, options: &SolveOptions) -> (String, String) {
    let result = solver.solve(facelets, options);
    let probes = result
        .metrics
        .probes
        .map_or_else(|| "-".to_owned(), |probes| probes.to_string());
    let outcome = match &result.outcome {
        Ok(solution) => format!("ok {solution}"),
        Err(failure @ (Failure::InvalidInput | Failure::BudgetExhausted | Failure::Exhausted)) => {
            format!("err {failure:?}")
        }
    };
    (probes, outcome)
}

#[test]
fn solver_output_matches_the_golden_record() {
    let three = Min2PhaseSolver::shared();
    let two = PocketSolver::new();
    let mut rows = 0;
    for (number, line) in GOLDEN.lines().enumerate() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let [kind, max_moves, facelets, probes, outcome] = fields[..] else {
            panic!("line {}: malformed", number + 1);
        };
        let mut options = SolveOptions::default();
        if max_moves != "-" {
            options = options.with_max_moves(max_moves.parse().expect("max moves"));
        }
        let facelets = Facelets::parse(facelets).expect("golden facelets");
        let solver: &dyn Solver = if kind == "2" { &two } else { &three };
        let (got_probes, got_outcome) = describe(solver, &facelets, &options);
        assert_eq!(got_outcome, outcome, "line {}: solution", number + 1);
        assert_eq!(got_probes, probes, "line {}: probes", number + 1);
        rows += 1;
    }
    assert!(rows >= 500, "only {rows} golden rows");
}
