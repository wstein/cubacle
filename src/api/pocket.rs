//! An optimal 2x2x2 solver.
//!
//! One corner (DBL) is kept fixed, so only the faces U, R and F ever turn.
//! A state is then the arrangement of the other seven corners (7! = 5040)
//! together with the twists of six of them (3^6 = 729; the seventh follows),
//! 3,674,160 states in all. A breadth-first search from the solved state fills
//! a table of exact distances, and a solution is read off by walking downhill.
//! The result is optimal in the half-turn metric restricted to U, R and F,
//! which equals the optimal length over all six faces.
//!
//! A cube whose DBL piece is not in the DBL slot is first rotated as a whole
//! until it is. The solution then uses the faces that the rotation moved into
//! the U, R and F positions, so that it solves the cube as it was given.

use std::sync::OnceLock;
use std::time::Instant;

use super::orbit64::{CORNER_COLOURS, CORNER_FACELETS};
use super::{
    Face, Facelets, Failure, Move, MoveSequence, PuzzleKind, SolveMetrics, SolveOptions,
    SolveResult, Solver, Turn,
};

const PERMS: usize = 5040;
const TWISTS: usize = 729;
const STATES: usize = PERMS * TWISTS;
const UNSEEN: u8 = u8::MAX;
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
    dist: Vec<u8>,
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

        let mut dist = vec![UNSEEN; STATES];
        dist[0] = 0;
        let mut frontier = vec![0u32];
        let mut depth = 0u8;
        while !frontier.is_empty() {
            depth += 1;
            let mut next = Vec::new();
            for &state in &frontier {
                let (perm, twist) = (state as usize / TWISTS, state as usize % TWISTS);
                for m in 0..9 {
                    let to = usize::from(perm_move[perm][m]) * TWISTS
                        + usize::from(twist_move[twist][m]);
                    if dist[to] == UNSEEN {
                        dist[to] = depth;
                        next.push(to as u32);
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

    fn step(&self, state: usize, m: usize) -> usize {
        usize::from(self.perm_move[state / TWISTS][m]) * TWISTS
            + usize::from(self.twist_move[state % TWISTS][m])
    }

    /// Moves along strictly decreasing distance until the cube is solved.
    fn walk(&self, mut state: usize) -> Vec<(usize, Turn)> {
        let mut moves = Vec::new();
        while self.dist[state] > 0 {
            let want = self.dist[state] - 1;
            let m = (0..9)
                .find(|&m| self.dist[self.step(state, m)] == want)
                .expect("every state but the solved one has a neighbour one closer");
            moves.push(search_move(m));
            state = self.step(state, m);
        }
        moves
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

fn colour_letters(colours: &[usize]) -> String {
    colours.iter().map(|&c| Face::ALL[c].letter()).collect()
}

/// Reads the corners off a 2x2 facelet string.
fn read_corners(facelets: &Facelets) -> Result<Cubie, String> {
    let stickers = facelets.stickers();
    let mut cube = SOLVED;
    let mut seen = [false; 8];
    for slot in 0..8 {
        let colours = CORNER_FACELETS[slot].map(|f| stickers[thinned(f)].index());
        let twist = colours
            .iter()
            .position(|&c| c == 0 || c == 3)
            .ok_or_else(|| format!("corner {} has no U or D coloured sticker", SLOT_NAMES[slot]))?;
        let ordered = [
            colours[twist],
            colours[(twist + 1) % 3],
            colours[(twist + 2) % 3],
        ];
        let piece = CORNER_COLOURS
            .iter()
            .position(|c| *c == ordered)
            .ok_or_else(|| {
                format!(
                    "the stickers {} at corner {} are not a corner of the cube",
                    colour_letters(&colours),
                    SLOT_NAMES[slot]
                )
            })?;
        if std::mem::replace(&mut seen[piece], true) {
            return Err(format!(
                "the corner with colours {} appears twice",
                colour_letters(&CORNER_COLOURS[piece])
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
struct Normalised {
    state: usize,
    origin: [usize; 6],
}

fn normalise(facelets: &Facelets) -> Result<Normalised, String> {
    let cube = read_corners(facelets)?;
    let rotation = rotations()
        .iter()
        .find(|r| {
            let turned = r.apply(&cube);
            turned.piece[FIXED] == FIXED as u8 && turned.twist[FIXED] == 0
        })
        .expect("the 24 rotations bring any corner to any slot and orientation");
    let turned = rotation.apply(&cube);
    Ok(Normalised {
        state: perm_rank(&turned) * TWISTS + twist_rank(&turned),
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

    fn solve(&self, facelets: &Facelets, options: &SolveOptions) -> SolveResult {
        let started = Instant::now();
        let finish =
            |outcome: Result<MoveSequence, Failure>, message: Option<String>| SolveResult {
                outcome,
                message,
                metrics: SolveMetrics {
                    elapsed: started.elapsed(),
                    probes: None,
                },
            };

        if facelets.kind() != PuzzleKind::Cube2 {
            let size = facelets.kind().size();
            return finish(
                Err(Failure::InvalidInput),
                Some(format!("expected a 2x2x2, got a {size}x{size}x{size}")),
            );
        }
        let normalised = match normalise(facelets) {
            Ok(normalised) => normalised,
            Err(message) => return finish(Err(Failure::InvalidInput), Some(message)),
        };

        // The table is built once per process. A budget cannot interrupt that
        // build, but it is honoured as soon as the build returns.
        let tables = tables();
        if options.time_budget.is_some_and(|b| started.elapsed() > b) {
            return finish(
                Err(Failure::BudgetExhausted),
                Some("the time budget ran out building the distance table".into()),
            );
        }

        let length = usize::from(tables.dist[normalised.state]);
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
            .walk(normalised.state)
            .into_iter()
            .map(|(face, turn)| Move::new(Face::ALL[normalised.origin[face]], 1, false, turn))
            .collect();
        finish(Ok(solution), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_moves, random_moves};
    use std::collections::HashMap;
    use std::time::Duration;

    type Perms = HashMap<String, [usize; 24]>;

    /// Per-move sticker permutations for the 2x2, derived from the 3x3 engine
    /// alone: `perm[dest] = source`, for all six faces and three turns.
    fn sticker_perms() -> Perms {
        let solved: Vec<char> = from_moves("").unwrap().chars().collect();
        let mut perms = HashMap::new();
        for face in Face::ALL {
            for suffix in ["", "2", "'"] {
                let name = format!("{}{suffix}", face.letter());
                let moved: Vec<char> = from_moves(&name).unwrap().chars().collect();
                let mut perm: [usize; 24] = std::array::from_fn(|i| i);
                for dest_slot in CORNER_FACELETS {
                    let mut colours: Vec<char> = dest_slot.iter().map(|&f| moved[f]).collect();
                    colours.sort();
                    let home = CORNER_FACELETS
                        .iter()
                        .find(|home| {
                            let mut c: Vec<char> = home.iter().map(|&f| solved[f]).collect();
                            c.sort();
                            c == colours
                        })
                        .unwrap();
                    for &dest in &dest_slot {
                        let src = home.iter().find(|&&f| solved[f] == moved[dest]).unwrap();
                        perm[thinned(dest)] = thinned(*src);
                    }
                }
                perms.insert(name, perm);
            }
        }
        perms
    }

    fn apply(perms: &Perms, state: &str, mv: &str) -> String {
        let chars: Vec<char> = state.chars().collect();
        perms[mv].iter().map(|&src| chars[src]).collect()
    }

    fn thin(full: &str) -> String {
        let chars: Vec<char> = full.chars().collect();
        let mut out = vec![' '; 24];
        for f in CORNER_FACELETS.iter().flatten() {
            out[thinned(*f)] = chars[*f];
        }
        out.into_iter().collect()
    }

    fn solved_2x2() -> String {
        thin(&from_moves("").unwrap())
    }

    fn all_moves() -> Vec<String> {
        let mut moves = Vec::new();
        for face in Face::ALL {
            for suffix in ["", "2", "'"] {
                moves.push(format!("{}{suffix}", face.letter()));
            }
        }
        moves
    }

    fn is_solved(state: &str) -> bool {
        state
            .as_bytes()
            .chunks(4)
            .all(|face| face.iter().all(|&c| c == face[0]))
    }

    fn replay(perms: &Perms, state: &str, solution: &MoveSequence) -> String {
        solution
            .moves()
            .iter()
            .fold(state.to_owned(), |s, mv| apply(perms, &s, &mv.to_string()))
    }

    #[test]
    fn distance_table_has_the_known_depth_distribution() {
        let started = Instant::now();
        let tables = tables();
        eprintln!("table ready after {:?}", started.elapsed());
        let mut counts = [0usize; 12];
        for &d in &tables.dist {
            assert_ne!(d, UNSEEN);
            counts[usize::from(d)] += 1;
        }
        assert_eq!(
            counts,
            [1, 9, 54, 321, 1847, 9992, 50136, 227536, 870072, 1887748, 623800, 2644]
        );
        assert_eq!(counts.iter().sum::<usize>(), 3_674_160);
    }

    #[test]
    fn the_cubie_model_agrees_with_the_3x3_engine_on_every_move() {
        let perms = sticker_perms();
        let solved = solved_2x2();
        for face in Face::ALL {
            for (suffix, turn) in [
                ("", Turn::Clockwise),
                ("2", Turn::Half),
                ("'", Turn::CounterClockwise),
            ] {
                let name = format!("{}{suffix}", face.letter());
                let moved = Facelets::parse(&apply(&perms, &solved, &name)).unwrap();
                assert_eq!(
                    read_corners(&moved).unwrap(),
                    SOLVED.turned(face.index(), turn),
                    "{name}"
                );
            }
        }
    }

    #[test]
    fn rotations_are_the_24_whole_cube_rotations() {
        assert_eq!(rotations().len(), 24);
    }

    #[test]
    fn random_states_replay_to_solved_with_table_length() {
        let perms = sticker_perms();
        let solver = PocketSolver::new();
        for _ in 0..2500 {
            let state = thin(&from_moves(&random_moves(30)).unwrap());
            let facelets = Facelets::parse(&state).unwrap();
            let result = solver.solve(&facelets, &SolveOptions::default());
            let solution = result.solution().expect("solvable");
            assert!(is_solved(&replay(&perms, &state, solution)), "{state}");
            let normalised = normalise(&facelets).unwrap();
            assert_eq!(
                solution.len(),
                usize::from(tables().dist[normalised.state]),
                "{state}"
            );
            assert!(solution.len() <= 11);
        }
    }

    #[test]
    fn near_states_are_solved_optimally_in_every_orientation() {
        // A multi-source search over facelet strings with all 18 moves, from
        // all 24 orientations of the solved cube, gives true distances that do
        // not depend on the table or the state model.
        let perms = sticker_perms();
        let moves = all_moves();
        let rotate = |s: &str, a: &str, b: &str| apply(&perms, &apply(&perms, s, a), b);
        let mut depth: HashMap<String, usize> = HashMap::new();
        let mut frontier = vec![solved_2x2()];
        depth.insert(solved_2x2(), 0);
        let mut i = 0;
        while i < frontier.len() {
            let s = frontier[i].clone();
            for next in [
                rotate(&s, "R", "L'"),
                rotate(&s, "U", "D'"),
                rotate(&s, "F", "B'"),
            ] {
                if !depth.contains_key(&next) {
                    depth.insert(next.clone(), 0);
                    frontier.push(next);
                }
            }
            i += 1;
        }
        assert_eq!(frontier.len(), 24);
        for d in 1..=4 {
            let mut next_frontier = Vec::new();
            for s in &frontier {
                for m in &moves {
                    let n = apply(&perms, s, m);
                    if !depth.contains_key(&n) {
                        depth.insert(n.clone(), d);
                        next_frontier.push(n);
                    }
                }
            }
            frontier = next_frontier;
        }
        assert!(depth.len() > 20_000);
        let solver = PocketSolver::new();
        for (state, &d) in &depth {
            let facelets = Facelets::parse(state).unwrap();
            let solution = solver
                .solve(&facelets, &SolveOptions::default())
                .outcome
                .unwrap();
            assert_eq!(solution.len(), d, "{state}");
            assert!(is_solved(&replay(&perms, state, &solution)), "{state}");
        }
    }

    #[test]
    fn move_limit_and_budget_are_honoured() {
        let perms = sticker_perms();
        let solver = PocketSolver::new();
        let state = apply(&perms, &apply(&perms, &solved_2x2(), "R"), "U");
        let facelets = Facelets::parse(&state).unwrap();
        let tight = SolveOptions::default().with_max_moves(1);
        assert_eq!(
            solver.solve(&facelets, &tight).outcome,
            Err(Failure::Exhausted)
        );
        let enough = SolveOptions::default().with_max_moves(2);
        assert_eq!(
            solver.solve(&facelets, &enough).solution().unwrap().len(),
            2
        );
        let none = SolveOptions::default().with_time_budget(Duration::ZERO);
        assert_eq!(
            solver.solve(&facelets, &none).outcome,
            Err(Failure::BudgetExhausted)
        );
    }

    #[test]
    fn invalid_inputs_are_reported_not_panicked_on() {
        let perms = sticker_perms();
        let solver = PocketSolver::new();
        let options = SolveOptions::default();

        let three = Facelets::solved(PuzzleKind::Cube3);
        assert_eq!(
            solver.solve(&three, &options).outcome,
            Err(Failure::InvalidInput)
        );

        // One corner twisted in place.
        let solved = solved_2x2();
        let mut chars: Vec<char> = solved.chars().collect();
        let slot = CORNER_FACELETS[0].map(thinned);
        chars.swap(slot[0], slot[1]);
        chars.swap(slot[1], slot[2]);
        let twisted: String = chars.into_iter().collect();
        let result = solver.solve(&Facelets::parse(&twisted).unwrap(), &options);
        assert_eq!(result.outcome, Err(Failure::InvalidInput));
        assert!(result.message.unwrap().contains("twist"));

        // Two stickers swapped between faces: counts stay valid, pieces do not.
        let mut chars: Vec<char> = solved.chars().collect();
        chars.swap(0, 4);
        let swapped: String = chars.into_iter().collect();
        let result = solver.solve(&Facelets::parse(&swapped).unwrap(), &options);
        assert_eq!(result.outcome, Err(Failure::InvalidInput));
        assert!(result.message.is_some());

        // Sticker shuffles with valid counts must never panic, and whatever
        // is accepted must really be solved by the answer.
        let mut state = solved;
        for round in 0..300 {
            let mut chars: Vec<char> = state.chars().collect();
            chars.swap(round % 24, (round * 7 + 3) % 24);
            state = chars.into_iter().collect();
            let result = solver.solve(&Facelets::parse(&state).unwrap(), &options);
            if let Some(solution) = result.solution() {
                assert!(is_solved(&replay(&perms, &state, solution)));
            }
        }
    }
}
