//! Orbit64 V2 facelet bridge through the public API: the independent facelet
//! vectors of the format in both directions, and facelet strings that are no
//! cube, each with its exact error.
//! Vectors: flix-orbit64 (Apache-2.0, Werner Stein).

use cubacle::notation::orbit64::v2::facelets::{decode, encode, from_facelets, to_facelets};
use cubacle::notation::orbit64::v2::{entries, entry, Cube, Model, Pose, V2Error};

const ROWS: &str = include_str!("../fixtures/orbit64_v2_facelets.txt");

struct Row {
    id: usize,
    moves: &'static str,
    facelets: &'static str,
    token: &'static str,
    /// The facelets already canonical, or the canonical reading of them.
    canonical: &'static str,
}

fn rows() -> Vec<Row> {
    ROWS.lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| {
            let f: Vec<&str> = line.split('|').collect();
            assert_eq!(f.len(), 5, "{line}");
            Row {
                id: f[0].parse().unwrap(),
                moves: f[1],
                facelets: f[2],
                token: f[3],
                canonical: if f[4].is_empty() { f[2] } else { f[4] },
            }
        })
        .collect()
}

/// The facelets of an entry's cube in the pose of the entry.
fn state_of(id: usize, facelets: &str) -> Cube {
    let e = entry(id).unwrap();
    from_facelets(e.n, e.model, e.pose, facelets).unwrap()
}

#[test]
fn the_fixture_has_every_row_of_the_reference_file() {
    let rows = rows();
    assert_eq!(rows.len(), 448);
    let color_ids: Vec<usize> = entries()
        .iter()
        .filter(|e| e.model == Model::Color)
        .map(|e| e.id)
        .collect();
    for id in color_ids {
        let seen = rows.iter().filter(|r| r.id == id).count();
        assert!(seen >= 2, "entry {id}: {seen} rows");
    }
    assert!(rows
        .iter()
        .all(|r| entry(r.id).unwrap().model == Model::Color));
}

#[test]
fn every_row_encodes_to_its_token() {
    for row in rows() {
        assert_eq!(
            encode(row.id, row.facelets).as_deref(),
            Ok(row.token),
            "entry {} after {:?}",
            row.id,
            row.moves
        );
        assert_eq!(
            encode(row.id, row.canonical).as_deref(),
            Ok(row.token),
            "entry {} canonical after {:?}",
            row.id,
            row.moves
        );
    }
}

#[test]
fn every_token_decodes_to_its_facelets() {
    for row in rows() {
        let (found, text) = decode(row.token).unwrap();
        assert_eq!(found.id, row.id, "{:?}", row.moves);
        assert_eq!(
            text, row.canonical,
            "entry {} after {:?}",
            row.id, row.moves
        );
    }
}

#[test]
fn cube_and_facelets_agree_in_both_directions_for_every_row() {
    for row in rows() {
        let e = entry(row.id).unwrap();
        let cube = state_of(row.id, row.facelets);
        // The cube is the one the token holds.
        assert_eq!(
            cubacle::notation::orbit64::v2::decode(row.token).unwrap().1,
            cube
        );
        // Written back, a framed odd cube keeps the orientation it was read in;
        // every other type writes its canonical picture.
        let written = to_facelets(e.n, e.model, e.pose, &cube).unwrap();
        assert_eq!(
            written, row.canonical,
            "entry {} after {:?}",
            row.id, row.moves
        );
        if e.pose == Pose::Framed && e.n % 2 == 1 {
            assert_eq!(
                written, row.facelets,
                "entry {} after {:?}",
                row.id, row.moves
            );
        }
        assert_eq!(state_of(row.id, &written), cube);
    }
}

#[test]
fn a_canonical_read_accepts_any_whole_cube_orientation() {
    // The odd framed rows are the same cubes in 24 orientations; the canonical
    // entry of the same size reads them all as one state.
    for row in rows().iter().filter(|r| {
        let e = entry(r.id).unwrap();
        e.pose == Pose::Framed
    }) {
        let e = entry(row.id).unwrap();
        let canonical = from_facelets(e.n, e.model, Pose::Canonical, row.facelets).unwrap();
        let again = from_facelets(e.n, e.model, Pose::Canonical, row.canonical).unwrap();
        assert_eq!(canonical, again, "entry {} after {:?}", row.id, row.moves);
        if e.n % 2 == 1 {
            assert_eq!(canonical.frame, 0);
        }
    }
}

#[test]
fn the_solved_facelets_are_the_solved_token_of_every_color_entry() {
    for e in entries().iter().filter(|e| e.model == Model::Color) {
        let solved = ["U", "R", "F", "D", "L", "B"]
            .map(|f| f.repeat(e.n * e.n))
            .join(" ");
        let token =
            cubacle::notation::orbit64::v2::encode(e.id, &Cube::solved(e.n, e.model).unwrap());
        assert_eq!(encode(e.id, &solved), token, "entry {}", e.id);
        assert_eq!(decode(token.as_ref().unwrap()).unwrap().1, solved);
    }
}

#[test]
fn spaces_are_ignored_on_input_and_one_space_separates_fields_on_output() {
    let packed = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    let spaced = "U U U U U U U U U  RRRRRRRRR FFFFFFFFF DDDDDDDDD LLLLLLLLL BBBBBBBBB ";
    let canonical = "UUUUUUUUU RRRRRRRRR FFFFFFFFF DDDDDDDDD LLLLLLLLL BBBBBBBBB";
    let cube = from_facelets(3, Model::Color, Pose::Framed, canonical).unwrap();
    assert_eq!(
        from_facelets(3, Model::Color, Pose::Framed, packed),
        Ok(cube.clone())
    );
    assert_eq!(
        from_facelets(3, Model::Color, Pose::Framed, spaced),
        Ok(cube.clone())
    );
    assert_eq!(
        to_facelets(3, Model::Color, Pose::Framed, &cube).as_deref(),
        Ok(canonical)
    );
}

// ------------------------------------------------------------- invalid input

const SOLVED3: &str = "UUUUUUUUU RRRRRRRRR FFFFFFFFF DDDDDDDDD LLLLLLLLL BBBBBBBBB";
const SOLVED2: &str = "UUUU RRRR FFFF DDDD LLLL BBBB";
const SOLVED4: &str = "UUUUUUUUUUUUUUUU RRRRRRRRRRRRRRRR FFFFFFFFFFFFFFFF DDDDDDDDDDDDDDDD \
                       LLLLLLLLLLLLLLLL BBBBBBBBBBBBBBBB";

fn framed(n: usize, text: &str) -> Result<Cube, V2Error> {
    from_facelets(n, Model::Color, Pose::Framed, text)
}

fn facelets_error(why: &str) -> Result<Cube, V2Error> {
    Err(V2Error::Facelets(why.to_string()))
}

fn invalid(why: &str) -> Result<Cube, V2Error> {
    Err(V2Error::Invalid(why.to_string()))
}

/// The solved cube with the facelets `at` showing the colors the facelets
/// `from` show on the solved cube.
fn showing(solved: &str, edits: &[(&[usize], &[usize])]) -> String {
    let letters: Vec<char> = solved.chars().filter(|&c| c != ' ').collect();
    let size = letters.len() / 6;
    let mut changed = letters.clone();
    for (at, from) in edits {
        for (&to, &source) in at.iter().zip(*from) {
            changed[to] = letters[source];
        }
    }
    changed
        .chunks(size)
        .map(|face| face.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn a_wrong_length_is_an_error() {
    let expected = |n: usize, got: usize| {
        facelets_error(&format!(
            "expected {} facelets for the {n}x{n}x{n}, got {got}",
            6 * n * n
        ))
    };
    assert_eq!(framed(3, ""), expected(3, 0));
    assert_eq!(framed(3, "UUU"), expected(3, 3));
    assert_eq!(framed(3, &format!("{SOLVED3}U")), expected(3, 55));
    assert_eq!(framed(2, SOLVED3), expected(2, 54));
    assert_eq!(framed(3, SOLVED2), expected(3, 24));
}

#[test]
fn a_bad_letter_is_an_error_and_only_the_space_is_ignored() {
    let bad = |c: &str| facelets_error(&format!("facelet '{c}' is not one of U R F D L B"));
    assert_eq!(framed(3, &SOLVED3.replacen('U', "X", 1)), bad("X"));
    assert_eq!(framed(3, &SOLVED3.replacen('U', "u", 1)), bad("u"));
    assert_eq!(framed(3, &SOLVED3.replacen(' ', "\t", 1)), bad("\t"));
    assert_eq!(framed(3, &SOLVED3.replacen(' ', "\n", 1)), bad("\n"));
    assert_eq!(framed(3, &SOLVED3.replacen(' ', "\r\n", 1)), bad("\r"));
    assert_eq!(
        framed(3, &SOLVED3.replacen('U', "\u{e9}", 1)),
        bad("\u{e9}")
    );
    // The first bad letter is reported, also when the length is wrong too.
    assert_eq!(framed(3, "UX?"), bad("X"));
}

#[test]
fn wrong_color_counts_are_errors() {
    // One U sticker swapped for an R sticker.
    let text = SOLVED3.replacen('U', "R", 1);
    assert_eq!(
        framed(3, &text),
        facelets_error("color U occurs 8 times, expected 9")
    );
    // Six stickers of one color in a 2x2x2 of four per face.
    let text = SOLVED2.replacen("BBBB", "BBBU", 1);
    assert_eq!(
        framed(2, &text),
        facelets_error("color U occurs 5 times, expected 4")
    );
    // All letters one color: the first color with a wrong count is U.
    assert_eq!(
        framed(2, &"D".repeat(24)),
        facelets_error("color U occurs 0 times, expected 4")
    );
}

#[test]
fn fixed_centers_of_the_wrong_handedness_are_an_error() {
    let right_handed = "the six fixed centers are not a right-handed whole-cube frame";
    // Two fixed centers exchanged.
    let text = "UUUUUUUUU RRRRFRRRR FFFFRFFFF DDDDDDDDD LLLLLLLLL BBBBBBBBB";
    assert_eq!(framed(3, text), facelets_error(right_handed));
    // The canonical read turns the cube first and fails the same way.
    assert_eq!(
        from_facelets(3, Model::Color, Pose::Canonical, text),
        facelets_error(right_handed)
    );
    // The mirror image of the solved cube: U L F D R B.
    let mirror = "UUUUUUUUU LLLLLLLLL FFFFFFFFF DDDDDDDDD RRRRRRRRR BBBBBBBBB";
    assert_eq!(framed(3, mirror), facelets_error(right_handed));
    assert_eq!(framed(5, &mirror5()), facelets_error(right_handed));
}

fn mirror5() -> String {
    ["U", "L", "F", "D", "R", "B"]
        .map(|f| f.repeat(25))
        .join(" ")
}

#[test]
fn a_twisted_corner_is_an_error() {
    // The URF corner turned in place.
    let text = showing(SOLVED3, &[(&[8, 9, 20], &[9, 20, 8])]);
    assert_eq!(
        framed(3, &text),
        invalid("corner twists must sum to 0 mod 3")
    );
    let text = showing(SOLVED2, &[(&[3, 4, 9], &[4, 9, 3])]);
    assert_eq!(
        framed(2, &text),
        invalid("corner twists must sum to 0 mod 3")
    );
}

#[test]
fn a_flipped_midge_is_an_error() {
    // The UR midge with its two stickers exchanged.
    let text = showing(SOLVED3, &[(&[5, 10], &[10, 5])]);
    assert_eq!(framed(3, &text), invalid("midge flips must sum to 0 mod 2"));
}

#[test]
fn a_piece_shown_twice_is_an_error() {
    // 2x2x2: the UFL slot shows the URF corner and the DRB slot shows the DBL
    // corner; the color counts stay intact, URF and DBL occur twice.
    let corners = "corner pieces are not a permutation of 8";
    let text = showing(
        SOLVED2,
        &[(&[2, 8, 17], &[3, 4, 9]), (&[15, 7, 22], &[14, 23, 18])],
    );
    assert_eq!(framed(2, &text), invalid(corners));
    let text = showing(
        SOLVED3,
        &[(&[6, 18, 38], &[8, 9, 20]), (&[35, 17, 51], &[33, 53, 42])],
    );
    assert_eq!(framed(3, &text), invalid(corners));
    // 3x3x3 midges: UF shows UR and DR shows DF.
    let text = showing(SOLVED3, &[(&[7, 19], &[5, 10]), (&[32, 16], &[28, 25])]);
    assert_eq!(
        framed(3, &text),
        invalid("midge pieces must be a permutation of 12 on odd cubes and empty otherwise")
    );
    // 4x4x4 wings: slot 1 shows wing 0 and slot 15 shows wing 13.
    let text = showing(SOLVED4, &[(&[7, 18], &[1, 82]), (&[62, 93], &[55, 29])]);
    assert_eq!(
        framed(4, &text),
        invalid("expected 1 wing permutations of 24")
    );
}

#[test]
fn a_slot_that_shows_no_piece_is_an_error() {
    // The U and R stickers of the URF corner exchanged: a mirrored corner.
    let text = showing(SOLVED3, &[(&[8, 9], &[9, 8])]);
    assert_eq!(
        framed(3, &text),
        facelets_error("corner slot 0 shows RUF, which is no corner of this cube")
    );
    // The UR midge shows U and D, the DR midge R and R: counts intact.
    let text = showing(SOLVED3, &[(&[5, 10], &[5, 32]), (&[32, 16], &[16, 16])]);
    assert_eq!(
        framed(3, &text),
        facelets_error("midge slot 0 shows UD, which is no midge of this cube")
    );
    // A 4x4x4 wing showing U and D, another B and B: counts intact.
    let text = showing(SOLVED4, &[(&[1, 82], &[1, 62]), (&[62, 93], &[93, 93])]);
    assert_eq!(
        framed(4, &text),
        facelets_error("wing slot 0 shows UD, which is no wing of this cube")
    );
}

/// The mirror image of a facelet string: left and right exchanged, columns
/// reversed, letters kept.
fn mirrored(n: usize, text: &str) -> String {
    let faces: Vec<Vec<char>> = text.split(' ').map(|face| face.chars().collect()).collect();
    let flip = |face: &[char]| -> Vec<char> {
        (0..n * n)
            .map(|i| face[i / n * n + (n - 1 - i % n)])
            .collect()
    };
    let order = [0, 4, 2, 3, 1, 5];
    order
        .iter()
        .map(|&f| flip(&faces[f]).iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn a_mirrored_cube_is_an_error_for_every_row() {
    let mut checked = 0;
    for row in rows().iter().filter(|r| entry(r.id).unwrap().n <= 7) {
        let e = entry(row.id).unwrap();
        let text = mirrored(e.n, row.facelets);
        let read = from_facelets(e.n, e.model, e.pose, &text);
        assert!(read.is_err(), "entry {} after {:?}", row.id, row.moves);
        checked += 1;
    }
    assert!(checked > 200, "{checked}");
    // Odd cubes fail at the fixed centers; even cubes at the first corner.
    let text = mirrored(3, SOLVED3);
    assert_eq!(
        framed(3, &text),
        facelets_error("the six fixed centers are not a right-handed whole-cube frame")
    );
    assert_eq!(
        framed(2, &mirrored(2, SOLVED2)),
        facelets_error("corner slot 0 shows ULF, which is no corner of this cube")
    );
    assert_eq!(
        from_facelets(2, Model::Color, Pose::Canonical, &mirrored(2, SOLVED2)),
        facelets_error("no whole-cube rotation puts the DBL corner home")
    );
}

#[test]
fn an_unsupported_type_is_an_error() {
    let picture = "the picture model has no letters";
    let unsupported = |why: &str| Err(V2Error::Unsupported(why.to_string()));
    for pose in [Pose::Canonical, Pose::Framed] {
        assert_eq!(
            from_facelets(3, Model::Picture, pose, SOLVED3),
            unsupported(picture)
        );
        let solved = Cube::solved(3, Model::Picture).unwrap();
        assert_eq!(
            to_facelets(3, Model::Picture, pose, &solved),
            Err(V2Error::Unsupported(picture.to_string()))
        );
    }
    // The picture entries of the registry, by id, and their tokens.
    let pictures: Vec<_> = entries()
        .iter()
        .filter(|e| e.model == Model::Picture)
        .collect();
    assert_eq!(pictures.len(), 26);
    for e in pictures {
        assert_eq!(
            encode(e.id, SOLVED3),
            Err(V2Error::Unsupported(picture.to_string()))
        );
        let token =
            cubacle::notation::orbit64::v2::encode(e.id, &Cube::solved(e.n, e.model).unwrap())
                .unwrap();
        assert_eq!(
            decode(&token),
            Err(V2Error::Unsupported(picture.to_string()))
        );
    }
    let no_type = |n: usize| unsupported(&format!("no V2 type for the {n}x{n}x{n}"));
    assert_eq!(
        from_facelets(16, Model::Color, Pose::Framed, SOLVED3),
        no_type(16)
    );
    assert_eq!(from_facelets(1, Model::Color, Pose::Framed, ""), no_type(1));
    assert_eq!(
        from_facelets(0, Model::Color, Pose::Canonical, ""),
        no_type(0)
    );
    assert_eq!(encode(10_000, SOLVED3), Err(V2Error::UnknownEntry(10_000)));
}

#[test]
fn a_cube_that_is_no_state_of_the_type_has_no_facelets() {
    let mut cube = Cube::solved(3, Model::Color).unwrap();
    cube.corner_twists[0] = 1;
    assert_eq!(
        to_facelets(3, Model::Color, Pose::Framed, &cube),
        Err(V2Error::Invalid(
            "corner twists must sum to 0 mod 3".to_string()
        ))
    );
    // A canonical cube has frame 0 only.
    let mut cube = Cube::solved(3, Model::Color).unwrap();
    cube.frame = 4;
    assert!(to_facelets(3, Model::Color, Pose::Canonical, &cube).is_err());
    assert!(to_facelets(3, Model::Color, Pose::Framed, &cube).is_ok());
}

#[test]
fn malformed_tokens_are_errors_for_the_facelet_helper_too() {
    assert_eq!(decode(""), Err(V2Error::UnknownToken));
    assert_eq!(
        decode("AAA+"),
        Err(V2Error::BadCharacter {
            position: 3,
            found: '+'
        })
    );
}
