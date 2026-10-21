//! Orbit64 state tokens: a canonical base64url encoding of a cube state.
//!
//! The format is defined by <https://github.com/wstein/flix-orbit64> (`FORMAT.md`).
//! A token's length identifies the cube size; the state is a mixed-radix number
//! over the cube's orbits, followed on odd cubes by one of 24 whole-cube frames.
//!
//! Only the 2x2x2 and 3x3x3 are decoded so far, under the slot convention
//! `orbit64-3x3-draft@1`, which is the standard Kociemba order: corners
//! URF UFL ULB UBR DFR DLF DBL DRB and edges UR UF UL UB DR DF DL DB FR FL BL BR.
//! A token carries ordinals only, so it cannot prove its producer used the same
//! numbering; the reference vectors of the format are the evidence it matches.

use std::fmt;

use super::{Face, Facelets, FaceletsError};

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Token widths by cube size 2..=7.
const WIDTHS: [usize; 6] = [5, 12, 27, 43, 66, 90];

/// Facelet indices of each corner slot; the first is the one twist counts from.
const CORNER_FACELETS: [[usize; 3]; 8] = [
    [8, 9, 20],
    [6, 18, 38],
    [0, 36, 47],
    [2, 45, 11],
    [29, 26, 15],
    [27, 44, 24],
    [33, 53, 42],
    [35, 17, 51],
];

/// Facelet indices of each edge slot ("midge" in Orbit64 terms).
const EDGE_FACELETS: [[usize; 2]; 12] = [
    [5, 10],
    [7, 19],
    [3, 37],
    [1, 46],
    [32, 16],
    [28, 25],
    [30, 43],
    [34, 52],
    [23, 12],
    [21, 41],
    [50, 39],
    [48, 14],
];

/// Face colours (indices into `URFDLB`) of each corner piece, clockwise.
const CORNER_COLOURS: [[usize; 3]; 8] = [
    [0, 1, 2],
    [0, 2, 4],
    [0, 4, 5],
    [0, 5, 1],
    [3, 2, 1],
    [3, 4, 2],
    [3, 5, 4],
    [3, 1, 5],
];

const EDGE_COLOURS: [[usize; 2]; 12] = [
    [0, 1],
    [0, 2],
    [0, 4],
    [0, 5],
    [3, 1],
    [3, 2],
    [3, 4],
    [3, 5],
    [2, 1],
    [2, 4],
    [5, 4],
    [5, 1],
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Orbit64Error {
    /// The token has a character outside the base64url alphabet.
    BadCharacter { position: usize, found: char },
    /// The token's length is not that of any cube size.
    BadLength(usize),
    /// The token is a valid width but no decoder exists for that size yet.
    Unsupported(usize),
    /// The number is at or above the number of states of the cube.
    OutOfRange,
    /// The token decoded to something that is not a cube.
    NotACube(FaceletsError),
}

impl fmt::Display for Orbit64Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Orbit64Error::BadCharacter { position, found } => write!(
                f,
                "orbit64 token has {found:?} at {position}, outside the base64url alphabet"
            ),
            Orbit64Error::BadLength(len) => write!(
                f,
                "{len} characters is not an orbit64 state width (5, 12, 27, 43, 66 or 90)"
            ),
            Orbit64Error::Unsupported(size) => write!(
                f,
                "orbit64 decoding of the {size}x{size}x{size} is not implemented yet"
            ),
            Orbit64Error::OutOfRange => write!(f, "orbit64 token is out of range for its cube"),
            Orbit64Error::NotACube(error) => write!(f, "orbit64 token is not a cube: {error}"),
        }
    }
}

impl std::error::Error for Orbit64Error {}

/// The cube size whose state tokens have this width.
pub fn size_of_width(width: usize) -> Option<usize> {
    WIDTHS
        .iter()
        .position(|&w| w == width)
        .map(|index| index + 2)
}

/// Whether the text could be an Orbit64 state token: a state width, base64url only.
pub fn looks_like_token(text: &str) -> bool {
    size_of_width(text.chars().count()).is_some()
        && text.bytes().all(|byte| ALPHABET.contains(&byte))
}

/// Decodes a state token into the facelets it shows, whole-cube frame applied.
pub fn decode(token: &str) -> Result<Facelets, Orbit64Error> {
    let size = size_of_width(token.chars().count())
        .ok_or_else(|| Orbit64Error::BadLength(token.chars().count()))?;
    if size > 3 {
        return Err(Orbit64Error::Unsupported(size));
    }
    let value = from_base64(token)?;

    let stickers = if size == 2 {
        if value >= CORNER_STATES {
            return Err(Orbit64Error::OutOfRange);
        }
        let (corners, twists) = corners(value);
        let full = place(&corners, &twists, &Edges::solved());
        thin_to_2x2(&full)
    } else {
        if value >= STATES_3X3 {
            return Err(Orbit64Error::OutOfRange);
        }
        let (coordinate, frame) = (value / 24, (value % 24) as usize);
        let (corner_rank, edge_value) = (coordinate / EDGE_STATES, coordinate % EDGE_STATES);
        let (corner_perm, twists) = corners(corner_rank);
        let edges = edges(edge_value, parity(&corner_perm));
        let full = place(&corner_perm, &twists, &edges);
        apply_frame(&full, frame)
    };
    let faces = stickers.iter().map(|&index| Face::ALL[index]).collect();
    Facelets::from_stickers(faces).map_err(Orbit64Error::NotACube)
}

const CORNER_STATES: u128 = 40320 * 2187;
/// Edge states of an odd cube: parity-selected permutations times flips.
const EDGE_STATES: u128 = 479_001_600 / 2 * 2048;
const STATES_3X3: u128 = CORNER_STATES * EDGE_STATES * 24;

fn from_base64(token: &str) -> Result<u128, Orbit64Error> {
    let mut value: u128 = 0;
    for (position, found) in token.chars().enumerate() {
        let digit = u8::try_from(found)
            .ok()
            .and_then(|byte| ALPHABET.iter().position(|&known| known == byte))
            .ok_or(Orbit64Error::BadCharacter { position, found })?;
        value = value * 64 + digit as u128;
    }
    Ok(value)
}

fn corners(rank: u128) -> ([usize; 8], [usize; 8]) {
    let permutation = unrank_permutation(rank / 2187, 8);
    let twists = orientations(rank % 2187, 7, 3);
    (
        permutation.try_into().expect("8 corners"),
        twists.try_into().expect("8 twists"),
    )
}

struct Edges {
    permutation: [usize; 12],
    flips: [usize; 12],
}

impl Edges {
    fn solved() -> Edges {
        Edges {
            permutation: std::array::from_fn(|slot| slot),
            flips: [0; 12],
        }
    }
}

fn edges(value: u128, corner_parity: usize) -> Edges {
    let permutation = unrank_permutation_with_parity(value / 2048, 12, corner_parity);
    let flips = orientations(value % 2048, 11, 2);
    Edges {
        permutation: permutation.try_into().expect("12 edges"),
        flips: flips.try_into().expect("12 flips"),
    }
}

/// Lehmer-code unranking with the least significant digit last.
fn unrank_permutation(mut rank: u128, n: usize) -> Vec<usize> {
    let mut digits = Vec::with_capacity(n);
    for radix in 1..=n as u128 {
        digits.push((rank % radix) as usize);
        rank /= radix;
    }
    let mut available: Vec<usize> = (0..n).collect();
    digits
        .iter()
        .rev()
        .map(|&digit| available.remove(digit))
        .collect()
}

/// Unranks a permutation of the given parity; the last two entries are implied.
fn unrank_permutation_with_parity(rank: u128, n: usize, wanted: usize) -> Vec<usize> {
    let mut available: Vec<usize> = (0..n).collect();
    let mut picked = Vec::with_capacity(n);
    while available.len() > 2 {
        let remaining = available.len() as u128;
        let divisor = (1..remaining).product::<u128>() / 2;
        let digit = ((rank / divisor) % remaining) as usize;
        picked.push(available.remove(digit));
    }
    let mut direct = picked.clone();
    direct.extend(&available);
    if parity(&direct) == wanted {
        direct
    } else {
        picked.extend(available.iter().rev());
        picked
    }
}

fn parity(permutation: &[usize]) -> usize {
    let inversions = (0..permutation.len())
        .flat_map(|i| (i + 1..permutation.len()).map(move |j| (i, j)))
        .filter(|&(i, j)| permutation[i] > permutation[j])
        .count();
    inversions % 2
}

/// Little-endian digits of `count` orientations, the last closing the sum to 0.
fn orientations(mut value: u128, count: usize, radix: usize) -> Vec<usize> {
    let mut digits = Vec::with_capacity(count + 1);
    for _ in 0..count {
        digits.push((value % radix as u128) as usize);
        value /= radix as u128;
    }
    let sum: usize = digits.iter().sum();
    digits.push((radix - sum % radix) % radix);
    digits
}

fn place(corner_perm: &[usize; 8], twists: &[usize; 8], edges: &Edges) -> [usize; 54] {
    let mut sticker = [0usize; 54];
    for face in 0..6 {
        sticker[face * 9 + 4] = face;
    }
    for slot in 0..8 {
        for k in 0..3 {
            let at = CORNER_FACELETS[slot][(k + twists[slot]) % 3];
            sticker[at] = CORNER_COLOURS[corner_perm[slot]][k];
        }
    }
    for slot in 0..12 {
        for k in 0..2 {
            let at = EDGE_FACELETS[slot][(k + edges.flips[slot]) % 2];
            sticker[at] = EDGE_COLOURS[edges.permutation[slot]][k];
        }
    }
    sticker
}

/// A 2x2x2 is drawn on the 3x3x3 grid and keeps only the corner cells.
fn thin_to_2x2(full: &[usize; 54]) -> Vec<usize> {
    (0..24)
        .map(|i| {
            let (face, row, col) = (i / 4, (i / 2) % 2, i % 2);
            full[face * 9 + row * 6 + col * 2]
        })
        .collect()
}

type Sticker = (i32, i32, i32, i32, i32, i32);

fn sticker_of(index: usize) -> Sticker {
    let (face, row, col) = (
        (index / 9) as i32,
        ((index / 3) % 3) as i32,
        (index % 3) as i32,
    );
    match face {
        0 => (col, 2, row, 0, 1, 0),
        1 => (2, 2 - row, 2 - col, 1, 0, 0),
        2 => (col, 2 - row, 2, 0, 0, 1),
        3 => (col, 0, 2 - row, 0, -1, 0),
        4 => (0, 2 - row, col, -1, 0, 0),
        _ => (2 - col, 2 - row, 0, 0, 0, -1),
    }
}

fn facelet_of((x, y, z, nx, ny, nz): Sticker) -> usize {
    let at = |face: i32, row: i32, col: i32| (face * 9 + row * 3 + col) as usize;
    if ny == 1 {
        at(0, z, x)
    } else if ny == -1 {
        at(3, 2 - z, x)
    } else if nz == 1 {
        at(2, 2 - y, x)
    } else if nz == -1 {
        at(5, 2 - y, 2 - x)
    } else if nx == 1 {
        at(1, 2 - y, 2 - z)
    } else {
        at(4, 2 - y, z)
    }
}

fn turn_x((x, y, z, nx, ny, nz): Sticker) -> Sticker {
    (x, z, 2 - y, nx, nz, -ny)
}

fn turn_y((x, y, z, nx, ny, nz): Sticker) -> Sticker {
    (2 - z, y, x, -nz, ny, nx)
}

fn turn_z((x, y, z, nx, ny, nz): Sticker) -> Sticker {
    (y, 2 - x, z, ny, -nx, nz)
}

fn repeat(turn: fn(Sticker) -> Sticker, mut sticker: Sticker, times: usize) -> Sticker {
    for _ in 0..times {
        sticker = turn(sticker);
    }
    sticker
}

/// Applies a frame: ranks 0..16 are x turns then y, 16..20 add one z, 20..24 three.
fn apply_frame(full: &[usize; 54], rank: usize) -> Vec<usize> {
    let x = if rank < 16 { rank / 4 } else { 0 };
    let z = match rank {
        16..=19 => 1,
        20.. => 3,
        _ => 0,
    };
    let y = rank % 4;
    (0..54)
        .map(|index| {
            let mut sticker = sticker_of(index);
            sticker = repeat(turn_y, sticker, (4 - y) % 4);
            sticker = repeat(turn_z, sticker, (4 - z) % 4);
            sticker = repeat(turn_x, sticker, (4 - x) % 4);
            full[facelet_of(sticker)]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::PuzzleKind;

    #[test]
    fn solved_tokens_decode_to_solved_cubes() {
        assert!(decode("AAAAAAAAAAAA").unwrap().is_solved());
        assert!(decode("AAAAA").unwrap().is_solved());
    }

    #[test]
    fn the_format_compatibility_vectors_decode() {
        let vectors = [
            (
                "AAAAAAAAAL_o",
                "UBULURUFURURFRBRDRFUFLFRFDFDFDLDRDBDLULBLFLDLBUBRBLBDB",
            ),
            (
                "AAAAAAAACC-Y",
                "UUUUURUUURURBRLRDRFFFLFRFFFDDDLDRDDDLLLFLFLDLBBBBBRBBB",
            ),
            (
                "FRot3QyvoAAA",
                "UUUUUUUUUBBBRRRRRRRRRFFFFFFDDDDDDDDDFFFLLLLLLLLLBBBBBB",
            ),
        ];
        for (token, expected) in vectors {
            assert_eq!(decode(token).unwrap().to_string(), expected, "{token}");
        }
    }

    #[test]
    fn reference_3x3_examples_including_nonzero_frames() {
        let vectors = [
            (
                "AebrMQDGIp0Y",
                "DUDLLFUDULUBFULFRUFBBRFDDDRBRURRFDBRFFLLDBFULRLRDBBBUL",
            ),
            (
                "B_7na2IjVReI",
                "RBDLLDBFURFBDUDUFFLRFBFBFLLRUFRRUBLLUDDRDLLUDRUBRBBDFU",
            ),
            (
                "M3Fo9OBFtLeH",
                "BDBLFLLFBRUDFDRFURBUULLDULLFDDRBBFRFDBUBUFRFLLBRDRRDUU",
            ),
            (
                "D43T566hd_BK",
                "LDFRDRRDLFURDLUUBFULDLFBBLLUUFBUURRDDBBFRFDLLURBFBDRFB",
            ),
            (
                "AAAAAAAAAAAA",
                "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
            ),
            (
                "AAAABeBQZgAA",
                "UDUDUDUDURLRLRLRLRFBFBFBFBFDUDUDUDUDLRLRLRLRLBFBFBFBFB",
            ),
            (
                "KJZ7jorgHRfo",
                "LLLLULLLLFFFFRFFFFDDDDFDDDDRRRRDRRRRBBBBLBBBBUUUUBUUUU",
            ),
            (
                "KCvKi0WENEAA",
                "DDDDUDDDDLLLLRLLLLFFFFFFFFFUUUUDUUUURRRRLRRRRBBBBBBBBB",
            ),
            (
                "AAAAA-3ogAAA",
                "UDUDUDUDURRRRRRRRRFFFFFFFFFDUDUDUDUDLLLLLLLLLBBBBBBBBB",
            ),
            (
                "AAAAAAAAAL_o",
                "UBULURUFURURFRBRDRFUFLFRFDFDFDLDRDBDLULBLFLDLBUBRBLBDB",
            ),
        ];
        for (token, expected) in vectors {
            assert_eq!(decode(token).unwrap().to_string(), expected, "{token}");
        }
    }

    #[test]
    fn reference_2x2_examples() {
        let vectors = [
            ("ASj79", "LFDRFLDBFURLBBLUURUDDFRB"),
            ("CrRYm", "FURRDLUBFFLRDBBLDURBFLUD"),
            ("Amjka", "RFDURDFURFFLLDLBUBBURBLD"),
            ("BVQaG", "UURBULBDFLDRFDDFBULRFRLB"),
            ("ALV8G", "BULRFRLFBUDDFFLUDUBRBRLD"),
            ("CMS0m", "RUBBRFDDUUDLRBUFDLLBRFLF"),
        ];
        for (token, expected) in vectors {
            assert_eq!(decode(token).unwrap().to_string(), expected, "{token}");
        }
    }

    #[test]
    fn the_user_token_decodes_to_the_known_cube() {
        assert_eq!(
            decode("A4ib8aejxj4Y").unwrap().to_string(),
            "FUFUUFFFFUUUURRURURRRFFRRFRBDBBDDBBBDLDDLLDDDLBLBBLLLL"
        );
    }

    #[test]
    fn rejects_bad_tokens() {
        assert_eq!(decode("AAAA"), Err(Orbit64Error::BadLength(4)));
        assert!(matches!(
            decode("AAAA+"),
            Err(Orbit64Error::BadCharacter {
                position: 4,
                found: '+'
            })
        ));
        assert_eq!(decode("__________"), Err(Orbit64Error::BadLength(10)));
        assert_eq!(decode("____________"), Err(Orbit64Error::OutOfRange));
        assert_eq!(decode(&"A".repeat(27)), Err(Orbit64Error::Unsupported(4)));
    }

    #[test]
    fn token_detection_uses_width_and_alphabet() {
        assert!(looks_like_token("A4ib8aejxj4Y"));
        assert!(!looks_like_token("A4ib8aejxj4"));
        assert!(!looks_like_token("A4ib8aejxj4+"));
        assert_eq!(
            PuzzleKind::of_size(size_of_width(5).unwrap()),
            Some(PuzzleKind::Cube2)
        );
    }
}
