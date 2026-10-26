use std::sync::Arc;

use cubacle::engines::dwalton::DwaltonSolver;
use cubacle::engines::five::FiveSolver;
use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::generate;
use cubacle::phases::phases;
use cubacle::puzzle::{Facelets, Failure, PuzzleKind, SolveOptions, SolveResult, Solver};

fn solver() -> DwaltonSolver {
    let three = Arc::new(Min2PhaseSolver::new());
    DwaltonSolver::new(
        PuzzleKind::Cube6,
        three.clone(),
        Arc::new(FourSolver::new(three.clone())),
        Arc::new(FiveSolver::new(three)),
    )
}

/// The stages of an answer: the six groups in order (a long one cut into
/// numbered parts), covering the moves, with the engine's own state facts at
/// their ends (checked against the cube state).
fn check_stages(cube: &Facelets, result: &SolveResult) {
    let moves = result.solution().expect("a solution");
    // the end of the last part of each group, by its name without `(1/2)`
    let mut groups: Vec<(String, usize)> = Vec::new();
    for stage in &result.stages {
        let name = match stage.name.rsplit_once(" (") {
            Some((name, part)) if part.ends_with(')') => name,
            _ => stage.name,
        };
        match groups.last_mut() {
            Some((last, end)) if last == name => *end = stage.end,
            _ => groups.push((name.to_string(), stage.end)),
        }
    }
    let got: Vec<&str> = groups.iter().map(|g| g.0.as_str()).collect();
    let size = cube.kind().size();
    let want: [&str; 6] = if size == 6 {
        [
            "LR centers on L/R",
            "UD/FB centers on their axes",
            "inner centers solved",
            "inner wings paired",
            "outer wings paired, centers solved",
            "3x3 stage",
        ]
    } else {
        [
            "LR centers on L/R",
            "UD/FB centers on their axes",
            "centers solved",
            "middle wings paired",
            "outer wings paired",
            "3x3 stage",
        ]
    };
    assert_eq!(got, want);
    let ends: Vec<usize> = result.stages.iter().map(|s| s.end).collect();
    assert!(ends.windows(2).all(|w| w[0] <= w[1]), "{ends:?}");
    assert_eq!(ends.last(), Some(&moves.len()));
    let group_ends: Vec<usize> = groups.iter().map(|g| g.1).collect();
    // the 3x3 stage turns outer faces only
    assert!(moves.moves()[group_ends[4]..]
        .iter()
        .all(|m| m.depth == 1 && !m.wide));
    // the state facts hold from the engine's ends on, so the state-based
    // phases cannot end later
    let found = phases(cube, moves).unwrap();
    assert!(found[0].moves.end <= group_ends[3], "{found:?} {ends:?}");
    assert!(found[1].moves.end <= group_ends[4], "{found:?} {ends:?}");
}

/// A short scramble used to put every move into the first stage, because the
/// repair of the stage ends ran on to the end of the answer.
#[test]
fn a_short_scramble_has_more_than_one_phase() {
    let scramble: cubacle::puzzle::MoveSequence = "R U F' 2R 3U Lw2 D'".parse().unwrap();
    let cube = Facelets::solved(PuzzleKind::Cube6)
        .apply(&scramble)
        .unwrap();
    let (result, _) = solver().solve_tallied(&cube);
    assert!(result.is_solved(), "{:?}", result.message);
    check_stages(&cube, &result);
    let total = result.solution().unwrap().len();
    let mut from = 0;
    for stage in &result.stages {
        assert!(stage.end - from < total, "{:?}", result.stages);
        assert!(stage.end - from <= 21, "{:?}", result.stages);
        from = stage.end;
    }
}

/// A slice turn that closes the wing stages used to be merged into the run of
/// moves that opens the 3x3 stage, which then turned a second layer (found
/// by a random scramble in a stress loop; the failure looked load-dependent).
#[test]
fn a_slice_turn_at_the_3x3_boundary_stays_out_of_the_3x3_stage() {
    let scramble: cubacle::puzzle::MoveSequence = "3L F' U 2L 2D2 F D 2L 3B' 2F2 L' 3U 3R B 2F 3U 3R U 3F 3L2 3B2 3D2 3L 3R 3U2 3B' D' 3L2 B 3D2 B D2 R' U' 2L2 2U2 L 2F2 2D 3L' D2 3U' 2L2 3D2 3R B2 3D 3F R' 2B2 2R' 2U 3F' U 2F D R F 2R' 3B 2R2 F' U2 2B 2D 2B D B' 3F L F2 3D' 3B' 2R2 2U2 2F 3D2 3B2 3L' R"
        .parse()
        .unwrap();
    let cube = Facelets::solved(PuzzleKind::Cube6)
        .apply(&scramble)
        .unwrap();
    let (result, _) = solver().solve_tallied(&cube);
    assert!(result.is_solved(), "{:?}", result.message);
    check_stages(&cube, &result);
    let moves = result.solution().unwrap();
    assert!(cube.apply(moves).unwrap().is_solved());
}

/// Solves the reference cubes in `$DWALTON_CUBES` (`N-1.txt` ... `N-20.txt`,
/// N from `$DWALTON_SIZE`, 6 or 7);
/// run with `--ignored --nocapture`.
#[test]
#[ignore]
fn reference_cubes_are_solved() {
    let Ok(dir) = std::env::var("DWALTON_CUBES") else {
        return;
    };
    let size: usize = std::env::var("DWALTON_SIZE").map_or(6, |s| s.parse().unwrap());
    let solver = if size == 7 { solver7() } else { solver() };
    for i in 1..=20 {
        let text = std::fs::read_to_string(format!("{dir}/{size}-{i}.txt")).unwrap();
        let cube = Facelets::parse(text.trim()).unwrap();
        let (result, tally) = solver.solve_tallied(&cube);
        let moves = result
            .outcome
            .unwrap_or_else(|f| panic!("{i}: {f:?} {:?}", result.message));
        assert!(cube.apply(&moves).unwrap().is_solved(), "cube {i}");
        println!(
            "CUBE {i} total {} ms {:?} {tally:?}",
            moves.len(),
            result.metrics.elapsed
        );
    }
}

fn solver7() -> DwaltonSolver {
    let three = Arc::new(Min2PhaseSolver::new());
    DwaltonSolver::new(
        PuzzleKind::Cube7,
        three.clone(),
        Arc::new(FourSolver::new(three.clone())),
        Arc::new(FiveSolver::new(three)),
    )
}

#[test]
fn seven_by_seven_scrambles_are_solved_and_replay() {
    let solver = solver7();
    for _ in 0..2 {
        let scramble = generate::scramble(PuzzleKind::Cube7, 100);
        let cube: Facelets = Facelets::solved(PuzzleKind::Cube7)
            .apply(&scramble)
            .unwrap();
        println!("scramble: {scramble}");
        let (result, tally) = solver.solve_tallied(&cube);
        println!("{tally:?}");
        assert!(result.is_solved(), "{:?}", result.message);
        check_stages(&cube, &result);
        let moves = result
            .outcome
            .unwrap_or_else(|f| panic!("{f:?} {:?}", result.message));
        assert!(cube.apply(&moves).unwrap().is_solved());
    }
}

#[test]
fn an_impossible_7x7x7_is_refused_before_any_search() {
    // Corner and edge permutations of different parity: no turns lead here.
    let text = "WWYYYYWWWWYWYWWWYRYYWWWOWOYWWWYRYYWWWWYWYWWYWWWWWOOOOOOOORBOBROORRGRROORYRYROORRGRROORBOBROOOOOOOOBBGGGBBBBOGOBBBBBWBBBBBGGGBBBBBWBBBBBOGOBBBBBBBBBYYWWWWYYYYWYWYYYWOWWYYYRYRWYYYWOWWYYYYWYWYYWYYYYYRRRRRRRROGRGORROOBOORROWOWORROOBOORROGRGORRRRRRRRGGGGGGGGGRBRGGGGGYGGGGGBBBGGGGGYGGGGGRBRGGGGBBBGG";
    // colors to face letters: white up, red right, green front, yellow down, orange left, blue back
    let letters: String = text
        .chars()
        .map(|c| match c {
            'W' => 'U',
            'G' => 'F',
            'Y' => 'D',
            'O' => 'L',
            other => other,
        })
        .collect();
    let cube = Facelets::parse(&letters).expect("facelets");
    let result = solver7().solve(&cube, &SolveOptions::default());
    assert_eq!(result.outcome.unwrap_err(), Failure::InvalidInput);
    assert!(result.message.unwrap().contains("parity"));
}
