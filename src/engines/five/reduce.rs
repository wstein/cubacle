//! Reduction: centres solved, wings paired with their edge.
//!
//! The wings are the 24 distinguishable pieces of the two wing orbits. A
//! sticker pair read in the slot's handedness names the piece. A wing is
//! paired when its colours are those of the middle edge beside it, so the
//! goal piece of a slot follows from the middle edge of its edge. Quarter
//! turns of an inner layer swap the parity of the wing permutation while
//! leaving the corners and middle edges alone; the 3-cycles keep it, so one
//! such turn up front is all the parity handling an odd cube needs.

use super::library::{Cycle, Library};
use super::model::{id_of, Colours, Model, MoveId};
use super::orbit;
use crate::puzzle::{Face, Move, Turn};

/// Why a cube could not be reduced.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Reduction {
    /// The state cannot exist.
    Impossible(String),
    /// A stage found no way on; not expected for any valid cube.
    Stuck(&'static str),
}

pub(super) struct Tables {
    pub model: Model,
    pub library: Library,
}

impl Tables {
    pub(super) fn build() -> Tables {
        let model = Model::new();
        let library = Library::build(&model);
        Tables { model, library }
    }
}

/// Checks everything a reduction relies on about the state.
pub(super) fn validate(tables: &Tables, colours: &Colours) -> Result<(), Reduction> {
    let model = &tables.model;
    for face in 0..6 {
        if usize::from(colours[face * 25 + 12]) != face {
            return Err(Reduction::Impossible(
                "the centres have moved; the face centres must show their own colour".to_owned(),
            ));
        }
    }
    for (name, slots) in [("x-centre", &model.x), ("t-centre", &model.t)] {
        let mut counts = [0usize; 6];
        for &slot in slots {
            counts[usize::from(colours[slot])] += 1;
        }
        if counts != [4; 6] {
            return Err(Reduction::Impossible(format!(
                "each colour must appear four times among the {name}s"
            )));
        }
    }
    wing_pieces(model, colours)?;
    wing_goal(model, colours).map(|_| ())
}

/// The piece in every wing slot, by the slot it belongs to.
fn wing_pieces(model: &Model, colours: &Colours) -> Result<Vec<u8>, Reduction> {
    let mut home = [[u8::MAX; 6]; 6];
    for (slot, wing) in model.wings.iter().enumerate() {
        home[wing[0] / 25][wing[1] / 25] = slot as u8;
    }
    let mut seen = vec![false; model.wings.len()];
    let mut pieces = Vec::with_capacity(model.wings.len());
    for wing in &model.wings {
        let piece = home[usize::from(colours[wing[0]])][usize::from(colours[wing[1]])];
        if piece == u8::MAX || std::mem::replace(&mut seen[usize::from(piece)], true) {
            return Err(Reduction::Impossible(
                "the wing pieces are not the 24 pieces of a 5x5x5".to_owned(),
            ));
        }
        pieces.push(piece);
    }
    Ok(pieces)
}

/// The piece that pairs with the middle edge beside every wing slot.
fn wing_goal(model: &Model, colours: &Colours) -> Result<Vec<u8>, Reduction> {
    let mut home = [[u8::MAX; 6]; 6];
    for (slot, wing) in model.wings.iter().enumerate() {
        home[wing[0] / 25][wing[1] / 25] = slot as u8;
    }
    let mut seen = vec![false; model.wings.len()];
    let mut goal = Vec::with_capacity(model.wings.len());
    for beside in &model.beside {
        let beside = beside.map(|sticker| usize::from(colours[sticker]));
        let piece = home[beside[0]][beside[1]];
        if piece == u8::MAX || std::mem::replace(&mut seen[usize::from(piece)], true) {
            return Err(Reduction::Impossible(
                "the middle edges are not the 12 edges of a cube".to_owned(),
            ));
        }
        goal.push(piece);
    }
    Ok(goal)
}

/// Whether the permutation taking `pieces` to `goal` is odd.
fn is_odd(pieces: &[u8], goal: &[u8]) -> bool {
    let target = |piece: u8| goal.iter().position(|&g| g == piece).expect("a bijection");
    let permutation: Vec<usize> = pieces.iter().map(|&p| target(p)).collect();
    let mut seen = vec![false; permutation.len()];
    let mut odd = false;
    for start in 0..permutation.len() {
        let mut at = start;
        let mut length = 0;
        while !std::mem::replace(&mut seen[at], true) {
            at = permutation[at];
            length += 1;
        }
        odd ^= length % 2 == 0 && length > 0;
    }
    odd
}
/// Solves the centres and pairs the wings, applying the moves to `colours`
/// and returning them.
pub(super) fn reduce(tables: &Tables, colours: &mut Colours) -> Result<Vec<MoveId>, Reduction> {
    validate(tables, colours)?;
    let model = &tables.model;
    let mut moves = Vec::new();

    if is_odd(&wing_pieces(model, colours)?, &wing_goal(model, colours)?) {
        let turn = id_of(&Move::new(Face::R, 2, false, Turn::Clockwise)).expect("an inner turn");
        model.apply(colours, turn);
        moves.push(turn);
    }

    centres(
        model,
        &tables.library.x,
        &model.x,
        colours,
        &mut moves,
        "x-centres",
    )?;
    centres(
        model,
        &tables.library.t,
        &model.t,
        colours,
        &mut moves,
        "t-centres",
    )?;

    let mut labels = wing_pieces(model, colours)?;
    let goal = wing_goal(model, colours)?;
    let steps = orbit::solve(&tables.library.wings, &mut labels, &goal)
        .ok_or(Reduction::Stuck("the wings"))?;
    perform(model, &tables.library.wings, &steps, colours, &mut moves);
    Ok(moves)
}

fn centres(
    model: &Model,
    cycles: &[Cycle],
    slots: &[usize],
    colours: &mut Colours,
    moves: &mut Vec<MoveId>,
    name: &'static str,
) -> Result<(), Reduction> {
    let mut labels: Vec<u8> = slots.iter().map(|&s| colours[s]).collect();
    let goal: Vec<u8> = slots.iter().map(|&s| (s / 25) as u8).collect();
    let steps = orbit::solve(cycles, &mut labels, &goal).ok_or(Reduction::Stuck(name))?;
    perform(model, cycles, &steps, colours, moves);
    Ok(())
}

fn perform(
    model: &Model,
    cycles: &[Cycle],
    steps: &[usize],
    colours: &mut Colours,
    moves: &mut Vec<MoveId>,
) {
    for &step in steps {
        model.apply_all(colours, &cycles[step].moves);
        moves.extend(&cycles[step].moves);
    }
}

/// Whether the centres are solid and every wing sits in its own slot.
pub(super) fn is_reduced(tables: &Tables, colours: &Colours) -> bool {
    let model = &tables.model;
    let solid = model
        .x
        .iter()
        .chain(&model.t)
        .all(|&slot| usize::from(colours[slot]) == slot / 25);
    solid && wing_pieces(model, colours).ok() == wing_goal(model, colours).ok()
}

/// Merges turns of the same layer that only commute turns apart, and drops
/// turns that cancel. The result does the same to the cube.
pub(super) fn simplify(moves: &[MoveId]) -> Vec<MoveId> {
    // Quarter turns clockwise per move: 1, 2 or 3.
    let quarters = |id: MoveId| usize::from(id % 3) + 1;
    let axis = |id: MoveId| (id / 6) % 3;
    let mut out: Vec<MoveId> = Vec::with_capacity(moves.len());
    'next: for &id in moves {
        let mut at = out.len();
        while at > 0 && axis(out[at - 1]) == axis(id) {
            at -= 1;
            if out[at] / 3 == id / 3 {
                let total = (quarters(out[at]) + quarters(id)) % 4;
                if total == 0 {
                    out.remove(at);
                } else {
                    out[at] = id - id % 3 + (total - 1) as u8;
                }
                continue 'next;
            }
        }
        out.push(id);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::model::support::{scramble, scrambled};
    use super::super::model::{colours_of, id_of, invert, move_of, sequence};
    use super::*;
    use crate::puzzle::{Face, Facelets, PuzzleKind};

    fn tables() -> &'static Tables {
        static TABLES: std::sync::OnceLock<Tables> = std::sync::OnceLock::new();
        TABLES.get_or_init(Tables::build)
    }

    fn three_of(colours: &Colours) -> String {
        tables().model.three_by_three(colours).to_string()
    }

    #[test]
    fn the_hidden_3x3x3_of_a_solved_cube_is_solved() {
        let solved = colours_of(&Facelets::solved(PuzzleKind::Cube5));
        assert!(three_of(&solved).starts_with("UUUUUUUUURRRRRRRRR"));
        assert!(tables().model.three_by_three(&solved).is_solved());
    }

    #[test]
    fn reduction_solves_the_centres_and_pairs_the_wings() {
        for seed in 0..40 {
            let mut colours = colours_of(&scrambled(seed, 120));
            assert!(!is_reduced(tables(), &colours), "seed {seed}");
            let before = three_of(&colours);
            let moves = reduce(tables(), &mut colours).expect("reducible");
            assert!(is_reduced(tables(), &colours), "seed {seed}");
            // Only centres and wings moved: the corners and middle edges stay.
            assert_eq!(three_of(&colours), before, "seed {seed}");
            assert!(!moves.is_empty());
        }
    }

    #[test]
    fn reduction_replays_on_the_facelets() {
        let cube = scrambled(7, 80);
        let mut colours = colours_of(&cube);
        let moves = reduce(tables(), &mut colours).unwrap();
        let end = cube.apply(&sequence(&moves)).unwrap();
        assert_eq!(colours_of(&end), colours);
    }

    #[test]
    fn an_odd_wing_permutation_gets_one_inner_quarter_turn_first() {
        // 2R as the very first move is the parity repair; both parities occur.
        let repair = id_of(&"2R".parse().unwrap()).unwrap();
        let (mut odd, mut even) = (0, 0);
        for seed in 0..60 {
            let cube = scrambled(seed, 40);
            let mut colours = colours_of(&cube);
            let moves = reduce(tables(), &mut colours).expect("reducible");
            assert!(is_reduced(tables(), &colours), "seed {seed}");
            if moves.first() == Some(&repair) {
                odd += 1;
            } else {
                even += 1;
            }
        }
        assert!(odd > 5 && even > 5, "odd {odd}, even {even}");
    }

    #[test]
    fn simplify_keeps_the_effect_and_never_grows() {
        for seed in 0..30 {
            let mut moves = scramble(seed, 60);
            let back = invert(&moves[20..40]);
            moves.splice(40..40, back);
            let short = simplify(&moves);
            assert!(short.len() <= moves.len());
            let solved = Facelets::solved(PuzzleKind::Cube5);
            assert_eq!(
                solved.apply(&sequence(&moves)).unwrap(),
                solved.apply(&sequence(&short)).unwrap(),
                "seed {seed}"
            );
        }
        assert!(simplify(&[0, 2]).is_empty(), "U U' cancels");
        assert_eq!(simplify(&[0, 0]), [1], "U U is U2");
        // U and D turn the same axis, so the second U meets the first.
        let (u, d) = (0, 18);
        assert_eq!(move_of(d).face, Face::D);
        assert_eq!(simplify(&[u, d, 2]), [d]);
    }

    #[test]
    fn impossible_states_are_rejected_by_the_stage_checks() {
        let solved = colours_of(&Facelets::solved(PuzzleKind::Cube5));
        assert_eq!(validate(tables(), &solved), Ok(()));

        let mut moved_centre = solved;
        moved_centre.swap(12, 25 + 12);
        assert!(matches!(
            validate(tables(), &moved_centre),
            Err(Reduction::Impossible(_))
        ));

        // Two x-centres swapped across colours leave the counts right but a
        // third colour moved in makes them wrong.
        let mut counts = solved;
        counts[tables().model.x[0]] = 1;
        counts[tables().model.x[5]] = 0;
        assert!(matches!(
            validate(tables(), &counts),
            Err(Reduction::Impossible(_))
        ));

        // A wing showing two stickers of the same colour is no wing.
        let mut wing = solved;
        let [first, second] = tables().model.wings[0];
        wing[second] = wing[first];
        wing[tables().model.x[0]] = solved[second];
        assert!(matches!(
            validate(tables(), &wing),
            Err(Reduction::Impossible(_))
        ));
    }
}
