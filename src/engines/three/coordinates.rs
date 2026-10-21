use super::consts::*;
use super::cubie::*;
use super::tables::*;

#[derive(Clone, Copy)]
pub(super) struct Coord {
    pub(super) twst: u16,
    pub(super) tsym: u16,
    pub(super) flip: u16,
    pub(super) fsym: u16,
    pub(super) slice: u16,
    pub(super) prun: i8,
}

#[derive(Clone, Copy)]
pub(super) struct Coord2 {
    pub(super) edge: u16,
    pub(super) esym: u16,
    pub(super) corn: u16,
    pub(super) csym: u16,
    pub(super) mid: u16,
}

impl Coord {
    pub(super) fn new() -> Self {
        Coord {
            twst: 0u16,
            tsym: 0u16,
            flip: 0u16,
            fsym: 0u16,
            slice: 0u16,
            prun: 0i8,
        }
    }

    pub(super) fn set_from_cubie(&mut self, stbl: &StaticTables, src: &Cubie) -> i8 {
        self.slice = src.get_slice();
        self.flip = stbl.flip_raw2sym[src.get_flip() as usize];
        self.fsym = self.flip & 7;
        self.flip >>= 3;
        self.twst = stbl.twst_raw2sym[src.get_twst() as usize];
        self.tsym = self.twst & 7;
        self.twst >>= 3;
        self.prun = std::cmp::max(
            get_pruning(
                &stbl.slice_twst_prun,
                self.twst as usize * N_SLICE
                    + stbl.slice_conj[(self.slice * 8 + self.tsym) as usize] as usize,
            ),
            get_pruning(
                &stbl.slice_flip_prun,
                self.flip as usize * N_SLICE
                    + stbl.slice_conj[(self.slice * 8 + self.fsym) as usize] as usize,
            ),
        ) as i8;
        self.prun
    }

    #[inline]
    pub(super) fn move_slice_twist(
        &mut self,
        sctx: &StaticContext,
        stbl: &StaticTables,
        src: &Coord,
        mv: usize,
    ) -> i8 {
        self.slice = stbl.slice_move[src.slice as usize * N_MOVES_P1 + mv];
        self.twst = stbl.twst_move
            [src.twst as usize * N_MOVES_P1 + sctx.symmove[mv][src.tsym as usize] as usize];
        self.tsym = (self.twst & 7) ^ src.tsym;
        self.twst >>= 3;
        get_pruning(
            &stbl.slice_twst_prun,
            self.twst as usize * N_SLICE
                + stbl.slice_conj[(self.slice * 8 + self.tsym) as usize] as usize,
        ) as i8
    }

    #[inline]
    pub(super) fn move_flip_prun(
        &mut self,
        sctx: &StaticContext,
        stbl: &StaticTables,
        src: &Coord,
        mv: usize,
    ) -> i8 {
        self.flip = stbl.flip_move
            [src.flip as usize * N_MOVES_P1 + sctx.symmove[mv][src.fsym as usize] as usize];
        self.fsym = (self.flip & 7) ^ src.fsym;
        self.flip >>= 3;
        get_pruning(
            &stbl.slice_flip_prun,
            self.flip as usize * N_SLICE
                + stbl.slice_conj[(self.slice * 8 + self.fsym) as usize] as usize,
        ) as i8
    }
}

impl Coord2 {
    pub(super) fn new() -> Self {
        Coord2 {
            edge: 0u16,
            esym: 0u16,
            corn: 0u16,
            csym: 0u16,
            mid: 0u16,
        }
    }

    pub(super) fn set_from_cubie(
        &mut self,
        sctx: &StaticContext,
        stbl: &StaticTables,
        src: &Cubie,
    ) -> i8 {
        self.corn = esym2csym(stbl.eperm_raw2sym[src.get_cperm() as usize]);
        self.csym = self.corn & 0xf;
        self.corn >>= 4;
        self.edge = stbl.eperm_raw2sym[src.get_eperm() as usize];
        self.esym = self.edge & 0xf;
        self.edge >>= 4;
        self.mid = src.get_mperm() as u16;
        let edgei = get_perm_sym_inv(sctx, stbl, self.edge, self.esym, 0);
        let corni = get_perm_sym_inv(sctx, stbl, self.corn, self.csym, 1);
        std::cmp::max(
            get_pruning(
                &stbl.ccomb_eperm_prun,
                (edgei >> 4) as usize * N_CCOMB
                    + stbl.ccomb_conj[stbl.cperm2comb[corni as usize >> 4] as usize * 16
                        + sctx.symmuli[edgei as usize & 0xf][corni as usize & 0xf] as usize]
                        as usize,
            ),
            std::cmp::max(
                get_pruning(
                    &stbl.ccomb_eperm_prun,
                    self.edge as usize * N_CCOMB
                        + stbl.ccomb_conj[stbl.cperm2comb[self.corn as usize] as usize * 16
                            + sctx.symmuli[self.esym as usize][self.csym as usize] as usize]
                            as usize,
                ),
                get_pruning(
                    &stbl.mperm_cperm_prun,
                    self.corn as usize * N_MPERM
                        + stbl.mperm_conj[self.mid as usize * 16 + self.csym as usize] as usize,
                ),
            ),
        ) as i8
    }
}
