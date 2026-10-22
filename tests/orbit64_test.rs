//! Orbit64 through the public API: the official vectors, input detection and
//! round trips of random states. Vectors: flix-orbit64 (Apache-2.0, Werner Stein).

use cubacle::generate;
use cubacle::notation::orbit64::{decode, encode, token_width, Orbit64Error};
use cubacle::notation::parse_cube;
use cubacle::puzzle::{Facelets, PuzzleKind};

const VECTORS: &str = include_str!("fixtures/orbit64.txt");

fn vectors() -> impl Iterator<Item = (usize, &'static str, &'static str)> {
    VECTORS
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split(' ').collect();
            (parts[0].parse().unwrap(), parts[1], parts[2])
        })
}

#[test]
fn parse_cube_reads_every_official_token_by_its_length() {
    let mut per_size = [0usize; 6];
    for (size, token, facelets) in vectors() {
        let cube = parse_cube(token, None).unwrap();
        assert_eq!(cube.kind().size(), size, "{token}");
        assert_eq!(cube.to_string(), facelets, "{token}");
        assert_eq!(encode(&cube).as_deref(), Ok(token));
        per_size[size - 2] += 1;
    }
    // 20 official examples each for 2x2x2 to 5x5x5, and the solved 6x6x6 and 7x7x7.
    assert!(
        per_size[..4].iter().all(|&count| count >= 20),
        "{per_size:?}"
    );
    assert_eq!(per_size[4..], [1, 1]);
}

#[test]
fn random_states_round_trip_through_tokens() {
    for kind in [PuzzleKind::Cube2, PuzzleKind::Cube3, PuzzleKind::Cube5] {
        for _ in 0..100 {
            let cube = generate::random_state(kind).unwrap();
            let token = encode(&cube).unwrap();
            assert_eq!(token.len(), token_width(kind.size()).unwrap());
            assert_eq!(decode(&token).unwrap(), cube);
        }
    }
}

#[test]
fn scrambled_big_cubes_round_trip_through_tokens() {
    for kind in [PuzzleKind::Cube4, PuzzleKind::Cube6, PuzzleKind::Cube7] {
        let scramble = "R U 2R 2U' 3F' Rw2 Lw B2 D' 2L 3D2 Uw' F 2B' R2 3U Fw";
        let cube = Facelets::solved(kind)
            .apply(&scramble.parse().unwrap())
            .unwrap();
        assert!(!cube.is_solved());
        let token = encode(&cube).unwrap();
        assert_eq!(decode(&token).unwrap(), cube, "{kind:?}");
    }
}

#[test]
fn bad_tokens_are_errors_not_panics() {
    assert_eq!(decode("AAAA"), Err(Orbit64Error::BadLength(4)));
    assert_eq!(decode(&"_".repeat(90)), Err(Orbit64Error::OutOfRange));
    assert!(parse_cube(&"-".repeat(66), None).is_err());
    assert!(decode(&format!("{}!", "A".repeat(11))).is_err());
}
