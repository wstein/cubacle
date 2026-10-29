//! An optimal 2x2x2 solver.
//!
//! One corner (DBL) is kept fixed, so only the faces U, R and F ever turn.
//! A state is then the arrangement of the other seven corners (7! = 5040)
//! together with the twists of six of them (3^6 = 729; the seventh follows),
//! 3,674,160 states in all. A breadth-first search from the solved state fills
//! a table of exact distances (4 bits each, two states per byte), and a solution is read off by walking downhill.
//! The result is optimal in the half-turn metric restricted to U, R and F,
//! which equals the optimal length over all six faces.
//!
//! A cube whose DBL piece is not in the DBL slot is first rotated as a whole
//! until it is. The solution then uses the faces that the rotation moved into
//! the U, R and F positions, so that it solves the cube as it was given.

pub mod fixed;
mod partial;

use std::sync::OnceLock;
use std::time::Instant;

use crate::notation::orbit64::{CORNER_COLORS, CORNER_FACELETS};
use crate::puzzle::{
    Face, Facelets, Failure, Move, MoveSequence, PuzzleKind, SolveEvidence, SolveFrame,
    SolveMetrics, SolveOptions, SolveResult, Solver, Target, Turn,
};

const PERMS: usize = 5040;
const TWISTS: usize = 729;
const STATES: usize = PERMS * TWISTS;
/// The nibble of a state the search has not reached; real depths are at most 11.
const UNSEEN: u8 = 0xF;
/// The corner that never moves: DBL, in the Kociemba corner order.
const FIXED: usize = 6;
/// The slots whose occupants are permuted, in coordinate order.
const FREE_SLOTS: [usize; 7] = [0, 1, 2, 3, 4, 5, 7];
const SLOT_NAMES: [&str; 8] = ["URF", "UFL", "ULB", "UBR", "DFR", "DLF", "DBL", "DRB"];

/// The optimal 2x2x2 engine behind the [`Solver`] facade.
#[derive(Debug, Clone, Copy, Default)]
pub struct PocketSolver;

impl PocketSolver {
    pub fn new() -> PocketSolver {
        PocketSolver
    }
}

/// Corners by slot: which piece sits there and how it is twisted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cubie {
    piece: [u8; 8],
    twist: [u8; 8],
}

const SOLVED: Cubie = Cubie {
    piece: [0, 1, 2, 3, 4, 5, 6, 7],
    twist: [0; 8],
};

/// One clockwise quarter turn per face, in `URFDLB` order.
const FACE_TURNS: [Cubie; 6] = [
    Cubie {
        piece: [3, 0, 1, 2, 4, 5, 6, 7],
        twist: [0; 8],
    },
    Cubie {
        piece: [4, 1, 2, 0, 7, 5, 6, 3],
        twist: [2, 0, 0, 1, 1, 0, 0, 2],
    },
    Cubie {
        piece: [1, 5, 2, 3, 0, 4, 6, 7],
        twist: [1, 2, 0, 0, 2, 1, 0, 0],
    },
    Cubie {
        piece: [0, 1, 2, 3, 5, 6, 7, 4],
        twist: [0; 8],
    },
    Cubie {
        piece: [0, 2, 6, 3, 4, 1, 5, 7],
        twist: [0, 1, 2, 0, 0, 2, 1, 0],
    },
    Cubie {
        piece: [0, 1, 3, 7, 4, 5, 2, 6],
        twist: [0, 0, 1, 2, 0, 0, 2, 1],
    },
];

impl Cubie {
    /// The state after performing `mv` on this one.
    fn then(&self, mv: &Cubie) -> Cubie {
        let mut out = *self;
        for slot in 0..8 {
            let from = usize::from(mv.piece[slot]);
            out.piece[slot] = self.piece[from];
            out.twist[slot] = (self.twist[from] + mv.twist[slot]) % 3;
        }
        out
    }

    fn turned(&self, face: usize, turn: Turn) -> Cubie {
        let times = match turn {
            Turn::Clockwise => 1,
            Turn::Half => 2,
            Turn::CounterClockwise => 3,
        };
        (0..times).fold(*self, |cube, _| cube.then(&FACE_TURNS[face]))
    }
}

/// The nine moves of the search: U, R, F, each clockwise, half and counter-clockwise.
fn search_move(index: usize) -> (usize, Turn) {
    let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise][index % 3];
    (index / 3, turn)
}

fn perm_rank(cube: &Cubie) -> usize {
    let values: Vec<usize> = FREE_SLOTS
        .iter()
        .map(|&slot| match usize::from(cube.piece[slot]) {
            7 => FIXED,
            piece => piece,
        })
        .collect();
    let mut rank = 0;
    for i in 0..7 {
        let smaller = values[i + 1..].iter().filter(|&&v| v < values[i]).count();
        rank = rank * (7 - i) + smaller;
    }
    rank
}

fn perm_unrank(mut rank: usize) -> Cubie {
    let mut digits = [0usize; 7];
    for i in (0..7).rev() {
        let base = 7 - i;
        digits[i] = rank % base;
        rank /= base;
    }
    let mut pool: Vec<usize> = (0..7).collect();
    let mut cube = SOLVED;
    for (i, &slot) in FREE_SLOTS.iter().enumerate() {
        let value = pool.remove(digits[i]);
        cube.piece[slot] = if value == FIXED { 7 } else { value as u8 };
    }
    cube
}

fn twist_rank(cube: &Cubie) -> usize {
    cube.twist[..6]
        .iter()
        .fold(0, |rank, &t| rank * 3 + usize::from(t))
}

fn twist_unrank(mut rank: usize) -> Cubie {
    let mut cube = SOLVED;
    for slot in (0..6).rev() {
        cube.twist[slot] = (rank % 3) as u8;
        rank /= 3;
    }
    let sum: u8 = cube.twist[..6].iter().sum();
    cube.twist[7] = (3 - sum % 3) % 3;
    cube
}

struct Tables {
    perm_move: Vec<[u16; 9]>,
    twist_move: Vec<[u16; 9]>,
    /// Distances to the solved state, two states per byte (low nibble first).
    dist: Box<[u8]>,
}

impl Tables {
    fn build() -> Tables {
        let mut perm_move = vec![[0u16; 9]; PERMS];
        for (rank, row) in perm_move.iter_mut().enumerate() {
            let cube = perm_unrank(rank);
            for (m, entry) in row.iter_mut().enumerate() {
                let (face, turn) = search_move(m);
                *entry = perm_rank(&cube.turned(face, turn)) as u16;
            }
        }
        let mut twist_move = vec![[0u16; 9]; TWISTS];
        for (rank, row) in twist_move.iter_mut().enumerate() {
            let cube = twist_unrank(rank);
            for (m, entry) in row.iter_mut().enumerate() {
                let (face, turn) = search_move(m);
                *entry = twist_rank(&cube.turned(face, turn)) as u16;
            }
        }

        let mut dist = vec![0xFF_u8; STATES / 2].into_boxed_slice();
        set_depth(&mut dist, 0, 0);
        let mut frontier = vec![0u32];
        let mut depth = 0u8;
        while !frontier.is_empty() {
            depth += 1;
            let mut next = Vec::new();
            for &state in &frontier {
                let (perm, twist) = (state as usize / TWISTS, state as usize % TWISTS);
                for m in 0..9 {
                    let to = u32::from(perm_move[perm][m]) * TWISTS as u32
                        + u32::from(twist_move[twist][m]);
                    if get_depth(&dist, to) == UNSEEN {
                        set_depth(&mut dist, to, depth);
                        next.push(to);
                    }
                }
            }
            frontier = next;
        }
        Tables {
            perm_move,
            twist_move,
            dist,
        }
    }

    fn step(&self, state: u32, m: usize) -> u32 {
        let (perm, twist) = (state as usize / TWISTS, state as usize % TWISTS);
        u32::from(self.perm_move[perm][m]) * TWISTS as u32 + u32::from(self.twist_move[twist][m])
    }

    fn depth(&self, state: u32) -> u8 {
        get_depth(&self.dist, state)
    }

    /// Moves along strictly decreasing distance until the cube is solved.
    fn walk(&self, mut state: u32) -> Vec<(usize, Turn)> {
        let mut moves = Vec::new();
        while self.depth(state) > 0 {
            let want = self.depth(state) - 1;
            let m = (0..9)
                .find(|&m| self.depth(self.step(state, m)) == want)
                .expect("every state but the solved one has a neighbor one closer");
            moves.push(search_move(m));
            state = self.step(state, m);
        }
        moves
    }
}

fn get_depth(table: &[u8], state: u32) -> u8 {
    (table[state as usize >> 1] >> ((state & 1) * 4)) & 0xF
}

fn set_depth(table: &mut [u8], state: u32, depth: u8) {
    let shift = (state & 1) * 4;
    let byte = &mut table[state as usize >> 1];
    *byte = (*byte & !(0xF << shift)) | (depth << shift);
}

/// Why a solve missed its time budget, if it did. Only the one-off table
/// build can: a lookup in a ready table takes microseconds and is treated as
/// meeting every budget.
fn budget_verdict(
    built_table: bool,
    elapsed: std::time::Duration,
    budget: Option<std::time::Duration>,
) -> Option<&'static str> {
    match budget {
        Some(budget) if built_table && elapsed > budget => {
            Some("building the distance table took longer than the time budget")
        }
        _ => None,
    }
}

static TABLES: OnceLock<Tables> = OnceLock::new();

fn tables() -> &'static Tables {
    TABLES.get_or_init(Tables::build)
}

/// A whole-cube rotation, built from the generators x, y and z.
struct Rotation {
    /// Generators applied in order: 0 is x (R L'), 1 is y (U D'), 2 is z (F B').
    word: Vec<u8>,
    /// `origin[p]` is the position whose content the rotation brings to
    /// position `p`, in `URFDLB` order.
    origin: [usize; 6],
}

fn rotations() -> &'static [Rotation] {
    static ROTATIONS: OnceLock<Vec<Rotation>> = OnceLock::new();
    ROTATIONS.get_or_init(|| {
        // Faces in URFDLB order: U=0 R=1 F=2 D=3 L=4 B=5.
        const GENERATORS: [[usize; 6]; 3] = [
            [2, 1, 3, 5, 4, 0], // x: U<-F, F<-D, D<-B, B<-U
            [0, 5, 1, 3, 2, 4], // y: F<-R, L<-F, B<-L, R<-B
            [4, 0, 2, 1, 3, 5], // z: R<-U, D<-R, L<-D, U<-L
        ];
        let mut found = vec![Rotation {
            word: Vec::new(),
            origin: [0, 1, 2, 3, 4, 5],
        }];
        let mut next = 0;
        while next < found.len() {
            for (g, generator) in GENERATORS.iter().enumerate() {
                // The content at p after this generator came from generator[p]
                // before it, and from there the existing rotation traces it back.
                let origin = found[next].origin;
                let composed: [usize; 6] = std::array::from_fn(|p| origin[generator[p]]);
                if found.iter().all(|r| r.origin != composed) {
                    let mut word = found[next].word.clone();
                    word.push(g as u8);
                    found.push(Rotation {
                        word,
                        origin: composed,
                    });
                }
            }
            next += 1;
        }
        found
    })
}

impl Rotation {
    fn apply(&self, cube: &Cubie) -> Cubie {
        self.word.iter().fold(*cube, |cube, &g| {
            let (cw, ccw) = [(1, 4), (0, 3), (2, 5)][usize::from(g)];
            cube.turned(cw, Turn::Clockwise)
                .turned(ccw, Turn::CounterClockwise)
        })
    }
}

/// The 2x2 facelet index of a 3x3 corner facelet (thinning keeps the corners).
fn thinned(facelet: usize) -> usize {
    let (face, row, col) = (facelet / 9, facelet % 9 / 3, facelet % 3);
    face * 4 + row / 2 * 2 + col / 2
}

fn color_letters(colors: &[usize]) -> String {
    colors.iter().map(|&c| Face::ALL[c].letter()).collect()
}

/// Reads the corners off a 2x2 facelet string.
fn read_corners(facelets: &Facelets) -> Result<Cubie, String> {
    let stickers = facelets.stickers();
    let mut cube = SOLVED;
    let mut seen = [false; 8];
    for slot in 0..8 {
        let colors = CORNER_FACELETS[slot].map(|f| stickers[thinned(f)].index());
        let twist = colors
            .iter()
            .position(|&c| c == 0 || c == 3)
            .ok_or_else(|| format!("corner {} has no U or D colored sticker", SLOT_NAMES[slot]))?;
        let ordered = [
            colors[twist],
            colors[(twist + 1) % 3],
            colors[(twist + 2) % 3],
        ];
        let piece = CORNER_COLORS
            .iter()
            .position(|c| *c == ordered)
            .ok_or_else(|| {
                format!(
                    "the stickers {} at corner {} are not a corner of the cube",
                    color_letters(&colors),
                    SLOT_NAMES[slot]
                )
            })?;
        if std::mem::replace(&mut seen[piece], true) {
            return Err(format!(
                "the corner with colors {} appears twice",
                color_letters(&CORNER_COLORS[piece])
            ));
        }
        cube.piece[slot] = piece as u8;
        cube.twist[slot] = twist as u8;
    }
    if cube.twist.iter().map(|&t| u32::from(t)).sum::<u32>() % 3 != 0 {
        return Err("the corner twists do not sum to 0 mod 3, so no turns reach this state".into());
    }
    Ok(cube)
}

/// A cube rotated so that the fixed corner is home: its table index, and the
/// face each position of the rotated cube held originally.
struct Normalized {
    state: u32,
    origin: [usize; 6],
}

fn normalize(facelets: &Facelets) -> Result<Normalized, String> {
    let cube = read_corners(facelets)?;
    let rotation = rotations()
        .iter()
        .find(|r| {
            let turned = r.apply(&cube);
            turned.piece[FIXED] == FIXED as u8 && turned.twist[FIXED] == 0
        })
        .expect("the 24 rotations bring any corner to any slot and orientation");
    let turned = rotation.apply(&cube);
    Ok(Normalized {
        state: (perm_rank(&turned) * TWISTS + twist_rank(&turned)) as u32,
        origin: rotation.origin,
    })
}

impl Solver for PocketSolver {
    fn kind(&self) -> PuzzleKind {
        PuzzleKind::Cube2
    }

    fn prepare(&self) {
        tables();
        rotations();
    }

    fn solve_to(&self, start: &Facelets, target: &Target, options: &SolveOptions) -> SolveResult {
        partial::solve_to(self, start, target, options)
    }

    fn solve(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        let started = Instant::now();
        let finish = |outcome: Result<MoveSequence, Failure>, message: Option<String>| {
            let evidence = outcome.as_ref().map_or_else(
                |_| SolveEvidence::default(),
                |path| SolveEvidence::optimal(SolveFrame::AnyOrientation, path.len()),
            );
            SolveResult {
                outcome,
                message,
                metrics: SolveMetrics {
                    elapsed: started.elapsed(),
                    probes: None,
                    evidence,
                },
                stages: Vec::new(),
            }
        };

        if facelets.kind() != PuzzleKind::Cube2 {
            let size = facelets.kind().size();
            return finish(
                Err(Failure::InvalidInput),
                Some(format!("expected a 2x2x2, got a {size}x{size}x{size}")),
            );
        }
        if let Err(reason) = crate::notation::check(facelets) {
            return finish(Err(Failure::InvalidInput), Some(reason.to_string()));
        }
        let normalized = match normalize(facelets) {
            Ok(normalized) => normalized,
            Err(message) => return finish(Err(Failure::InvalidInput), Some(message)),
        };

        // The table is built once per process, by whichever solve comes first.
        // That build cannot be interrupted, so a budget is only judged against
        // it; once the table exists a lookup is instantaneous and always meets
        // any budget. `prepare()` moves the build out of the first solve.
        let built_here = TABLES.get().is_none();
        let tables = tables();
        if let Some(message) = budget_verdict(built_here, started.elapsed(), options.time_budget) {
            return finish(Err(Failure::BudgetExhausted), Some(message.into()));
        }

        let length = usize::from(tables.depth(normalized.state));
        if options
            .max_moves
            .is_some_and(|max| length > usize::from(max))
        {
            return finish(
                Err(Failure::Exhausted),
                Some(format!(
                    "the shortest solution has {length} moves, over the limit"
                )),
            );
        }
        let solution = tables
            .walk(normalized.state)
            .into_iter()
            .map(|(face, turn)| Move::new(Face::ALL[normalized.origin[face]], 1, false, turn))
            .collect();
        finish(Ok(solution), None)
    }
}

#[cfg(test)]
mod tests;
