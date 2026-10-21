use std::sync::atomic::{AtomicI8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::consts::*;
use super::coordinates::*;
use super::cubie::*;
use super::tables::*;

#[repr(C)]
pub(super) struct Solution {
    pub(super) depth1: i8,
    pub(super) verbose: u8,
    pub(super) urf_idx: u8,
    pub(super) premv_len: i8,
    pub(super) length: i8,
    pub(super) moves: [u8; 31],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TimeLimit {
    Continue,
    ReturnSolution,
    TimedOut,
}

impl Solution {
    pub(super) fn append_move(&mut self, cur_move: u8) {
        if self.length == 0 {
            self.moves[self.length as usize] = cur_move;
            self.length += 1;
            return;
        }

        let cur_axis = cur_move / 3;
        let last_axis = self.moves[(self.length - 1) as usize] / 3;

        if cur_axis == last_axis {
            let pow = (cur_move % 3 + self.moves[(self.length - 1) as usize] % 3 + 1) % 4;
            if pow == 3 {
                self.length -= 1;
            } else {
                self.moves[(self.length - 1) as usize] = cur_axis * 3 + pow;
            }
            return;
        }

        if self.length > 1
            && cur_axis % 3 == last_axis % 3
            && cur_axis == self.moves[(self.length - 2) as usize] / 3
        {
            let pow = (cur_move % 3 + self.moves[(self.length - 2) as usize] % 3 + 1) % 4;
            if pow == 3 {
                self.moves[(self.length - 2) as usize] = self.moves[(self.length - 1) as usize];
                self.length -= 1;
            } else {
                self.moves[(self.length - 2) as usize] = cur_axis * 3 + pow;
            }
            return;
        }

        self.moves[self.length as usize] = cur_move;
        self.length += 1;
    }

    pub(super) fn format_moves(&self) -> String {
        let mut buf = String::new();
        let urf = if self.verbose & INVERSE_SOLUTION != 0 {
            (self.urf_idx + 3) % 6
        } else {
            self.urf_idx
        };

        if urf < 3 {
            for (s, &mv) in self.moves.iter().enumerate().take(self.length as usize) {
                if self.verbose & USE_SEPARATOR != 0 && s == self.depth1 as usize {
                    buf.push_str(".  ");
                }
                buf.push_str(MOVE2STR[URF_MOVE[urf as usize][mv as usize] as usize]);
                buf.push(' ');
            }
        } else {
            for (s, &mv) in self
                .moves
                .iter()
                .enumerate()
                .take(self.length as usize)
                .rev()
            {
                buf.push_str(MOVE2STR[URF_MOVE[urf as usize][mv as usize] as usize]);
                buf.push(' ');
                if self.verbose & USE_SEPARATOR != 0 && s == self.depth1 as usize {
                    buf.push_str(".  ");
                }
            }
        }

        if self.verbose & APPEND_LENGTH != 0 {
            buf.push_str(&format!("({}f)", self.length));
        }

        buf
    }
}

#[repr(C)]
pub(super) struct IdaContext {
    pub(super) mv: [u8; 30],
    pub(super) allow_shorter: bool,
    pub(super) depth1: i8,
    pub(super) length1: i8,
    pub(super) valid1: i8,
    pub(super) urf_idx: u8,
    pub(super) root_only_axis: i8,
    pub(super) shared_best: Option<Arc<AtomicI8>>,
    pub(super) improve: bool,
    pub(super) p1_cubies: [Cubie; 20],
    pub(super) urf_cubies: [Cubie; 6],
    pub(super) premv: [u8; 15],
    pub(super) premv_len: i8,
    pub(super) max_depth2: i8,
    pub(super) target_length: i8,
    pub(super) probes: u64,
    pub(super) phase1_nodes: u64,
    pub(super) phase1_rejected_slice_twist: u64,
    pub(super) phase1_rejected_slice_flip: u64,
    pub(super) phase1_slice_twist_breaks: u64,
    pub(super) phase1_slice_twist_equals: u64,
    pub(super) phase1_slice_flip_breaks: u64,
    pub(super) phase1_slice_flip_equals: u64,
    pub(super) phase1_children_recurred: u64,
    pub(super) deadline: Option<Instant>,
    pub(super) minimum_deadline: Option<Instant>,
    pub(super) timed_out: bool,
    pub(super) deadline_with_solution: bool,
    pub(super) has_solution: bool,
    pub(super) solution: Solution,
}

impl IdaContext {
    pub fn new() -> Self {
        IdaContext {
            mv: [0; 30],
            allow_shorter: false,
            depth1: 0,
            length1: 0,
            valid1: 0,
            urf_idx: 0,
            root_only_axis: -1,
            shared_best: None,
            improve: false,
            p1_cubies: [Cubie::new(); 20],
            urf_cubies: [Cubie::new(); 6],
            premv: [0; 15],
            premv_len: 0,
            max_depth2: 0,
            target_length: 0,
            probes: 0,
            phase1_nodes: 0,
            phase1_rejected_slice_twist: 0,
            phase1_rejected_slice_flip: 0,
            phase1_slice_twist_breaks: 0,
            phase1_slice_twist_equals: 0,
            phase1_slice_flip_breaks: 0,
            phase1_slice_flip_equals: 0,
            phase1_children_recurred: 0,
            deadline: None,
            minimum_deadline: None,
            timed_out: false,
            deadline_with_solution: false,
            has_solution: false,
            solution: Solution {
                depth1: 0,
                verbose: 0,
                urf_idx: 0,
                premv_len: 0,
                length: 0,
                moves: [0; 31],
            },
        }
    }

    pub(super) fn set_time_limits(&mut self, minimum: Duration, maximum: Option<Duration>) {
        let started = Instant::now();
        self.set_deadlines(
            maximum.and_then(|budget| started.checked_add(budget)),
            maximum
                .map(|maximum| minimum.min(maximum))
                .and_then(|minimum| started.checked_add(minimum)),
        )
    }

    pub(super) fn set_deadlines(
        &mut self,
        deadline: Option<Instant>,
        minimum_deadline: Option<Instant>,
    ) {
        self.deadline = deadline;
        self.minimum_deadline = minimum_deadline;
        self.timed_out = false;
        self.deadline_with_solution = false;
        self.has_solution = false;
    }

    pub(super) fn time_limit(&mut self) -> TimeLimit {
        let Some(maximum_deadline) = self.deadline else {
            return if self.has_solution && !self.improve {
                TimeLimit::ReturnSolution
            } else {
                TimeLimit::Continue
            };
        };
        let now = Instant::now();
        if now >= maximum_deadline {
            if self.has_solution {
                self.deadline_with_solution = true;
                return TimeLimit::ReturnSolution;
            }
            self.timed_out = true;
            return TimeLimit::TimedOut;
        }
        if self.has_solution
            && self
                .minimum_deadline
                .is_none_or(|minimum_deadline| now >= minimum_deadline)
        {
            return TimeLimit::ReturnSolution;
        }
        TimeLimit::Continue
    }

    pub fn solve_cubie(
        &mut self,
        sctx: &StaticContext,
        stbl: &StaticTables,
        cc: &Cubie,
        target_length: i8,
    ) -> String {
        match self.time_limit() {
            TimeLimit::Continue => {}
            TimeLimit::ReturnSolution => return self.solution.format_moves(),
            TimeLimit::TimedOut => return String::from("Error 8"),
        }
        let mut cc1 = *cc;
        let mut cc2 = Cubie::new();
        self.target_length = target_length + 1;
        self.probes = 0;
        for i in 0..6 {
            self.urf_cubies[i] = cc1;
            Cubie::corn_mult(&sctx.symurfi, &cc1, &mut cc2);
            Cubie::edge_mult(&sctx.symurfi, &cc1, &mut cc2);
            Cubie::corn_mult(&cc2, &sctx.symurf, &mut cc1);
            Cubie::edge_mult(&cc2, &sctx.symurf, &mut cc1);
            if i == 2 {
                Cubie::inv(&cc1, &mut cc2);
                cc1 = cc2;
            }
        }
        for length1 in 0..21 {
            match self.time_limit() {
                TimeLimit::Continue => {}
                TimeLimit::ReturnSolution => return self.solution.format_moves(),
                TimeLimit::TimedOut => return String::from("Error 8"),
            }
            // A solution of at most target_length - 1 moves cannot have a longer first phase.
            if length1 >= self.target_length {
                break;
            }
            self.length1 = length1;
            self.max_depth2 =
                std::cmp::min(MAX_DEPTH2 as i8, self.target_length - self.length1 - 1);
            self.depth1 = self.length1 - self.premv_len;
            self.allow_shorter = false;
            for urf_idx in 0..6 {
                self.urf_idx = urf_idx;
                let cc = self.urf_cubies[self.urf_idx as usize];
                let ret = self.phase1_pre_moves(sctx, stbl, MAX_PREMV_LEN, -30, &cc, 0);
                if ret == 0 {
                    let solbuf = self.solution.format_moves();
                    #[cfg(debug_assertions)]
                    println!(
                        "solution found in {:2}+{:2} moves urf={} premv={} probe={:5}: {}",
                        self.length1,
                        self.solution.length - self.length1,
                        self.solution.urf_idx,
                        self.solution.premv_len,
                        self.probes,
                        solbuf
                    );
                    return solbuf;
                }
                if ret == PREMOVE_TIMEOUT {
                    return if self.has_solution {
                        self.solution.format_moves()
                    } else {
                        String::from("Error 8")
                    };
                }
            }
        }
        if self.has_solution {
            self.solution.format_moves()
        } else {
            String::from("Error 8")
        }
    }

    pub(super) fn solve_one_urf(
        &mut self,
        sctx: &StaticContext,
        stbl: &StaticTables,
        cc: &Cubie,
        urf_idx: u8,
        target_length: i8,
    ) -> String {
        match self.time_limit() {
            TimeLimit::Continue => {}
            TimeLimit::ReturnSolution => return self.solution.format_moves(),
            TimeLimit::TimedOut => return String::from("Error 8"),
        }
        self.target_length = target_length + 1;
        self.probes = 0;
        self.urf_idx = urf_idx;
        for length1 in 0..21 {
            match self.time_limit() {
                TimeLimit::Continue => {}
                TimeLimit::ReturnSolution => return self.solution.format_moves(),
                TimeLimit::TimedOut => return String::from("Error 8"),
            }
            // A solution of at most target_length - 1 moves cannot have a longer first phase.
            if length1 >= self.target_length {
                break;
            }
            self.length1 = length1;
            if let Some(best) = &self.shared_best {
                self.target_length = self.target_length.min(best.load(Ordering::Relaxed));
                if self.length1 >= self.target_length {
                    break;
                }
            }
            self.max_depth2 = std::cmp::min(MAX_DEPTH2 as i8, self.target_length - length1 - 1);
            self.depth1 = self.length1 - self.premv_len;
            self.allow_shorter = false;
            let ret = self.phase1_pre_moves(sctx, stbl, MAX_PREMV_LEN, -30, cc, 0);
            if ret == 0 {
                return self.solution.format_moves();
            }
            if ret == PREMOVE_TIMEOUT {
                return if self.has_solution {
                    self.solution.format_moves()
                } else {
                    String::from("Error 8")
                };
            }
        }
        if self.has_solution {
            self.solution.format_moves()
        } else {
            String::from("Error 8")
        }
    }

    pub(super) fn phase1_pre_moves(
        &mut self,
        sctx: &StaticContext,
        stbl: &StaticTables,
        maxl: i8,
        lm: i8,
        cc: &Cubie,
        _ssym: i32,
    ) -> i32 {
        match self.time_limit() {
            TimeLimit::Continue => {}
            TimeLimit::ReturnSolution => return 0,
            TimeLimit::TimedOut => return PREMOVE_TIMEOUT,
        }
        self.premv_len = MAX_PREMV_LEN - maxl;
        if self.premv_len == 0 || ((0o667667 >> lm) & 1) == 0 {
            self.depth1 = self.length1 - self.premv_len;
            self.allow_shorter = self.depth1 == MIN_P1PRE_LEN && self.premv_len != 0;
            self.p1_cubies[0] = *cc;
            let mut node = Coord::new();
            if node.set_from_cubie(stbl, &self.p1_cubies[0]) <= self.depth1 {
                let ret = self.phase1(sctx, stbl, &node, 0, self.depth1, -1);
                if ret == 0 {
                    return 0;
                }
                if ret == SEARCH_TIMEOUT {
                    return PREMOVE_TIMEOUT;
                }
            }
        }
        if maxl == 0 || self.premv_len + MIN_P1PRE_LEN >= self.length1 {
            return 1;
        }
        let skip_moves = if maxl == 1 || self.premv_len + 1 + MIN_P1PRE_LEN >= self.length1 {
            0o227227
        } else {
            0
        };
        let mut cd = Cubie::new();
        let lm = lm / 3;
        for m in 0..18 {
            if m / 3 == lm || m / 3 == lm - 3 || m / 3 == lm + 3 {
                continue;
            }
            if (skip_moves & (1 << m)) != 0 {
                continue;
            }
            Cubie::corn_mult(&sctx.movecube[m as usize], cc, &mut cd);
            Cubie::edge_mult(&sctx.movecube[m as usize], cc, &mut cd);
            self.premv[MAX_PREMV_LEN as usize - maxl as usize] = m as u8;
            let ret = self.phase1_pre_moves(sctx, stbl, maxl - 1, m, &cd, 0);
            if ret == 0 {
                return 0;
            }
            if ret == PREMOVE_TIMEOUT {
                return PREMOVE_TIMEOUT;
            }
        }
        1
    }

    pub(super) fn phase1(
        &mut self,
        sctx: &StaticContext,
        stbl: &StaticTables,
        node: &Coord,
        _ssym: i32,
        maxl: i8,
        lm: i8,
    ) -> i8 {
        match self.time_limit() {
            TimeLimit::Continue => {}
            TimeLimit::ReturnSolution => return 0,
            TimeLimit::TimedOut => return SEARCH_TIMEOUT,
        }
        self.phase1_nodes += 1;
        let mut next_node: Coord = Coord::new();
        if node.prun == 0 && maxl < 5 {
            if self.allow_shorter || maxl == 0 {
                self.depth1 -= maxl;
                let ret = self.init_phase2(sctx, stbl);
                self.depth1 += maxl;
                return if ret == SEARCH_COMPLETE { 0 } else { ret };
            } else {
                return 1;
            }
        }
        for axis in (0..N_MOVES_P1 as i8).step_by(3) {
            if maxl == self.depth1 && self.root_only_axis >= 0 && axis != self.root_only_axis {
                continue;
            }
            if axis == lm || axis == lm - 9 {
                continue;
            }
            for power in 0..3 {
                let m = axis + power;
                let slice_twist = next_node.move_slice_twist(sctx, stbl, node, m as usize);
                if slice_twist > maxl {
                    self.phase1_rejected_slice_twist += 1;
                    self.phase1_slice_twist_breaks += 1;
                    break;
                }
                let slice_flip = next_node.move_flip_prun(sctx, stbl, node, m as usize);
                let prun = slice_twist.max(slice_flip);
                next_node.prun = prun;
                if prun > maxl {
                    self.phase1_rejected_slice_flip += 1;
                    self.phase1_slice_flip_breaks += 1;
                    break;
                } else if prun == maxl {
                    if slice_twist == maxl {
                        self.phase1_rejected_slice_twist += 1;
                        self.phase1_slice_twist_equals += 1;
                    } else {
                        self.phase1_rejected_slice_flip += 1;
                        self.phase1_slice_flip_equals += 1;
                    }
                    continue;
                }
                self.mv[self.depth1 as usize - maxl as usize] = m as u8;
                self.valid1 = self.valid1.min(self.depth1 - maxl);
                self.phase1_children_recurred += 1;
                let ret = self.phase1(sctx, stbl, &next_node, 0, maxl - 1, axis);
                if ret == 0 {
                    return 0;
                } else if ret == SEARCH_TIMEOUT {
                    return SEARCH_TIMEOUT;
                } else if ret >= 2 {
                    break;
                }
            }
        }
        1
    }

    pub(super) fn init_phase2(&mut self, sctx: &StaticContext, stbl: &StaticTables) -> i8 {
        match self.time_limit() {
            TimeLimit::Continue => {}
            TimeLimit::ReturnSolution => return SEARCH_COMPLETE,
            TimeLimit::TimedOut => return SEARCH_TIMEOUT,
        }
        if let Some(best) = &self.shared_best {
            self.target_length = self.target_length.min(best.load(Ordering::Relaxed));
            self.max_depth2 =
                std::cmp::min(MAX_DEPTH2 as i8, self.target_length - self.length1 - 1);
        }
        self.probes += 1;
        let mut cc = if self.depth1 == 0 {
            self.p1_cubies[0]
        } else {
            Cubie::new()
        };
        for i in self.valid1 as usize..self.depth1 as usize {
            Cubie::corn_mult(
                &self.p1_cubies[i],
                &sctx.movecube[self.mv[i] as usize],
                &mut cc,
            );
            Cubie::edge_mult(
                &self.p1_cubies[i],
                &sctx.movecube[self.mv[i] as usize],
                &mut cc,
            );
            self.p1_cubies[i + 1] = cc;
        }
        self.valid1 = self.depth1;
        let mut node1 = Coord2::new();
        let mut prun = node1.set_from_cubie(sctx, stbl, &cc);
        let mut node2 = Coord2::new();
        if self.premv_len > 0 {
            let m = self.premv[self.premv_len as usize - 1] as usize / 3 * 3 + 1;
            let mut cd = Cubie::new();
            Cubie::corn_mult(&sctx.movecube[m], &cc, &mut cd);
            Cubie::edge_mult(&sctx.movecube[m], &cc, &mut cd);
            prun = prun.min(node2.set_from_cubie(sctx, stbl, &cd));
        }
        if prun > self.max_depth2 {
            return prun - self.max_depth2;
        }
        let mut depth2 = self.max_depth2;
        while depth2 >= prun {
            let mut sol_src = 0;
            let mut ret = self.phase2(sctx, stbl, &node1, depth2, self.depth1, 10);
            if ret == SEARCH_COMPLETE {
                return SEARCH_COMPLETE;
            }
            if ret == SEARCH_TIMEOUT {
                return SEARCH_TIMEOUT;
            }
            if ret < 0 && self.premv_len > 0 {
                sol_src = 1;
                ret = self.phase2(sctx, stbl, &node2, depth2, self.depth1, 10);
                if ret == SEARCH_COMPLETE {
                    return SEARCH_COMPLETE;
                }
                if ret == SEARCH_TIMEOUT {
                    return SEARCH_TIMEOUT;
                }
            }
            if ret < 0 {
                break;
            }
            depth2 -= ret;
            self.target_length = 0;
            self.solution.length = 0;
            self.solution.urf_idx = self.urf_idx;
            self.solution.depth1 = self.depth1;
            self.solution.premv_len = self.premv_len;
            for i in 0..self.depth1 + depth2 {
                self.solution.append_move(self.mv[i as usize]);
            }
            if sol_src == 1 {
                self.solution
                    .append_move(self.premv[self.premv_len as usize - 1] / 3 * 3 + 1);
            }
            for i in (0..self.premv_len).rev() {
                self.solution.append_move(self.premv[i as usize]);
            }
            self.target_length = self.solution.length;
            if let Some(best) = &self.shared_best {
                best.fetch_min(self.solution.length, Ordering::Relaxed);
            }
            self.has_solution = true;
            depth2 -= 1;
        }

        if depth2 != self.max_depth2 {
            self.max_depth2 =
                std::cmp::min(MAX_DEPTH2 as i8, self.target_length - self.length1 - 1);
            return match self.time_limit() {
                TimeLimit::Continue => 1,
                TimeLimit::ReturnSolution => 0,
                TimeLimit::TimedOut => SEARCH_TIMEOUT,
            };
        }
        1
    }

    #[allow(clippy::needless_range_loop)]
    pub(super) fn phase2(
        &mut self,
        sctx: &StaticContext,
        stbl: &StaticTables,
        node: &Coord2,
        maxl: i8,
        depth: i8,
        lm: i8,
    ) -> i8 {
        match self.time_limit() {
            TimeLimit::Continue => {}
            TimeLimit::ReturnSolution => return SEARCH_COMPLETE,
            TimeLimit::TimedOut => return SEARCH_TIMEOUT,
        }
        if node.edge == 0 && node.corn == 0 && node.mid == 0 {
            return maxl;
        }
        let move_mask = sctx.canon_masks2[lm as usize];
        let mut nodex = Coord2::new();
        for m in 0..N_MOVES_P2 {
            if (move_mask >> m & 1) != 0 {
                continue;
            }
            nodex.mid = stbl.mperm_move[node.mid as usize * N_MOVES_P2 + m];
            nodex.corn = stbl.cperm_move
                [node.corn as usize * N_MOVES_P2 + sctx.symmove2[m][node.csym as usize] as usize];
            nodex.csym = sctx.symmult[nodex.corn as usize & 0xf][node.csym as usize] as u16;
            nodex.corn >>= 4;
            nodex.edge = stbl.eperm_move
                [node.edge as usize * N_MOVES_P2 + sctx.symmove2[m][node.esym as usize] as usize];
            nodex.esym = sctx.symmult[nodex.edge as usize & 0xf][node.esym as usize] as u16;
            nodex.edge >>= 4;
            let edgei = get_perm_sym_inv(sctx, stbl, nodex.edge, nodex.esym, 0) as usize;
            let corni = get_perm_sym_inv(sctx, stbl, nodex.corn, nodex.csym, 1) as usize;
            let prun = get_pruning(
                &stbl.ccomb_eperm_prun,
                (edgei >> 4) * N_CCOMB
                    + stbl.ccomb_conj[stbl.cperm2comb[corni >> 4] as usize * 16
                        + sctx.symmuli[edgei & 0xf][corni & 0xf] as usize]
                        as usize,
            ) as i8;
            if prun > maxl + 1 {
                return maxl - prun + 1;
            } else if prun >= maxl {
                continue;
            }
            let prun = std::cmp::max(
                get_pruning(
                    &stbl.mperm_cperm_prun,
                    nodex.corn as usize * N_MPERM
                        + stbl.mperm_conj[nodex.mid as usize * 16 + nodex.csym as usize] as usize,
                ),
                get_pruning(
                    &stbl.ccomb_eperm_prun,
                    nodex.edge as usize * N_CCOMB
                        + stbl.ccomb_conj[stbl.cperm2comb[nodex.corn as usize] as usize * 16
                            + sctx.symmuli[nodex.esym as usize][nodex.csym as usize] as usize]
                            as usize,
                ),
            ) as i8;
            if prun >= maxl {
                continue;
            }
            let ret = self.phase2(sctx, stbl, &nodex, maxl - 1, depth + 1, m as i8);
            if ret == SEARCH_COMPLETE {
                return SEARCH_COMPLETE;
            } else if ret >= 0 {
                self.mv[depth as usize] = P2MOVES[m];
                return ret;
            } else if ret == SEARCH_TIMEOUT {
                return SEARCH_TIMEOUT;
            } else if ret < -2 {
                break;
            }
        }
        -1
    }
}
