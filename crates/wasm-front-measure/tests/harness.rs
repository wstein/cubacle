//! The harness text interface answers like the public front-goal entry.
//! Input: `facelets TAB twists TAB accepted TAB center_up` (the columns 6 to 8 of
//! tests/fixtures/front_goal_calendar.tsv, then 1 when the center cell is not blank).

use cubacle_wasm_front_measure::solve_text;

const FIXTURE: &str = include_str!("../../../tests/fixtures/front_goal_calendar.tsv");

fn input_of(print: &str, step: &str) -> (String, usize) {
    let line = FIXTURE
        .lines()
        .find(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            !l.starts_with('#') && c[0] == print && c[1] == step
        })
        .unwrap();
    let c: Vec<&str> = line.split('\t').collect();
    let center = c[4].split('|').nth(4).unwrap();
    let up = if center.is_empty() { "0" } else { "1" };
    (
        format!("{}\t{}\t{}\t{}", c[5], c[6], c[7], up),
        c[3].parse().unwrap(),
    )
}

#[test]
fn a_calendar_case_gives_the_484_optimum() {
    let (input, expected) = input_of("de-a", "1");
    let out = solve_text(&input, 120_000, 0, 0);
    let f: Vec<&str> = out.split('\t').collect();
    assert_eq!(f[0], "ok", "{out}");
    assert_eq!(f[1].split_whitespace().count(), expected, "{out}");
}

#[test]
fn a_node_cap_ends_as_budget() {
    let (input, _) = input_of("de-a", "1");
    let out = solve_text(&input, 120_000, 10, 0);
    assert!(out.starts_with("budget\t"), "{out}");
}

#[test]
fn a_bad_input_is_invalid() {
    assert!(solve_text("x", 1000, 0, 0).starts_with("invalid\t"));
}
