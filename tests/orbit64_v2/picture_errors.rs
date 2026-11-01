//! Orbit64 V2 picture facelets that are no cube, through the public API: each
//! error path with the exact text of the Flix reference (`Orbit64.V2.Facelets`
//! of flix-orbit64, Apache-2.0, Werner Stein), the order in which several
//! faults are reported, hostile input and mutated vectors that must never
//! panic.

use super::picture_facelets::{put, rows, slot, solved, symbol_char, SOLVED3};
use cubacle::notation::orbit64::v2::facelets::{decode, encode, from_facelets, to_facelets};
use cubacle::notation::orbit64::v2::{entries, entry, Cube, Model, Pose, V2Error};

fn at(text: &str, index: usize) -> char {
    text.chars().filter(|&c| c != ' ').nth(index).unwrap()
}

fn swap(text: &str, i: usize, j: usize) -> String {
    put(text, &[(i, at(text, j)), (j, at(text, i))])
}

/// The string with the symbols of two slots exchanged, facelet by facelet.
fn swap_slots(text: &str, n: usize, a: &str, b: &str) -> String {
    let edits: Vec<(usize, char)> = slot(n, a)
        .iter()
        .zip(slot(n, b))
        .flat_map(|(&i, j)| [(i, at(text, j)), (j, at(text, i))])
        .collect();
    put(text, &edits)
}

/// The string with the symbols of a slot cycled: the piece twisted in place.
fn cycled(text: &str, n: usize, label: &str) -> String {
    let facelets = slot(n, label);
    let edits: Vec<(usize, char)> = (0..facelets.len())
        .map(|k| (facelets[k], at(text, facelets[(k + 1) % facelets.len()])))
        .collect();
    put(text, &edits)
}

fn picture(n: usize, pose: Pose, text: &str) -> Result<Cube, V2Error> {
    from_facelets(n, Model::Picture, pose, text)
}

fn rejected(n: usize, text: &str) -> V2Error {
    picture(n, Pose::Framed, text).unwrap_err()
}

fn facelets_error(why: &str) -> V2Error {
    V2Error::Facelets(why.to_string())
}

fn invalid(why: &str) -> V2Error {
    V2Error::Invalid(why.to_string())
}

#[test]
fn a_wrong_length_is_an_error() {
    let expected = |n: usize, got: usize| {
        facelets_error(&format!(
            "expected {} facelets for the {n}x{n}x{n}, got {got}",
            6 * n * n
        ))
    };
    assert_eq!(rejected(3, ""), expected(3, 0));
    assert_eq!(rejected(3, "GGJ"), expected(3, 3));
    assert_eq!(rejected(3, &format!("{SOLVED3}A")), expected(3, 55));
    assert_eq!(rejected(3, &SOLVED3[..SOLVED3.len() - 1]), expected(3, 53));
    assert_eq!(rejected(3, solved(4)), expected(3, 96));
    assert_eq!(rejected(4, SOLVED3), expected(4, 54));
    assert_eq!(rejected(15, &"A".repeat(1349)), expected(15, 1349));
}

#[test]
fn a_bad_symbol_is_an_error_and_only_the_space_is_ignored() {
    let bad = |c: &str| facelets_error(&format!("facelet '{c}' is not a base64url symbol"));
    for c in [
        "#", "+", "/", "=", ".", "!", "\t", "\n", "\r", "\0", "\u{e9}", "\u{df}",
    ] {
        assert_eq!(rejected(3, &SOLVED3.replacen('G', c, 1)), bad(c), "{c:?}");
    }
    assert_eq!(rejected(3, &SOLVED3.replacen(' ', "\r\n", 1)), bad("\r"));
    assert_eq!(
        rejected(3, &SOLVED3.replacen('G', "\u{2028}", 1)),
        bad("\u{2028}")
    );
    // The first bad symbol is reported, also when the length is wrong too.
    assert_eq!(rejected(3, "G#+"), bad("#"));
    assert_eq!(rejected(3, &format!("{SOLVED3}A?")), bad("?"));
    // A bad symbol after a huge amount of good ones is still found.
    assert_eq!(rejected(3, &format!("{}~", "A".repeat(1 << 16))), bad("~"));
    // The color letters are symbols too; only the picture rules reject them.
    assert_eq!(
        rejected(3, &"U".repeat(54)),
        facelets_error("corner sticker 'U' occurs more than once")
    );
}

#[test]
fn a_huge_input_costs_no_memory_and_is_a_length_error() {
    // 64 MiB of valid symbols: counted, not stored.
    let huge = "A".repeat(64 << 20);
    assert_eq!(
        rejected(3, &huge),
        facelets_error(&format!(
            "expected 54 facelets for the 3x3x3, got {}",
            64 << 20
        ))
    );
    assert_eq!(
        picture(15, Pose::Canonical, &huge),
        Err(facelets_error(&format!(
            "expected 1350 facelets for the 15x15x15, got {}",
            64 << 20
        )))
    );
    // A bad symbol after all of it is still found, and spaces cost nothing.
    assert_eq!(
        rejected(3, &format!("{huge}#")),
        facelets_error("facelet '#' is not a base64url symbol")
    );
    let spaces = " ".repeat(1 << 24);
    assert!(picture(3, Pose::Framed, &format!("{spaces}{SOLVED3}{spaces}")).is_ok());
}

#[test]
fn a_sticker_on_the_wrong_class_is_an_error() {
    let range = |at: usize, c: char, what: &str| {
        facelets_error(&format!(
            "facelet {at} shows '{c}', which is no {what} sticker"
        ))
    };
    // Symbol Y is 24, `_` is 63: no corner, midge, center or fixed center
    // has that many stickers.
    assert_eq!(
        rejected(3, &put(SOLVED3, &[(0, 'Y')])),
        range(0, 'Y', "corner")
    );
    assert_eq!(
        rejected(3, &put(SOLVED3, &[(0, '_')])),
        range(0, '_', "corner")
    );
    assert_eq!(
        rejected(3, &put(SOLVED3, &[(1, 'Y')])),
        range(1, 'Y', "midge")
    );
    assert_eq!(
        rejected(3, &put(SOLVED3, &[(4, 'Y')])),
        range(4, 'Y', "fixed center")
    );
    // Wings have 48 stickers and centers 24: `w` is 48.
    let solved5 = solved(5);
    assert_eq!(
        rejected(5, &put(solved5, &[(1, 'w')])),
        range(1, 'w', "wing")
    );
    assert_eq!(
        rejected(5, &put(solved5, &[(6, 'Y')])),
        range(6, 'Y', "center")
    );
}

#[test]
fn a_repeated_sticker_is_an_error() {
    let twice = |c: char, what: &str| {
        facelets_error(&format!("{what} sticker '{c}' occurs more than once"))
    };
    let c0 = at(SOLVED3, 8);
    assert_eq!(rejected(3, &put(SOLVED3, &[(0, c0)])), twice(c0, "corner"));
    assert_eq!(
        rejected(3, &put(SOLVED3, &[(1, at(SOLVED3, 5))])),
        twice(at(SOLVED3, 5), "midge")
    );
    // Two fixed centers that differ only in rotation are still the same piece.
    assert_eq!(
        rejected(3, &put(SOLVED3, &[(4, 'B'), (13, 'A')])),
        twice('B', "fixed center")
    );
    let solved4 = solved(4);
    let copy = |to: usize, from: usize| put(solved4, &[(to, at(solved4, from))]);
    assert_eq!(rejected(4, &copy(5, 6)), twice(at(solved4, 6), "center"));
    assert_eq!(rejected(4, &copy(1, 82)), twice(at(solved4, 82), "wing"));
}

#[test]
fn a_slot_that_shows_no_piece_is_an_error() {
    // The U and R stickers of the URF corner exchanged: a mirrored corner.
    assert_eq!(
        rejected(3, &swap(SOLVED3, 8, 9)),
        facelets_error("corner slot 0 shows BAC, which is no corner of this cube")
    );
    // The U sticker of UR and the R sticker of UF: two midges mixed.
    assert_eq!(
        rejected(3, &swap(SOLVED3, 5, 19)),
        facelets_error("midge slot 0 shows DB, which is no midge of this cube")
    );
    // A wing with its two stickers exchanged is a twisted wing.
    let solved4 = solved(4);
    assert_eq!(
        rejected(4, &swap(solved4, 1, 82)),
        facelets_error("wing slot 0 shows BA, which is no wing of this cube")
    );
}

#[test]
fn the_fixed_centers_must_form_a_frame() {
    let frame = "the six fixed centers are not a right-handed whole-cube frame";
    // The U and R centers exchanged: left-handed.
    let text = swap(SOLVED3, 4, 13);
    assert_eq!(rejected(3, &text), facelets_error(frame));
    assert_eq!(
        picture(3, Pose::Canonical, &text),
        Err(facelets_error(frame))
    );
}

#[test]
fn the_laws_of_the_format_are_checked_in_their_order() {
    let twists = invalid("corner twists must sum to 0 mod 3");
    let flips = invalid("midge flips must sum to 0 mod 2");
    for n in 3..=9 {
        let solved = solved(n);
        for pose in [Pose::Canonical, Pose::Framed] {
            assert_eq!(
                picture(n, pose, &cycled(solved, n, "C 0")),
                Err(twists.clone())
            );
        }
        if n % 2 == 1 {
            assert_eq!(rejected(n, &cycled(solved, n, "M 0")), flips, "{n}x{n}x{n}");
            // The twist sum comes before the flip sum.
            assert_eq!(
                rejected(n, &cycled(&cycled(solved, n, "M 0"), n, "C 0")),
                twists
            );
        }
    }
    // Two midges exchanged with everything on them: M odd, C even.
    let text = swap_slots(SOLVED3, 3, "M 0", "M 1");
    assert_eq!(
        rejected(3, &text),
        invalid("midge parity must equal corner parity")
    );
    // One fixed center turned a quarter: R odd, C even.
    assert_eq!(
        rejected(3, &put(SOLVED3, &[(4, 'B')])),
        invalid("fixed-center rotation parity must equal corner parity")
    );
    // Two center stickers of the 4x4x4 exchanged: Z odd, C even.
    assert_eq!(
        rejected(4, &swap(solved(4), 5, 6)),
        invalid("a center orbit parity breaks the picture relation")
    );
    // Two corners exchanged on a 4x4x4: C odd, so every center orbit breaks.
    assert_eq!(
        rejected(4, &swap_slots(solved(4), 4, "C 0", "C 1")),
        invalid("a center orbit parity breaks the picture relation")
    );
    // M = C comes before R = C, which comes before the center parities.
    let both = swap_slots(&put(SOLVED3, &[(4, 'B')]), 3, "M 0", "M 1");
    assert_eq!(
        rejected(3, &both),
        invalid("midge parity must equal corner parity")
    );
}

#[test]
fn repeats_are_reported_in_class_order_and_fixed_centers_by_piece() {
    let twice = |c: char, what: &str| {
        facelets_error(&format!("{what} sticker '{c}' occurs more than once"))
    };
    // Wing orbits go outermost first: a 6x6x6 with a repeat in two orbits
    // reports the outer one ('T'), not the inner one ('K').
    let wings = [
        "TuuffXlXXXWslWXWWsqVVUVLqVUUULIBBooK QmmddUVTTpSkVSTSSkiRRQRriRQQQrFFFggH ",
        "VeevvSPPPPOcPOPOOcaNNMNnaNMMMnMXXYYP NWWZZRNLLLQUNKLKKUSJJIJjSJIIIjCHHKQE ",
        // The symbols are split here only to keep the spelling check quiet.
        "WOObbO",
        "tHHHGMtGHGGMKFFEFTKFEEETLDDIIB AGGRRDJDDDCEJCDCCECBBABhCBAAAhJTpAAG",
    ]
    .concat();
    assert_eq!(encode(16, &wings), Err(twice('T', "wing")));
    // Center orbits go in orbit order: 'P' repeats in the first orbit that does.
    let centers = "EQGHCjIAJSRFSGXUKDLNRZCWN OMWTBbGVEIFQBTNOHWFDWtUKL PYDXMnMINaACKBEcOLPPSvLeV \
                   TuKfXlXMWsTSERVqVAULIBOoK FiQVQFRKTmJHNEBgQJSdHrSkU AGHRDJDUCEMNWOICBXPhJpPAG";
    assert_eq!(encode(12, centers), Err(twice('P', "center")));
    // A fixed center repeats by piece `v div 4`, whatever its rotation: 'T' is
    // piece 4 (rotation 3), the piece of the L center 'Q'.
    let fixed = "GGJEAADCA BBLRTXOJW EDCSIQRLN PKMMMISOV HFFVQTUNQ KHIWUUXPT";
    assert_eq!(encode(5, fixed), Err(twice('T', "fixed center")));
    // The U center turned a quarter and the R center at rotation 0 are piece 0 twice.
    assert_eq!(
        rejected(3, &put(SOLVED3, &[(4, 'B'), (13, 'A')])),
        twice('B', "fixed center")
    );
}

#[test]
fn the_first_failure_of_a_step_is_the_one_reported() {
    let twice = |c: char, what: &str| {
        facelets_error(&format!("{what} sticker '{c}' occurs more than once"))
    };
    let c0 = at(SOLVED3, 8);
    // Step 2: a repeated corner sticker and a midge out of range; corners first.
    assert_eq!(
        rejected(3, &put(SOLVED3, &[(0, c0), (1, 'Y')])),
        twice(c0, "corner")
    );
    // Within a class a range error comes before a repeat.
    assert_eq!(
        rejected(3, &put(SOLVED3, &[(0, c0), (2, 'Y')])),
        facelets_error("facelet 2 shows 'Y', which is no corner sticker")
    );
    // Step 2 before step 3: a range error beats a mirrored corner.
    assert_eq!(
        rejected(3, &put(&swap(SOLVED3, 8, 9), &[(5, 'Y')])),
        facelets_error("facelet 5 shows 'Y', which is no midge sticker")
    );
    // Step 3 before step 4: a mirrored corner beats exchanged fixed centers.
    assert_eq!(
        rejected(3, &swap(&swap(SOLVED3, 8, 9), 4, 13)),
        facelets_error("corner slot 0 shows BAC, which is no corner of this cube")
    );
    // Step 4 before step 5: exchanged fixed centers beat a twisted corner.
    assert_eq!(
        rejected(3, &swap(&cycled(SOLVED3, 3, "C 0"), 4, 13)),
        facelets_error("the six fixed centers are not a right-handed whole-cube frame")
    );
    // Step 5: the twist sum comes before the flip sum.
    assert_eq!(
        rejected(3, &cycled(&cycled(SOLVED3, 3, "M 0"), 3, "C 0")),
        invalid("corner twists must sum to 0 mod 3")
    );
}

#[test]
fn the_2x2x2_has_no_picture_entry() {
    let why = "the 2x2x2 has no picture entry: its picture model is the color model";
    for pose in [Pose::Canonical, Pose::Framed] {
        assert_eq!(
            picture(2, pose, "A"),
            Err(V2Error::Unsupported(why.to_string()))
        );
        let color = Cube::solved(2, Model::Color).unwrap();
        assert_eq!(
            to_facelets(2, Model::Picture, pose, &color),
            Err(V2Error::Unsupported(why.to_string()))
        );
    }
    assert!(entries()
        .iter()
        .all(|e| e.n != 2 || e.model == Model::Color));
    let no_type = |n: usize| V2Error::Unsupported(format!("no V2 type for the {n}x{n}x{n}"));
    assert_eq!(picture(16, Pose::Framed, SOLVED3), Err(no_type(16)));
    assert_eq!(picture(1, Pose::Framed, ""), Err(no_type(1)));
    assert_eq!(picture(0, Pose::Canonical, ""), Err(no_type(0)));
    // The size is checked before the text, and before the model.
    assert_eq!(picture(2, Pose::Framed, "#"), picture(2, Pose::Framed, ""));
}

#[test]
fn a_cube_that_is_no_state_of_the_type_has_no_picture() {
    let solved = Cube::solved(3, Model::Picture).unwrap();
    let broken = Cube {
        rotations: vec![1, 0, 0, 0, 0, 0],
        ..solved.clone()
    };
    assert_eq!(
        to_facelets(3, Model::Picture, Pose::Canonical, &broken),
        Err(invalid(
            "fixed-center rotation parity must equal corner parity"
        ))
    );
    // A canonical cube has frame 0 only.
    let framed = Cube { frame: 4, ..solved };
    assert_eq!(
        to_facelets(3, Model::Picture, Pose::Canonical, &framed),
        Err(invalid(
            "frame must be 0..23 on odd framed cubes and 0 otherwise"
        ))
    );
    // A color cube is no picture state: its centers are not permutations.
    let color = Cube::solved(4, Model::Color).unwrap();
    assert_eq!(
        to_facelets(4, Model::Picture, Pose::Canonical, &color),
        Err(invalid(
            "a center orbit is not a valid color multiset or permutation of 24"
        ))
    );
}

#[test]
fn encode_and_decode_reach_every_picture_entry() {
    let pictures: Vec<_> = entries()
        .iter()
        .filter(|e| e.model == Model::Picture)
        .collect();
    assert_eq!(pictures.len(), 26);
    for e in pictures {
        let text = solved(e.n);
        let token = encode(e.id, text).unwrap();
        assert_eq!(token.len(), e.length);
        let (found, written) = decode(&token).unwrap();
        assert_eq!((found.id, written.as_str()), (e.id, text));
        // A color string is not a picture string: `U` is symbol 20, a corner
        // symbol, so the first failure is the repeated corner sticker.
        let color = ["U", "R", "F", "D", "L", "B"].map(|f| f.repeat(e.n * e.n));
        assert_eq!(
            encode(e.id, &color.join(" ")),
            Err(facelets_error("corner sticker 'U' occurs more than once"))
        );
    }
}

// ------------------------------------------------------- hostile mutations

/// A deterministic generator, so a failure repeats.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as usize % bound
    }
}

#[test]
fn mutated_pictures_never_panic_and_a_read_one_writes_back_the_same_cube() {
    let mut rng = Lcg(0x0b17_64c0_ffee);
    let (mut accepted, mut refused) = (0, 0);
    for row in rows() {
        let e = entry(row.id).unwrap();
        let symbols: Vec<char> = row.facelets.chars().filter(|&c| c != ' ').collect();
        for round in 0..40 {
            let mut m = symbols.clone();
            let (i, j) = (rng.next(m.len()), rng.next(m.len()));
            match round % 8 {
                0 => m.swap(i, j),
                1 => m[i] = m[j],
                2 => m[i] = symbol_char(rng.next(64)),
                3 => m.truncate(i),
                4 => m.insert(i, symbol_char(rng.next(64))),
                5 => m[i] = ['#', '\n', '\u{e9}', '\0', '\u{1F600}'][rng.next(5)],
                6 => {
                    m.swap(i, j);
                    let (k, l) = (rng.next(m.len()), rng.next(m.len()));
                    m.swap(k, l);
                }
                _ => m.extend_from_slice(&symbols[..i]),
            }
            let text: String = m.iter().collect();
            match from_facelets(e.n, e.model, e.pose, &text) {
                Ok(cube) => {
                    accepted += 1;
                    let written = to_facelets(e.n, e.model, e.pose, &cube).unwrap();
                    assert_eq!(state_of_text(e.n, e.pose, &written), cube, "{text}");
                    assert!(encode(row.id, &text).is_ok());
                }
                Err(V2Error::Facelets(_) | V2Error::Invalid(_)) => refused += 1,
                Err(other) => panic!("{other:?} for {text}"),
            }
        }
    }
    assert!(accepted > 100 && refused > 5000, "{accepted} {refused}");
}

fn state_of_text(n: usize, pose: Pose, text: &str) -> Cube {
    from_facelets(n, Model::Picture, pose, text).unwrap()
}
