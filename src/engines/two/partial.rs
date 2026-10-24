//! Partial targets on the 2x2x2: a breadth-first search from the whole goal set.
//!
//! The 3,674,160 states (DBL fixed) are tested against the target; every state
//! that meets it is a source at distance 0, and one pass of the usual move
//! tables gives the exact distance from every state to the nearest goal. Like
//! a complete target on the 2x2x2, a goal counts when the cube shows it after a
//! rotation of the whole cube, so the test uses the 24 rotated masks.

use std::time::Instant;

use super::{
    get_depth, normalise, set_depth, tables, Cubie, Face, Facelets, Failure, Move, MoveSequence,
    PocketSolver, PuzzleKind, SolveMetrics, SolveOptions, SolveResult, Turn, PERMS, SOLVED, STATES,
    TWISTS, UNSEEN,
};
use crate::notation::orbit64::{CORNER_COLOURS, CORNER_FACELETS};
use crate::puzzle::{destinations, Target};

/// Sticker positions (2x2 index) of the three stickers of every corner slot.
fn slot_positions() -> [[usize; 3]; 8] {
    CORNER_FACELETS.map(|slot| slot.map(super::thinned))
}

/// The 24 whole-cube rotations as sticker maps: the sticker at `i` moves to
/// `map[i]`.
fn rotation_maps() -> Vec<Vec<usize>> {
    let turn = |face, turn| Move::new(face, 1, false, turn);
    let generators: Vec<Vec<usize>> = [(Face::R, Face::L), (Face::U, Face::D), (Face::F, Face::B)]
        .iter()
        .map(|&(cw, ccw)| {
            let first = destinations(PuzzleKind::Cube2, &turn(cw, Turn::Clockwise));
            let second = destinations(PuzzleKind::Cube2, &turn(ccw, Turn::CounterClockwise));
            (0..24).map(|i| second[first[i]]).collect()
        })
        .collect();
    let identity: Vec<usize> = (0..24).collect();
    let mut found = vec![identity];
    let mut next = 0;
    while next < found.len() {
        for generator in &generators {
            let composed: Vec<usize> = found[next].iter().map(|&p| generator[p]).collect();
            if !found.contains(&composed) {
                found.push(composed);
            }
        }
        next += 1;
    }
    found
}

/// For every rotation, the constrained stickers as (position in the unrotated
/// state, allowed colour set): a state `T` rotated by `map` shows an allowed
/// colour at `map[i]` exactly when `T[i]` is allowed there.
fn cared_lists(target: &Target) -> Vec<Vec<(usize, u8)>> {
    rotation_maps()
        .iter()
        .map(|map| {
            (0..24)
                .filter_map(|i| {
                    let set = target.sets()[map[i]];
                    (set != 0b11_1111).then_some((i, set))
                })
                .collect()
        })
        .collect()
}

/// The stickers of a corner state, by 2x2 position.
fn stickers(cube: &Cubie, positions: &[[usize; 3]; 8]) -> [u8; 24] {
    let mut out = [0u8; 24];
    for slot in 0..8 {
        let piece = usize::from(cube.piece[slot]);
        let twist = usize::from(cube.twist[slot]);
        for k in 0..3 {
            out[positions[slot][k]] = CORNER_COLOURS[piece][(k + 3 - twist) % 3] as u8;
        }
    }
    out
}

/// Distances from every state to the nearest goal; `None` when no state is a goal.
fn goal_distances(target: &Target) -> Option<Box<[u8]>> {
    let t = tables();
    let positions = slot_positions();
    let cared = cared_lists(target);
    let twists: Vec<Cubie> = (0..TWISTS).map(super::twist_unrank).collect();
    let mut dist = vec![0xFF_u8; STATES / 2].into_boxed_slice();
    let mut frontier = Vec::new();
    for perm in 0..PERMS {
        let base = super::perm_unrank(perm);
        for (twist, twisted) in twists.iter().enumerate() {
            let mut cube = SOLVED;
            cube.piece = base.piece;
            cube.twist = twisted.twist;
            let shown = stickers(&cube, &positions);
            let goal = cared
                .iter()
                .any(|list| list.iter().all(|&(i, set)| set >> shown[i] & 1 == 1));
            if goal {
                let state = (perm * TWISTS + twist) as u32;
                set_depth(&mut dist, state, 0);
                frontier.push(state);
            }
        }
    }
    if frontier.is_empty() {
        return None;
    }
    let mut depth = 0u8;
    while !frontier.is_empty() {
        depth += 1;
        let mut next = Vec::new();
        for &state in &frontier {
            for m in 0..9 {
                let to = t.step(state, m);
                if get_depth(&dist, to) == UNSEEN {
                    set_depth(&mut dist, to, depth);
                    next.push(to);
                }
            }
        }
        frontier = next;
    }
    Some(dist)
}

pub(super) fn solve_to(
    _engine: &PocketSolver,
    facelets: &Facelets,
    target: &Target,
    options: &SolveOptions,
) -> SolveResult {
    let started = Instant::now();
    let finish = |outcome: Result<MoveSequence, Failure>, message: Option<String>| SolveResult {
        outcome,
        message,
        metrics: SolveMetrics {
            elapsed: started.elapsed(),
            probes: None,
        },
        stages: Vec::new(),
    };
    if facelets.kind() != PuzzleKind::Cube2 || target.kind() != PuzzleKind::Cube2 {
        return finish(
            Err(Failure::InvalidInput),
            Some("expected a 2x2x2 start and target".into()),
        );
    }
    let normalised = match normalise(facelets) {
        Ok(normalised) => normalised,
        Err(message) => return finish(Err(Failure::InvalidInput), Some(message)),
    };
    let Some(dist) = goal_distances(target) else {
        return finish(
            Err(Failure::Exhausted),
            Some(
                "unreachable: no state of the 2x2x2 meets the target (all 3674160 states tested)"
                    .into(),
            ),
        );
    };
    let t = tables();
    let length = usize::from(get_depth(&dist, normalised.state));
    if options
        .max_moves
        .is_some_and(|max| length > usize::from(max))
    {
        return finish(
            Err(Failure::Exhausted),
            Some(format!(
                "the shortest solution has {length} moves, over the limit"
            )),
        );
    }
    let mut state = normalised.state;
    let mut solution = Vec::with_capacity(length);
    while get_depth(&dist, state) > 0 {
        let want = get_depth(&dist, state) - 1;
        let m = (0..9)
            .find(|&m| get_depth(&dist, t.step(state, m)) == want)
            .expect("every state off the goal set has a neighbour one closer");
        let (face, turn) = super::search_move(m);
        solution.push(Move::new(
            Face::ALL[normalised.origin[face]],
            1,
            false,
            turn,
        ));
        state = t.step(state, m);
    }
    finish(
        Ok(solution.into_iter().collect()),
        Some(format!(
            "shortest: {length} moves, proved by a search from the whole goal set"
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::Solver;
    use std::collections::HashSet;

    /// A small deterministic generator: the tests need fixed "random" cubes.
    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self, below: usize) -> usize {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (self.0 >> 33) as usize % below
        }
    }

    fn scramble(rng: &mut Lcg, length: usize) -> Facelets {
        let moves = ["U", "U2", "U'", "R", "R2", "R'", "F", "F2", "F'"];
        let text: Vec<&str> = (0..length).map(|_| moves[rng.next(9)]).collect();
        Facelets::solved(PuzzleKind::Cube2)
            .apply(&text.join(" ").parse().unwrap())
            .unwrap()
    }

    /// The least number of turns of U, R and F after which a rotation of the
    /// cube shows the target: the brute-force answer to compare with.
    fn brute_force(start: &Facelets, target: &Target) -> usize {
        let moves: Vec<MoveSequence> = ["U", "U2", "U'", "R", "R2", "R'", "F", "F2", "F'"]
            .iter()
            .map(|m| m.parse().unwrap())
            .collect();
        let mut seen: HashSet<Facelets> = HashSet::from([start.clone()]);
        let mut frontier = vec![start.clone()];
        for depth in 0.. {
            let none = MoveSequence::new(Vec::new());
            if frontier.iter().any(|s| target.reached_by(s, &none)) {
                return depth;
            }
            let mut next = Vec::new();
            for state in &frontier {
                for turn in &moves {
                    let to = state.apply(turn).unwrap();
                    if seen.insert(to.clone()) {
                        next.push(to);
                    }
                }
            }
            frontier = next;
        }
        unreachable!()
    }

    fn solve(start: &Facelets, target: &Target) -> SolveResult {
        PocketSolver.solve_to(start, target, &SolveOptions::default())
    }

    #[test]
    fn the_distance_equals_the_brute_force_minimum_over_completions() {
        let mut rng = Lcg(7);
        for round in 0..6 {
            // a few turns from the goal, so the brute force stays shallow
            let goal = scramble(&mut rng, 6);
            let away = ["U", "R'", "F2", "U'", "R", "F'"];
            let text: Vec<&str> = (0..4 + round % 2).map(|_| away[rng.next(6)]).collect();
            let start = goal.apply(&text.join(" ").parse().unwrap()).unwrap();
            // keep about a third of the stickers of a reachable goal
            let keep: Vec<bool> = (0..24).map(|_| rng.next(3) == 0).collect();
            let target = Target::masked(&goal, |position| keep[position]);
            let result = solve(&start, &target);
            let moves = result.outcome.clone().expect("a reachable goal");
            assert!(target.reached_by(&start, &moves), "round {round}");
            assert_eq!(moves.len(), brute_force(&start, &target), "round {round}");
            assert!(result.message.unwrap().starts_with("shortest"));
        }
    }

    #[test]
    fn every_constrained_sticker_matches_and_freeing_stickers_never_lengthens() {
        let mut rng = Lcg(99);
        for _ in 0..3 {
            let start = scramble(&mut rng, 9);
            let goal = scramble(&mut rng, 8);
            let mut last = usize::MAX;
            for cared in [24, 16, 8, 4, 0] {
                let target = Target::masked(&goal, |position| position < cared);
                let moves = solve(&start, &target).outcome.unwrap();
                assert!(target.reached_by(&start, &moves));
                assert!(moves.len() <= last, "{cared} cared stickers");
                last = moves.len();
            }
            assert_eq!(last, 0, "nothing cared: no move");
        }
    }

    #[test]
    fn a_fully_constrained_target_is_the_old_solve() {
        let mut rng = Lcg(5);
        for _ in 0..3 {
            let start = scramble(&mut rng, 10);
            let moves = solve(
                &start,
                &Target::complete(&Facelets::solved(PuzzleKind::Cube2)),
            )
            .outcome
            .unwrap();
            let old = PocketSolver.solve(&start, &SolveOptions::default());
            assert_eq!(Some(&moves), old.solution());
        }
    }

    #[test]
    fn a_target_no_state_meets_is_reported_as_unreachable() {
        let start = Facelets::solved(PuzzleKind::Cube2);
        // the three stickers of one corner all showing U
        let positions = slot_positions()[0];
        let mut sets = vec![0b11_1111u8; 24];
        for position in positions {
            sets[position] = 1;
        }
        let target = Target::from_sets(sets).unwrap();
        let result = solve(&start, &target);
        assert_eq!(result.outcome, Err(Failure::Exhausted));
        assert!(result.message.unwrap().starts_with("unreachable"));
    }

    #[test]
    fn the_move_limit_is_honoured() {
        let start = Facelets::solved(PuzzleKind::Cube2)
            .apply(&"R U F".parse().unwrap())
            .unwrap();
        let target = Target::complete(&Facelets::solved(PuzzleKind::Cube2));
        let limited = SolveOptions::default().with_max_moves(1);
        let result = PocketSolver.solve_to(&start, &target, &limited);
        assert_eq!(result.outcome, Err(Failure::Exhausted));
    }
}
