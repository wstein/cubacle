//! Twist-flip pruning with conjugate pruning for phase 1.
//!
//! The distance table is indexed by the flip symmetry class (336) and the raw
//! twist (2187) and holds 4-bit entries, about 367 KB. A cube is brought to
//! the flip representative with the symmetry stored in its flip coordinate,
//! and its raw twist is conjugated the same way. Every distance in the table
//! is a true distance of a projection of the cube group, so it is a lower
//! bound for phase 1; the bound is taken a second time on the cube conjugated
//! by a quarter turn about the U-D axis, which sees other twists and flips and
//! is again a lower bound because that rotation fixes the phase 2 subgroup.

use super::consts::*;
use super::coordinates::Coord;
use super::cubie::*;
use super::tables::*;

pub(super) struct TwistFlipTables {
    /// Raw twist after a move: `N_TWST * 18`.
    twist_move: Vec<u16>,
    /// Raw twist conjugated by flip symmetry `s`: `N_TWST * 8`.
    twist_conj: Vec<u16>,
    /// 4-bit distances, index `flip_class * N_TWST + conjugated twist`.
    prun: Vec<u32>,
    /// The move that a move becomes when the cube is conjugated by U4.
    conj_move: [u8; 18],
    conj_sym: Cubie,
    conj_sym_inv: Cubie,
}

impl TwistFlipTables {
    pub(super) fn new() -> Self {
        TwistFlipTables {
            twist_move: Vec::new(),
            twist_conj: Vec::new(),
            prun: Vec::new(),
            conj_move: [0; 18],
            conj_sym: Cubie::new(),
            conj_sym_inv: Cubie::new(),
        }
    }

    pub(super) fn init(&mut self, sctx: &StaticContext, flip_move: &[u16], flip_selfsym: &[u16]) {
        let mut c = Cubie::new();
        self.twist_move = vec![0; N_TWST * N_MOVES_P1];
        self.twist_conj = vec![0; N_TWST * 8];
        for i in 0..N_TWST {
            c.reset();
            c.set_twst(i as u16);
            for j in 0..N_MOVES_P1 {
                let mut d = Cubie::new();
                Cubie::corn_mult(&c, &sctx.movecube[j], &mut d);
                self.twist_move[i * N_MOVES_P1 + j] = d.get_twst() as u16;
            }
            for j in 0..8 {
                let (mut e, mut d) = (Cubie::new(), Cubie::new());
                let s = j << 1;
                Cubie::corn_mult(&sctx.symcube[s], &c, &mut e);
                Cubie::corn_mult(&e, &sctx.symcube[sctx.symmuli[0][s] as usize], &mut d);
                self.twist_conj[i * 8 + j] = d.get_twst() as u16;
            }
        }
        self.prun = vec![0; N_FLIP_SYM * N_TWST / 8 + 1];
        init_raw_sym_prun(
            &mut self.prun,
            &self.twist_move,
            &self.twist_conj,
            flip_move,
            flip_selfsym,
            N_TWST,
            N_FLIP_SYM,
            0xe603,
        );
        // Conjugation by U4 (symmetry 1).
        let s = sctx.symcube[1];
        let si = sctx.symcube[sctx.symmuli[0][1] as usize];
        let (mut e, mut d) = (Cubie::new(), Cubie::new());
        for m in 0..N_MOVES_P1 {
            Cubie::corn_mult(&si, &sctx.movecube[m], &mut e);
            Cubie::edge_mult(&si, &sctx.movecube[m], &mut e);
            Cubie::corn_mult(&e, &s, &mut d);
            Cubie::edge_mult(&e, &s, &mut d);
            let k = (0..N_MOVES_P1)
                .find(|&k| Cubie::cmp(&sctx.movecube[k], &d) == 0)
                .expect("U4 maps moves to moves");
            self.conj_move[m] = k as u8;
        }
        self.conj_sym = s;
        self.conj_sym_inv = si;
    }

    #[inline]
    fn lookup(&self, flip_class: u16, flip_sym: u16, twist: u16) -> i8 {
        let t = self.twist_conj[twist as usize * 8 + flip_sym as usize] as usize;
        get_pruning(&self.prun, flip_class as usize * N_TWST + t) as i8
    }
}

impl Coord {
    /// Fills the twist-flip fields of a root node and returns its bound.
    pub(super) fn set_twist_flip(&mut self, stbl: &StaticTables, src: &Cubie) -> i8 {
        let tf = &stbl.twist_flip;
        self.twr = src.get_twst() as u16;
        let direct = tf.lookup(self.flip, self.fsym, self.twr);
        let (mut e, mut d) = (Cubie::new(), Cubie::new());
        Cubie::corn_mult(&tf.conj_sym_inv, src, &mut e);
        Cubie::edge_mult(&tf.conj_sym_inv, src, &mut e);
        Cubie::corn_mult(&e, &tf.conj_sym, &mut d);
        Cubie::edge_mult(&e, &tf.conj_sym, &mut d);
        self.twc = d.get_twst() as u16;
        let f = stbl.flip_raw2sym[d.get_flip() as usize];
        self.fsc = f & 7;
        self.fc = f >> 3;
        direct.max(tf.lookup(self.fc, self.fsc, self.twc))
    }

    /// Moves the twist-flip state; `self.flip` and `self.fsym` must already
    /// be those of the child. Returns the direct bound.
    #[inline]
    pub(super) fn move_twist_flip(&mut self, stbl: &StaticTables, src: &Coord, mv: usize) -> i8 {
        let tf = &stbl.twist_flip;
        self.twr = tf.twist_move[src.twr as usize * N_MOVES_P1 + mv];
        tf.lookup(self.flip, self.fsym, self.twr)
    }

    /// Moves the conjugate state and returns the conjugate bound.
    #[inline]
    pub(super) fn move_twist_flip_conj(
        &mut self,
        sctx: &StaticContext,
        stbl: &StaticTables,
        src: &Coord,
        mv: usize,
    ) -> i8 {
        let tf = &stbl.twist_flip;
        let mc = tf.conj_move[mv] as usize;
        self.twc = tf.twist_move[src.twc as usize * N_MOVES_P1 + mc];
        let f = stbl.flip_move
            [src.fc as usize * N_MOVES_P1 + sctx.symmove[mc][src.fsc as usize] as usize];
        self.fsc = (f & 7) ^ src.fsc;
        self.fc = f >> 3;
        tf.lookup(self.fc, self.fsc, self.twc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reduced table agrees with a plain breadth-first search over all
    /// 4,478,976 (twist, flip) pairs.
    #[test]
    fn reduced_table_equals_raw_distances() {
        let shared = fresh();
        let built = get(&shared);
        let (sctx, stbl) = (&built.sctx, &built.stbl);
        let tf = &stbl.twist_flip;
        let mut flip_move = vec![0u16; N_FLIP * N_MOVES_P1];
        let mut c = Cubie::new();
        for i in 0..N_FLIP {
            c.reset();
            c.set_flip(i as u16);
            for j in 0..N_MOVES_P1 {
                let mut d = Cubie::new();
                Cubie::edge_mult(&c, &sctx.movecube[j], &mut d);
                flip_move[i * N_MOVES_P1 + j] = d.get_flip() as u16;
            }
        }
        let n = N_TWST * N_FLIP;
        let mut dist = vec![255u8; n];
        dist[0] = 0;
        let (mut depth, mut done) = (0u8, 1usize);
        while done < n {
            for i in 0..n {
                if dist[i] != depth {
                    continue;
                }
                let (t, f) = (i / N_FLIP, i % N_FLIP);
                for m in 0..N_MOVES_P1 {
                    let k = tf.twist_move[t * N_MOVES_P1 + m] as usize * N_FLIP
                        + flip_move[f * N_MOVES_P1 + m] as usize;
                    if dist[k] == 255 {
                        dist[k] = depth + 1;
                        done += 1;
                    }
                }
            }
            depth += 1;
        }
        assert!(depth < 15, "distance 15 is the unfilled marker");
        for t in 0..N_TWST {
            for f in 0..N_FLIP {
                let r = stbl.flip_raw2sym[f];
                let got = tf.lookup(r >> 3, r & 7, t as u16);
                assert_eq!(got as u8, dist[t * N_FLIP + f], "twist {t} flip {f}");
            }
        }
    }
}
