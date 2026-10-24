//! Properties of the detector over the whole pattern library and random cubes.

use super::geometry::image;
use super::{canon, colours, detect};
use crate::patterns;
use crate::puzzle::{Facelets, PuzzleKind};

/// A deterministic scrambled cube: `length` random turns of random layers.
pub(super) fn random_cube(kind: PuzzleKind, seed: u64, length: usize) -> Facelets {
    let mut x = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut next = |n: usize| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        (x % n as u64) as usize
    };
    let mut text = Vec::new();
    for _ in 0..length {
        let face = ["U", "R", "F", "D", "L", "B"][next(6)];
        let depth = 1 + next(kind.size() - 1);
        let suffix = ["", "'", "2"][next(3)];
        let layer = if depth == 1 {
            String::new()
        } else {
            depth.to_string()
        };
        text.push(format!("{layer}{face}{suffix}"));
    }
    Facelets::solved(kind)
        .apply(&text.join(" ").parse().expect("valid moves"))
        .expect("the layers exist")
}

/// A state with its colours renamed by `names`.
fn renamed(state: &[u8], names: [u8; 6]) -> Vec<u8> {
    state.iter().map(|&c| names[c as usize]).collect()
}

const NAMINGS: [[u8; 6]; 4] = [
    [0, 1, 2, 3, 4, 5],
    [3, 4, 5, 0, 1, 2],
    [1, 2, 0, 4, 5, 3],
    [5, 0, 4, 2, 1, 3],
];

#[test]
fn every_library_pattern_is_found_under_all_48_symmetries_and_renamings() {
    for pattern in patterns::all() {
        for size in pattern.sizes() {
            let kind = PuzzleKind::of_size(size).expect("a cube size");
            let state = colours(&pattern.state(kind).expect("defined"));
            for g in 0..48 {
                for names in NAMINGS {
                    let seen = renamed(&image(kind, &state, g), names);
                    let report = detect(&canon::facelets(&seen));
                    assert!(
                        report.matches.iter().any(|m| m.id == pattern.id),
                        "{} {size}x{size}x{size} under symmetry {g} {names:?}: {:?}",
                        pattern.id,
                        report.matches
                    );
                }
            }
        }
    }
}

#[test]
fn a_pattern_is_named_exactly_when_given_as_defined() {
    for pattern in patterns::all() {
        for size in pattern.sizes() {
            let kind = PuzzleKind::of_size(size).expect("a cube size");
            let report = detect(&pattern.state(kind).expect("defined"));
            let own = report
                .matches
                .iter()
                .find(|m| m.id == pattern.id)
                .expect("found");
            assert_eq!(own.relation, "exactly", "{} {size}", pattern.id);
        }
    }
}

#[test]
fn the_solved_cube_is_named_on_every_size() {
    for kind in PuzzleKind::ALL {
        let report = detect(&Facelets::solved(kind));
        assert_eq!(report.matches[0].id, "solved");
    }
}

#[test]
fn the_canonical_form_is_stable_and_idempotent() {
    for kind in PuzzleKind::ALL {
        let cube = random_cube(kind, 11, 30);
        let report = detect(&cube);
        assert_eq!(detect(&report.canonical).canonical, report.canonical);
        let state = colours(&cube);
        for g in [1, 7, 19, 30, 47] {
            let moved = renamed(&image(kind, &state, g), NAMINGS[3]);
            assert_eq!(detect(&canon::facelets(&moved)).canonical, report.canonical);
        }
    }
}

#[test]
fn random_cubes_match_nothing() {
    for kind in PuzzleKind::ALL {
        for seed in 1..6 {
            let report = detect(&random_cube(kind, seed, 40));
            assert!(report.matches.is_empty(), "{kind:?} {seed}");
        }
    }
}

#[test]
fn symmetry_orders_are_invariant_for_the_library() {
    for pattern in patterns::all() {
        for size in pattern.sizes() {
            let kind = PuzzleKind::of_size(size).expect("a cube size");
            let state = colours(&pattern.state(kind).expect("defined"));
            let order = detect(&pattern.state(kind).unwrap()).symmetry.order();
            for g in [3, 20, 41] {
                let seen = renamed(&image(kind, &state, g), NAMINGS[2]);
                assert_eq!(detect(&canon::facelets(&seen)).symmetry.order(), order);
            }
        }
    }
}

#[test]
fn every_generator_builds_a_state_the_detector_names() {
    for pattern in patterns::all() {
        for generator in pattern.generators() {
            let kind = PuzzleKind::of_size(generator.size).expect("a cube size");
            let moves = generator.sequence().expect("parsable moves");
            let built = Facelets::solved(kind).apply(&moves).expect("moves fit");
            let report = detect(&built);
            assert!(
                report.matches.iter().any(|m| m.id == pattern.id),
                "{} {}x{}: {}",
                pattern.id,
                generator.size,
                generator.size,
                generator.moves
            );
        }
    }
}

#[test]
fn the_fingerprint_is_the_same_for_every_turn_and_renaming() {
    for kind in PuzzleKind::ALL {
        for seed in [3, 4] {
            let cube = random_cube(kind, seed, 25);
            let state = colours(&cube);
            let want = detect(&cube).fingerprint;
            for g in 0..48 {
                let seen = renamed(&image(kind, &state, g), NAMINGS[1 + g % 3]);
                let got = detect(&canon::facelets(&seen)).fingerprint;
                assert_eq!(got.code(), want.code(), "{kind:?} symmetry {g}");
            }
        }
    }
}

#[test]
fn known_states_have_the_expected_fingerprint() {
    let code = |name: &str, size: usize| {
        let kind = PuzzleKind::of_size(size).unwrap();
        let cube = patterns::find(name).unwrap().state(kind).unwrap();
        detect(&cube).fingerprint.code()
    };
    assert_eq!(code("superflip", 3), "C0t0 E0f12");
    assert_eq!(code("checkerboard", 3), "C0t0 E12f0(2.2.2.2.2.2)");
    assert_eq!(
        detect(&Facelets::solved(PuzzleKind::Cube3))
            .fingerprint
            .code(),
        "C0t0 E0f0"
    );
    assert_eq!(
        detect(&Facelets::solved(PuzzleKind::Cube2))
            .fingerprint
            .code(),
        "C0t0"
    );
}

#[test]
fn a_random_cube_is_displaced_nearly_everywhere() {
    let cube = random_cube(PuzzleKind::Cube3, 9, 40);
    let fp = detect(&cube).fingerprint;
    assert!(fp.orbits[0].displaced >= 5, "{}", fp.code());
    assert!(fp.orbits[1].displaced >= 8, "{}", fp.code());
}
