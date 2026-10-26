use super::*;
use crate::generate;
use std::collections::HashMap;
use std::time::Duration;

type Perms = HashMap<String, [usize; 24]>;

/// The solved 3x3 with `moves` applied, as facelets.
fn cube3(moves: &str) -> String {
    Facelets::solved(PuzzleKind::Cube3)
        .apply(&moves.parse().unwrap())
        .unwrap()
        .to_string()
}

/// Per-move sticker permutations for the 2x2, derived from
/// `Facelets::apply` on a 3x3: `perm[dest] = source`, for all six faces and three turns.
fn sticker_perms() -> Perms {
    let solved: Vec<char> = cube3("").chars().collect();
    let mut perms = HashMap::new();
    for face in Face::ALL {
        for suffix in ["", "2", "'"] {
            let name = format!("{}{suffix}", face.letter());
            let moved: Vec<char> = cube3(&name).chars().collect();
            let mut perm: [usize; 24] = std::array::from_fn(|i| i);
            for dest_slot in CORNER_FACELETS {
                let mut colors: Vec<char> = dest_slot.iter().map(|&f| moved[f]).collect();
                colors.sort();
                let home = CORNER_FACELETS
                    .iter()
                    .find(|home| {
                        let mut c: Vec<char> = home.iter().map(|&f| solved[f]).collect();
                        c.sort();
                        c == colors
                    })
                    .unwrap();
                for &dest in &dest_slot {
                    let src = home.iter().find(|&&f| solved[f] == moved[dest]).unwrap();
                    perm[thinned(dest)] = thinned(*src);
                }
            }
            perms.insert(name, perm);
        }
    }
    perms
}

fn apply(perms: &Perms, state: &str, mv: &str) -> String {
    let chars: Vec<char> = state.chars().collect();
    perms[mv].iter().map(|&src| chars[src]).collect()
}

fn thin(full: &str) -> String {
    let chars: Vec<char> = full.chars().collect();
    let mut out = vec![' '; 24];
    for f in CORNER_FACELETS.iter().flatten() {
        out[thinned(*f)] = chars[*f];
    }
    out.into_iter().collect()
}

fn solved_2x2() -> String {
    thin(&cube3(""))
}

fn all_moves() -> Vec<String> {
    let mut moves = Vec::new();
    for face in Face::ALL {
        for suffix in ["", "2", "'"] {
            moves.push(format!("{}{suffix}", face.letter()));
        }
    }
    moves
}

fn is_solved(state: &str) -> bool {
    state
        .as_bytes()
        .chunks(4)
        .all(|face| face.iter().all(|&c| c == face[0]))
}

fn replay(perms: &Perms, state: &str, solution: &MoveSequence) -> String {
    solution
        .moves()
        .iter()
        .fold(state.to_owned(), |s, mv| apply(perms, &s, &mv.to_string()))
}

#[test]
fn distance_table_has_the_known_depth_distribution() {
    let started = Instant::now();
    let tables = tables();
    eprintln!("table ready after {:?}", started.elapsed());
    let mut counts = [0usize; 12];
    assert_eq!(tables.dist.len(), 1_837_080);
    for state in 0..STATES as u32 {
        let d = tables.depth(state);
        assert_ne!(d, UNSEEN);
        counts[usize::from(d)] += 1;
    }
    assert_eq!(
        counts,
        [1, 9, 54, 321, 1847, 9992, 50136, 227536, 870072, 1887748, 623800, 2644]
    );
    assert_eq!(counts.iter().sum::<usize>(), 3_674_160);
}

#[test]
fn the_cubie_model_agrees_with_the_3x3_engine_on_every_move() {
    let perms = sticker_perms();
    let solved = solved_2x2();
    for face in Face::ALL {
        for (suffix, turn) in [
            ("", Turn::Clockwise),
            ("2", Turn::Half),
            ("'", Turn::CounterClockwise),
        ] {
            let name = format!("{}{suffix}", face.letter());
            let moved = Facelets::parse(&apply(&perms, &solved, &name)).unwrap();
            assert_eq!(
                read_corners(&moved).unwrap(),
                SOLVED.turned(face.index(), turn),
                "{name}"
            );
        }
    }
}

#[test]
fn rotations_are_the_24_whole_cube_rotations() {
    assert_eq!(rotations().len(), 24);
}

#[test]
fn random_states_replay_to_solved_with_table_length() {
    let perms = sticker_perms();
    let solver = PocketSolver::new();
    for _ in 0..2500 {
        let scramble = generate::scramble(PuzzleKind::Cube3, 30);
        let state = thin(&cube3(&scramble.to_string()));
        let facelets = Facelets::parse(&state).unwrap();
        let result = solver.solve(&facelets, &SolveOptions::default());
        let solution = result.solution().expect("solvable");
        assert!(is_solved(&replay(&perms, &state, solution)), "{state}");
        let normalized = normalize(&facelets).unwrap();
        assert_eq!(
            solution.len(),
            usize::from(tables().depth(normalized.state)),
            "{state}"
        );
        assert!(solution.len() <= 11);
    }
}

#[test]
fn generated_states_solve_and_replay() {
    let perms = sticker_perms();
    let solver = PocketSolver::new();
    for _ in 0..300 {
        let cube = generate::random_state(PuzzleKind::Cube2).unwrap();
        let result = solver.solve(&cube, &SolveOptions::default());
        let solution = result.solution().expect("a generated state is reachable");
        assert!(is_solved(&replay(&perms, &cube.to_string(), solution)));
    }
}

#[test]
fn near_states_are_solved_optimally_in_every_orientation() {
    // A multi-source search over facelet strings with all 18 moves, from
    // all 24 orientations of the solved cube, gives true distances that do
    // not depend on the table or the state model.
    let perms = sticker_perms();
    let moves = all_moves();
    let rotate = |s: &str, a: &str, b: &str| apply(&perms, &apply(&perms, s, a), b);
    let mut depth: HashMap<String, usize> = HashMap::new();
    let mut frontier = vec![solved_2x2()];
    depth.insert(solved_2x2(), 0);
    let mut i = 0;
    while i < frontier.len() {
        let s = frontier[i].clone();
        for next in [
            rotate(&s, "R", "L'"),
            rotate(&s, "U", "D'"),
            rotate(&s, "F", "B'"),
        ] {
            if !depth.contains_key(&next) {
                depth.insert(next.clone(), 0);
                frontier.push(next);
            }
        }
        i += 1;
    }
    assert_eq!(frontier.len(), 24);
    for d in 1..=4 {
        let mut next_frontier = Vec::new();
        for s in &frontier {
            for m in &moves {
                let n = apply(&perms, s, m);
                if !depth.contains_key(&n) {
                    depth.insert(n.clone(), d);
                    next_frontier.push(n);
                }
            }
        }
        frontier = next_frontier;
    }
    assert!(depth.len() > 20_000);
    let solver = PocketSolver::new();
    for (state, &d) in &depth {
        let facelets = Facelets::parse(state).unwrap();
        let solution = solver
            .solve(&facelets, &SolveOptions::default())
            .outcome
            .unwrap();
        assert_eq!(solution.len(), d, "{state}");
        assert!(is_solved(&replay(&perms, state, &solution)), "{state}");
    }
}

#[test]
fn move_limit_and_budget_are_honored() {
    let perms = sticker_perms();
    let solver = PocketSolver::new();
    let state = apply(&perms, &apply(&perms, &solved_2x2(), "R"), "U");
    let facelets = Facelets::parse(&state).unwrap();
    let tight = SolveOptions::default().with_max_moves(1);
    assert_eq!(
        solver.solve(&facelets, &tight).outcome,
        Err(Failure::Exhausted)
    );
    let enough = SolveOptions::default().with_max_moves(2);
    assert_eq!(
        solver.solve(&facelets, &enough).solution().unwrap().len(),
        2
    );
    // With the table ready, even a zero budget is met.
    solver.prepare();
    let none = SolveOptions::default().with_time_budget(Duration::ZERO);
    assert_eq!(solver.solve(&facelets, &none).solution().unwrap().len(), 2);
}

#[test]
fn only_a_table_build_can_miss_the_budget() {
    let (ms5, ms1) = (Duration::from_millis(5), Duration::from_millis(1));
    assert_eq!(budget_verdict(false, ms5, Some(Duration::ZERO)), None);
    assert_eq!(budget_verdict(true, ms1, None), None);
    assert_eq!(budget_verdict(true, ms1, Some(ms5)), None);
    assert_eq!(
        budget_verdict(true, ms5, Some(ms1)),
        Some("building the distance table took longer than the time budget")
    );
}

#[test]
fn invalid_inputs_are_reported_not_panicked_on() {
    let perms = sticker_perms();
    let solver = PocketSolver::new();
    let options = SolveOptions::default();

    let three = Facelets::solved(PuzzleKind::Cube3);
    assert_eq!(
        solver.solve(&three, &options).outcome,
        Err(Failure::InvalidInput)
    );

    // One corner twisted in place.
    let solved = solved_2x2();
    let mut chars: Vec<char> = solved.chars().collect();
    let slot = CORNER_FACELETS[0].map(thinned);
    chars.swap(slot[0], slot[1]);
    chars.swap(slot[1], slot[2]);
    let twisted: String = chars.into_iter().collect();
    let result = solver.solve(&Facelets::parse(&twisted).unwrap(), &options);
    assert_eq!(result.outcome, Err(Failure::InvalidInput));
    assert_eq!(
        result.message.as_deref(),
        Some("the corners are invalid: one corner is twisted on its own")
    );

    // Two stickers swapped between faces: counts stay valid, pieces do not.
    let mut chars: Vec<char> = solved.chars().collect();
    chars.swap(0, 4);
    let swapped: String = chars.into_iter().collect();
    let result = solver.solve(&Facelets::parse(&swapped).unwrap(), &options);
    assert_eq!(result.outcome, Err(Failure::InvalidInput));
    assert!(result.message.is_some());

    // Near-valid states: a legal scramble plus one sticker swap or one
    // in-place corner twist. Each must be rejected or solved for real,
    // and both outcomes must occur.
    let moves = all_moves();
    let (mut rejected, mut solved_count) = (0, 0);
    let mut seed = 12345u32;
    let mut next = |bound: usize| {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (seed >> 8) as usize % bound
    };
    for _ in 0..300 {
        let mut state = solved_2x2();
        for _ in 0..8 {
            state = apply(&perms, &state, &moves[next(18)]);
        }
        let mut chars: Vec<char> = state.chars().collect();
        if next(2) == 0 {
            chars.swap(next(24), next(24));
        } else {
            let slot = CORNER_FACELETS[next(8)].map(thinned);
            chars.swap(slot[0], slot[1]);
            chars.swap(slot[1], slot[2]);
        }
        let state: String = chars.into_iter().collect();
        let result = solver.solve(&Facelets::parse(&state).unwrap(), &options);
        match result.outcome {
            Err(Failure::InvalidInput) => rejected += 1,
            Ok(solution) => {
                assert!(is_solved(&replay(&perms, &state, &solution)), "{state}");
                solved_count += 1;
            }
            Err(other) => panic!("{state}: unexpected {other:?}"),
        }
    }
    assert!(
        rejected > 0 && solved_count > 0,
        "{rejected}/{solved_count}"
    );
}
