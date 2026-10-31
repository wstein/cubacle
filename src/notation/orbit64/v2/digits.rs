//! Origin: Rust re-implementation of the digit layout of `Orbit64.V2.State`
//! (flix-orbit64, Apache-2.0, Werner Stein), following `FORMAT-V2.md`.
//!
//! A state's rank is the Horner combination of one digit per component, most
//! significant first. Permutation digits are written as their Lehmer digits
//! (radices `k, k-1, ..., 1`, or `k, ..., 3` when the parity is implied) and
//! the center multisets as five combination ranks, so every radix is small and
//! a flat list of `u32` digits describes the whole state, exactly as in V1.

use super::cube::{center_orbits, drops_corner, is_odd, wing_orbits, Cube};
use super::registry::{Model, Pose};
use super::V2Error;
use crate::notation::orbit64::big::Big;
use crate::notation::orbit64::coords::{
    center_digits, choose, lehmer, orientation_value, orientations, parity, unlehmer,
    unrank_centers,
};

/// The piece and slot number of the DBL corner.
const DBL: usize = 6;
/// Twist states of seven corners, flip states of eleven midges or fixed
/// centers' rotations (`2^11`), and the number of whole-cube frames.
const FLIP_STATES: u32 = 2048;
const FRAME_STATES: u32 = 24;

/// The components of a state, in digit order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    CornerPerm,
    CornerTwist,
    MidgePerm,
    MidgeFlip,
    Wing,
    Center,
    FixedRotation,
    Frame,
}

/// The components a type has: wing and center orbits repeat per orbit.
fn parts(n: usize, model: Model, pose: Pose) -> Vec<Part> {
    let odd = is_odd(n);
    let mut parts = vec![Part::CornerPerm, Part::CornerTwist];
    if odd {
        parts.extend([Part::MidgePerm, Part::MidgeFlip]);
    }
    parts.extend(std::iter::repeat_n(Part::Wing, wing_orbits(n)));
    parts.extend(std::iter::repeat_n(Part::Center, center_orbits(n).len()));
    if odd && model == Model::Picture {
        parts.push(Part::FixedRotation);
    }
    if odd && pose == Pose::Framed {
        parts.push(Part::Frame);
    }
    parts
}

/// The corner slots that are stored: eight, or seven without DBL.
fn stored_corners(n: usize, pose: Pose) -> usize {
    if drops_corner(n, pose) {
        7
    } else {
        8
    }
}

/// Lehmer radices of a permutation of `k`; the last two are implied when the
/// parity is known.
fn permutation_radices(k: usize, parity_known: bool) -> impl Iterator<Item = u32> {
    let last = if parity_known { 3 } else { 1 };
    (last..=k as u32).rev()
}

fn part_radices(n: usize, model: Model, pose: Pose, part: Part) -> Vec<u32> {
    match part {
        Part::CornerPerm => permutation_radices(stored_corners(n, pose), false).collect(),
        Part::CornerTwist => vec![3u32.pow(stored_corners(n, pose) as u32 - 1)],
        Part::MidgePerm => permutation_radices(12, true).collect(),
        Part::MidgeFlip | Part::FixedRotation => vec![FLIP_STATES],
        Part::Wing => permutation_radices(24, false).collect(),
        Part::Center => match model {
            Model::Color => (0..5).map(|t| choose(24 - 4 * t, 4)).collect(),
            Model::Picture => permutation_radices(24, true).collect(),
        },
        Part::Frame => vec![FRAME_STATES],
    }
}

/// The radix of every digit of a state of the type, most significant first.
pub(super) fn radices(n: usize, model: Model, pose: Pose) -> Vec<u32> {
    parts(n, model, pose)
        .into_iter()
        .flat_map(|part| part_radices(n, model, pose, part))
        .collect()
}

/// The number of states of a type.
pub(super) fn state_count(n: usize, model: Model, pose: Pose) -> Big {
    let mut count = Big::one();
    for radix in radices(n, model, pose) {
        count.mul_add(radix, 0);
    }
    count
}

/// Piece numbers of an even canonical cube are renumbered `s - [s > 6]`.
fn stored_pieces(n: usize, pose: Pose, cube: &Cube) -> Vec<usize> {
    let shrink = drops_corner(n, pose);
    (0..8)
        .filter(|&slot| !(shrink && slot == DBL))
        .map(|slot| {
            let piece = cube.corner_pieces[slot];
            piece - usize::from(shrink && piece > DBL)
        })
        .collect()
}

fn stored_twists(n: usize, pose: Pose, cube: &Cube) -> Vec<usize> {
    let shrink = drops_corner(n, pose);
    (0..8)
        .filter(|&slot| !(shrink && slot == DBL))
        .map(|slot| cube.corner_twists[slot])
        .collect()
}

fn rotation_digit(rotations: &[usize]) -> u32 {
    let weights = [1u32, 4, 16, 64, 256, 1024];
    rotations
        .iter()
        .zip(weights)
        .enumerate()
        .map(|(index, (&r, weight))| weight * (if index == 5 { r / 2 } else { r }) as u32)
        .sum()
}

/// The digits of a valid state of the type, in the order of `radices`.
pub(super) fn digits(n: usize, model: Model, pose: Pose, cube: &Cube) -> Result<Vec<u32>, V2Error> {
    let mut out = Vec::new();
    lehmer(
        &stored_pieces(n, pose, cube),
        stored_corners(n, pose),
        &mut out,
    );
    out.push(orientation_value(&stored_twists(n, pose, cube), 3));
    if is_odd(n) {
        lehmer(&cube.midge_pieces, 10, &mut out);
        out.push(orientation_value(&cube.midge_flips, 2));
    }
    for wing in &cube.wings {
        lehmer(wing, 24, &mut out);
    }
    for center in &cube.centers {
        match model {
            Model::Color => center_digits(center, &mut out)
                .map_err(|_| V2Error::Invalid("center colors are not four of each".to_string()))?,
            Model::Picture => lehmer(center, 22, &mut out),
        }
    }
    if is_odd(n) && model == Model::Picture {
        out.push(rotation_digit(&cube.rotations));
    }
    if is_odd(n) && pose == Pose::Framed {
        out.push(cube.frame as u32);
    }
    Ok(out)
}

/// Combines the digits of a valid state into its rank.
pub(super) fn rank(n: usize, model: Model, pose: Pose, digits: &[u32]) -> Big {
    let mut rank = Big::zero();
    for (radix, digit) in radices(n, model, pose).into_iter().zip(digits) {
        rank.mul_add(radix, *digit);
    }
    rank
}

/// Splits a rank below the state count of the type into its digits.
pub(super) fn split(n: usize, model: Model, pose: Pose, mut rank: Big) -> Vec<u32> {
    let mut digits: Vec<u32> = radices(n, model, pose)
        .iter()
        .rev()
        .map(|&radix| rank.div_rem(radix))
        .collect();
    digits.reverse();
    digits
}

/// Reinserts the left-out DBL corner (slot 6) and undoes the renumbering.
fn restore_dbl(stored: &[usize], dbl: usize, renumber: bool) -> Vec<usize> {
    let unshift = |value: usize| value + usize::from(renumber && value >= DBL);
    (0..8)
        .map(|slot| match slot {
            DBL => dbl,
            7 => unshift(stored[6]),
            _ => unshift(stored[slot]),
        })
        .collect()
}

/// Splits the rotation digit; the low bit of `r_B` follows from `R = C`.
fn unrank_rotations(digit: u32, corner_parity: usize) -> Vec<usize> {
    let mut rotations: Vec<usize> = (0..5).map(|i| (digit >> (2 * i)) as usize % 4).collect();
    let low = (corner_parity + rotations.iter().sum::<usize>()) % 2;
    rotations.push(2 * (digit >> 10) as usize + low);
    rotations
}

/// The state with the given digits; the inverse of `digits`.
pub(super) fn from_digits(n: usize, model: Model, pose: Pose, digits: &[u32]) -> Cube {
    let mut digits = digits.iter().copied();
    let odd = is_odd(n);
    let stored = stored_corners(n, pose);
    let pieces = unlehmer(&mut digits, stored, None);
    let twists = orientations(digits.next().unwrap_or(0), stored - 1, 3);
    let (corner_pieces, corner_twists) = if drops_corner(n, pose) {
        (
            restore_dbl(&pieces, DBL, true),
            restore_dbl(&twists, 0, false),
        )
    } else {
        (pieces, twists)
    };
    let corners = parity(&corner_pieces);
    let mut cube = Cube {
        corner_pieces,
        corner_twists,
        ..Cube::default()
    };
    if odd {
        cube.midge_pieces = unlehmer(&mut digits, 12, Some(corners));
        cube.midge_flips = orientations(digits.next().unwrap_or(0), 11, 2);
    }
    cube.wings = (0..wing_orbits(n))
        .map(|_| unlehmer(&mut digits, 24, None))
        .collect();
    cube.centers = (0..center_orbits(n).len())
        .map(|index| match model {
            Model::Color => unrank_centers(&mut digits),
            Model::Picture => {
                let wanted = cube.required_center_parity(n, corners, index);
                unlehmer(&mut digits, 24, Some(wanted))
            }
        })
        .collect();
    if odd && model == Model::Picture {
        cube.rotations = unrank_rotations(digits.next().unwrap_or(0), corners);
    }
    if odd && pose == Pose::Framed {
        cube.frame = digits.next().unwrap_or(0) as usize;
    }
    cube
}
