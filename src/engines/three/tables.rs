use super::consts::*;
use std::sync::{Arc, OnceLock};

use super::cubie::*;
use super::twist_flip::TwistFlipTables;

pub(super) fn esym2csym(esym: u16) -> u16 {
    esym ^ (0x00dddd00u32 >> ((esym & 0xf) << 1) & 3) as u16
}

pub(super) struct StaticContext {
    pub(super) movecube: [Cubie; 18],
    pub(super) symcube: [Cubie; 16],
    pub(super) symmult: [[u8; 16]; 16],
    pub(super) symmuli: [[u8; 16]; 16],
    pub(super) symmove: [[u8; 8]; N_MOVES_P1],
    pub(super) symmove2: [[u8; 16]; N_MOVES_P1],
    pub(super) canon_masks2: [u16; 11],
    pub(super) symurf: Cubie,
    pub(super) symurfi: Cubie,
}

#[allow(clippy::needless_range_loop)]
impl StaticContext {
    pub(super) fn box_new() -> Box<Self> {
        let mut sctx = Box::new(StaticContext {
            movecube: [Cubie::new(); 18],
            symcube: [Cubie::new(); 16],
            symmult: [[0; 16]; 16],
            symmuli: [[0; 16]; 16],
            symmove: [[0; 8]; 18],
            symmove2: [[0; 16]; 18],
            canon_masks2: [0; 11],
            symurf: Cubie::new(),
            symurfi: Cubie::new(),
        });
        sctx.init();
        sctx
    }

    pub(super) fn init(&mut self) {
        let movebase: [Cubie; 6] = [
            Cubie {
                ca: [3, 0, 1, 2, 4, 5, 6, 7],
                ea: [6, 0, 2, 4, 8, 10, 12, 14, 16, 18, 20, 22],
            },
            Cubie {
                ca: [20, 1, 2, 8, 15, 5, 6, 19],
                ea: [16, 2, 4, 6, 22, 10, 12, 14, 8, 18, 20, 0],
            },
            Cubie {
                ca: [9, 21, 2, 3, 16, 12, 6, 7],
                ea: [0, 19, 4, 6, 8, 17, 12, 14, 3, 11, 20, 22],
            },
            Cubie {
                ca: [0, 1, 2, 3, 5, 6, 7, 4],
                ea: [0, 2, 4, 6, 10, 12, 14, 8, 16, 18, 20, 22],
            },
            Cubie {
                ca: [0, 10, 22, 3, 4, 17, 13, 7],
                ea: [0, 2, 20, 6, 8, 10, 18, 14, 16, 4, 12, 22],
            },
            Cubie {
                ca: [0, 1, 11, 23, 4, 5, 18, 14],
                ea: [0, 2, 4, 23, 8, 10, 12, 21, 16, 18, 7, 15],
            },
        ];
        for i in 0..18 {
            if i % 3 == 0 {
                self.movecube[i] = movebase[i / 3];
            } else {
                let mut cc = Cubie::new();
                Cubie::corn_mult(&self.movecube[i - 1], &movebase[i / 3], &mut cc);
                Cubie::edge_mult(&self.movecube[i - 1], &movebase[i / 3], &mut cc);
                self.movecube[i] = cc;
            }
        }

        let u4 = Cubie {
            ca: [3, 0, 1, 2, 7, 4, 5, 6],
            ea: [6, 0, 2, 4, 14, 8, 10, 12, 23, 17, 19, 21],
        };
        let lr2 = Cubie {
            ca: [25, 24, 27, 26, 29, 28, 31, 30],
            ea: [4, 2, 0, 6, 12, 10, 8, 14, 18, 16, 22, 20],
        };
        let f2 = Cubie {
            ca: [5, 4, 7, 6, 1, 0, 3, 2],
            ea: [12, 10, 8, 14, 4, 2, 0, 6, 18, 16, 22, 20],
        };

        let mut cc = Cubie::new();
        let mut cd = Cubie::new();

        for i in 0..16 {
            self.symcube[i] = cc;
            Cubie::corn_mult(&cc, &u4, &mut cd);
            Cubie::edge_mult(&cc, &u4, &mut cd);
            cc = cd;
            if i % 4 == 3 {
                Cubie::corn_mult(&cc, &lr2, &mut cd);
                Cubie::edge_mult(&cc, &lr2, &mut cd);
                cc = cd;
            }
            if i % 8 == 7 {
                Cubie::corn_mult(&cc, &f2, &mut cd);
                Cubie::edge_mult(&cc, &f2, &mut cd);
                cc = cd;
            }
        }

        self.symurf = Cubie {
            ca: [8, 20, 13, 17, 19, 15, 22, 10],
            ea: [3, 16, 11, 18, 7, 22, 15, 20, 1, 9, 13, 5],
        };
        Cubie::corn_mult(&self.symurf, &self.symurf, &mut self.symurfi);
        Cubie::edge_mult(&self.symurf, &self.symurf, &mut self.symurfi);

        for i in 0..16 {
            for j in 0..16 {
                Cubie::corn_mult(&self.symcube[i], &self.symcube[j], &mut cc);
                Cubie::edge_mult(&self.symcube[i], &self.symcube[j], &mut cc);
                for k in 0..16 {
                    if Cubie::cmp(&cc, &self.symcube[k]) == 0 {
                        self.symmult[i][j] = k as u8;
                        self.symmuli[k][j] = i as u8;
                    }
                }
            }
        }

        let mut p2moves_imap = [0; 18];
        for i in 0..18 {
            p2moves_imap[P2MOVES[i] as usize] = i;
        }

        for i in 0..18 {
            for j in 0..16 {
                Cubie::corn_mult(&self.symcube[j], &self.movecube[i], &mut cc);
                Cubie::corn_mult(&cc, &self.symcube[self.symmuli[0][j] as usize], &mut cd);
                Cubie::edge_mult(&self.symcube[j], &self.movecube[i], &mut cc);
                Cubie::edge_mult(&cc, &self.symcube[self.symmuli[0][j] as usize], &mut cd);
                for k in 0..18 {
                    if Cubie::cmp(&self.movecube[k], &cd) == 0 {
                        self.symmove2[p2moves_imap[i]][j] = p2moves_imap[k] as u8;
                        if j % 2 == 0 {
                            self.symmove[i][j / 2] = k as u8;
                        }
                        break;
                    }
                }
            }
        }

        for i in 0..10 {
            let ix = P2MOVES[i] as usize / 3;
            self.canon_masks2[i] = 0;
            for j in 0..10 {
                let jx = P2MOVES[j] as usize / 3;
                self.canon_masks2[i] |= if (ix == jx) || ((ix % 3 == jx % 3) && (ix >= jx)) {
                    1
                } else {
                    0
                } << j;
            }
        }
        self.canon_masks2[10] = 0;
    }
}

#[allow(clippy::needless_range_loop)]
pub(super) fn init_sym2raw(
    sctx: &StaticContext,
    n_raw: usize,
    coord: usize,
    sym2raw: &mut [u16],
    raw2sym: &mut [u16],
    selfsym: &mut [u16],
) -> usize {
    let mut c = Cubie::new();
    let mut e = Cubie::new();
    let mut d = Cubie::new();
    let sym_inc = if coord >= 2 { 1 } else { 2 };
    let sym_shift = if coord >= 2 { 0 } else { 1 };
    for i in 0..n_raw {
        raw2sym[i] = 0;
    }
    let mut count = 0;
    for i in 0..n_raw {
        if raw2sym[i] != 0 {
            continue;
        }
        match coord {
            0 => c.set_flip(i as u16),
            1 => c.set_twst(i as u16),
            2 => c.set_eperm(i as u16),
            _ => unreachable!(),
        }
        for s in (0..16).step_by(sym_inc) {
            if coord == 1 {
                Cubie::corn_mult(&sctx.symcube[sctx.symmuli[0][s] as usize], &c, &mut e);
                Cubie::corn_mult(&e, &sctx.symcube[s], &mut d);
            } else {
                Cubie::edge_mult(&sctx.symcube[sctx.symmuli[0][s] as usize], &c, &mut e);
                Cubie::edge_mult(&e, &sctx.symcube[s], &mut d);
            }
            let idx = match coord {
                0 => d.get_flip(),
                1 => d.get_twst(),
                2 => d.get_eperm(),
                _ => unreachable!(),
            };
            if idx == i as i32 {
                selfsym[count] |= 1 << (s >> sym_shift);
            }
            raw2sym[idx as usize] = ((count << 4 | s) >> sym_shift) as u16;
        }
        sym2raw[count] = i as u16;
        count += 1;
    }
    #[cfg(debug_assertions)]
    println!("init sym2raw coord={} count={}", coord, count);
    count
}

pub(super) struct StaticTables {
    pub(super) perm_sym_inv: [u16; N_PERM_SYM],
    pub(super) cperm2comb: [u8; N_PERM_SYM],
    pub(super) flip_sym2raw: [u16; N_FLIP_SYM],
    pub(super) flip_raw2sym: [u16; N_FLIP],
    pub(super) flip_selfsym: [u16; N_FLIP_SYM],
    pub(super) twst_sym2raw: [u16; N_TWST_SYM],
    pub(super) twst_raw2sym: [u16; N_TWST],
    pub(super) twst_selfsym: [u16; N_TWST_SYM],
    pub(super) eperm_sym2raw: [u16; N_PERM_SYM],
    pub(super) eperm_raw2sym: [u16; N_PERM],
    pub(super) eperm_selfsym: [u16; N_PERM_SYM],
    pub(super) flip_move: [u16; N_FLIP_SYM * N_MOVES_P1],
    pub(super) twst_move: [u16; N_TWST_SYM * N_MOVES_P1],
    pub(super) slice_move: [u16; N_SLICE * N_MOVES_P1],
    pub(super) slice_conj: [u16; N_SLICE * 8],
    pub(super) cperm_move: [u16; N_PERM_SYM * N_MOVES_P2],
    pub(super) eperm_move: [u16; N_PERM_SYM * N_MOVES_P2],
    pub(super) mperm_move: [u16; N_MPERM * N_MOVES_P2],
    pub(super) mperm_conj: [u16; N_MPERM * 16],
    pub(super) ccomb_move: [u16; N_CCOMB * N_MOVES_P2],
    pub(super) ccomb_conj: [u16; N_CCOMB * 16],
    pub(super) slice_flip_prun: [u32; N_SLICE * N_FLIP_SYM / 8 + 1],
    pub(super) slice_twst_prun: [u32; N_SLICE * N_TWST_SYM / 8 + 1],
    pub(super) ccomb_eperm_prun: [u32; N_CCOMB * N_PERM_SYM / 8 + 1],
    pub(super) mperm_cperm_prun: [u32; N_MPERM * N_PERM_SYM / 8 + 1],
    pub(super) twist_flip: TwistFlipTables,
}

#[allow(clippy::needless_range_loop)]
pub(super) fn init_move_tables(sctx: &StaticContext, stbl: &mut StaticTables) {
    let mut c = Cubie::new();
    c.reset();

    for i in 0..N_FLIP_SYM {
        c.set_flip(stbl.flip_sym2raw[i]);
        for j in 0..N_MOVES_P1 {
            let mut d = Cubie::new();
            Cubie::edge_mult(&c, &sctx.movecube[j], &mut d);
            stbl.flip_move[i * N_MOVES_P1 + j] = stbl.flip_raw2sym[d.get_flip() as usize];
        }
    }

    for i in 0..N_TWST_SYM {
        c.set_twst(stbl.twst_sym2raw[i]);
        for j in 0..N_MOVES_P1 {
            let mut d = Cubie::new();
            Cubie::corn_mult(&c, &sctx.movecube[j], &mut d);
            stbl.twst_move[i * N_MOVES_P1 + j] = stbl.twst_raw2sym[d.get_twst() as usize];
        }
    }

    for i in 0..N_SLICE {
        c.set_slice(i as i32);
        for j in 0..N_MOVES_P1 {
            let mut d = Cubie::new();
            Cubie::edge_mult(&c, &sctx.movecube[j], &mut d);
            stbl.slice_move[i * N_MOVES_P1 + j] = d.get_slice();
        }
        for j in 0..8 {
            let mut e = Cubie::new();
            let mut d = Cubie::new();
            Cubie::edge_mult(&sctx.symcube[j << 1], &c, &mut e);
            Cubie::edge_mult(&e, &sctx.symcube[j << 1], &mut d);
            stbl.slice_conj[i * 8 + j] = d.get_slice();
        }
    }

    c.reset();
    for i in 0..N_PERM_SYM {
        c.set_cperm(stbl.eperm_sym2raw[i]);
        c.set_eperm(stbl.eperm_sym2raw[i]);
        for j in 0..N_MOVES_P2 {
            let mut d = Cubie::new();
            Cubie::corn_mult(&c, &sctx.movecube[P2MOVES[j] as usize], &mut d);
            Cubie::edge_mult(&c, &sctx.movecube[P2MOVES[j] as usize], &mut d);
            stbl.cperm_move[i * N_MOVES_P2 + j] =
                esym2csym(stbl.eperm_raw2sym[d.get_cperm() as usize]);
            stbl.eperm_move[i * N_MOVES_P2 + j] = stbl.eperm_raw2sym[d.get_eperm() as usize];
        }
        let mut d = Cubie::new();
        Cubie::inv(&c, &mut d);
        stbl.perm_sym_inv[i] = stbl.eperm_raw2sym[d.get_eperm() as usize];
        stbl.cperm2comb[i] = c.get_ccomb() as u8;
    }

    for i in 0..N_MPERM {
        c.set_mperm(i as u16);
        for j in 0..N_MOVES_P2 {
            let mut d = Cubie::new();
            Cubie::edge_mult(&c, &sctx.movecube[P2MOVES[j] as usize], &mut d);
            stbl.mperm_move[i * N_MOVES_P2 + j] = d.get_mperm() as u16;
        }
        for j in 0..16 {
            let mut e = Cubie::new();
            let mut d = Cubie::new();
            Cubie::edge_mult(&sctx.symcube[j], &c, &mut e);
            Cubie::edge_mult(&e, &sctx.symcube[sctx.symmuli[0][j] as usize], &mut d);
            stbl.mperm_conj[i * 16 + j] = d.get_mperm() as u16;
        }
    }

    for i in 0..N_CCOMB {
        c.set_ccomb(i as i32);
        for j in 0..N_MOVES_P2 {
            let mut d = Cubie::new();
            Cubie::corn_mult(&c, &sctx.movecube[P2MOVES[j] as usize], &mut d);
            stbl.ccomb_move[i * N_MOVES_P2 + j] = d.get_ccomb() as u16;
        }
        for j in 0..16 {
            let mut e = Cubie::new();
            let mut d = Cubie::new();
            Cubie::corn_mult(&sctx.symcube[j], &c, &mut e);
            Cubie::corn_mult(&e, &sctx.symcube[sctx.symmuli[0][j] as usize], &mut d);
            stbl.ccomb_conj[i * 16 + j] = d.get_ccomb() as u16;
        }
    }
}

pub(super) fn set_pruning(table: &mut [u32], index: usize, value: u32) {
    table[index >> 3] ^= value << ((index & 7) << 2);
}

#[inline]
pub(super) fn get_pruning(table: &[u32], index: usize) -> u32 {
    (table[index >> 3] >> ((index & 7) << 2)) & 0xf
}

#[allow(clippy::needless_range_loop, clippy::too_many_arguments)]
pub(super) fn init_raw_sym_prun(
    prun_table: &mut [u32],
    raw_move: &[u16],
    raw_conj: &[u16],
    sym_move: &[u16],
    sym_selfsym: &[u16],
    n_raw: usize,
    n_sym: usize,
    prun_flag: usize,
) {
    let sym_shift: usize = prun_flag & 0xf;
    let sym_e2c_magic: usize = if (prun_flag >> 4) & 1 == 1 {
        0x00DDDD00
    } else {
        0
    };
    let is_phase2: usize = if (prun_flag >> 5) & 1 == 1 { 1 } else { 0 };
    let inv_depth: usize = (prun_flag >> 8) & 0xf;
    let max_depth: usize = (prun_flag >> 12) & 0xf;

    let sym_mask: usize = (1 << sym_shift) - 1;
    let n_entries: usize = n_raw * n_sym;
    let n_moves: usize = if is_phase2 != 0 {
        N_MOVES_P2
    } else {
        N_MOVES_P1
    };

    let mut depth: usize = 0;
    let mut _done: usize = 1;

    for i in 0..(n_entries / 8 + 1) {
        prun_table[i] = 0xffffffff;
    }
    set_pruning(prun_table, 0, 0xf);

    while depth < max_depth {
        let inv = depth > inv_depth;
        let select = (if inv { 0xf } else { depth }) as u32;
        let check = (if inv { depth } else { 0xf }) as u32;
        depth += 1;
        let xor_val = (depth ^ 0xf) as u32;
        let mut val: u32 = 0;
        let mut i = 0;
        while i < n_entries {
            if (i & 7) == 0 {
                val = prun_table[i >> 3];
                if !inv && val == 0xffffffff {
                    i += 8;
                    continue;
                }
            }

            if (val & 0xf) != select {
                i += 1;
                val >>= 4;
                continue;
            }

            let raw = i % n_raw;
            let sym = i / n_raw;

            for m in 0..n_moves {
                let symx = sym_move[sym * n_moves + m] as usize;
                let rawx = raw_conj
                    [(raw_move[raw * n_moves + m] as usize) << sym_shift | (symx & sym_mask)]
                    as usize;
                let symx = symx >> sym_shift;
                let idx = symx * n_raw + rawx;
                let prun = get_pruning(prun_table, idx);

                if prun != check {
                    continue;
                }

                _done += 1;
                if inv {
                    set_pruning(prun_table, i, xor_val);
                    break;
                }

                set_pruning(prun_table, idx, xor_val);
                let idx = idx - rawx;

                for j in 1..=15 {
                    let ssmask = sym_selfsym[symx];
                    if (ssmask >> j) & 1 == 0 {
                        continue;
                    }
                    let idxx = idx
                        + raw_conj[(rawx << sym_shift) | (j ^ (sym_e2c_magic >> (j << 1) & 3))]
                            as usize;
                    if get_pruning(prun_table, idxx) == check {
                        set_pruning(prun_table, idxx, xor_val);
                        _done += 1;
                    }
                }
            }
            i += 1;
            val >>= 4;
        }
        #[cfg(debug_assertions)]
        println!("depth={:2} entry_cnt={:10}", depth, _done);
    }
}

impl StaticTables {
    pub(super) fn box_new(sctx: &StaticContext) -> Box<Self> {
        let mut stbl = Box::new(StaticTables {
            perm_sym_inv: [0u16; N_PERM_SYM],
            cperm2comb: [0u8; N_PERM_SYM],
            flip_sym2raw: [0u16; N_FLIP_SYM],
            flip_raw2sym: [0u16; N_FLIP],
            flip_selfsym: [0u16; N_FLIP_SYM],
            twst_sym2raw: [0u16; N_TWST_SYM],
            twst_raw2sym: [0u16; N_TWST],
            twst_selfsym: [0u16; N_TWST_SYM],
            eperm_sym2raw: [0u16; N_PERM_SYM],
            eperm_raw2sym: [0u16; N_PERM],
            eperm_selfsym: [0u16; N_PERM_SYM],
            flip_move: [0u16; N_FLIP_SYM * N_MOVES_P1],
            twst_move: [0u16; N_TWST_SYM * N_MOVES_P1],
            slice_move: [0u16; N_SLICE * N_MOVES_P1],
            slice_conj: [0u16; N_SLICE * 8],
            cperm_move: [0u16; N_PERM_SYM * N_MOVES_P2],
            eperm_move: [0u16; N_PERM_SYM * N_MOVES_P2],
            mperm_move: [0u16; N_MPERM * N_MOVES_P2],
            mperm_conj: [0u16; N_MPERM * 16],
            ccomb_move: [0u16; N_CCOMB * N_MOVES_P2],
            ccomb_conj: [0u16; N_CCOMB * 16],
            slice_flip_prun: [0u32; N_SLICE * N_FLIP_SYM / 8 + 1],
            slice_twst_prun: [0u32; N_SLICE * N_TWST_SYM / 8 + 1],
            ccomb_eperm_prun: [0u32; N_CCOMB * N_PERM_SYM / 8 + 1],
            mperm_cperm_prun: [0u32; N_MPERM * N_PERM_SYM / 8 + 1],
            twist_flip: TwistFlipTables::new(),
        });
        stbl.init(sctx);
        stbl
    }

    pub(super) fn init(&mut self, sctx: &StaticContext) {
        init_sym2raw(
            sctx,
            N_FLIP,
            0,
            &mut self.flip_sym2raw,
            &mut self.flip_raw2sym,
            &mut self.flip_selfsym,
        );
        init_sym2raw(
            sctx,
            N_TWST,
            1,
            &mut self.twst_sym2raw,
            &mut self.twst_raw2sym,
            &mut self.twst_selfsym,
        );
        init_sym2raw(
            sctx,
            N_PERM,
            2,
            &mut self.eperm_sym2raw,
            &mut self.eperm_raw2sym,
            &mut self.eperm_selfsym,
        );
        init_move_tables(sctx, self);
        self.twist_flip
            .init(sctx, &self.flip_move, &self.flip_selfsym);
        init_raw_sym_prun(
            &mut self.slice_twst_prun,
            &self.slice_move,
            &self.slice_conj,
            &self.twst_move,
            &self.twst_selfsym,
            N_SLICE,
            N_TWST_SYM,
            0x69603,
        );
        init_raw_sym_prun(
            &mut self.slice_flip_prun,
            &self.slice_move,
            &self.slice_conj,
            &self.flip_move,
            &self.flip_selfsym,
            N_SLICE,
            N_FLIP_SYM,
            0x69603,
        );
        init_raw_sym_prun(
            &mut self.ccomb_eperm_prun,
            &self.ccomb_move,
            &self.ccomb_conj,
            &self.eperm_move,
            &self.eperm_selfsym,
            N_CCOMB,
            N_PERM_SYM,
            0x7c824,
        );
        init_raw_sym_prun(
            &mut self.mperm_cperm_prun,
            &self.mperm_move,
            &self.mperm_conj,
            &self.cperm_move,
            &self.eperm_selfsym,
            N_MPERM,
            N_PERM_SYM,
            0x8ea34,
        );
    }
}

pub(super) fn get_perm_sym_inv(
    sctx: &StaticContext,
    stbl: &StaticTables,
    idx: u16,
    sym: u16,
    is_corner: i32,
) -> u16 {
    let idxi = stbl.perm_sym_inv[idx as usize];
    let mut result = if is_corner != 0 {
        esym2csym(idxi)
    } else {
        idxi
    };
    result = (result & 0xfff0) | (sctx.symmult[result as usize & 0xf][sym as usize] as u16);
    result
}

/// Both table sets of the engine, built together.
pub(super) struct Tables {
    pub(super) sctx: Box<StaticContext>,
    pub(super) stbl: Box<StaticTables>,
}

impl Tables {
    fn build() -> Tables {
        let sctx = StaticContext::box_new();
        let stbl = StaticTables::box_new(&sctx);
        Tables { sctx, stbl }
    }
}

/// Tables that are built on first use and shared between the clones of one
/// solver, including the worker threads it hands them to.
pub(super) type SharedTables = Arc<OnceLock<Tables>>;

/// A new table set, not yet built and shared with no other solver.
pub(super) fn fresh() -> SharedTables {
    Arc::new(OnceLock::new())
}

/// The tables, building them if this is the first use.
pub(super) fn get(shared: &OnceLock<Tables>) -> &Tables {
    shared.get_or_init(Tables::build)
}
