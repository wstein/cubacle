//! A free orientation on a cube whose centers a turn moved.

use crate::puzzle::{oriented_by_centers, Face, Facelets, Target};

/// The start and target to search instead, when `start` has its centers off
/// their faces.
///
/// The start is read by its centers: its colors are renamed so that each
/// center carries its face's color. The target gets the same renaming, so a
/// state meets the renamed target after some rotation exactly when the given
/// state meets the given target after that rotation, and moves that solve the
/// renamed pair solve the given one.
pub(super) fn through_centers(start: &Facelets, target: &Target) -> Option<(Facelets, Target)> {
    let renamed = oriented_by_centers(start)?;
    let stickers = start.stickers();
    let mut new_color = [0usize; 6];
    for face in Face::ALL {
        new_color[stickers[face.index() * 9 + 4].index()] = face.index();
    }
    let sets = target
        .sets()
        .iter()
        .map(|&set| {
            (0..6)
                .filter(|&color| set >> color & 1 == 1)
                .fold(0u8, |acc, color| acc | 1 << new_color[color])
        })
        .collect();
    Some((renamed, Target::from_sets(sets).ok()?))
}
