//! The order of a move sequence from its sticker cycles.

use std::process::Command;

use cubacle::notation::parse_expression;
use cubacle::patterns::cycles_of_moves;
use cubacle::puzzle::{sticker_cycles, Facelets, MoveSequence, PuzzleKind};

fn moves(text: &str, kind: PuzzleKind) -> MoveSequence {
    parse_expression(text, kind).unwrap()
}

fn order(text: &str, kind: PuzzleKind) -> u128 {
    sticker_cycles(kind, &moves(text, kind)).unwrap().order()
}

/// Repeats the sequence on the solved cube until the colors are solved again.
fn by_repetition(text: &str, kind: PuzzleKind, bound: u128) -> Option<u128> {
    let seq = moves(text, kind);
    let mut cube = Facelets::solved(kind);
    for n in 1..=bound {
        cube = cube.apply(&seq).unwrap();
        if cube.is_solved() {
            return Some(n);
        }
    }
    None
}

/// The first n for which n copies of the sequence move no sticker at all:
/// the moves are written out n times and their cycles are taken afresh.
fn by_sticker_repetition(text: &str, kind: PuzzleKind, bound: usize) -> Option<u128> {
    let seq = moves(text, kind);
    (1..=bound)
        .find(|&n| {
            let all: Vec<_> = seq
                .moves()
                .iter()
                .copied()
                .cycle()
                .take(seq.moves().len() * n)
                .collect();
            let cycles = sticker_cycles(kind, &MoveSequence::new(all)).unwrap();
            cycles.orbits.iter().all(|orbit| orbit.lengths.is_empty())
        })
        .map(|n| n as u128)
}

#[test]
fn known_orders_on_the_3x3x3_match_repetition() {
    let kind = PuzzleKind::Cube3;
    for (text, expected) in [("", 1), ("R", 4), ("R2", 2), ("R U", 105), ("R U R' U'", 6)] {
        assert_eq!(order(text, kind), expected, "{text}");
        assert_eq!(by_repetition(text, kind, 200), Some(expected), "{text}");
    }
}

#[test]
fn every_size_agrees_with_repetition() {
    let sequences = ["R U", "R U2 F'", "R U R' U'", "F2 D' L", "R L' U D' F B'"];
    for kind in PuzzleKind::ALL {
        for text in sequences {
            let n = order(text, kind);
            assert_eq!(
                by_sticker_repetition(text, kind, 1300),
                Some(n),
                "{kind:?} {text}"
            );
            // colors cannot tell center stickers apart, so they may come
            // back sooner, never later
            let colors = by_repetition(text, kind, 1300).unwrap();
            assert_eq!(n % colors, 0, "{kind:?} {text}");
        }
    }
}

#[test]
fn slices_on_the_5x5x5_agree_with_repetition() {
    let kind = PuzzleKind::Cube5;
    for text in ["2R U", "R 2U' 3F", "Rw Uw'", "2R2 2U2 2F2", "R U 3L' 2D"] {
        let n = order(text, kind);
        assert_eq!(by_sticker_repetition(text, kind, 3000), Some(n), "{text}");
        assert_eq!(n % by_repetition(text, kind, 3000).unwrap(), 0, "{text}");
    }
}

#[test]
fn the_3x3x3_sticker_order_equals_the_cubie_order() {
    let kind = PuzzleKind::Cube3;
    for text in [
        "R",
        "R U",
        "R U R' U'",
        "R U2 D' B D'",
        "F R U' R' U' R U R' F' R U R' U' R' F R F'",
        "M' U M U2",
        "(R U)3 F",
        "R2 L2 U2 D2",
    ] {
        let cubie = cycles_of_moves(kind, &moves(text, kind)).unwrap();
        assert_eq!(order(text, kind), cubie.order.unwrap().whole, "{text}");
    }
}

#[test]
fn an_orbit_order_divides_the_whole() {
    let kind = PuzzleKind::Cube4;
    let cycles = sticker_cycles(kind, &moves("R U 2R' Fw", kind)).unwrap();
    for orbit in &cycles.orbits {
        assert_eq!(cycles.order() % orbit.order(), 0);
    }
}

fn cubacle(args: &[&str]) -> (Option<i32>, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "XDG_CONFIG_HOME",
            std::env::temp_dir().join("cubacle-test-xdg"),
        )
        .output()
        .expect("cubacle runs");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn the_command_prints_the_order() {
    let (code, out, _) = cubacle(&["order", "R", "U"]);
    assert_eq!((code, out.as_str()), (Some(0), "order 105\n"));
    let (_, out, _) = cubacle(&["order", "--size", "5", "2R U"]);
    assert!(out.starts_with("order "), "{out}");
}

#[test]
fn verbose_lists_the_cycles_per_orbit() {
    let (code, out, _) = cubacle(&["order", "-v", "R U"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        out,
        "order 105\ncorners: 3 15 (order 15)\nedges: 7x2 (order 7)\ncenters: in place (order 1)\n"
    );
}

#[test]
fn bad_input_exits_with_one() {
    let (code, out, err) = cubacle(&["order", "R Q"]);
    assert_eq!(code, Some(1));
    assert!(out.is_empty());
    assert!(err.starts_with("error:"), "{err}");
    assert_eq!(cubacle(&["order", "--size", "9", "R"]).0, Some(1));
    assert_eq!(cubacle(&["order", "--size", "3", "3R"]).0, Some(1));
}
