use super::*;

fn seq(text: &str) -> MoveSequence {
    text.parse().expect("moves")
}

fn inverse(text: &str) -> MoveSequence {
    let flipped: Vec<String> = text
        .split_whitespace()
        .rev()
        .map(|t| match t.strip_suffix('\'') {
            Some(plain) => plain.to_owned(),
            None if t.ends_with('2') => t.to_owned(),
            None => format!("{t}'"),
        })
        .collect();
    seq(&flipped.join(" "))
}

fn after(kind: PuzzleKind, moves: &str) -> Facelets {
    Facelets::solved(kind).apply(&seq(moves)).expect("fits")
}

fn holds(predicate: Predicate, state: &Facelets) -> bool {
    predicate(&Layout::new(state.kind()), state.stickers())
}

#[test]
fn g1_is_closed_under_its_generators_and_left_by_the_others() {
    let cube = PuzzleKind::Cube3;
    assert!(holds(in_g1, &Facelets::solved(cube)));
    assert!(holds(in_g1, &after(cube, "U D' R2 L2 F2 B2 U2 R2 D F2 U'")));
    for outside in ["R", "R'", "L", "F", "B'", "R U", "U F2 R"] {
        assert!(!holds(in_g1, &after(cube, outside)), "{outside}");
    }
    // a pure edge flip of the middle layer: F then B' then a G1 move keeps the
    // domino colours wrong
    assert!(!holds(in_g1, &after(cube, "F R F'")));
}

#[test]
fn g1_agrees_with_the_generators_on_many_walks() {
    let cube = PuzzleKind::Cube3;
    let generators = ["U", "U'", "U2", "D", "D'", "D2", "R2", "L2", "F2", "B2"];
    let mut seed = 0x2545_f491_u64;
    let mut walk = Vec::new();
    for _ in 0..200 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        walk.push(generators[(seed >> 33) as usize % generators.len()]);
        if walk.len() % 7 == 0 {
            assert!(holds(in_g1, &after(cube, &walk.join(" "))));
        }
    }
}

#[test]
fn centres_and_edges_on_a_4x4() {
    let cube = PuzzleKind::Cube4;
    assert!(holds(centres_solved, &after(cube, "R U' F2 B")));
    // outer turns move both wings of an edge together
    assert!(holds(edges_paired, &after(cube, "R U' F2 B")));
    assert!(!holds(edges_paired, &after(cube, "Rw")));
    assert!(!holds(centres_solved, &after(cube, "Rw")));
    assert!(holds(edges_paired, &Facelets::solved(cube)));
}

#[test]
fn the_middle_sticker_of_an_odd_cube_counts() {
    let cube = PuzzleKind::Cube5;
    assert!(holds(centres_solved, &after(cube, "R U")));
    assert!(!holds(centres_solved, &after(cube, "2R")));
    assert!(holds(edges_paired, &Facelets::solved(cube)));
}

#[test]
fn a_3x3_solution_is_cut_where_it_enters_g1_for_good() {
    let solution = "R U' F U2 R2 D";
    let start = after(PuzzleKind::Cube3, &inverse(solution).to_string());
    let found = phases(&start, &seq(solution)).unwrap();
    assert_eq!(
        found,
        vec![
            Phase {
                number: 1,
                name: "to the G1 subgroup",
                moves: 0..3
            },
            Phase {
                number: 2,
                name: "solve",
                moves: 3..6
            }
        ]
    );
}

#[test]
fn an_empty_phase_is_kept_and_counted() {
    // a 4x4 that has its centres and edges: only the 3x3 stage is left
    let solution = "R U R'";
    let start = after(PuzzleKind::Cube4, &inverse(solution).to_string());
    let found = phases(&start, &seq(solution)).unwrap();
    assert_eq!(found.len(), 3, "{found:?}");
    assert_eq!((found[0].number, found[0].moves.clone()), (1, 0..0));
    assert_eq!((found[1].number, found[1].moves.clone()), (2, 0..0));
    assert_eq!((found[2].number, found[2].name), (3, "3x3 stage"));
    assert_eq!(found[2].moves, 0..3);
    // a cube that is already in G1 has an empty phase 1
    let start = after(PuzzleKind::Cube3, "R2 U");
    let found = phases(&start, &seq("U' R2")).unwrap();
    assert_eq!(found.len(), 2);
    assert!(found[0].moves.is_empty());
    assert_eq!((found[1].number, found[1].name), (2, "solve"));
}

#[test]
fn nothing_to_do_gives_no_phases_and_a_2x2_has_one() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    assert!(phases(&solved, &MoveSequence::default())
        .unwrap()
        .is_empty());
    let start = after(PuzzleKind::Cube2, "R U");
    let found = phases(&start, &seq("U' R'")).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].number, found[0].name, found[0].moves.clone()),
        (1, "solve", 0..2)
    );
}

#[test]
fn the_phases_cover_the_solution_without_gaps() {
    let solution = "Rw U' 2F Rw' U Fw2 R U2 R'";
    let start = after(PuzzleKind::Cube5, &inverse(solution).to_string());
    let found = phases(&start, &seq(solution)).unwrap();
    let mut at = 0;
    for phase in &found {
        assert_eq!(phase.moves.start, at);
        assert!(phase.moves.end >= at);
        at = phase.moves.end;
    }
    assert_eq!(at, 9);
}

fn stage(name: &'static str, end: usize) -> SolveStage {
    SolveStage { name, end }
}

#[test]
fn engine_stages_become_numbered_phases_and_empty_ones_stay() {
    let stages = [stage("a", 0), stage("b", 4), stage("c", 4), stage("d", 9)];
    let found = from_stages(&stages, 9).unwrap();
    let got: Vec<(usize, &str, Range<usize>)> = found
        .iter()
        .map(|p| (p.number, p.name, p.moves.clone()))
        .collect();
    assert_eq!(
        got,
        vec![
            (1, "a", 0..0),
            (2, "b", 0..4),
            (3, "c", 4..4),
            (4, "d", 4..9)
        ]
    );
}

#[test]
fn stages_that_do_not_cover_the_answer_are_refused() {
    let ok = [stage("a", 3), stage("b", 9)];
    assert!(from_stages(&ok, 9).is_some());
    assert!(from_stages(&ok, 10).is_none(), "ends short");
    assert!(from_stages(&ok, 8).is_none(), "ends long");
    assert!(from_stages(&[stage("a", 5), stage("b", 3)], 3).is_none());
    assert!(from_stages(&[], 4).is_none());
    // nothing to solve: the caller falls back to the state-based answer
    assert!(from_stages(&[stage("a", 0)], 0).is_none());
}
