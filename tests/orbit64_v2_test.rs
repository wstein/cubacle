//! Orbit64 V2 through the public API: the reference vectors of the format, the
//! V1 state tokens read as V2, and malformed input.
//! Vectors: flix-orbit64 (Apache-2.0, Werner Stein).

use cubacle::notation::orbit64::v2::{decode, encode, entries, Cube, Model, Pose, V2Error};
use cubacle::notation::orbit64::{decode as decode_v1, encode as encode_v1};

const VECTORS: &str = include_str!("fixtures/orbit64_v2.txt");
const V1_VECTORS: &str = include_str!("fixtures/orbit64.txt");
const V1_ORBITS: &str = include_str!("fixtures/orbit64_orbits.txt");

/// The state text `key=v,v,...;key=...` of a vector, as a cube.
fn parse_cube(text: &str) -> Cube {
    let mut cube = Cube::default();
    let mut wings = Vec::new();
    let mut centers = Vec::new();
    for field in text.split(';').filter(|field| !field.is_empty()) {
        let (key, values) = field.split_once('=').unwrap();
        let values: Vec<usize> = values
            .split(',')
            .filter(|v| !v.is_empty())
            .map(|v| v.parse().unwrap())
            .collect();
        match key {
            "cp" => cube.corner_pieces = values,
            "ct" => cube.corner_twists = values,
            "mp" => cube.midge_pieces = values,
            "mf" => cube.midge_flips = values,
            "fr" => cube.rotations = values,
            "fm" => cube.frame = values[0],
            _ if key.starts_with('w') => wings.push((key[1..].parse::<usize>().unwrap(), values)),
            _ if key.starts_with('z') => centers.push((key[1..].parse::<usize>().unwrap(), values)),
            _ => panic!("unknown key {key}"),
        }
    }
    wings.sort();
    centers.sort();
    cube.wings = wings.into_iter().map(|(_, v)| v).collect();
    cube.centers = centers.into_iter().map(|(_, v)| v).collect();
    cube
}

fn vectors() -> Vec<(usize, Cube, &'static str)> {
    VECTORS
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split(' ').collect();
            (parts[0].parse().unwrap(), parse_cube(parts[1]), parts[2])
        })
        .collect()
}

/// The color framed entry of a cube size.
fn framed_color_id(n: usize) -> usize {
    entries()
        .iter()
        .find(|e| e.n == n && e.model == Model::Color && e.pose == Pose::Framed)
        .unwrap()
        .id
}

#[test]
fn every_reference_vector_encodes_to_its_token() {
    let vectors = vectors();
    assert_eq!(vectors.len(), 195);
    for (id, cube, token) in &vectors {
        assert_eq!(encode(*id, cube).as_deref(), Ok(*token), "entry {id}");
    }
}

#[test]
fn every_reference_vector_decodes_to_its_state() {
    for (id, cube, token) in vectors() {
        let (entry, decoded) = decode(token).unwrap_or_else(|e| panic!("{token}: {e}"));
        assert_eq!(entry.id, id, "{token}");
        assert_eq!(entry.length, token.len());
        assert_eq!(decoded, cube, "{token}");
    }
}

#[test]
fn the_vectors_cover_every_entry() {
    let mut seen = vec![0usize; entries().len()];
    for (id, _, _) in vectors() {
        seen[id] += 1;
    }
    assert!(seen.iter().all(|&count| count >= 2), "{seen:?}");
}

#[test]
fn the_initial_registry_has_the_lengths_of_the_format() {
    #[rustfmt::skip]
    let lengths = [
        4, 5, 11, 12, 13, 14, 26, 27, 30, 31, 42, 42, 52, 53, 65, 66, 83, 83, 89, 90, 118, 118,
        121, 122, 161, 161, 154, 155, 209, 210, 194, 195, 265, 266, 236, 237, 326, 327, 285, 285,
        395, 396, 335, 336, 469, 470, 392, 393, 551, 552, 451, 452, 639, 639,
    ];
    let found: Vec<usize> = entries().iter().map(|e| e.length).collect();
    assert_eq!(found, lengths);
    for (index, entry) in entries().iter().enumerate() {
        assert_eq!(entry.id, index);
    }
}

#[test]
fn a_v1_state_token_reads_as_the_color_framed_v2_entry() {
    let mut checked = 0;
    for line in V1_ORBITS
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let mut fields = line.split(' ');
        let n: usize = fields.next().unwrap().parse().unwrap();
        let token = fields.next().unwrap();
        if n == 5 {
            continue;
        }
        let (entry, cube) = decode(token).unwrap();
        assert_eq!((entry.id, entry.n), (framed_color_id(n), n), "{token}");
        assert_eq!((entry.model, entry.pose), (Model::Color, Pose::Framed));
        let (mut wings, mut centers) = (Vec::new(), Vec::new());
        for field in fields {
            let (tag, body) = field.split_once(':').unwrap();
            let orbit = |part: &str| -> Vec<usize> {
                part.split(',').map(|v| v.parse().unwrap()).collect()
            };
            match tag {
                "C" => {
                    let (pieces, twists) = body.split_once(';').unwrap();
                    assert_eq!(cube.corner_pieces, orbit(pieces), "{token}");
                    assert_eq!(cube.corner_twists, orbit(twists), "{token}");
                }
                "M" => {
                    let (pieces, flips) = body.split_once(';').unwrap();
                    assert_eq!(cube.midge_pieces, orbit(pieces), "{token}");
                    assert_eq!(cube.midge_flips, orbit(flips), "{token}");
                }
                "W" => wings.push(orbit(body)),
                _ => centers.push(orbit(body)),
            }
        }
        assert_eq!(cube.wings, wings, "{token}");
        assert_eq!(cube.centers, centers, "{token}");
        assert_eq!(encode(entry.id, &cube).as_deref(), Ok(token));
        checked += 1;
    }
    assert_eq!(checked, 10);
}

#[test]
fn the_v1_vectors_of_the_even_and_odd_sizes_round_trip_as_v2() {
    let mut per_size = [0usize; 6];
    for line in V1_VECTORS
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let mut fields = line.split(' ');
        let n: usize = fields.next().unwrap().parse().unwrap();
        let token = fields.next().unwrap();
        if n == 5 {
            // A V1 5x5x5 token has 43 characters, where V2 has no entry.
            assert_eq!(decode(token), Err(V2Error::UnknownToken));
            continue;
        }
        let (entry, cube) = decode(token).unwrap();
        assert_eq!(entry.id, framed_color_id(n), "{token}");
        assert_eq!(encode(entry.id, &cube).as_deref(), Ok(token));
        // The V1 decoder reads the same token as a cube of the same size.
        assert_eq!(decode_v1(token).map(|f| f.kind().size()), Ok(n));
        per_size[n - 2] += 1;
    }
    assert!(
        per_size[..3].iter().all(|&count| count >= 20),
        "{per_size:?}"
    );
    assert_eq!(per_size[4..], [1, 1]);
    let solved = encode_v1(&decode_v1(&"A".repeat(12)).unwrap());
    assert_eq!(solved.as_deref(), Ok("AAAAAAAAAAAA"));
}

#[test]
fn malformed_tokens_are_errors_not_panics() {
    assert_eq!(decode(""), Err(V2Error::UnknownToken));
    assert_eq!(decode("A"), Err(V2Error::UnknownToken));
    assert_eq!(decode(&"A".repeat(43)), Err(V2Error::UnknownToken));
    assert_eq!(decode(&"A".repeat(100_000)), Err(V2Error::UnknownToken));
    assert_eq!(
        decode("AAA+"),
        Err(V2Error::BadCharacter {
            position: 3,
            found: '+'
        })
    );
    assert_eq!(
        decode("AAA\u{e9}"),
        Err(V2Error::BadCharacter {
            position: 3,
            found: '\u{e9}'
        })
    );
    // 4 characters hold 3,674,160 states of 2^24 values: the rest is unallocated.
    assert_eq!(decode("____"), Err(V2Error::UnknownToken));
    // The last 5-character value is above the 2x2x2 framed range as well.
    assert_eq!(decode("_____"), Err(V2Error::UnknownToken));
    // No entry has a 6-character token, and a 42-character one needs both ranges.
    assert_eq!(decode("AAAAAA"), Err(V2Error::UnknownToken));
    assert_eq!(decode(&"_".repeat(42)), Err(V2Error::UnknownToken));
}

#[test]
fn malformed_states_are_errors_not_panics() {
    let solved = |n, model| Cube::solved(n, model);
    let color3 = framed_color_id(3);
    let ok = solved(3, Model::Color);
    assert_eq!(encode(color3, &ok).as_deref(), Ok("AAAAAAAAAAAA"));
    assert_eq!(encode(54, &ok), Err(V2Error::UnknownEntry(54)));
    assert_eq!(
        encode(usize::MAX, &ok),
        Err(V2Error::UnknownEntry(usize::MAX))
    );

    // Short vectors, a missing component and a wrong entry.
    for cut in 0..8 {
        let mut cube = ok.clone();
        cube.corner_pieces.truncate(cut);
        assert!(matches!(encode(color3, &cube), Err(V2Error::Invalid(_))));
    }
    assert!(encode(color3, &Cube::default()).is_err());
    assert!(encode(framed_color_id(4), &ok).is_err());
    assert!(encode(framed_color_id(2), &ok).is_err());

    type Damage = fn(&mut Cube);
    let broken: [(&str, Damage); 9] = [
        ("twist sum", |c| c.corner_twists[0] = 1),
        ("twist range", |c| c.corner_twists[0] = 3),
        ("flip sum", |c| c.midge_flips[0] = 1),
        ("duplicate piece", |c| c.midge_pieces[0] = 1),
        ("piece range", |c| c.corner_pieces[0] = 99),
        ("parity", |c| c.corner_pieces.swap(0, 1)),
        ("frame", |c| c.frame = 24),
        ("short flips", |c| c.midge_flips.truncate(11)),
        ("extra wing", |c| c.wings.push((0..24).collect())),
    ];
    for (what, damage) in broken {
        let mut cube = ok.clone();
        damage(&mut cube);
        assert!(
            matches!(encode(color3, &cube), Err(V2Error::Invalid(_))),
            "{what}"
        );
    }

    // Unbalanced colors, wrong parities of a picture cube, DBL not home.
    let mut cube = solved(5, Model::Color);
    cube.centers[0][0] = 1;
    assert!(encode(framed_color_id(5), &cube).is_err());
    let picture = entries()
        .iter()
        .find(|e| e.n == 5 && e.model == Model::Picture && e.pose == Pose::Canonical)
        .unwrap();
    let mut cube = solved(5, Model::Picture);
    assert!(encode(picture.id, &cube).is_ok());
    cube.centers[0].swap(0, 1);
    assert!(encode(picture.id, &cube).is_err());
    let mut cube = solved(5, Model::Picture);
    cube.rotations[0] = 1;
    assert!(encode(picture.id, &cube).is_err());
    let canonical4 = entries()
        .iter()
        .find(|e| e.n == 4 && e.pose == Pose::Canonical)
        .unwrap();
    let mut cube = solved(4, Model::Color);
    assert!(encode(canonical4.id, &cube).is_ok());
    cube.corner_twists[6] = 1;
    cube.corner_twists[0] = 2;
    assert!(encode(canonical4.id, &cube).is_err());
    let mut cube = solved(4, Model::Color);
    cube.corner_pieces.swap(6, 7);
    cube.corner_pieces.swap(0, 1);
    assert!(encode(canonical4.id, &cube).is_err());
}

#[test]
fn solved_states_are_the_zero_ranks() {
    for entry in entries() {
        let token = encode(entry.id, &Cube::solved(entry.n, entry.model)).unwrap();
        assert_eq!(token.len(), entry.length);
        let (found, cube) = decode(&token).unwrap();
        assert_eq!(found.id, entry.id);
        assert_eq!(cube, Cube::solved(entry.n, entry.model));
        if ![11, 17, 21, 25, 39, 53].contains(&entry.id) {
            assert!(token.bytes().all(|b| b == b'A'), "entry {}", entry.id);
        }
    }
}
