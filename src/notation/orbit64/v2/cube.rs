//! Origin: Rust re-implementation of the `Cube` record and the validation of
//! `Orbit64.V2.State` (flix-orbit64, Apache-2.0, Werner Stein).

use super::registry::{Model, Pose};
use super::V2Error;
use crate::notation::orbit64::coords::{is_permutation, parity};

/// Orbit coordinates of one cube, in the slot convention of Orbit64 V1.
///
/// Components a type lacks are left empty. The caller normalizes the pose
/// first: the fixed centers of an odd cube are implicitly home, and `frame`
/// carries the whole-cube orientation, which is 0 unless the type is odd and
/// framed. `rotations` holds `r_U r_R r_F r_D r_L r_B`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cube {
    /// The corner piece in each of the eight slots.
    pub corner_pieces: Vec<usize>,
    /// The twist (`0..=2`) in each corner slot.
    pub corner_twists: Vec<usize>,
    /// The midge piece in each of the twelve slots; odd cubes only.
    pub midge_pieces: Vec<usize>,
    /// The flip (`0..=1`) in each midge slot; odd cubes only.
    pub midge_flips: Vec<usize>,
    /// One 24-slot permutation per wing orbit, outermost first.
    pub wings: Vec<Vec<usize>>,
    /// One 24-slot center orbit each: colors (`0..=5`, four of each) in the
    /// color model, a permutation of the pieces in the picture model.
    pub centers: Vec<Vec<usize>>,
    /// The fixed-center rotations (`0..=3`); odd picture cubes only.
    pub rotations: Vec<usize>,
    /// The frame rank (`0..24`); odd framed cubes only.
    pub frame: usize,
}

pub(super) fn is_odd(n: usize) -> bool {
    n % 2 == 1
}

/// Even canonical cubes leave the DBL corner out: it is home by definition.
pub(super) fn drops_corner(n: usize, pose: Pose) -> bool {
    !is_odd(n) && pose == Pose::Canonical
}

/// The number of wing orbits; zero below the 4x4x4, and for any `n < 2`.
pub(super) fn wing_orbits(n: usize) -> usize {
    n.saturating_sub(2) / 2
}

/// The `(row, column)` names of the center orbits, ordered by lowest facelet.
/// Empty below the 4x4x4, and for any `n < 2`.
pub(super) fn center_orbits(n: usize) -> Vec<(usize, usize)> {
    let last = n.saturating_sub(1);
    (1..last)
        .flat_map(|r| (r..last - r).map(move |c| (r, c)))
        .collect()
}

impl Cube {
    /// The solved state of an `n x n x n` cube in the given model: every digit
    /// is zero. `None` when `n < 2`, a cube of fewer than two layers; the
    /// solved state is defined for every larger `n`, also beyond the sizes of
    /// the registry. The pose does not change the state.
    pub fn solved(n: usize, model: Model) -> Option<Cube> {
        if n < 2 {
            return None;
        }
        let odd = is_odd(n);
        let identity = |k: usize| (0..k).collect::<Vec<_>>();
        let center = match model {
            Model::Color => (0..24).map(|slot| slot / 4).collect(),
            Model::Picture => identity(24),
        };
        Some(Cube {
            corner_pieces: identity(8),
            corner_twists: vec![0; 8],
            midge_pieces: if odd { identity(12) } else { Vec::new() },
            midge_flips: if odd { vec![0; 12] } else { Vec::new() },
            wings: vec![identity(24); wing_orbits(n)],
            centers: vec![center; center_orbits(n).len()],
            rotations: if odd && model == Model::Picture {
                vec![0; 6]
            } else {
                Vec::new()
            },
            frame: 0,
        })
    }

    /// The parity `W_k` of the wing orbit at depth `k`; zero without one.
    pub(super) fn wing_parity(&self, k: usize) -> usize {
        match k.checked_sub(1).and_then(|i| self.wings.get(i)) {
            Some(wing) => parity(wing),
            None => 0,
        }
    }

    /// Why this cube is not a state of the type, or `Ok`.
    pub(super) fn validate(&self, n: usize, model: Model, pose: Pose) -> Result<(), V2Error> {
        self.check_layout(n, model, pose)?;
        self.check_laws(n, model, pose)
    }

    fn check_layout(&self, n: usize, model: Model, pose: Pose) -> Result<(), V2Error> {
        let odd = is_odd(n);
        let wings = wing_orbits(n);
        let centers = center_orbits(n).len();
        let picture = model == Model::Picture;
        let bad = |why: &str| Err(V2Error::Invalid(why.to_string()));
        if !is_permutation(&self.corner_pieces, 8) {
            return bad("corner pieces are not a permutation of 8");
        }
        if !in_range(&self.corner_twists, 8, 3) {
            return bad("corner twists must be 8 values in 0..2");
        }
        if odd && !is_permutation(&self.midge_pieces, 12) || !odd && !self.midge_pieces.is_empty() {
            return bad(
                "midge pieces must be a permutation of 12 on odd cubes and empty otherwise",
            );
        }
        if odd && !in_range(&self.midge_flips, 12, 2) || !odd && !self.midge_flips.is_empty() {
            return bad("midge flips must be 12 values in 0..1 on odd cubes and empty otherwise");
        }
        if self.wings.len() != wings || !self.wings.iter().all(|w| is_permutation(w, 24)) {
            return Err(V2Error::Invalid(format!(
                "expected {wings} wing permutations of 24"
            )));
        }
        if self.centers.len() != centers {
            return Err(V2Error::Invalid(format!(
                "expected {centers} center orbits"
            )));
        }
        if !self.centers.iter().all(|c| center_valid(model, c)) {
            return bad("a center orbit is not a valid color multiset or permutation of 24");
        }
        let rotations_ok = if odd && picture {
            in_range(&self.rotations, 6, 4)
        } else {
            self.rotations.is_empty()
        };
        if !rotations_ok {
            return bad(
                "rotations must be 6 values in 0..3 on odd picture cubes and empty otherwise",
            );
        }
        let framed = odd && pose == Pose::Framed;
        if self.frame >= if framed { 24 } else { 1 } {
            return bad("frame must be 0..23 on odd framed cubes and 0 otherwise");
        }
        Ok(())
    }

    fn check_laws(&self, n: usize, model: Model, pose: Pose) -> Result<(), V2Error> {
        let odd = is_odd(n);
        let corners = parity(&self.corner_pieces);
        let bad = |why: &str| Err(V2Error::Invalid(why.to_string()));
        if self.corner_twists.iter().sum::<usize>() % 3 != 0 {
            return bad("corner twists must sum to 0 mod 3");
        }
        if odd && self.midge_flips.iter().sum::<usize>() % 2 != 0 {
            return bad("midge flips must sum to 0 mod 2");
        }
        if drops_corner(n, pose) && (self.corner_pieces[6] != 6 || self.corner_twists[6] != 0) {
            return bad("a canonical even cube has its DBL corner home with twist 0");
        }
        if odd && parity(&self.midge_pieces) != corners {
            return bad("midge parity must equal corner parity");
        }
        if odd && model == Model::Picture && self.rotations.iter().sum::<usize>() % 2 != corners {
            return bad("fixed-center rotation parity must equal corner parity");
        }
        if model == Model::Picture {
            for (index, center) in self.centers.iter().enumerate() {
                if parity(center) != self.required_center_parity(n, corners, index) {
                    return bad("a center orbit parity breaks the picture relation");
                }
            }
        }
        Ok(())
    }

    /// The parity the `index`-th (from 0) center orbit must have in the
    /// picture model: `C + W_r + W_d` with `d = min(c, n - 1 - c)`.
    pub(super) fn required_center_parity(&self, n: usize, corners: usize, index: usize) -> usize {
        let (r, c) = center_orbits(n)[index];
        let d = c.min(n - 1 - c);
        (corners + self.wing_parity(r) + self.wing_parity(d)) % 2
    }
}

fn in_range(values: &[usize], size: usize, bound: usize) -> bool {
    values.len() == size && values.iter().all(|&v| v < bound)
}

fn center_valid(model: Model, center: &[usize]) -> bool {
    match model {
        Model::Color => {
            in_range(center, 24, 6)
                && (0..6).all(|c| center.iter().filter(|&&x| x == c).count() == 4)
        }
        Model::Picture => is_permutation(center, 24),
    }
}
