use min2phase::{apply_moves, from_moves, solve_pattern};

const SOLVED: &str = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";

fn main() {
    let from = from_moves("R U R' U'").expect("the source scramble is valid");
    let to = from_moves("F2 D L2").expect("the target scramble is valid");
    let result = solve_pattern(&from, &to, 20);

    println!("moves: {}", result.solution);
    println!("outcome: {:?}", result.outcome);
    assert_eq!(
        apply_moves(&from, &result.solution).as_deref(),
        Some(to.as_str())
    );
    assert_ne!(to, SOLVED);
}
