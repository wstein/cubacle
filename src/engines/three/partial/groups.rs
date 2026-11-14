//! Four-cubie group tables: small pruning tables for the front goal.
//!
//! A table tracks four cubies, each by the location digit of the cubie (the
//! slot and the orientation it sits in: 24 values for a corner and for an
//! edge), so it has 24^4 = 331,776 entries of one byte. An entry is the
//! distance to the nearest abstract goal: a placement of the four cubies such
//! that some view accepts them where they sit and an untracked cubie could
//! fill every other slot the view constrains. This relaxes the goal, so the
//! entry never exceeds the true distance. Several tables are combined by the
//! maximum. The center twists have their own table (`twist`).
//!
//! The cubies are chosen per goal: the ones the views need most come first,
//! and the groups are the nine of the research on wstein/cubistry#484 (four
//! corners, the other four, edges 1 to 4, 5 to 8, 9 to 12, and four mixed). The
//! ranking of the cubies is not the research's (see `ranked`).

use super::gate::{Gate, Stop};
use super::goal::{Goal, View};
use super::model::{Pos, Turns, TURNS};
use super::tables::Built;

const DIGITS: usize = 24;
const STATES: usize = DIGITS * DIGITS * DIGITS * DIGITS;
const UNSEEN: u8 = u8::MAX;

/// What kind of cubie a table tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Orbit {
    Corner,
    Edge,
}

/// A tracked cubie: its orbit and its piece number in the cubie model.
pub(super) type Tracked = (Orbit, u8);

/// How the 9 groups are cut from the corners (`c`) and the edges (`e`), each
/// ranked by need: indices into the two rankings.
const GROUPS: [[Tracked; 4]; 9] = {
    use Orbit::{Corner as C, Edge as E};
    [
        [(C, 0), (C, 1), (C, 2), (C, 3)],
        [(C, 4), (C, 5), (C, 6), (C, 7)],
        [(E, 0), (E, 1), (E, 2), (E, 3)],
        [(E, 4), (E, 5), (E, 6), (E, 7)],
        [(E, 8), (E, 9), (E, 10), (E, 11)],
        [(C, 0), (C, 1), (E, 0), (E, 1)],
        [(C, 2), (C, 3), (E, 2), (E, 3)],
        [(C, 0), (C, 1), (C, 2), (E, 0)],
        [(E, 0), (E, 1), (E, 2), (C, 0)],
    ]
};

/// Which of the nine groups the sets of two and four tables use.
fn chosen(count: usize) -> &'static [usize] {
    match count {
        2 => &[0, 2],
        4 => &[0, 2, 3, 5],
        _ => &[0, 1, 2, 3, 4, 5, 6, 7, 8],
    }
}

/// The pieces of `orbit` that the views need most come first: each constrained
/// slot gives its weight 1 / (pieces it accepts) to each piece it accepts.
///
/// This differs from `build_heuristic` in `FrontGoal.jl` (wstein/cubistry#484),
/// which adds 1 / (stickers accepted at the slot) once per sticker of a cubie
/// that the slot accepts, and breaks ties by its own cubie order. Here a piece
/// counts once per slot and ties go by piece number, so the ranking, and with
/// it the node counts, can differ from the research. Any ranking gives an
/// admissible bound, so lengths are not affected.
fn ranked(goal: &Goal, orbit: Orbit) -> Vec<u8> {
    let (n, width) = match orbit {
        Orbit::Corner => (8usize, 3),
        Orbit::Edge => (12, 2),
    };
    let mut score = vec![0f64; n];
    for view in &goal.views {
        let list = match orbit {
            Orbit::Corner => &view.corners,
            Orbit::Edge => &view.edges,
        };
        for &(_, bits) in list {
            let accepted: Vec<usize> = (0..n)
                .filter(|&p| bits >> (p * width) & ((1 << width) - 1) != 0)
                .collect();
            for &p in &accepted {
                score[p] += 1.0 / accepted.len() as f64;
            }
        }
    }
    let mut pieces: Vec<u8> = (0..n as u8).collect();
    pieces.sort_by(|&a, &b| {
        score[usize::from(b)]
            .partial_cmp(&score[usize::from(a)])
            .expect("scores are numbers")
            .then(a.cmp(&b))
    });
    pieces
}

/// The groups of `count` tables (2, 4 or, for any other number, 9) for `goal`.
pub(super) fn plan_groups(goal: &Goal, count: usize) -> Vec<[Tracked; 4]> {
    let corners = ranked(goal, Orbit::Corner);
    let edges = ranked(goal, Orbit::Edge);
    chosen(count)
        .iter()
        .map(|&g| {
            GROUPS[g].map(|(orbit, rank)| match orbit {
                Orbit::Corner => (orbit, corners[usize::from(rank)]),
                Orbit::Edge => (orbit, edges[usize::from(rank)]),
            })
        })
        .collect()
}

/// Where each digit goes under each turn, for corners and edges.
struct Digits {
    corner: [[u8; DIGITS]; TURNS],
    edge: [[u8; DIGITS]; TURNS],
}

impl Digits {
    fn new(turns: &Turns) -> Digits {
        let mut digits = Digits {
            corner: [[0; DIGITS]; TURNS],
            edge: [[0; DIGITS]; TURNS],
        };
        for m in 0..TURNS {
            for t in 0..8 {
                let from = usize::from(turns.cfrom[m][t]);
                for o in 0..3 {
                    let moved = (o + usize::from(turns.cadd[m][t])) % 3;
                    digits.corner[m][from * 3 + o] = (t * 3 + moved) as u8;
                }
            }
            for t in 0..12 {
                let from = usize::from(turns.efrom[m][t]);
                for o in 0..2 {
                    let moved = o ^ usize::from(turns.eadd[m][t]);
                    digits.edge[m][from * 2 + o] = (t * 2 + moved) as u8;
                }
            }
        }
        digits
    }
}

/// The distance table of four tracked cubies. A coordinate is the four digits,
/// one per byte (the lowest byte is the first cubie).
pub(super) struct GroupTable {
    tracked: [Tracked; 4],
    step: [[[u8; DIGITS]; TURNS]; 4],
    dist: Vec<u8>,
}

#[inline]
fn flat(at: u32) -> usize {
    let [a, b, c, d] = at.to_le_bytes().map(usize::from);
    ((d * DIGITS + c) * DIGITS + b) * DIGITS + a
}

impl GroupTable {
    /// Bytes a table takes.
    pub(super) const BYTES: usize = STATES;

    /// Unsatisfiable when no placement of the four cubies is abstractly a goal.
    pub(super) fn build(
        goal: &Goal,
        turns: &Turns,
        tracked: &[Tracked; 4],
        gate: &mut Gate,
    ) -> Result<Built<GroupTable>, Stop> {
        let digits = Digits::new(turns);
        let step: [[[u8; DIGITS]; TURNS]; 4] = std::array::from_fn(|i| match tracked[i].0 {
            Orbit::Corner => digits.corner,
            Orbit::Edge => digits.edge,
        });
        let accepts = Acceptance::new(goal, tracked);
        let mut dist = vec![UNSEEN; STATES];
        let mut seeds = 0usize;
        for (flat_at, cell) in dist.iter_mut().enumerate() {
            if flat_at.is_multiple_of(4096) {
                gate.check((flat_at * 300 / STATES) as u16)?;
            }
            let digit = [
                flat_at % DIGITS,
                flat_at / DIGITS % DIGITS,
                flat_at / (DIGITS * DIGITS) % DIGITS,
                flat_at / (DIGITS * DIGITS * DIGITS),
            ];
            if accepts.met(tracked, digit) {
                *cell = 0;
                seeds += 1;
            }
        }
        if seeds == 0 {
            return Ok(Built::Unsatisfiable);
        }
        // Breadth-first search over the placements, level by level.
        let mut frontier: Vec<u32> = (0..STATES as u32)
            .filter(|&f| dist[f as usize] == 0)
            .map(unflat)
            .collect();
        let mut level = 0u8;
        let mut done = 0usize;
        while !frontier.is_empty() {
            level += 1;
            let mut next = Vec::new();
            for &at in &frontier {
                done += 1;
                if done.is_multiple_of(4096) {
                    gate.check((300 + (done * 700 / STATES).min(699)) as u16)?;
                }
                for m in 0..TURNS {
                    let to = advance(&step, at, m);
                    let slot = &mut dist[flat(to)];
                    if *slot == UNSEEN {
                        *slot = level;
                        next.push(to);
                    }
                }
            }
            frontier = next;
        }
        gate.check(1000)?;
        Ok(Built::Table(GroupTable {
            tracked: *tracked,
            step,
            dist,
        }))
    }

    /// The coordinate of `pos`.
    pub(super) fn index(&self, pos: &Pos) -> u32 {
        let mut bytes = [0u8; 4];
        for (i, &(orbit, piece)) in self.tracked.iter().enumerate() {
            bytes[i] = match orbit {
                Orbit::Corner => {
                    let s = (0..8)
                        .find(|&s| pos.cp[s] == piece)
                        .expect("a cube has every corner");
                    (s * 3 + usize::from(pos.co[s])) as u8
                }
                Orbit::Edge => {
                    let s = (0..12)
                        .find(|&s| pos.ep[s] == piece)
                        .expect("a cube has every edge");
                    (s * 2 + usize::from(pos.eo[s])) as u8
                }
            };
        }
        u32::from_le_bytes(bytes)
    }

    #[inline]
    pub(super) fn step(&self, at: u32, m: usize) -> u32 {
        advance(&self.step, at, m)
    }

    #[inline]
    pub(super) fn bound(&self, at: u32) -> u8 {
        self.dist[flat(at)]
    }
}

fn unflat(f: u32) -> u32 {
    let f = f as usize;
    u32::from_le_bytes([
        (f % DIGITS) as u8,
        (f / DIGITS % DIGITS) as u8,
        (f / (DIGITS * DIGITS) % DIGITS) as u8,
        (f / (DIGITS * DIGITS * DIGITS)) as u8,
    ])
}

#[inline]
fn advance(step: &[[[u8; DIGITS]; TURNS]; 4], at: u32, m: usize) -> u32 {
    let b = at.to_le_bytes();
    u32::from_le_bytes([
        step[0][m][usize::from(b[0])],
        step[1][m][usize::from(b[1])],
        step[2][m][usize::from(b[2])],
        step[3][m][usize::from(b[3])],
    ])
}

/// The abstract goal of a group, precomputed per view so that a placement is
/// judged with a few table reads.
struct Acceptance {
    /// Per tracked cubie and digit: the views that do not reject it there.
    ok: [[u32; DIGITS]; 4],
    /// Per view: the corner slots and edge slots a tracked cubie must occupy
    /// because no untracked cubie fits there.
    must_corner: Vec<u8>,
    must_edge: Vec<u16>,
}

impl Acceptance {
    fn new(goal: &Goal, tracked: &[Tracked; 4]) -> Acceptance {
        assert!(goal.views.len() <= 32, "a goal has at most 32 views here");
        let corner_mask: u32 = tracked
            .iter()
            .filter(|t| t.0 == Orbit::Corner)
            .fold(0, |m, t| m | 7 << (usize::from(t.1) * 3));
        let edge_mask: u32 = tracked
            .iter()
            .filter(|t| t.0 == Orbit::Edge)
            .fold(0, |m, t| m | 3 << (usize::from(t.1) * 2));
        let constraint = |view: &View, orbit: Orbit, slot: usize| -> Option<u32> {
            let list = match orbit {
                Orbit::Corner => &view.corners,
                Orbit::Edge => &view.edges,
            };
            list.iter()
                .find(|&&(s, _)| usize::from(s) == slot)
                .map(|&(_, bits)| bits)
        };
        let mut ok = [[0u32; DIGITS]; 4];
        for (i, &(orbit, piece)) in tracked.iter().enumerate() {
            let width = if orbit == Orbit::Corner { 3 } else { 2 };
            for (d, cell) in ok[i].iter_mut().enumerate() {
                let (slot, o) = (d / width, d % width);
                for (v, view) in goal.views.iter().enumerate() {
                    let fits = constraint(view, orbit, slot)
                        .is_none_or(|bits| bits >> (usize::from(piece) * width + o) & 1 == 1);
                    if fits {
                        *cell |= 1 << v;
                    }
                }
            }
        }
        let mut must_corner = Vec::new();
        let mut must_edge = Vec::new();
        for view in &goal.views {
            let mut mc = 0u8;
            for &(slot, bits) in &view.corners {
                if bits & !corner_mask == 0 {
                    mc |= 1 << slot;
                }
            }
            let mut me = 0u16;
            for &(slot, bits) in &view.edges {
                if bits & !edge_mask == 0 {
                    me |= 1 << slot;
                }
            }
            must_corner.push(mc);
            must_edge.push(me);
        }
        Acceptance {
            ok,
            must_corner,
            must_edge,
        }
    }

    /// Whether the placement `digit` is an abstract goal of some view.
    fn met(&self, tracked: &[Tracked; 4], digit: [usize; 4]) -> bool {
        let mut views = u32::MAX;
        let (mut corners, mut edges) = (0u8, 0u16);
        for i in 0..4 {
            views &= self.ok[i][digit[i]];
            match tracked[i].0 {
                Orbit::Corner => corners |= 1 << (digit[i] / 3),
                Orbit::Edge => edges |= 1 << (digit[i] / 2),
            }
        }
        (0..self.must_corner.len()).any(|v| {
            views >> v & 1 == 1
                && self.must_corner[v] & !corners == 0
                && self.must_edge[v] & !edges == 0
        })
    }
}
