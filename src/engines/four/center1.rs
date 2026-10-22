//! Phase 1: eight centres of one kind onto two opposite faces.
//!
//! Ported from cs0x7f/TPR-4x4x4-Solver src/Center1.java, MIT, Copyright (c)
//! 2023 Chen Shuang; this file is AGPL-3.0-or-later, see NOTICE.md.
//!
//! The state is which 8 of the 24 centre places hold the chosen kind
//! (colours `c % 3 == urf`): C(24,8) = 735,471 states, 15,582 classes under the
//! 48 symmetries. A state is written `class << 6 | symmetry`, the symmetry
//! that takes the state to its class representative.

use super::cube::move_centres;
use super::sym::{step, Syms};
use crate::reduction::comb::CNK;

/// Raw states, and classes under the 48 symmetries.
pub(super) const N_RAW: usize = 735_471;
pub(super) const N_CLASS: usize = 15_582;
/// Moves in the pruning table's search: `U R F D L B u r f`.
const N_SEARCH_MOVES: usize = 27;

/// Which centre places hold the chosen kind (1) or not (0).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Flags {
    ct: [u8; 24],
}

impl Flags {
    /// The state with number `idx`.
    pub(super) fn from_number(idx: u32) -> Flags {
        let mut idx = idx as i32;
        let mut ct = [0u8; 24];
        let mut r = 8;
        for i in (0..24).rev() {
            if idx >= CNK[i][r] {
                idx -= CNK[i][r];
                r -= 1;
                ct[i] = 1;
            }
        }
        Flags { ct }
    }

    /// The flags for the kind `urf` (0 to 2) of the centre colours `ct`.
    pub(super) fn of_centres(ct: &[u8; 24], urf: u8) -> Flags {
        let mut flags = [0u8; 24];
        for (flag, &colour) in flags.iter_mut().zip(ct) {
            *flag = u8::from(colour % 3 == urf);
        }
        Flags { ct: flags }
    }

    pub(super) fn number(&self) -> u32 {
        let mut idx = 0;
        let mut r = 8usize;
        for i in (0..24).rev() {
            if self.ct[i] == 1 {
                idx += CNK[i][r];
                r = r.wrapping_sub(1);
            }
        }
        idx as u32
    }

    pub(super) fn step(&mut self, j: usize) {
        step(&mut self.ct, j);
    }

    fn moved(mut self, m: usize) -> Flags {
        move_centres(&mut self.ct, m);
        self
    }

    /// `class << 6 | symmetry` by looking through the 48 symmetries.
    pub(super) fn class_sym(&self, sym2raw: &[u32]) -> Option<u32> {
        let mut c = *self;
        for j in 0..48 {
            if let Ok(class) = sym2raw.binary_search(&c.number()) {
                return Some((class as u32) << 6 | j as u32);
            }
            c.step(j);
        }
        None
    }
}

/// The tables of phase 1.
pub(super) struct Center1 {
    /// Raw state of the representative of each class (ascending).
    pub sym2raw: Vec<u32>,
    /// `ctsmv[class * 36 + m]`: the state after move `m`.
    pub ctsmv: Vec<u32>,
    /// Distance of each class from the goal; 255 for unknown.
    pub csprun: Vec<u8>,
}

impl Center1 {
    pub(super) fn build(syms: &Syms) -> Center1 {
        // Representatives, and for every raw state the symmetry to its class.
        let mut raw2sym = vec![0u32; N_RAW];
        let mut seen = vec![false; N_RAW];
        let mut sym2raw = Vec::with_capacity(N_CLASS);
        for i in 0..N_RAW {
            if seen[i] {
                continue;
            }
            let mut c = Flags::from_number(i as u32);
            for j in 0..48 {
                let idx = c.number() as usize;
                seen[idx] = true;
                raw2sym[idx] = (sym2raw.len() as u32) << 6 | u32::from(syms.inv[j]);
                c.step(j);
            }
            sym2raw.push(i as u32);
        }
        assert_eq!(sym2raw.len(), N_CLASS);

        let mut ctsmv = vec![0u32; N_CLASS * 36];
        for (i, &raw) in sym2raw.iter().enumerate() {
            let d = Flags::from_number(raw);
            for m in 0..36 {
                ctsmv[i * 36 + m] = raw2sym[d.moved(m).number() as usize];
            }
        }
        let csprun = prune(&ctsmv);
        Center1 {
            sym2raw,
            ctsmv,
            csprun,
        }
    }
}

/// Distances of the classes from the solved class (0), by breadth-first
/// layers, the last ones from the unknown side.
fn prune(ctsmv: &[u32]) -> Vec<u8> {
    let mut prun = vec![255u8; N_CLASS];
    prun[0] = 0;
    let mut depth = 0u8;
    let mut done = 1;
    while done != N_CLASS {
        let inverse = depth > 4;
        let select = if inverse { 255 } else { depth };
        let check = if inverse { depth } else { 255 };
        depth += 1;
        for i in 0..N_CLASS {
            if prun[i] != select {
                continue;
            }
            for m in 0..N_SEARCH_MOVES {
                let idx = (ctsmv[i * 36 + m] >> 6) as usize;
                if prun[idx] != check {
                    continue;
                }
                done += 1;
                if inverse {
                    prun[i] = depth;
                    break;
                }
                prun[idx] = depth;
            }
        }
    }
    prun
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_round_trip_and_the_solved_state_is_class_zero() {
        for idx in (0..N_RAW as u32).step_by(997) {
            assert_eq!(Flags::from_number(idx).number(), idx);
        }
        assert_eq!(Flags::from_number(0).ct[..8], [1; 8]);
    }
}

#[cfg(test)]
mod reference_tests {
    use super::super::testing::fnv;
    use super::*;

    #[test]
    fn tables_match_the_reference_checksums() {
        let syms = Syms::new();
        assert_eq!(
            fnv(syms.mult.iter().flatten().map(|&v| i64::from(v))),
            3189489955994858883
        );
        assert_eq!(
            fnv(syms.inv.iter().map(|&v| i64::from(v))),
            -6015769024500312893
        );
        assert_eq!(
            fnv(syms.mv.iter().flatten().map(|&v| i64::from(v))),
            -3029094272301040045
        );
        assert_eq!(
            fnv(syms.finish.iter().map(|&v| i64::from(v))),
            6043733436547256691
        );
        let c1 = Center1::build(&syms);
        assert_eq!(
            fnv(c1.ctsmv.iter().map(|&v| i64::from(v))),
            7518774406779946637
        );
        assert_eq!(
            fnv(c1.sym2raw.iter().map(|&v| i64::from(v))),
            5512513337953774344
        );
        assert_eq!(
            fnv(c1.csprun.iter().map(|&v| i64::from(v as i8))),
            3403341935679631689
        );
        let mut histogram = [0usize; 9];
        for &d in &c1.csprun {
            histogram[usize::from(d)] += 1;
        }
        assert_eq!(histogram, [1, 1, 4, 37, 338, 2685, 8867, 3629, 20]);
    }
}
