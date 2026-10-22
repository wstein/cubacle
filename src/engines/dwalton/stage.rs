//! One stage: iterative-deepening search over pattern tables.
//!
//! New code on top of the generic search of `crate::reduction::ida`. A node
//! is a mask per sticker group (bit set: the sticker shows a marked colour);
//! a move permutes the bits through byte lookup tables.

use super::geometry::MoveDef;
use super::pattern::Table;
use crate::puzzle::Face;
use crate::reduction::ida::{self, Collector, Stage, Start};

pub(super) const MAXG: usize = 12;

#[derive(Clone, Copy)]
pub(super) struct Node(pub [u32; MAXG]);

/// A set of stickers and the colours that count as marked on it.
pub(super) struct GroupSpec {
    pub cells: Vec<usize>,
    pub marks: u8,
}

/// A heuristic that is not a table, and the goal test that goes with it.
pub(super) trait Extra: Sync {
    /// 0 exactly when the extra goal conditions hold, else an estimate of at
    /// least 1.
    fn eval(&self, masks: &[u32]) -> i32;
}

pub(super) struct PatStage<'a> {
    pub groups: Vec<GroupSpec>,
    /// Names of the moves in use, by move number.
    pub names: Vec<String>,
    /// `perm[m][g]`: four byte tables of 256 entries.
    perm: Vec<Vec<Vec<u32>>>,
    pub tables: Vec<&'a Table>,
    pub extra: Option<&'a dyn Extra>,
    skips: Vec<u64>,
    /// The longest solution to look for.
    pub max_length: i32,
}

fn axis(face: Face) -> usize {
    match face {
        Face::U | Face::D => 0,
        Face::R | Face::L => 1,
        Face::F | Face::B => 2,
    }
}

/// The moves of `defs` that `allow` accepts.
pub(super) fn select(defs: &[MoveDef], allow: impl Fn(&str) -> bool) -> Vec<&MoveDef> {
    defs.iter().filter(|d| allow(&d.name)).collect()
}

impl<'a> PatStage<'a> {
    pub(super) fn new(
        groups: Vec<GroupSpec>,
        used: &[&MoveDef],
        tables: Vec<&'a Table>,
        extra: Option<&'a dyn Extra>,
    ) -> PatStage<'a> {
        assert!(used.len() < 64 && groups.len() <= MAXG);
        let perm = used
            .iter()
            .map(|d| {
                groups
                    .iter()
                    .map(|g| {
                        assert!(g.cells.len() <= 32);
                        let local: Vec<u32> = g
                            .cells
                            .iter()
                            .map(|&s| {
                                g.cells
                                    .iter()
                                    .position(|&t| t == d.dest[s])
                                    .unwrap_or_else(|| panic!("{} leaves a group", d.name))
                                    as u32
                            })
                            .collect();
                        let mut t = vec![0u32; 1024];
                        for b in 0..4 {
                            for v in 0..256usize {
                                for j in 0..8 {
                                    if (v >> j) & 1 == 1 && b * 8 + j < local.len() {
                                        t[b * 256 + v] |= 1 << local[b * 8 + j];
                                    }
                                }
                            }
                        }
                        t
                    })
                    .collect()
            })
            .collect();
        // Moves about one axis commute: try them in one order only.
        let family = |d: &MoveDef| (d.mv.face, d.mv.depth, d.mv.wide);
        let mut skips = vec![0u64; used.len() + 1];
        for (last, l) in used.iter().enumerate() {
            for (m, d) in used.iter().enumerate() {
                if axis(l.mv.face) == axis(d.mv.face) && (family(l) == family(d) || m < last) {
                    skips[last] |= 1 << m;
                }
            }
        }
        PatStage {
            groups,
            names: used.iter().map(|d| d.name.clone()).collect(),
            perm,
            tables,
            extra,
            skips,
            max_length: 100,
        }
    }

    /// The node of a cube given as colour indices per sticker.
    pub(super) fn node_of(&self, colours: impl Fn(usize) -> u8) -> Node {
        let mut node = Node([0; MAXG]);
        for (g, spec) in self.groups.iter().enumerate() {
            for (i, &s) in spec.cells.iter().enumerate() {
                node.0[g] |= u32::from((spec.marks >> colours(s)) & 1) << i;
            }
        }
        node
    }

    #[inline(always)]
    fn heuristic(&self, m: &[u32; MAXG]) -> i32 {
        let mut h = 0;
        for t in &self.tables {
            h = h.max(t.depth(m));
        }
        if let Some(e) = self.extra {
            h = h.max(e.eval(m));
        }
        h
    }
}

impl Stage for PatStage<'_> {
    type Node = Node;
    fn move_count(&self) -> usize {
        self.names.len()
    }
    fn move_code(&self, m: usize) -> u8 {
        m as u8
    }
    fn skips(&self) -> &[u64] {
        &self.skips
    }
    fn max_length(&self) -> i32 {
        self.max_length
    }
    fn next_axis(&self) -> u64 {
        0
    }
    fn min_back_depth(&self) -> i32 {
        1
    }
    fn prun(&self, node: &Node) -> i32 {
        self.heuristic(&node.0)
    }
    #[inline(always)]
    fn step(&self, node: &Node, mv: usize, _maxl: i32) -> (Node, i32) {
        let mut next = Node([0; MAXG]);
        for (g, t) in self.perm[mv].iter().enumerate() {
            let m = node.0[g];
            next.0[g] = t[(m & 255) as usize]
                | t[256 + ((m >> 8) & 255) as usize]
                | t[512 + ((m >> 16) & 255) as usize]
                | t[768 + (m >> 24) as usize];
        }
        let p = self.heuristic(&next.0);
        (next, p)
    }
}

struct Record;
impl Collector for Record {
    type Out = Vec<u8>;
    fn collect(&self, _owner: usize, moves: &[u8], out: &mut Vec<Vec<u8>>) {
        out.push(moves.to_vec());
    }
}

/// The first shortest solution found from `node`, as move names. The same
/// for every thread count.
pub(super) fn solve(stage: &PatStage, node: Node, threads: usize) -> Option<Vec<String>> {
    let start = Start {
        node,
        cost: 0,
        owner: 0,
    };
    let found = ida::search(stage, &[start], &Record, 1, threads, None).ok()?;
    let first = found.into_iter().next()?;
    Some(
        first
            .iter()
            .map(|&m| stage.names[usize::from(m)].clone())
            .collect(),
    )
}
