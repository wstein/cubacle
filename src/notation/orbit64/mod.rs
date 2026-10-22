//! Orbit64 state tokens: a canonical base64url encoding of a cube state.
//!
//! The format is defined by <https://github.com/wstein/flix-orbit64> (`FORMAT.md`).
//! A token's length identifies the cube size; the state is a mixed-radix number
//! over the cube's orbits, followed on odd cubes by one of 24 whole-cube frames.
//!
//! Every size from the 2x2x2 to the 7x7x7 is decoded and encoded by the same
//! code; the orbit structure and the slot tables are derived from the size
//! (see `layout`). Slots follow the convention `orbit64-Nxn-draft@1` of the
//! format: the standard Kociemba order for corners (URF UFL ULB UBR DFR DLF DBL
//! DRB) and midges (UR UF UL UB DR DF DL DB FR FL BL BR), wings and centres by
//! facelet index. A token carries ordinals only, so it cannot prove its
//! producer used the same numbering; the reference vectors of the format are
//! the evidence it matches, and they exist for the 2x2x2 to 5x5x5 (the 6x6x6
//! and 7x7x7 only for the solved cube).
//!
//! Origin: Rust re-implementation of flix-orbit64 (Apache-2.0, Werner Stein).

use std::fmt;

use crate::puzzle::{Face, Facelets, FaceletsError};

mod big;
mod coords;
mod frame;
mod layout;
mod pieces;

use big::Big;
use coords::{radices, Coords};
use layout::Layout;

pub(crate) use layout::{CORNER_COLOURS, CORNER_FACELETS};

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// The cube sizes with an Orbit64 state token.
const SIZES: std::ops::RangeInclusive<usize> = 2..=7;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Orbit64Error {
    /// The token has a character outside the base64url alphabet.
    BadCharacter { position: usize, found: char },
    /// The token's length is not that of any cube size.
    BadLength(usize),
    /// The number is at or above the number of states of the cube.
    OutOfRange,
    /// The token decoded to something that is not a cube.
    NotACube(FaceletsError),
    /// The facelets are well formed but their pieces cannot be those of a cube,
    /// for example a twisted corner or mismatched corner and edge parity.
    Impossible(String),
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
            Orbit64Error::OutOfRange => write!(f, "orbit64 token is out of range for its cube"),
            Orbit64Error::NotACube(error) => write!(f, "orbit64 token is not a cube: {error}"),
            Orbit64Error::Impossible(why) => write!(f, "this is not a cube state: {why}"),
        }
    }
}

impl std::error::Error for Orbit64Error {}

/// The number of characters of a state token of the `size`x`size`x`size`.
///
/// It is the smallest width for which every state fits while the first
/// character stays in the state class (`A` to `P`, a leading `00` sextet).
pub fn token_width(size: usize) -> Option<usize> {
    if !SIZES.contains(&size) {
        return None;
    }
    let mut states = Big::one();
    for radix in radices(size) {
        states.mul_add(radix, 0);
    }
    (1..).find(|&width| states.at_most_power_of_two(4 + 6 * (width - 1)))
}

/// The cube size whose state tokens have this width.
pub fn size_of_width(width: usize) -> Option<usize> {
    SIZES
        .into_iter()
        .find(|&size| token_width(size) == Some(width))
}

/// Whether the text could be an Orbit64 state token: a state width, base64url only.
pub fn looks_like_token(text: &str) -> bool {
    size_of_width(text.chars().count()).is_some()
        && text.bytes().all(|byte| ALPHABET.contains(&byte))
}

/// Decodes a state token into the facelets it shows, whole-cube frame applied.
///
/// Never panics: a token that is too long, too short, outside the alphabet or
/// above the number of states of its cube is an error.
pub fn decode(token: &str) -> Result<Facelets, Orbit64Error> {
    let size = size_of_width(token.chars().count())
        .ok_or_else(|| Orbit64Error::BadLength(token.chars().count()))?;
    let mut value = Big::from_base64(token)
        .map_err(|(position, found)| Orbit64Error::BadCharacter { position, found })?;

    let mut digits: Vec<u32> = radices(size)
        .iter()
        .rev()
        .map(|&radix| value.div_rem(radix))
        .collect();
    if !value.is_zero() {
        return Err(Orbit64Error::OutOfRange);
    }
    digits.reverse();

    let layout = Layout::new(size);
    let odd = layout.has_fixed_centres();
    let frame = if odd {
        digits.pop().unwrap_or(0) as usize
    } else {
        0
    };
    let coords = Coords::from_digits(size, &digits);
    let mut stickers = pieces::place(&layout, &coords);
    if odd {
        stickers = frame::apply(size, &stickers, frame);
    }
    let faces = stickers.iter().map(|&index| Face::ALL[index]).collect();
    Facelets::from_stickers(faces).map_err(Orbit64Error::NotACube)
}

/// Encodes facelets as a state token; the inverse of [`decode`].
///
/// An odd cube's fixed centres select the stored frame. Facelets whose pieces
/// are not those of a cube (a stickers swapped in place, a twisted corner,
/// corner and edge parity that disagree, moved fixed centres that are not a
/// rotation) are an error, not a token. An even cube has no fixed centres and
/// is encoded as it stands.
pub fn encode(facelets: &Facelets) -> Result<String, Orbit64Error> {
    let size = facelets.kind().size();
    let layout = Layout::new(size);
    let mut stickers: Vec<usize> = facelets
        .stickers()
        .iter()
        .map(|face| face.index())
        .collect();

    let mut frame = None;
    if layout.has_fixed_centres() {
        let found = (0..frame::FRAME_COUNT).find_map(|rank| {
            let upright = frame::unapply(size, &stickers, rank);
            let centred = layout
                .fixed
                .iter()
                .enumerate()
                .all(|(face, &at)| upright[at] == face);
            centred.then_some((rank, upright))
        });
        let (rank, upright) = found.ok_or_else(|| {
            Orbit64Error::Impossible("the fixed centres are not a rotation of the cube".into())
        })?;
        frame = Some(rank);
        stickers = upright;
    }

    let coords = pieces::read(&layout, &stickers).map_err(Orbit64Error::Impossible)?;
    let mut digits = coords.digits(size).map_err(Orbit64Error::Impossible)?;
    digits.extend(frame.map(|rank| rank as u32));

    let radices = radices(size);
    debug_assert_eq!(digits.len(), radices.len());
    let mut value = Big::zero();
    for (digit, radix) in digits.iter().zip(&radices) {
        value.mul_add(*radix, *digit);
    }
    let width = token_width(size).unwrap_or(0);
    value
        .to_base64(width)
        .ok_or_else(|| Orbit64Error::Impossible("the state does not fit its token".into()))
}

#[cfg(test)]
mod tests;
