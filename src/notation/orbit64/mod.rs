//! Orbit64 state tokens: a canonical base64url encoding of a cube state.
//!
//! State tokens are Orbit64 V2, specified by `FORMAT-V2.md` in
//! <https://github.com/wstein/flix-orbit64>. This module is the color-cube
//! facelet bridge: it writes the color framed entry and reads any color entry
//! Cubacle can show (sizes 2 through 7). Move sequences and structured
//! algorithms stay in `FORMAT.md` (classes `01` and `10`). The algorithm DAG
//! envelope is a separate unaccepted draft and is not implemented here.
//!
//! Every size from the 2x2x2 to the 7x7x7 is decoded and encoded by the same
//! code; the orbit structure and the slot tables are derived from the size
//! (see `layout`). Slots follow the convention `orbit64-Nxn-draft@1` of the
//! format: the standard Kociemba order for corners (URF UFL ULB UBR DFR DLF DBL
//! DRB) and midges (UR UF UL UB DR DF DL DB FR FL BL BR), wings and centers by
//! facelet index. A token carries ordinals only, so it cannot prove its
//! producer used the same numbering; the reference vectors of the format are
//! the evidence it matches: facelet vectors for the 2x2x2, 3x3x3, 4x4x4, 6x6x6
//! and 7x7x7. A 5x5x5 class-`00` token is 43 characters and is rejected; the
//! color framed 5x5x5 entry is 42.
//!
//! Origin: Rust re-implementation of flix-orbit64 (Apache-2.0, Werner Stein).

use std::fmt;

use super::impossible::Impossible;
use crate::puzzle::{Facelets, FaceletsError};

use crate::puzzle::Face;

mod big;
mod coords;
mod frame;
mod layout;
mod pieces;
pub mod v2;

use big::Big;
use coords::Coords;
use layout::Layout;

pub(crate) use layout::{CORNER_COLORS, CORNER_FACELETS};

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
    Impossible(Impossible),
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
                "{len} characters is not an Orbit64 V2 color-token length this solver reads"
            ),
            Orbit64Error::OutOfRange => write!(f, "orbit64 token is out of range for its cube"),
            Orbit64Error::NotACube(error) => write!(f, "orbit64 token is not a cube: {error}"),
            Orbit64Error::Impossible(why) => write!(f, "this is not a cube state ({why})"),
        }
    }
}

impl std::error::Error for Orbit64Error {}

/// The facelets of the eight corner slots of an `n`x`n`x`n`, in the format's slot
/// order (URF UFL ULB UBR DFR DLF DBL DRB), each in turning order. `None`
/// outside the sizes with a token.
pub(crate) fn corner_slots(n: usize) -> Option<Vec<Vec<usize>>> {
    SIZES.contains(&n).then(|| Layout::new(n).corners)
}

/// The facelets of the six fixed centers of an odd `n`x`n`x`n` in `URFDLB`
/// order; empty on even cubes and outside the sizes with a token.
pub(crate) fn fixed_centers(n: usize) -> Vec<usize> {
    if SIZES.contains(&n) {
        Layout::new(n).fixed
    } else {
        Vec::new()
    }
}

/// The color framed V2 entry for a cube size Cubacle can show, if there is one.
fn framed_color(size: usize) -> Option<&'static v2::Entry> {
    if !SIZES.contains(&size) {
        return None;
    }
    v2::entries().iter().find(|entry| {
        entry.n == size && entry.model == v2::Model::Color && entry.pose == v2::Pose::Framed
    })
}

/// Whether a color V2 entry of a supported size has this token length.
fn color_length(width: usize) -> bool {
    v2::entries().iter().any(|entry| {
        entry.length == width && entry.model == v2::Model::Color && SIZES.contains(&entry.n)
    })
}

/// Whether the color framed entry of a supported size has this token length.
///
/// These are the lengths [`encode`] writes: 5, 12, 27, 42, 66 and 90. The
/// canonical-pose lengths 4, 11, 26, 65 and 89 are real V2 entries, and they
/// are also the lengths of ordinary move expressions (`F2 B2`, `D L2 B`),
/// so the reader does not treat them as tokens.
fn framed_color_length(width: usize) -> bool {
    v2::entries().iter().any(|entry| {
        entry.length == width
            && entry.model == v2::Model::Color
            && entry.pose == v2::Pose::Framed
            && SIZES.contains(&entry.n)
    })
}

/// The number of characters of the color framed state token of the
/// `size`x`size`x`size`.
///
/// This is the token [`encode`] writes. V2 has no class bits, so the length is
/// the registry entry's length, not the old class-`00` width.
pub fn token_width(size: usize) -> Option<usize> {
    framed_color(size).map(|entry| entry.length)
}

/// The cube size whose color framed state tokens have this width.
pub fn size_of_width(width: usize) -> Option<usize> {
    let mut found = v2::entries().iter().filter_map(|entry| {
        (entry.length == width
            && entry.model == v2::Model::Color
            && entry.pose == v2::Pose::Framed
            && SIZES.contains(&entry.n))
        .then_some(entry.n)
    });
    let size = found.next()?;
    found.next().is_none().then_some(size)
}

/// Whether the text could be an Orbit64 V2 color framed state token: a framed
/// color length for a supported size, and base64url characters only.
///
/// Canonical-pose lengths are not included. A string of one of those lengths
/// is read as moves when it is a move expression.
pub fn looks_like_token(text: &str) -> bool {
    framed_color_length(text.chars().count()) && text.bytes().all(|byte| ALPHABET.contains(&byte))
}

/// Whether `text` is a retired class-`00` 5×5×5 token: 43 base64url characters.
///
/// That width is not a V2 color length. Every other class-`00` width of a
/// size Cubacle reads is also the color framed V2 length.
pub fn is_retired_v1_token(text: &str) -> bool {
    text.chars().count() == 43 && text.bytes().all(|byte| ALPHABET.contains(&byte))
}

fn from_v2(error: v2::V2Error) -> Orbit64Error {
    match error {
        v2::V2Error::BadCharacter { position, found } => {
            Orbit64Error::BadCharacter { position, found }
        }
        v2::V2Error::UnknownToken | v2::V2Error::UnknownEntry(_) => Orbit64Error::OutOfRange,
        v2::V2Error::Invalid(_) | v2::V2Error::Facelets(_) | v2::V2Error::Unsupported(_) => {
            Orbit64Error::OutOfRange
        }
    }
}

/// Decodes a V2 color state token into the facelets it shows.
///
/// Never panics: a token that is too long, too short, outside the alphabet,
/// outside every color entry, or not a color cube of a supported size is an
/// error. A picture token is out of range here; picture states are read
/// through [`v2`].
pub fn decode(token: &str) -> Result<Facelets, Orbit64Error> {
    let len = token.chars().count();
    let (entry, text) = match v2::facelets::decode(token) {
        Ok(pair) => pair,
        Err(v2::V2Error::BadCharacter { position, found }) => {
            return Err(Orbit64Error::BadCharacter { position, found });
        }
        Err(v2::V2Error::UnknownToken) if !color_length(len) => {
            return Err(Orbit64Error::BadLength(len));
        }
        Err(error) => return Err(from_v2(error)),
    };
    if entry.model != v2::Model::Color || !SIZES.contains(&entry.n) {
        return Err(Orbit64Error::OutOfRange);
    }
    let compact: String = text.chars().filter(|c| *c != ' ').collect();
    Facelets::parse(&compact).map_err(Orbit64Error::NotACube)
}

/// Encodes facelets as the color framed V2 state token; the inverse of [`decode`]
/// for tokens this function writes.
///
/// An odd cube's fixed centers select the stored frame. Facelets whose pieces
/// are not those of a cube (stickers swapped in place, a twisted corner,
/// corner and edge parity that disagree, moved fixed centers that are not a
/// rotation) are an error, not a token. An even cube has no fixed centers and
/// is encoded as it stands. The token is the V2 color framed entry, not the
/// retired class-`00` ranking. A 5×5×5 token is 42 characters.
pub fn encode(facelets: &Facelets) -> Result<String, Orbit64Error> {
    ensure_reachable(facelets)?;
    let entry = framed_color(facelets.kind().size())
        .ok_or(Orbit64Error::BadLength(facelets.kind().size()))?;
    v2::facelets::encode(entry.id, &facelets.to_string()).map_err(from_v2)
}

/// The class-`00` width. Sizes other than 5×5×5 share it with the framed V2
/// token; the 5×5×5 class-`00` token is 43 characters.
fn class00_width(size: usize) -> Option<usize> {
    match size {
        2 => Some(5),
        3 => Some(12),
        4 => Some(27),
        5 => Some(43),
        6 => Some(66),
        7 => Some(90),
        _ => None,
    }
}

/// The class-`00` token of a reachable cube.
///
/// The canonical identifier still uses this encoding, so a 5×5×5 identifier
/// stays the 43-character token of flix-orbit64 v0.9.0. State tokens go
/// through [`encode`], which writes the color framed V2 entry.
pub(crate) fn encode_class00(facelets: &Facelets) -> Result<String, Orbit64Error> {
    let size = facelets.kind().size();
    let layout = Layout::new(size);
    let mut stickers: Vec<usize> = facelets
        .stickers()
        .iter()
        .map(|face| face.index())
        .collect();

    let mut frame = None;
    if layout.has_fixed_centers() {
        let found = (0..frame::FRAME_COUNT).find_map(|rank| {
            let upright = frame::unapply(size, &stickers, rank);
            let centered = layout
                .fixed
                .iter()
                .enumerate()
                .all(|(face, &at)| upright[at] == face);
            centered.then_some((rank, upright))
        });
        let (rank, upright) = found.ok_or(Orbit64Error::Impossible(Impossible::FixedCenters))?;
        frame = Some(rank);
        stickers = upright;
    }

    let coords = pieces::read(&layout, &stickers).map_err(Orbit64Error::Impossible)?;
    let mut digits = coords.digits(size).map_err(Orbit64Error::Impossible)?;
    digits.extend(frame.map(|rank| rank as u32));

    let radices = coords::radices(size);
    debug_assert_eq!(digits.len(), radices.len());
    let mut value = Big::zero();
    for (digit, radix) in digits.iter().zip(&radices) {
        value.mul_add(*radix, *digit);
    }
    let width = class00_width(size).ok_or(Orbit64Error::BadLength(size))?;
    value.to_base64(width).ok_or(Orbit64Error::OutOfRange)
}

/// The facelets of a class-`00` token. State input does not call this: a
/// 43-character token is refused there. The canonical form reads the
/// identifier it still writes.
pub(crate) fn decode_class00(token: &str) -> Result<Facelets, Orbit64Error> {
    let size = class00_width_of(token.chars().count())
        .ok_or_else(|| Orbit64Error::BadLength(token.chars().count()))?;
    let mut value = Big::from_base64(token)
        .map_err(|(position, found)| Orbit64Error::BadCharacter { position, found })?;

    let mut digits: Vec<u32> = coords::radices(size)
        .iter()
        .rev()
        .map(|&radix| value.div_rem(radix))
        .collect();
    if !value.is_zero() {
        return Err(Orbit64Error::OutOfRange);
    }
    digits.reverse();

    let layout = Layout::new(size);
    let odd = layout.has_fixed_centers();
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

fn class00_width_of(width: usize) -> Option<usize> {
    (2..=7).find(|&size| class00_width(size) == Some(width))
}

/// Refuses a coloring that is not a reachable cube. The token is written
/// from the original facelets, frame included.
fn ensure_reachable(facelets: &Facelets) -> Result<(), Orbit64Error> {
    let size = facelets.kind().size();
    let layout = Layout::new(size);
    let mut stickers: Vec<usize> = facelets
        .stickers()
        .iter()
        .map(|face| face.index())
        .collect();

    if layout.has_fixed_centers() {
        let upright = (0..frame::FRAME_COUNT).find_map(|rank| {
            let upright = frame::unapply(size, &stickers, rank);
            let centered = layout
                .fixed
                .iter()
                .enumerate()
                .all(|(face, &at)| upright[at] == face);
            centered.then_some(upright)
        });
        stickers = upright.ok_or(Orbit64Error::Impossible(Impossible::FixedCenters))?;
    }

    let coords = pieces::read(&layout, &stickers).map_err(Orbit64Error::Impossible)?;
    // `digits` is the reachability check (twist, flip, parity, distinct pieces).
    coords.digits(size).map_err(Orbit64Error::Impossible)?;
    Ok(())
}

#[cfg(test)]
mod tests;
