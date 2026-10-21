use min2phase::{apply_moves, solve};

const MAX_SOL_LEN: u8 = 20;
const SOLVED: &str = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";

#[test]
fn string_slices_are_accepted_by_public_apis() {
    let cube = apply_moves(SOLVED, "R U R' U'").expect("valid move sequence");
    let solution = solve(cube.as_str(), MAX_SOL_LEN);
    let solved = apply_moves(cube.as_str(), solution.as_str()).expect("valid solver result");

    assert_eq!(solved, SOLVED);
}

#[test]
fn test_issue1_solve() {
    let cube = "UDUDUDUDURLRLRLRLRFBFBFBFBFDUDUDUDUDLRLRLRLRLBFBFBFBFB".to_string();
    let solution = solve(&cube, MAX_SOL_LEN);
    let solved_cubie = apply_moves(&cube, &solution);
    assert!(
        solved_cubie == Some(SOLVED.to_string()),
        "unexpected result: solution={solution} solved_cubie={solved_cubie:?}"
    );
}
