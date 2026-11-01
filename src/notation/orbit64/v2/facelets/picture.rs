//! Origin: Rust re-implementation of the picture reading of
//! `Orbit64.V2.Facelets` (flix-orbit64, Apache-2.0, Werner Stein), following
//! "Picture facelets" in `FORMAT-V2.md`.
//!
//! Reading a picture string, in the order the format fixes: the symbols and
//! their count (`parse`), then per sticker class a value out of range or a
//! repeated sticker, then every slot spelling one piece, then the pose, then
//! the laws, which `Cube::validate` checks. The steps up to the pieces
//! guarantee the layout, so the laws see a sound state.

use super::super::cube::{is_odd, Cube};
use super::super::geometry::{unrotate, unrotate_picture, FRAME_COUNT, FRAME_FACES};
use super::super::registry::{Model, Pose};
use super::super::{digits, V2Error};
use super::{malformed, no_piece, parse, symbol, DBL};
use crate::notation::orbit64::layout::Layout;

/// The stickers of one class: where they sit, how many values they can show,
/// and `per`, how many values name the same piece.
struct Group {
    what: &'static str,
    facelets: Vec<usize>,
    limit: usize,
    per: usize,
}

fn group(what: &'static str, facelets: Vec<usize>, limit: usize, per: usize) -> Group {
    Group {
        what,
        facelets,
        limit,
        per,
    }
}

/// Every sticker class of a cube, in the order the format checks them: corner,
/// midge, wings (outermost orbit first), centers (orbit order), fixed. A
/// sticker never leaves its class.
fn groups(layout: &Layout) -> Vec<Group> {
    let flat = |table: &[Vec<usize>]| table.iter().flatten().copied().collect::<Vec<_>>();
    let mut groups = vec![group("corner", flat(&layout.corners), 24, 1)];
    if !layout.midges.is_empty() {
        groups.push(group("midge", flat(&layout.midges), 24, 1));
    }
    for table in &layout.wings {
        groups.push(group("wing", flat(table), 48, 1));
    }
    for table in &layout.centers {
        groups.push(group("center", table.clone(), 24, 1));
    }
    if !layout.fixed.is_empty() {
        groups.push(group("fixed center", layout.fixed.clone(), 24, 4));
    }
    groups
}

/// A sticker on a facelet of the wrong class, or a piece shown twice.
fn check_group(values: &[usize], g: &Group) -> Result<(), V2Error> {
    if let Some(&at) = g.facelets.iter().find(|&&at| values[at] >= g.limit) {
        return Err(malformed(format!(
            "facelet {at} shows '{}', which is no {} sticker",
            symbol(Model::Picture, values[at]),
            g.what
        )));
    }
    // Every value is below the limit, so a piece is below `limit / per`.
    let mut count = vec![0usize; g.limit / g.per];
    for &at in &g.facelets {
        count[values[at] / g.per] += 1;
    }
    match g
        .facelets
        .iter()
        .map(|&at| values[at])
        .find(|&v| count[v / g.per] > 1)
    {
        Some(v) => Err(malformed(format!(
            "{} sticker '{}' occurs more than once",
            g.what,
            symbol(Model::Picture, v)
        ))),
        None => Ok(()),
    }
}

/// The piece and twist spelled by the `m` symbols of a slot: the `k`-th one
/// is `m p + (k - o) mod m`.
fn read_slot(seen: &[usize]) -> Option<(usize, usize)> {
    let m = seen.len();
    let (p, o) = (seen[0] / m, (m - seen[0] % m) % m);
    (0..m)
        .all(|k| seen[k] == m * p + (k + m - o) % m)
        .then_some((p, o))
}

fn seen_at(table: &[Vec<usize>], slot: usize, values: &[usize]) -> Vec<usize> {
    table[slot].iter().map(|&at| values[at]).collect()
}

/// The pieces and twists of a table of slots; a `strict` slot admits no twist.
fn read_oriented(
    what: &str,
    table: &[Vec<usize>],
    strict: bool,
    values: &[usize],
) -> Result<(Vec<usize>, Vec<usize>), V2Error> {
    let mut pieces = Vec::with_capacity(table.len());
    let mut twists = Vec::with_capacity(table.len());
    for slot in 0..table.len() {
        let seen = seen_at(table, slot, values);
        match read_slot(&seen) {
            Some((piece, twist)) if !(strict && twist != 0) => {
                pieces.push(piece);
                twists.push(twist);
            }
            _ => return Err(no_piece(Model::Picture, what, slot, &seen)),
        }
    }
    Ok((pieces, twists))
}

/// The coordinates of a canonical-pose picture; the frame is `frame`.
fn read_cube(n: usize, layout: &Layout, frame: usize, values: &[usize]) -> Result<Cube, V2Error> {
    let (corner_pieces, corner_twists) = read_oriented("corner", &layout.corners, false, values)?;
    let (midge_pieces, midge_flips) = if is_odd(n) {
        read_oriented("midge", &layout.midges, false, values)?
    } else {
        (Vec::new(), Vec::new())
    };
    let wings = layout
        .wings
        .iter()
        .map(|table| read_oriented("wing", table, true, values).map(|(pieces, _)| pieces))
        .collect::<Result<Vec<_>, _>>()?;
    let centers = layout
        .centers
        .iter()
        .map(|table| table.iter().map(|&at| values[at]).collect())
        .collect();
    let rotations = layout.fixed.iter().map(|&at| values[at] % 4).collect();
    Ok(Cube {
        corner_pieces,
        corner_twists,
        midge_pieces,
        midge_flips,
        wings,
        centers,
        rotations,
        frame,
    })
}

/// Every sticker is of its class and every slot spells one piece, in any pose.
fn check_pieces(n: usize, layout: &Layout, values: &[usize]) -> Result<(), V2Error> {
    for g in groups(layout) {
        check_group(values, &g)?;
    }
    read_cube(n, layout, 0, values).map(|_| ())
}

/// The frame to record and the values in the pose the type reads them in. The
/// fixed centers of an odd cube name the frame by their pieces `v div 4`.
fn posed(layout: &Layout, pose: Pose, values: Vec<usize>) -> Result<(usize, Vec<usize>), V2Error> {
    let n = layout.n;
    if is_odd(n) {
        let shown: Vec<usize> = layout.fixed.iter().map(|&at| values[at] / 4).collect();
        let frame = FRAME_FACES
            .iter()
            .position(|faces| faces[..] == shown[..])
            .ok_or_else(|| {
                malformed(
                    "the six fixed centers are not a right-handed whole-cube frame".to_string(),
                )
            })?;
        let upright = unrotate_picture(n, frame, &values);
        Ok((if pose == Pose::Framed { frame } else { 0 }, upright))
    } else if pose == Pose::Canonical {
        (0..FRAME_COUNT)
            .map(|frame| unrotate(n, frame, &values))
            .find(|rotated| read_slot(&seen_at(&layout.corners, DBL, rotated)) == Some((DBL, 0)))
            .map(|rotated| (0, rotated))
            .ok_or_else(|| malformed("no whole-cube rotation puts the DBL corner home".to_string()))
    } else {
        Ok((0, values))
    }
}

/// The cube state shown by a picture string, in the pose of `pose`.
pub(super) fn read(n: usize, pose: Pose, text: &str) -> Result<Cube, V2Error> {
    let layout = Layout::new(n);
    let values = parse(n, Model::Picture, text)?;
    check_pieces(n, &layout, &values)?;
    let (frame, values) = posed(&layout, pose, values)?;
    let cube = read_cube(n, &layout, frame, &values)?;
    cube.validate(n, Model::Picture, pose)?;
    digits::digits(n, Model::Picture, pose, &cube)?;
    Ok(cube)
}
