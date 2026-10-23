//! SiGN and WCA spellings of the same moves, read and written at the boundary.

use std::sync::Arc;

use cubacle::engines::four::FourSolver;
use cubacle::engines::three::Min2PhaseSolver;
use cubacle::engines::two::PocketSolver;
use cubacle::generate;
use cubacle::notation::{parse_moves, Notation};
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind, SolveOptions, Solver};

/// (WCA, SiGN) pairs; every pair names one physical move.
const PAIRS: [(&str, &str); 22] = [
    ("R", "R"),
    ("R2", "R2"),
    ("R'", "R'"),
    ("2R", "2R"),
    ("3U2", "3U2"),
    ("Rw", "r"),
    ("Rw2", "r2"),
    ("Rw'", "r'"),
    ("2Rw", "r"),
    ("3Rw", "3r"),
    ("3Lw2", "3l2"),
    ("4Fw'", "4f'"),
    ("5Dw", "5d"),
    ("6Bw2", "6b2"),
    ("Uw", "u"),
    ("Dw'", "d'"),
    ("Fw2", "f2"),
    ("Bw", "b"),
    ("Lw", "l"),
    ("6R", "6R"),
    ("2L'", "2L'"),
    ("3F2", "3F2"),
];

fn seq(text: &str) -> MoveSequence {
    parse_moves(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

#[test]
fn the_pairs_read_alike_and_write_each_way() {
    for (wca, sign) in PAIRS {
        let (a, b) = (seq(wca), seq(sign));
        assert_eq!(a, b, "{wca} / {sign}");
        assert_eq!(Notation::Sign.write(&a), sign.replace("2r", "r"));
        // `2Rw` is written back in the canonical `Rw`
        assert_eq!(Notation::Wca.write(&b), a.to_string());
    }
}

#[test]
fn every_depth_of_every_size_translates() {
    for size in 2..=7usize {
        for letter in ["U", "R", "F", "D", "L", "B"] {
            for turn in ["", "2", "'"] {
                for depth in 1..size {
                    let single = if depth == 1 {
                        format!("{letter}{turn}")
                    } else {
                        format!("{depth}{letter}{turn}")
                    };
                    let wide = match depth {
                        1 => None,
                        2 => Some((
                            format!("{letter}w{turn}"),
                            format!("{}{turn}", letter.to_lowercase()),
                        )),
                        _ => Some((
                            format!("{depth}{letter}w{turn}"),
                            format!("{depth}{}{turn}", letter.to_lowercase()),
                        )),
                    };
                    assert_eq!(Notation::Sign.write(&seq(&single)), single);
                    if let Some((w, s)) = wide {
                        let mv = seq(&w);
                        assert!(mv.fits(PuzzleKind::of_size(size).unwrap()));
                        assert_eq!(Notation::Sign.write(&mv), s, "size {size}: {w}");
                        assert_eq!(seq(&s), mv, "size {size}: {s}");
                        assert_eq!(Notation::Wca.write(&seq(&s)), w);
                    }
                }
            }
        }
    }
}

#[test]
fn mixed_spellings_in_one_sequence_are_read() {
    assert_eq!(seq("R u 3r 2R Rw f'"), seq("R Uw 3Rw 2R Rw Fw'"));
}

#[test]
fn bad_tokens_are_rejected_with_a_reason() {
    for text in [
        "rw", "3rw", "1r", "0r", "r4", "3-2r", "0-2r", "x", "R U r!", "r''",
    ] {
        assert!(parse_moves(text).is_err(), "{text}");
    }
    let error = parse_moves("R 5-2r").unwrap_err().to_string();
    assert!(error.contains("5-2r") && error.contains("range"), "{error}");
    let error = parse_moves("rw").unwrap_err().to_string();
    assert!(error.contains("wide"), "{error}");
}

/// Reads the solution in WCA, writes it in SiGN, reads that, compares, and
/// checks that both solve the scrambled cube.
fn round_trip(cube: &Facelets, solution: &MoveSequence) {
    let sign = Notation::Sign.write(solution);
    let back = parse_moves(&sign).expect("sign reads back");
    assert_eq!(&back, solution);
    assert_eq!(Notation::Wca.write(&back), solution.to_string());
    // (the 2x2x2 engine may end in a turned cube, so compare, not is_solved)
    assert_eq!(cube.apply(&back), cube.apply(solution));
    // the reverse direction: SiGN text -> WCA text -> SiGN text
    let wca = Notation::Wca.write(&parse_moves(&sign).unwrap());
    assert_eq!(Notation::Sign.write(&parse_moves(&wca).unwrap()), sign);
}

fn solve_and_round_trip(kind: PuzzleKind, solver: &dyn Solver) {
    let scramble = generate::scramble(kind, 30);
    let cube = Facelets::solved(kind).apply(&scramble).unwrap();
    let result = solver.solve(&cube, &SolveOptions::default());
    round_trip(&cube, result.solution().expect("a solution"));
}

#[test]
fn engine_solutions_of_the_small_sizes_round_trip() {
    let three = Arc::new(Min2PhaseSolver::shared());
    solve_and_round_trip(PuzzleKind::Cube2, &PocketSolver::new());
    solve_and_round_trip(PuzzleKind::Cube3, three.as_ref());
    solve_and_round_trip(PuzzleKind::Cube4, &FourSolver::new(three));
}

/// Recorded `cubacle scramble` and `cubacle solve` outputs (5x5x5 to 7x7x7).
const SCRAMBLE5: &str = "U' R 2B' F 2U2 2R2 B2 U' 2F2 2L2 2R' 2D' B2 L2 R' 2F' 2U' 2L' 2D' B2 2D U2 L' R2 2F R' 2U2 2F L' 2B U 2F2 R' 2U2 2L 2R' 2D' 2F2 D 2B2";
const SCRAMBLE6: &str = "2U2 R 3B2 3R2 D R' 3U 2L 2B 2U' L' 3R' 3D 2R2 U 3R' 3F2 L2 3D 3R' B L' 2F2 3U' B L2 D 3B' 3U2 L D2 2L 3B2 2L2 U' L 3F U' 3F2 3L";
const SCRAMBLE7: &str = "F' 2U' 2R2 U' 3R 2U2 3L2 3F 2R F U' 3R F2 U2 2R' 3D2 B2 2U2 2R 2F 3L2 D2 2B D' 2R' 2D' 3F' 2U2 3L2 3F2 2U' B' 2D 2R 2B R D U2 B2 3U2";
const SOLUTION5: &str = "L' Dw F' Uw Lw U' Bw2 D' U2 Fw D U2 Lw' F U' B' Lw' Bw2 L' Fw2 B D 2F2 L' Fw2 Dw2 Lw2 R Fw2 Uw2 Fw2 Dw2 R' U2 B2 Rw2 Fw2 B2 Rw2 D' L2 D' B2 D' Lw2 Fw2 Rw2 D' Fw2 D B2 R' L' F' L' B D' F2 D F2 B2 U2 F2 B2 L B' D2 F";
const SOLUTION6: &str = "Lw' Uw2 F' 3Lw 3Fw2 Rw' Fw Dw2 3Dw' 3Bw U2 Dw F Fw R2 Uw Lw Uw2 Bw' Dw F' Rw2 D' 3Lw Uw2 F 3Rw U' 3Fw2 Uw2 Rw' Uw2 B Dw2 Rw' U2 F Rw L2 3F2 L 3Uw2 L 3Bw2 Dw2 3Lw2 D' 3Lw2 B' Dw2 Lw2 Dw2 Bw2 Rw2 F D 3Lw2 U' 3Rw' F2 B2 3Rw F2 B2 3Rw U F 3Rw2 B' 3Rw2 U F' 3Uw2 B D F' B 3U2 R' B L2 D' F' Dw2 U B Lw2 F2 L' Fw2 Uw2 L' Uw2 L' D2 L' Dw2 U2 Fw2 U' R2 Bw2 Rw2 U' F2 R2 L2 U2 Bw2 Fw2 Rw2 D' Bw2 R2 F U2 R F L U' B L B2 U2 F2 L U2 R B2 R2 D2 L D' F";
const SOLUTION7: &str = "U 3Uw 3Lw' B' 3Rw 3Bw 3Uw 3Lw 3Bw2 3Uw Bw2 3Lw F2 3Uw2 D 3Rw2 Fw2 L' Dw' 3Rw Uw' 2B2 U Dw Fw B' D2 Bw Uw 3Rw U 3Uw2 3Lw' 3Rw2 B' D' 3Fw2 3L B 3Lw2 F' 3Rw2 U' 3Dw2 B 3Bw2 Lw' 3Fw2 Bw2 Lw' Rw2 F B' D Lw B' Dw2 Lw Fw2 3Uw2 Bw2 L2 3Fw2 L 3Uw2 R' Uw2 Lw2 U Fw2 3R2 U2 3R2 U 3Bw2 3Lw2 F Rw2 3Dw2 Lw2 Dw2 3Lw2 U2 B' Rw2 3Dw2 Rw2 B' 3Dw2 3Uw2 F' R D' 3Lw2 3Bw2 R' 3Dw2 D' F B2 D' L2 U' 3Bw2 U 3Lw2 R2 D 3Lw2 D 3Dw2 3Fw2 B2 3Dw2 3Bw2 D2 B2 U2 L' D2 3Bw2 R' 3Dw2 3Bw2 Rw' B2 F2 Lw D2 U2 Lw F L' D Fw2 B U' L' B' Dw2 L2 U2 Rw2 Fw2 B U2 L2 Dw2 Lw2 F Uw2 F Fw2 R2 U2 L Dw2 Fw2 L2 Dw2 Fw2 R' Uw2 D2 R' Uw2 D2 B2 L F L2 D B' R U' L2 U2 B2 U' B2 U D2 F2 L2 U' L2 R";

#[test]
fn recorded_big_solutions_round_trip() {
    for (size, scramble, solution) in [
        (5, SCRAMBLE5, SOLUTION5),
        (6, SCRAMBLE6, SOLUTION6),
        (7, SCRAMBLE7, SOLUTION7),
    ] {
        let moves = seq(solution);
        let cube = Facelets::solved(PuzzleKind::of_size(size).unwrap())
            .apply(&seq(scramble))
            .unwrap();
        assert!(cube.apply(&moves).unwrap().is_solved(), "{size}x{size}");
        // the scramble too reads and writes in both notations
        let scr = seq(scramble);
        assert_eq!(seq(&Notation::Sign.write(&scr)), scr);
        let sign = Notation::Sign.write(&moves);
        assert_ne!(sign, solution, "big solutions have wide turns");
        assert!(!sign.contains('w'));
        assert_eq!(seq(&sign), moves);
        assert_eq!(Notation::Wca.write(&seq(&sign)), solution);
    }
}
