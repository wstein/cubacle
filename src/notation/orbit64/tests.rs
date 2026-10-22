//! Tests of the codec: the official vectors, round trips on every size and
//! tokens that must be refused. Origin: flix-orbit64 (Apache-2.0, Werner Stein).

use super::*;
use crate::puzzle::{MoveSequence, PuzzleKind};

const VECTORS: &str = include_str!("../../../tests/fixtures/orbit64.txt");

fn vectors() -> Vec<(usize, &'static str, &'static str)> {
    VECTORS
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| {
            let mut parts = line.split(' ');
            let size = parts.next().unwrap().parse().unwrap();
            (size, parts.next().unwrap(), parts.next().unwrap())
        })
        .collect()
}

#[test]
fn every_official_vector_decodes_to_its_facelets_and_encodes_back() {
    let all = vectors();
    assert!(all.len() >= 100, "{} vectors", all.len());
    for (size, token, facelets) in all {
        let decoded = decode(token).unwrap_or_else(|e| panic!("{token}: {e}"));
        assert_eq!(decoded.kind().size(), size, "{token}");
        assert_eq!(decoded.to_string(), facelets, "{token}");
        assert_eq!(encode(&decoded).as_deref(), Ok(token), "{token}");
    }
}

#[test]
fn token_widths_are_those_of_the_format() {
    let widths: Vec<_> = (2..=7).map(|n| token_width(n).unwrap()).collect();
    assert_eq!(widths, [5, 12, 27, 43, 66, 90]);
    assert_eq!(token_width(1), None);
    assert_eq!(token_width(8), None);
    for (n, width) in (2..=7).zip(widths) {
        assert_eq!(size_of_width(width), Some(n));
    }
    assert_eq!(size_of_width(24), None);
}

#[test]
fn solved_cubes_are_runs_of_a() {
    for kind in PuzzleKind::ALL {
        let token = "A".repeat(token_width(kind.size()).unwrap());
        assert!(decode(&token).unwrap().is_solved());
        assert_eq!(encode(&Facelets::solved(kind)), Ok(token));
    }
}

#[test]
fn every_whole_cube_frame_of_an_odd_cube_round_trips() {
    for size in [3, 5, 7] {
        let width = token_width(size).unwrap();
        for (frame, &last) in ALPHABET.iter().enumerate().take(frame::FRAME_COUNT) {
            let mut token = "A".repeat(width - 1);
            token.push(char::from(last));
            let cube = decode(&token).unwrap();
            assert_eq!(
                encode(&cube).as_deref(),
                Ok(token.as_str()),
                "{size} {frame}"
            );
        }
    }
}

/// Scrambles using outer, wide and inner-layer turns of every depth.
fn scramble_of(size: usize, seed: usize) -> MoveSequence {
    let layers = size / 2;
    let mut state = seed as u64 * 2_654_435_761 + 12_345;
    let mut next = |bound: usize| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((state >> 33) as usize) % bound
    };
    let mut text = Vec::new();
    for _ in 0..40 {
        let face = ["U", "R", "F", "D", "L", "B"][next(6)];
        let depth = 1 + next(layers);
        let turn = ["", "2", "'"][next(3)];
        text.push(if depth == 1 {
            format!("{face}{turn}")
        } else {
            format!("{depth}{face}{turn}")
        });
    }
    text.join(" ").parse().unwrap()
}

#[test]
fn scrambled_cubes_of_every_size_round_trip_through_a_token() {
    for kind in PuzzleKind::ALL {
        let size = kind.size();
        for seed in 0..40 {
            let cube = Facelets::solved(kind)
                .apply(&scramble_of(size, seed))
                .unwrap();
            let token = encode(&cube).unwrap_or_else(|e| panic!("{size} seed {seed}: {e}"));
            assert_eq!(token.len(), token_width(size).unwrap());
            assert!(
                token.starts_with(|c: char| ('A'..='P').contains(&c)),
                "{token}"
            );
            assert_eq!(decode(&token).unwrap(), cube, "{size} seed {seed}");
        }
    }
}

#[test]
fn pseudo_random_tokens_of_every_size_round_trip_through_facelets() {
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    for size in 2..=7 {
        let radices = radices(size);
        for _ in 0..200 {
            // A uniformly random digit for every radix: a random point of the
            // whole state space, not only of the tokens that happen to fit.
            let mut value = Big::zero();
            for &radix in &radices {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                value.mul_add(radix, ((state >> 16) % u64::from(radix)) as u32);
            }
            let token = value.to_base64(token_width(size).unwrap()).unwrap();
            let cube = decode(&token).unwrap_or_else(|e| panic!("{token}: {e}"));
            assert_eq!(encode(&cube).as_deref(), Ok(token.as_str()), "{token}");
        }
    }
}

#[test]
fn the_alphabet_and_length_of_a_token_are_checked() {
    assert_eq!(decode(""), Err(Orbit64Error::BadLength(0)));
    assert_eq!(decode("AAAA"), Err(Orbit64Error::BadLength(4)));
    assert_eq!(decode("__________"), Err(Orbit64Error::BadLength(10)));
    assert_eq!(decode(&"A".repeat(91)), Err(Orbit64Error::BadLength(91)));
    assert!(matches!(
        decode("AAAA+"),
        Err(Orbit64Error::BadCharacter {
            position: 4,
            found: '+'
        })
    ));
    assert!(matches!(
        decode(&format!("{}é", "A".repeat(26))),
        Err(Orbit64Error::BadCharacter { found: 'é', .. })
    ));
}

#[test]
fn tokens_above_the_state_count_are_out_of_range() {
    for size in 2..=7 {
        let token = "_".repeat(token_width(size).unwrap());
        assert_eq!(decode(&token), Err(Orbit64Error::OutOfRange), "{size}");
        // The first sextet must stay in the state class.
        let token = format!("Q{}", "A".repeat(token_width(size).unwrap() - 1));
        assert_eq!(decode(&token), Err(Orbit64Error::OutOfRange), "{size}");
    }
}

#[test]
fn no_text_makes_the_decoder_panic() {
    let mut state = 0x2545_f491_4f6c_dd1du64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let pool: Vec<char> = ALPHABET
        .iter()
        .map(|&b| b as char)
        .chain("+/= \n\0éß€".chars())
        .collect();
    for _ in 0..3000 {
        let len = [0, 1, 5, 12, 27, 43, 66, 90, 91, 200][next() as usize % 10];
        let text: String = (0..len)
            .map(|_| pool[next() as usize % pool.len()])
            .collect();
        let _ = decode(&text);
        let _ = looks_like_token(&text);
    }
    for size in 2..=7 {
        let width = token_width(size).unwrap();
        for fill in ['A', 'P', 'Q', '_', 'g', 'w'] {
            let _ = decode(&fill.to_string().repeat(width));
        }
    }
}

#[test]
fn impossible_facelets_are_refused_with_a_sentence() {
    // One corner twisted in place.
    let mut stickers: Vec<Face> = Facelets::solved(PuzzleKind::Cube3).stickers().to_vec();
    let (a, b, c) = (8, 9, 20);
    let (fa, fb, fc) = (stickers[a], stickers[b], stickers[c]);
    stickers[a] = fb;
    stickers[b] = fc;
    stickers[c] = fa;
    let twisted = Facelets::from_stickers(stickers).unwrap();
    let error = encode(&twisted).unwrap_err();
    assert!(matches!(error, Orbit64Error::Impossible(_)), "{error}");
    assert!(error.to_string().contains("twists"), "{error}");

    // Two edges swapped: corner and edge parity disagree.
    let swapped = Facelets::solved(PuzzleKind::Cube3)
        .apply(&"R U R' U'".parse().unwrap())
        .unwrap();
    assert!(encode(&swapped).is_ok());
    let mut stickers: Vec<Face> = Facelets::solved(PuzzleKind::Cube3).stickers().to_vec();
    stickers.swap(5, 7);
    stickers.swap(10, 19);
    let parity = Facelets::from_stickers(stickers).unwrap();
    assert!(encode(&parity).unwrap_err().to_string().contains("parity"));

    // Fixed centres that are not a rotation.
    let mut stickers: Vec<Face> = Facelets::solved(PuzzleKind::Cube5).stickers().to_vec();
    stickers.swap(12, 37);
    let moved = Facelets::from_stickers(stickers).unwrap();
    assert!(encode(&moved).unwrap_err().to_string().contains("centres"));
}

#[test]
fn centres_with_the_wrong_colours_are_refused_on_even_cubes() {
    let mut stickers: Vec<Face> = Facelets::solved(PuzzleKind::Cube4).stickers().to_vec();
    // Swap a centre sticker with a corner sticker of another colour.
    stickers.swap(5, 16);
    let broken = Facelets::from_stickers(stickers).unwrap();
    assert!(matches!(encode(&broken), Err(Orbit64Error::Impossible(_))));
}

#[test]
fn detection_uses_width_and_alphabet() {
    assert!(looks_like_token("A4ib8aejxj4Y"));
    assert!(!looks_like_token("A4ib8aejxj4"));
    assert!(!looks_like_token("A4ib8aejxj4+"));
    assert!(looks_like_token(&"A".repeat(90)));
    assert_eq!(
        PuzzleKind::of_size(size_of_width(5).unwrap()),
        Some(PuzzleKind::Cube2)
    );
}

const ORBIT_VECTORS: &str = include_str!("../../../tests/fixtures/orbit64_orbits.txt");

fn numbers(text: &str) -> Vec<usize> {
    text.split(',').map(|n| n.parse().unwrap()).collect()
}

/// The orbit coordinates of the official vectors, as the format's reference
/// implementation lists them.
fn official_coords(fields: &[&str]) -> Coords {
    let mut coords = Coords::default();
    for field in fields {
        let (kind, rest) = field.split_once(':').unwrap();
        let mut parts = rest.split(';');
        let first = numbers(parts.next().unwrap());
        let second = parts.next().map(numbers);
        match kind {
            "C" => {
                coords.corner_perm = first;
                coords.corner_twist = second.unwrap();
            }
            "M" => {
                coords.midge_perm = first;
                coords.midge_flip = second.unwrap();
            }
            "W" => coords.wings.push(first),
            "Z" => coords.centres.push(first),
            other => panic!("unknown orbit {other}"),
        }
    }
    coords
}

/// The scrambled vectors of every size, including the 6x6x6 and 7x7x7 (which
/// have no other official vector than the solved cube), given as orbit
/// coordinates: placing the official pieces must give a cube whose token is
/// the official one, and decoding the token must give the same cube and the
/// same pieces.
#[test]
fn official_orbit_coordinates_match_their_tokens() {
    let mut seen = Vec::new();
    for line in ORBIT_VECTORS.lines().filter(|l| !l.starts_with('#')) {
        let fields: Vec<&str> = line.split(' ').collect();
        let size: usize = fields[0].parse().unwrap();
        let token = fields[1];
        let want = official_coords(&fields[2..]);
        let layout = Layout::new(size);

        let stickers = pieces::place(&layout, &want);
        let cube =
            Facelets::from_stickers(stickers.iter().map(|&i| Face::ALL[i]).collect()).unwrap();
        assert_eq!(encode(&cube).as_deref(), Ok(token), "{size} {token}");
        assert_eq!(decode(token).unwrap(), cube, "{size} {token}");
        assert_eq!(pieces::read(&layout, &stickers).unwrap(), want, "{token}");
        seen.push(size);
    }
    assert_eq!(seen, [2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7]);
}

/// Cubes of the large sizes scrambled by turns of every depth keep their
/// pieces: a token decodes to the cube, and the cube's pieces re-encode to
/// the same token after the frame of an odd cube is removed.
#[test]
fn scrambled_6x6x6_and_7x7x7_tokens_are_stable_under_more_turns() {
    for kind in [PuzzleKind::Cube6, PuzzleKind::Cube7] {
        let size = kind.size();
        let mut cube = Facelets::solved(kind);
        let mut tokens = std::collections::HashSet::new();
        for seed in 100..120 {
            cube = cube.apply(&scramble_of(size, seed)).unwrap();
            let token = encode(&cube).unwrap_or_else(|e| panic!("{size} {seed}: {e}"));
            assert_eq!(decode(&token).unwrap(), cube, "{size} {seed}");
            assert!(tokens.insert(token), "{size} {seed} repeats a state");
        }
    }
}
