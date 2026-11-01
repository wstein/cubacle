//! Origin: Rust re-implementation of `Orbit64.V2.Facelets` (flix-orbit64,
//! Apache-2.0, Werner Stein), following `FORMAT-V2.md`.
//!
//! The facelet bridge of Orbit64 V2: a cube state to its stickers and back.
//!
//! A facelet string writes the `6 n^2` stickers one per facelet, in six face
//! fields in the order `U R F D L B` separated by one space, in the layout of
//! `FORMAT.md` (facelet `face * n^2 + row * n + col`). Reading ignores spaces
//! and nothing else: a tab or a newline is a bad symbol.
//!
//! The color model writes the letters `U R F D L B`. The picture model, where
//! every sticker is distinct, writes one base64url symbol per facelet: the
//! sticker's index among the stickers of its class (corner `3 p + j`, midge
//! and wing `2 p + j`, center orbit `p`, fixed center `4 p + r`), see
//! "Picture facelets" in `FORMAT-V2.md`. The picture model has no 2x2x2.
//!
//! The slot tables are those of V1 (`layout.rs`), which extend to every
//! `n = 2..=15` by the rule of `FORMAT-V2.md`; `FORMAT-V2-SLOTS.md` is
//! normative and a test compares them. A piece is numbered by its home slot
//! and the solved cube shows face colors, so a piece's colors are the faces
//! of its home slot's facelets and no color table is needed.
//!
//! A canonical read turns the cube so that its reference is home first (the
//! fixed centers of an odd cube, the DBL corner of an even one), so it accepts
//! any whole-cube orientation. A framed read of an odd cube keeps the
//! orientation as `Cube::frame`; of an even cube it stays in the corners.

use super::cube::{is_odd, Cube};
use super::geometry::{rotate, rotate_picture, unrotate, FRAME_COUNT, FRAME_FACES};
use super::registry::{Entry, Model, Pose};
use super::{digits, entry, V2Error};
use crate::notation::orbit64::layout::Layout;
use crate::notation::orbit64::ALPHABET;

mod picture;

const LETTERS: [char; 6] = ['U', 'R', 'F', 'D', 'L', 'B'];

/// The corner slot of the DBL corner, home in a canonical even cube.
const DBL: usize = 6;

fn unsupported(why: String) -> V2Error {
    V2Error::Unsupported(why)
}

fn malformed(why: String) -> V2Error {
    V2Error::Facelets(why)
}

/// Why a type has no facelet string, if it has none.
fn type_fault(n: usize, model: Model) -> Result<(), V2Error> {
    if !(2..=15).contains(&n) {
        return Err(unsupported(format!("no V2 type for the {n}x{n}x{n}")));
    }
    if n == 2 && model == Model::Picture {
        return Err(unsupported(
            "the 2x2x2 has no picture entry: its picture model is the color model".to_string(),
        ));
    }
    Ok(())
}

/// The face color of a facelet: on the solved cube a sticker shows its face.
fn face_of(n: usize, facelet: usize) -> usize {
    facelet / (n * n)
}

/// The written form of a sticker value: a letter in the color model, a
/// base64url symbol in the picture model.
fn symbol(model: Model, value: usize) -> char {
    match model {
        Model::Color => LETTERS[value],
        Model::Picture => char::from(ALPHABET[value]),
    }
}

// ------------------------------------------------------------ cube to stickers

/// What the `j`-th sticker of the piece `piece` in a table of slots with `m`
/// stickers shows: the color of its home facelet, or in the picture model its
/// index `m p + j`.
fn shown(n: usize, model: Model, table: &[Vec<usize>], piece: usize, j: usize) -> usize {
    match model {
        Model::Color => face_of(n, table[piece][j]),
        Model::Picture => table[piece].len() * piece + j,
    }
}

fn write_oriented(
    n: usize,
    model: Model,
    sticker: &mut [usize],
    table: &[Vec<usize>],
    pieces: &[usize],
    twists: Option<&[usize]>,
) {
    for (slot, facelets) in table.iter().enumerate() {
        let m = facelets.len();
        let twist = twists.map_or(0, |t| t[slot]);
        for (k, &at) in facelets.iter().enumerate() {
            sticker[at] = shown(n, model, table, pieces[slot], (k + m - twist) % m);
        }
    }
}

/// The stickers of a canonical-pose cube. The fixed centers of an odd cube sit
/// at home, showing their face, or in the picture model their piece and
/// rotation `4 p + r`.
fn paint(n: usize, model: Model, layout: &Layout, cube: &Cube) -> Vec<usize> {
    let mut sticker = vec![0; 6 * n * n];
    for (face, &at) in layout.fixed.iter().enumerate() {
        sticker[at] = match model {
            Model::Color => face,
            Model::Picture => 4 * face + cube.rotations[face],
        };
    }
    let twists = Some(&cube.corner_twists[..]);
    let corners = &layout.corners;
    write_oriented(n, model, &mut sticker, corners, &cube.corner_pieces, twists);
    if is_odd(n) {
        let flips = Some(&cube.midge_flips[..]);
        let (midges, pieces) = (&layout.midges, &cube.midge_pieces);
        write_oriented(n, model, &mut sticker, midges, pieces, flips);
    }
    for (table, wing) in layout.wings.iter().zip(&cube.wings) {
        write_oriented(n, model, &mut sticker, table, wing, None);
    }
    for (table, colors) in layout.centers.iter().zip(&cube.centers) {
        for (&at, &color) in table.iter().zip(colors) {
            sticker[at] = color;
        }
    }
    sticker
}

/// The facelet string of a cube state of the type `(n, model, pose)`.
///
/// A canonical cube is written in its reference pose. A framed cube of an odd
/// size is that picture rotated by `cube.frame`; a framed cube of an even size
/// shows its pose in its corners already. The color model writes letters and
/// the picture model base64url symbols. A cube that is not a state of the
/// type is an error, and so is a size outside `2..=15` or a picture 2x2x2.
pub fn to_facelets(n: usize, model: Model, pose: Pose, cube: &Cube) -> Result<String, V2Error> {
    type_fault(n, model)?;
    cube.validate(n, model, pose)?;
    digits::digits(n, model, pose, cube)?;
    let stickers = paint(n, model, &Layout::new(n), cube);
    let stickers = match (is_odd(n), model) {
        (false, _) => stickers,
        (true, Model::Color) => rotate(n, cube.frame, &stickers),
        (true, Model::Picture) => rotate_picture(n, cube.frame, &stickers),
    };
    Ok(spell(n, model, &stickers))
}

fn spell(n: usize, model: Model, stickers: &[usize]) -> String {
    stickers
        .chunks(n * n)
        .map(|face| face.iter().map(|&v| symbol(model, v)).collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

// ------------------------------------------------------------ stickers to cube

/// The value of a letter or symbol, or why it is none.
fn value_of(model: Model, c: char) -> Result<usize, V2Error> {
    let found = match model {
        Model::Color => LETTERS.iter().position(|&letter| letter == c),
        Model::Picture => ALPHABET.iter().position(|&byte| char::from(byte) == c),
    };
    found.ok_or_else(|| {
        malformed(match model {
            Model::Color => format!("facelet '{c}' is not one of U R F D L B"),
            Model::Picture => format!("facelet '{c}' is not a base64url symbol"),
        })
    })
}

/// The values of a facelet string, or why it is no string of `n^2` per face.
fn parse(n: usize, model: Model, text: &str) -> Result<Vec<usize>, V2Error> {
    // Every character is checked, so the first bad one is reported before the
    // length; only the first 6 n^2 are kept, so a huge input costs no memory.
    let wanted = 6 * n * n;
    let mut values = Vec::with_capacity(wanted);
    let mut count = 0usize;
    for c in text.chars().filter(|&c| c != ' ') {
        let value = value_of(model, c)?;
        if count < wanted {
            values.push(value);
        }
        count += 1;
    }
    if count != wanted {
        return Err(malformed(format!(
            "expected {wanted} facelets for the {n}x{n}x{n}, got {count}"
        )));
    }
    Ok(values)
}
fn check_counts(n: usize, stickers: &[usize]) -> Result<(), V2Error> {
    for (face, letter) in LETTERS.iter().enumerate() {
        let count = stickers.iter().filter(|&&s| s == face).count();
        if count != n * n {
            return Err(malformed(format!(
                "color {letter} occurs {count} times, expected {}",
                n * n
            )));
        }
    }
    Ok(())
}

/// The stickers on the facelets of one slot.
fn seen_at(table: &[Vec<usize>], slot: usize, stickers: &[usize]) -> Vec<usize> {
    table[slot].iter().map(|&at| stickers[at]).collect()
}

/// The piece and twist whose colors, read from the slot's first facelet, are
/// `seen`.
fn read_piece(n: usize, table: &[Vec<usize>], seen: &[usize]) -> Option<(usize, usize)> {
    let m = seen.len();
    (0..table.len()).find_map(|piece| {
        (0..m)
            .find(|&twist| (0..m).all(|k| seen[k] == face_of(n, table[piece][(k + m - twist) % m])))
            .map(|twist| (piece, twist))
    })
}

fn no_piece(model: Model, what: &str, slot: usize, seen: &[usize]) -> V2Error {
    let shown: String = seen.iter().map(|&v| symbol(model, v)).collect();
    malformed(format!(
        "{what} slot {slot} shows {shown}, which is no {what} of this cube"
    ))
}

fn read_oriented(
    n: usize,
    what: &str,
    table: &[Vec<usize>],
    stickers: &[usize],
) -> Result<(Vec<usize>, Vec<usize>), V2Error> {
    (0..table.len())
        .map(|slot| {
            let seen = seen_at(table, slot, stickers);
            read_piece(n, table, &seen).ok_or_else(|| no_piece(Model::Color, what, slot, &seen))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|found| found.into_iter().unzip())
}

/// A wing carries no twist: its two stickers sit in the order of its home
/// slot or not at all.
fn read_wings(n: usize, table: &[Vec<usize>], stickers: &[usize]) -> Result<Vec<usize>, V2Error> {
    let solved: Vec<usize> = (0..6 * n * n).map(|i| face_of(n, i)).collect();
    let homes: Vec<Vec<usize>> = (0..table.len())
        .map(|piece| seen_at(table, piece, &solved))
        .collect();
    (0..table.len())
        .map(|slot| {
            let seen = seen_at(table, slot, stickers);
            homes
                .iter()
                .position(|home| *home == seen)
                .ok_or_else(|| no_piece(Model::Color, "wing", slot, &seen))
        })
        .collect()
}

/// The coordinates of a canonical-pose string; the frame is `frame`.
fn read_cube(n: usize, layout: &Layout, frame: usize, stickers: &[usize]) -> Result<Cube, V2Error> {
    let (corner_pieces, corner_twists) = read_oriented(n, "corner", &layout.corners, stickers)?;
    let (midge_pieces, midge_flips) = if is_odd(n) {
        read_oriented(n, "midge", &layout.midges, stickers)?
    } else {
        (Vec::new(), Vec::new())
    };
    let wings = layout
        .wings
        .iter()
        .map(|table| read_wings(n, table, stickers))
        .collect::<Result<Vec<_>, _>>()?;
    let centers = layout
        .centers
        .iter()
        .map(|table| table.iter().map(|&at| stickers[at]).collect())
        .collect();
    Ok(Cube {
        corner_pieces,
        corner_twists,
        midge_pieces,
        midge_flips,
        wings,
        centers,
        rotations: Vec::new(),
        frame,
    })
}

/// Rotates an odd cube so that its fixed centers are home; the frame is the
/// one they show.
fn normalize_odd(layout: &Layout, stickers: &[usize]) -> Result<(usize, Vec<usize>), V2Error> {
    let shown: Vec<usize> = layout.fixed.iter().map(|&at| stickers[at]).collect();
    let frame = FRAME_FACES
        .iter()
        .position(|faces| faces[..] == shown[..])
        .ok_or_else(|| {
            malformed("the six fixed centers are not a right-handed whole-cube frame".to_string())
        })?;
    Ok((frame, unrotate(layout.n, frame, stickers)))
}

/// Rotates an even cube so that its DBL corner is in slot DBL with twist 0.
fn normalize_even(layout: &Layout, stickers: &[usize]) -> Result<Vec<usize>, V2Error> {
    (0..FRAME_COUNT)
        .map(|frame| unrotate(layout.n, frame, stickers))
        .find(|rotated| {
            let seen = seen_at(&layout.corners, DBL, rotated);
            read_piece(layout.n, &layout.corners, &seen) == Some((DBL, 0))
        })
        .ok_or_else(|| malformed("no whole-cube rotation puts the DBL corner home".to_string()))
}

/// The frame to record and the stickers in the pose the type reads them in.
fn posed(
    layout: &Layout,
    pose: Pose,
    stickers: Vec<usize>,
) -> Result<(usize, Vec<usize>), V2Error> {
    if is_odd(layout.n) {
        let (frame, upright) = normalize_odd(layout, &stickers)?;
        Ok((if pose == Pose::Framed { frame } else { 0 }, upright))
    } else if pose == Pose::Canonical {
        Ok((0, normalize_even(layout, &stickers)?))
    } else {
        Ok((0, stickers))
    }
}

/// The cube state shown by a facelet string, in the pose of `pose`.
///
/// Anything that is no reachable cube is an error: a letter or symbol outside
/// the alphabet of the model, a wrong length, wrong color counts or a sticker
/// of the wrong class or twice (picture), a slot that shows no piece, fixed
/// centers that are not a right-handed frame, a piece twice, a twisted
/// corner, a flipped midge or a broken parity. Never panics.
pub fn from_facelets(n: usize, model: Model, pose: Pose, facelets: &str) -> Result<Cube, V2Error> {
    type_fault(n, model)?;
    if model == Model::Picture {
        return picture::read(n, pose, facelets);
    }
    let layout = Layout::new(n);
    let stickers = parse(n, model, facelets)?;
    check_counts(n, &stickers)?;
    let (frame, stickers) = posed(&layout, pose, stickers)?;
    let cube = read_cube(n, &layout, frame, &stickers)?;
    cube.validate(n, model, pose)?;
    digits::digits(n, model, pose, &cube)?;
    Ok(cube)
}

// ---------------------------------------------------- registry level helpers

/// The V2 token of the cube a facelet string shows, for registry entry `id`.
pub fn encode(id: usize, facelets: &str) -> Result<String, V2Error> {
    let entry = entry(id).ok_or(V2Error::UnknownEntry(id))?;
    let cube = from_facelets(entry.n, entry.model, entry.pose, facelets)?;
    super::encode(id, &cube)
}

/// The registry entry of a token and the facelet string of its state.
pub fn decode(token: &str) -> Result<(&'static Entry, String), V2Error> {
    let (entry, cube) = super::decode(token)?;
    let text = to_facelets(entry.n, entry.model, entry.pose, &cube)?;
    Ok((entry, text))
}

#[cfg(test)]
mod tests;
