//! Properties of the canonical token on random states of every size and on
//! the pattern library.

use super::*;
use crate::notation::{orbit64, parse_expression};
use crate::patterns::{self, canonical_cube, detect, image, random_cube, symmetry_of};
use crate::puzzle::{destinations, Face, Move, Turn};
use std::collections::HashMap;

fn colors(cube: &Facelets) -> Vec<u8> {
    cube.stickers().iter().map(|&f| f as u8).collect()
}

fn facelets(colors: &[u8]) -> Facelets {
    Facelets::from_stickers(colors.iter().map(|&c| Face::ALL[c as usize]).collect()).unwrap()
}

/// The next number of a xorshift generator, below `bound`.
fn draw(x: &mut u64, bound: usize) -> usize {
    *x ^= *x << 13;
    *x ^= *x >> 7;
    *x ^= *x << 17;
    (*x % bound as u64) as usize
}

/// A renaming of the six colors drawn at random.
fn naming(x: &mut u64) -> [u8; 6] {
    let mut names = [0, 1, 2, 3, 4, 5];
    for i in (1..6).rev() {
        names.swap(i, draw(x, i + 1));
    }
    names
}

/// `cube` under symmetry `g`, with its colors renamed by `names`.
fn variant(cube: &Facelets, g: usize, names: [u8; 6]) -> Facelets {
    let moved = image(cube.kind(), &colors(cube), g);
    facelets(&moved.iter().map(|&c| names[c as usize]).collect::<Vec<_>>())
}

fn token(cube: &Facelets) -> Token {
    canonical(cube).unwrap_or_else(|e| panic!("{cube}: {e}"))
}

/// How many random states of each size the heavier tests use.
fn samples(kind: PuzzleKind) -> usize {
    match kind.size() {
        2 | 3 => 40,
        4 | 5 => 12,
        _ => 6,
    }
}

#[test]
fn the_solved_cube_is_the_token_of_all_a() {
    for kind in PuzzleKind::ALL {
        let solved = Facelets::solved(kind);
        let width = orbit64::token_width(kind.size()).unwrap();
        assert_eq!(token(&solved).as_str(), "A".repeat(width));
        assert_eq!(token(&solved).kind(), kind);
    }
}

/// Odd cubes show their six centers at home (frame 0); even cubes show the
/// solved URF corner, `U R F` in turning order.
fn is_standard(cube: &Facelets) -> bool {
    let n = cube.kind().size();
    let faces = cube.stickers();
    if n % 2 == 1 {
        orbit64::fixed_centers(n)
            .iter()
            .zip(Face::ALL)
            .all(|(&at, face)| faces[at] == face)
    } else {
        let slot = &orbit64::corner_slots(n).unwrap()[0];
        slot.iter()
            .zip([Face::U, Face::R, Face::F])
            .all(|(&at, face)| faces[at] == face)
    }
}

#[test]
fn the_token_is_idempotent_and_round_trips() {
    for kind in PuzzleKind::ALL {
        for seed in 1..=samples(kind) as u64 {
            let cube = random_cube(kind, seed, 25);
            let tok = token(&cube);
            assert_eq!(token(&tok.state()), tok, "{kind:?} {seed}");
            assert!(is_standard(&tok.state()), "{kind:?} {seed}");
            assert_eq!(canonical_text(tok.as_str()).unwrap(), tok);
            assert_eq!(orbit64::encode(&tok.state()).unwrap(), tok.as_str());
        }
    }
}

#[test]
fn all_48_symmetries_and_any_renaming_give_one_token() {
    let mut x = 0x2545_F491_4F6C_DD1D;
    for kind in PuzzleKind::ALL {
        for seed in 1..=samples(kind) as u64 {
            let cube = random_cube(kind, seed + 100, 30);
            let tok = token(&cube);
            for g in 0..48 {
                for names in [[0, 1, 2, 3, 4, 5], naming(&mut x), naming(&mut x)] {
                    let other = variant(&cube, g, names);
                    assert_eq!(token(&other), tok, "{kind:?} {seed} g{g} {names:?}");
                }
            }
        }
    }
}

#[test]
fn different_color_states_never_share_a_token() {
    for kind in PuzzleKind::ALL {
        let mut by_token: HashMap<Token, Vec<u8>> = HashMap::new();
        let mut by_form: HashMap<Vec<u8>, Token> = HashMap::new();
        let mut cubes = Vec::new();
        let count = match kind.size() {
            2 | 3 => 4000,
            4 | 5 => 1500,
            _ => 400,
        };
        for seed in 0..count as u64 {
            cubes.push(random_cube(kind, seed + 7, 1 + (seed % 40) as usize));
        }
        for pattern in patterns::all() {
            cubes.extend(pattern.state(kind));
        }
        for cube in &cubes {
            let tok = token(cube);
            let form = detector_form(cube);
            // The token is in the class of the cube ...
            assert_eq!(detector_form(&tok.state()), form);
            // ... and tokens and classes correspond one to one.
            assert_eq!(by_token.entry(tok.clone()).or_insert(form.clone()), &form);
            assert_eq!(by_form.entry(form).or_insert(tok.clone()), &tok);
        }
        assert_eq!(by_token.len(), by_form.len());
        println!(
            "{kind:?}: {} states, {} tokens",
            cubes.len(),
            by_token.len()
        );
    }
}

fn detector_form(cube: &Facelets) -> Vec<u8> {
    colors(&canonical_cube(cube))
}

#[test]
fn every_library_pattern_has_a_token_and_matches_its_report() {
    for pattern in patterns::all() {
        for size in pattern.sizes() {
            let kind = PuzzleKind::of_size(size).unwrap();
            let cube = pattern.state(kind).unwrap();
            let tok = token(&cube);
            // The detector recognizes the pattern from the token's state, too.
            let report = detect(&tok.state());
            assert!(
                report.matches.iter().any(|m| m.id == pattern.id),
                "{} {size}",
                pattern.id
            );
        }
    }
}

#[test]
fn a_library_pattern_keeps_its_token_in_every_notation() {
    let pattern = patterns::find("checkerboard").unwrap();
    let cube = pattern.state(PuzzleKind::Cube3).unwrap();
    let tok = token(&cube);
    assert_eq!(canonical_text(&cube.to_string()).unwrap(), tok);
    let generator = pattern.generator(3, None).unwrap().sequence().unwrap();
    assert_eq!(canonical_moves(PuzzleKind::Cube3, &generator).unwrap(), tok);
    let scheme: String = cube
        .to_string()
        .chars()
        .map(|c| match c {
            'U' => 'W',
            'R' => 'R',
            'F' => 'G',
            'D' => 'Y',
            'L' => 'O',
            _ => 'B',
        })
        .collect();
    assert_eq!(canonical_text(&scheme).unwrap(), tok);
    let written = orbit64::encode(&cube).unwrap();
    assert_eq!(canonical_text(&written).unwrap(), tok);
}

/// Pairs of half-turn words with the same colors and different pieces.
const SAME_COLORS: [(usize, &str, &str); 4] = [
    (4, "2R2 2F2", "2F2 2L2 2R2 2B2 2L2 2B2"),
    (5, "2R2 2F2", "2F2 2L2 2R2 2B2 2L2 2B2"),
    (6, "2R2 2U2", "2L2 2U2 2R2 2L2"),
    (7, "2R2 2U2", "2L2 2U2 2R2 2L2"),
];

/// Where each sticker of the solved cube ends up: the pieces after `moves`.
fn pieces_of(kind: PuzzleKind, moves: &MoveSequence) -> Vec<usize> {
    let mut ids: Vec<usize> = (0..kind.facelet_count()).collect();
    for mv in moves.moves() {
        let mut next = ids.clone();
        for (from, &to) in destinations(kind, mv).iter().enumerate() {
            next[to] = ids[from];
        }
        ids = next;
    }
    ids
}

fn pieces(kind: PuzzleKind, moves: &str) -> Vec<usize> {
    pieces_of(kind, &moves.parse().unwrap())
}

#[test]
fn different_piece_states_of_one_color_state_have_one_token() {
    for (size, a, b) in SAME_COLORS {
        let kind = PuzzleKind::of_size(size).unwrap();
        assert_ne!(pieces(kind, a), pieces(kind, b), "{size}: pieces differ");
        let per_face = kind.face_len();
        let shown = |moves: &str| -> Vec<usize> {
            pieces(kind, moves).iter().map(|id| id / per_face).collect()
        };
        assert_eq!(shown(a), shown(b), "{size}: colors agree");
        let (ta, tb) = (
            canonical_moves(kind, &a.parse().unwrap()).unwrap(),
            canonical_moves(kind, &b.parse().unwrap()).unwrap(),
        );
        assert_eq!(ta, tb, "{size}");
        assert_ne!(
            ta.as_str(),
            "A".repeat(ta.as_str().len()),
            "{size}: not solved"
        );
    }
}

/// Center-only swaps of the repository's color test: the swap moves pieces
/// and no color, so a scramble with and without it has one token.
#[test]
fn a_swap_of_equal_colored_centers_changes_no_token() {
    for (size, scramble, swap) in [
        (4, "R U F' 2R 2U' L2 D", "2R2 2U2 2R2 3R2 2U2 3R2"),
        (5, "R U2 F' 2R 2U' L2 D", "2R 4R 2R2 2U2 4R' 2R' 2U2 2R2"),
        (6, "R U F' 2R 3U' L2 D'", "3R2 2U2 3R2 4R2 2U2 4R2"),
    ] {
        let kind = PuzzleKind::of_size(size).unwrap();
        let moves = |text: &str| parse_expression(text, kind).unwrap();
        let plain = canonical_moves(kind, &moves(scramble)).unwrap();
        let traded = canonical_moves(kind, &moves(&format!("{swap} {scramble}"))).unwrap();
        assert_eq!(plain, traded, "{size}");
        assert_ne!(
            pieces_of(kind, &moves(swap)),
            pieces_of(kind, &moves("R R'")),
            "{size}: the swap moves pieces"
        );
    }
}

/// Orbit64 reads colors only, so a color state has exactly one token before
/// any symmetry is considered: reading the colors back gives the same cube.
#[test]
fn orbit64_is_a_bijection_between_tokens_and_color_states() {
    for kind in PuzzleKind::ALL {
        for seed in 1..=samples(kind) as u64 {
            let cube = random_cube(kind, seed + 500, 35);
            let tok = orbit64::encode(&cube).unwrap();
            assert_eq!(orbit64::decode(&tok).unwrap(), cube, "{kind:?} {seed}");
        }
    }
}

/// A slice turn of the 3x3x3 moves the centers; the cube is still a class of
/// the standard scheme after recoloring.
#[test]
fn a_slice_turn_that_moves_the_centers_has_a_token() {
    let kind = PuzzleKind::Cube3;
    let moved = Facelets::solved(kind)
        .apply(&parse_expression("M R U", kind).unwrap())
        .unwrap();
    assert_ne!(moved.stickers()[4], Face::U);
    let tok = token(&moved);
    assert_ne!(tok.as_str(), "A".repeat(12));
    assert!(is_standard(&tok.state()));
}

#[test]
fn unreachable_states_are_errors_not_tokens() {
    let solved = Facelets::solved(PuzzleKind::Cube3);
    let mut stickers = solved.stickers().to_vec();
    stickers.swap(8, 9);
    let twisted = Facelets::from_stickers(stickers).unwrap();
    assert!(canonical(&twisted).is_err());
    let mut stickers = solved.stickers().to_vec();
    stickers.swap(4, 13);
    let centers = Facelets::from_stickers(stickers).unwrap();
    // Swapped centers are a recoloring: the corners then give the error.
    assert!(canonical(&centers).is_err());
    assert!(matches!(canonical_text("UUUU"), Err(CanonError::Input(_))));
}

#[test]
#[ignore]
fn numbers() {
    let start = std::time::Instant::now();
    let mut count = 0;
    for pattern in patterns::all() {
        for size in pattern.sizes() {
            let cube = pattern.state(PuzzleKind::of_size(size).unwrap()).unwrap();
            token(&cube);
            count += 1;
        }
    }
    println!("library: {count} entries in {:?}", start.elapsed());
    for kind in PuzzleKind::ALL {
        let cubes: Vec<Facelets> = (0..200).map(|s| random_cube(kind, s + 1, 40)).collect();
        let start = std::time::Instant::now();
        let tokens: Vec<Token> = cubes.iter().map(token).collect();
        let each = start.elapsed() / cubes.len() as u32;
        println!(
            "{}x{}: width {}, {:?} per call, e.g. {}",
            kind.size(),
            kind.size(),
            tokens[0].as_str().len(),
            each,
            tokens[0]
        );
    }
}

/// All 3 674 160 states of the 2x2x2 (the DBL corner fixed): the number of
/// tokens equals the number of classes that Burnside's count gives from the
/// symmetry groups of the states, and every class has a token that decodes
/// into it.
#[test]
#[ignore]
fn every_2x2_state_is_counted_exactly_once_by_burnside() {
    use std::collections::HashSet;
    let kind = PuzzleKind::Cube2;
    let mut perms = Vec::new();
    for face in [Face::R, Face::U, Face::F] {
        for turn in [Turn::Clockwise, Turn::Half, Turn::CounterClockwise] {
            perms.push(destinations(kind, &Move::new(face, 1, false, turn)));
        }
    }
    let start: [u8; 24] = std::array::from_fn(|i| (i / 4) as u8);
    let mut seen: HashSet<[u8; 24]> = HashSet::from([start]);
    let mut frontier = vec![start];
    while !frontier.is_empty() {
        let mut next = Vec::new();
        for state in &frontier {
            for perm in &perms {
                let mut moved = [0u8; 24];
                for (from, &to) in perm.iter().enumerate() {
                    moved[to] = state[from];
                }
                if seen.insert(moved) {
                    next.push(moved);
                }
            }
        }
        frontier = next;
    }
    assert_eq!(seen.len(), 3_674_160);
    let mut tokens = HashSet::new();
    let mut stabilizers = 0usize;
    for state in &seen {
        let cube = facelets(state);
        tokens.insert(token(&cube));
        stabilizers += symmetry_of(kind, state).order();
    }
    assert_eq!(stabilizers % 48, 0);
    println!(
        "2x2: {} tokens, Burnside {}",
        tokens.len(),
        stabilizers / 48
    );
    assert_eq!(tokens.len(), stabilizers / 48);
}

const VECTORS: &str = include_str!("../../tests/canonical_vectors.txt");

fn rows(kind: &str) -> Vec<Vec<&'static str>> {
    VECTORS
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| line.split(' ').collect::<Vec<_>>())
        .filter(|row| row[0] == kind)
        .collect()
}

/// The shared conformance suite: vectors of flix-orbit64 v0.9.0, the
/// reference of the canonical form. Every vector must give the same token.
#[test]
fn every_reference_vector_gives_the_reference_token() {
    let (tokens, facelets) = (rows("token"), rows("facelets"));
    assert_eq!((tokens.len(), facelets.len()), (17, 76));
    for row in tokens {
        let got = canonical_text(row[2]).unwrap_or_else(|e| panic!("{}: {e}", row[2]));
        assert_eq!(got.as_str(), row[3], "token {}", row[2]);
        assert_eq!(got.kind().size().to_string(), row[1]);
    }
    for row in facelets {
        let cube: Facelets = row[2].parse().unwrap();
        assert_eq!(cube.kind().size().to_string(), row[1]);
        assert_eq!(token(&cube).as_str(), row[3], "facelets {}", row[2]);
        assert_eq!(canonical_text(row[2]).unwrap().as_str(), row[3]);
    }
}

#[test]
fn every_reference_failure_is_an_error() {
    for row in rows("fail") {
        let cube: Facelets = row[2].parse().unwrap();
        assert!(canonical(&cube).is_err(), "{}", row[2]);
    }
    for row in rows("failtext") {
        assert!(canonical_text(row[1]).is_err(), "{}", row[1]);
    }
    assert_eq!((rows("fail").len(), rows("failtext").len()), (9, 7));
}

/// The physically mirrored cube (stickers across the R/L plane, R and L
/// colors exchanged) is another state of the same class with one token.
#[test]
fn a_chiral_pattern_and_its_mirror_image_share_a_token() {
    let mirror = |cube: &Facelets| {
        let flipped = crate::patterns::mirrored(cube);
        let swapped: Vec<u8> = colors(&flipped)
            .iter()
            .map(|&c| match c {
                1 => 4,
                4 => 1,
                c => c,
            })
            .collect();
        facelets(&swapped)
    };
    for row in rows("facelets") {
        let cube: Facelets = row[2].parse().unwrap();
        assert_eq!(token(&mirror(&cube)).as_str(), row[3], "{}", row[2]);
    }
    for token_text in [
        "AP7AYhnwDQbA",
        "EKYDgqvYC5rw",
        "BJStkD4cayBhsWj6eHgDZLTP1th",
        "AQshP-X3WpOUAtv878ZKcZyT5P3So75Lb-WOh2gDJ2w",
    ] {
        let cube = orbit64::decode(token_text).unwrap();
        let other = mirror(&cube);
        assert_ne!(orbit64::encode(&other).unwrap(), token_text);
        assert_eq!(token(&other), canonical_text(token_text).unwrap());
    }
}
