//! Names for the centre orbits, derived from where their stickers sit on a
//! face, never from colours.
//!
//! A centre sticker is `e1` and `e2` cells in from the two nearest edges of
//! its face (1 is the ring just inside the outer layer), `e1 <= e2`.
//!
//! - `e1 == e2`: on a diagonal of the face, an `x-centre`.
//! - `e2` the middle line of an odd face: on a mid-line, a `+-centre`.
//! - otherwise `oblique`; the two mirror images of such an orbit have the same
//!   name and are told apart by a trailing `a` and `b`.
//!
//! A diagonal or mid-line orbit is on the `corner side` or `edge side` when it
//! is the outermost one, on the `middle side` when it is the innermost one, and
//! otherwise it is named by its ring.

use crate::puzzle::{centre_orbits, locate, PuzzleKind};

/// The name and the fingerprint tag of one centre orbit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CentreClass {
    /// `x-centres (corner side)`; mirror images share it until [`pair_up`].
    pub(crate) name: String,
    /// The fingerprint token prefix: `X`, `+` or `O1-2:`, with a ring number
    /// (`X2:`) where a shape has more than one ring.
    pub(crate) tag: String,
}

fn class_of(n: usize, e1: usize, e2: usize) -> CentreClass {
    let inner = (n - 2) / 2;
    let ring = |outer: &str| match e1 {
        1 => format!("{outer} side"),
        e if e == inner => "middle side".to_owned(),
        e => format!("ring {e} from the {outer}"),
    };
    let numbered = |letter: &str| {
        if inner > 1 {
            format!("{letter}{e1}:")
        } else {
            letter.to_owned()
        }
    };
    if e1 == e2 {
        CentreClass {
            name: format!("x-centres ({})", ring("corner")),
            tag: numbered("X"),
        }
    } else if n % 2 == 1 && e2 == (n - 1) / 2 {
        CentreClass {
            name: format!("+-centres ({})", ring("edge")),
            tag: numbered("+"),
        }
    } else {
        CentreClass {
            name: format!("oblique centres ({e1} and {e2} cells in)"),
            tag: format!("O{e1}-{e2}:"),
        }
    }
}

/// The class of every orbit of [`centre_orbits`], in the same order.
pub(crate) fn centre_classes(kind: PuzzleKind) -> Vec<CentreClass> {
    let n = kind.size();
    centre_orbits(kind)
        .iter()
        .map(|orbit| {
            let (position, normal) = locate(kind, orbit[0]);
            let mut e: Vec<usize> = (0..3)
                .filter(|&axis| normal[axis] == 0)
                .map(|axis| (n - 1 - position[axis].unsigned_abs() as usize) / 2)
                .collect();
            e.sort_unstable();
            class_of(n, e[0], e[1])
        })
        .collect()
}

/// Appends ` a` and ` b` to a name that occurs twice (the mirror pair of an
/// oblique orbit), in the order given.
pub(crate) fn pair_up<'a>(names: impl Iterator<Item = &'a mut String>) {
    let mut names: Vec<&mut String> = names.collect();
    for i in 0..names.len() {
        let same: Vec<usize> = (0..names.len())
            .filter(|&j| names[j].as_str() == names[i].as_str())
            .collect();
        if same.len() == 2 && !names[i].ends_with(" a") && !names[i].ends_with(" b") {
            let (first, second) = (same[0], same[1]);
            names[first].push_str(" a");
            names[second].push_str(" b");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(size: usize) -> Vec<String> {
        let kind = PuzzleKind::of_size(size).unwrap();
        let mut names: Vec<String> = centre_classes(kind).into_iter().map(|c| c.name).collect();
        pair_up(names.iter_mut());
        names
    }

    #[test]
    fn the_five_has_an_x_and_a_plus_orbit() {
        assert_eq!(
            names(5),
            ["x-centres (corner side)", "+-centres (edge side)"]
        );
    }

    #[test]
    fn the_six_and_seven_name_every_orbit_once() {
        for size in [4, 5, 6, 7] {
            let list = names(size);
            let mut sorted = list.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), list.len(), "{size}: {list:?}");
        }
        assert_eq!(names(6).len(), 4);
        assert_eq!(names(7).len(), 6);
    }
}
