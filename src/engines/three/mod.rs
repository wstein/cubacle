//! The two-phase 3x3x3 engine.
//!
//! Files, from the bottom up: `consts` and `cubie` (the cube as pieces),
//! `tables` (move and pruning tables, built once per [`Min2PhaseSolver`]
//! family), `twist_flip` (the twist-flip pruning table), `coordinates` and `search` (the iterative-deepening search),
//! `engine` (string-level entry points, parsing and the worker pool) and
//! `solver` (the [`crate::puzzle::Solver`] adapter, the only public item).

mod consts;
mod coordinates;
mod cubie;
mod engine;
mod pool;
mod search;
mod solver;
mod stats;
mod tables;
mod twist_flip;

pub use solver::Min2PhaseSolver;

#[cfg(test)]
mod tests {
    use super::cubie::Cubie;
    use super::engine::solve_with_stats;
    use super::stats::SolveOutcome;
    use super::tables;
    use crate::puzzle::{Facelets, MoveSequence, PuzzleKind};

    /// Applies moves with the engine's own cubie model.
    fn engine_apply(moves: &MoveSequence) -> String {
        let shared = tables::fresh();
        let built = tables::get(&shared);
        let mut cube = Cubie::new();
        let mut next = Cubie::new();
        for mv in moves.moves() {
            let axis = mv.face.index();
            let power = match mv.turn {
                crate::puzzle::Turn::Clockwise => 0,
                crate::puzzle::Turn::Half => 1,
                crate::puzzle::Turn::CounterClockwise => 2,
            };
            let turn = &built.sctx.movecube[axis * 3 + power];
            Cubie::corn_mult(&cube, turn, &mut next);
            Cubie::edge_mult(&cube, turn, &mut next);
            cube = next;
        }
        cube.as_facelet()
    }

    #[test]
    fn facelets_apply_agrees_with_the_engine_cubie_model() {
        let mut seed = 0x2545_F491_4F6C_DD1Du64;
        let mut next = |bound: u64| {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) % bound
        };
        let letters = ["U", "R", "F", "D", "L", "B"];
        let suffixes = ["", "2", "'"];
        for _ in 0..200 {
            let text: Vec<String> = (0..1 + next(30))
                .map(|_| {
                    let face = letters[next(6) as usize];
                    format!("{face}{}", suffixes[next(3) as usize])
                })
                .collect();
            let moves: MoveSequence = text.join(" ").parse().unwrap();
            let typed = Facelets::solved(PuzzleKind::Cube3).apply(&moves).unwrap();
            assert_eq!(typed.to_string(), engine_apply(&moves), "{moves}");
        }
    }

    #[test]
    fn generated_states_are_reachable_and_solve_back() {
        use super::Min2PhaseSolver;
        use crate::puzzle::{SolveOptions, Solver};
        let solver = Min2PhaseSolver::new();
        for _ in 0..100 {
            let cube = crate::generate::random_state(PuzzleKind::Cube3).unwrap();
            let result = solver.solve(&cube, &SolveOptions::default());
            let solution = result.solution().expect("a generated state is reachable");
            assert!(cube.apply(solution).unwrap().is_solved());
        }
    }

    #[test]
    fn stats_count_probes_and_balance_the_pruning_tallies() {
        let cube = Facelets::solved(PuzzleKind::Cube3)
            .apply(&"R U R' U'".parse().unwrap())
            .unwrap();
        let result = solve_with_stats(&tables::fresh(), &cube.to_string(), 20);
        assert_eq!(result.outcome, SolveOutcome::Solved);
        assert!(result.probes > 0, "expected phase-two search probes");
        assert!(result.phase1_nodes > 0, "expected phase-one search nodes");
        assert!(result.phase1_children_recurred > 0);
        assert_eq!(
            result.phase1_rejected_slice_twist,
            result.phase1_slice_twist_breaks + result.phase1_slice_twist_equals
        );
        assert_eq!(
            result.phase1_rejected_slice_flip,
            result.phase1_slice_flip_breaks + result.phase1_slice_flip_equals
        );
    }
}
