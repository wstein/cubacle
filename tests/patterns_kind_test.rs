//! Pattern kinds: the regularity class of invented cube states.

use cubacle::patterns::{self, PatternKind};
use cubacle::puzzle::{Facelets, PuzzleKind};

fn cube(kind: PuzzleKind, moves: &str) -> Facelets {
    Facelets::solved(kind)
        .apply(&moves.parse().expect("valid moves"))
        .expect("moves exist on this cube")
}

#[test]
fn a_solved_cube_is_solved_on_every_size() {
    for kind in PuzzleKind::ALL {
        let measures = patterns::measures(&Facelets::solved(kind));
        assert_eq!(measures.kind(), PatternKind::Solved);
        assert_eq!(measures.moved, 0);
        assert_eq!(measures.fixed, 1.0);
        assert_eq!(patterns::kind(&Facelets::solved(kind)), PatternKind::Solved);
    }
}

#[test]
fn a_single_turn_is_mostly_in_place_with_a_symmetric_defect() {
    let measures = patterns::measures(&cube(PuzzleKind::Cube3, "R"));
    assert_eq!(measures.kind(), PatternKind::Mostly);
    assert_eq!(measures.moved, 8);
    assert!(measures.fixed > 0.7 && measures.mask_symmetry >= 0.5);
}

#[test]
fn a_few_moved_pieces_of_a_big_cube_are_few() {
    // 19 of the 218 pieces of a 7x7x7 (under 15 %).
    let m = patterns::measures(&cube(PuzzleKind::Cube7, "R U R' U'"));
    assert_eq!((m.moved, m.pieces), (19, 218));
    assert_eq!(m.kind(), PatternKind::Few);
    // The same sequence on a 3x3x3 moves 7 of 26.
    let m = patterns::measures(&cube(PuzzleKind::Cube3, "R U R' U'"));
    assert_eq!((m.moved, m.pieces), (7, 26));
    assert_eq!(m.kind(), PatternKind::Irreg);
}

#[test]
fn a_short_scramble_has_no_symmetry_and_is_irregular() {
    let m = patterns::measures(&cube(PuzzleKind::Cube3, "R U R' U'"));
    assert_eq!(m.kind(), PatternKind::Irreg);
    assert_eq!(m.symmetry_order, 1);
    assert_eq!(m.mask_symmetry, 0.0);
}

#[test]
fn the_library_checkerboard_is_symmetric() {
    let state = patterns::find("checkerboard")
        .and_then(|p| p.state(PuzzleKind::Cube3))
        .expect("the library has it");
    assert_eq!(patterns::kind(&state), PatternKind::Sym);
}

#[test]
fn no_library_pattern_is_irregular_or_solved() {
    for pattern in patterns::all() {
        for size in pattern.sizes() {
            let state = pattern.state(PuzzleKind::of_size(size).unwrap()).unwrap();
            let kind = patterns::kind(&state);
            assert!(
                !matches!(kind, PatternKind::Irreg | PatternKind::Solved),
                "{} on {size}x{size}x{size} is {kind}",
                pattern.id
            );
        }
    }
}

/// A deterministic scramble: `length` turns of random layers.
fn scrambled(kind: PuzzleKind, seed: u64, length: usize) -> Facelets {
    let mut x = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut next = |n: usize| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        (x % n as u64) as usize
    };
    let moves: Vec<String> = (0..length)
        .map(|_| {
            let face = ["U", "R", "F", "D", "L", "B"][next(6)];
            let depth = 1 + next(kind.size() - 1);
            let layer = if depth == 1 {
                String::new()
            } else {
                depth.to_string()
            };
            format!("{layer}{face}{}", ["", "'", "2"][next(3)])
        })
        .collect();
    cube(kind, &moves.join(" "))
}

#[test]
fn most_short_scrambles_are_irregular() {
    for size in [3, 4, 5] {
        let kind = PuzzleKind::of_size(size).unwrap();
        let irregular = (1..=60)
            .map(|seed| scrambled(kind, seed, 4 + seed as usize % 9))
            .filter(|c| patterns::kind(c) == PatternKind::Irreg)
            .count();
        assert!(irregular >= 48, "{size}x{size}x{size}: {irregular} of 60");
    }
}

#[test]
fn kind_names_round_trip_and_ignore_case() {
    for kind in PatternKind::ALL {
        assert_eq!(kind.name().parse(), Ok(kind));
        assert_eq!(kind.name().to_lowercase().parse(), Ok(kind));
        assert_eq!(kind.to_string(), kind.name());
    }
    assert!("bogus".parse::<PatternKind>().is_err());
}
