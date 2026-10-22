//! A library of pure 3-cycles, one set per orbit of pieces.
//!
//! A base commutator found offline cycles three pieces of one orbit and
//! leaves every other sticker alone. Conjugating it by a short setup `S`
//! (`S base S'`) cycles any three pieces the setup can reach, so the library
//! maps each 3-cycle of slots to the shortest move sequence found.

use super::model::{group, invert, parse, Model, MoveId, MOVES, STICKERS};

/// Base commutators per orbit. Each is verified to be a pure 3-cycle by the
/// tests; their inverses are added when the library is built.
const X_BASES: [&str; 1] = ["U 2R 2D 2R' U' 2R 2D' 2R'"];
const T_BASES: [&str; 4] = [
    "2B2 2D2 2B2 B2 L2 B2 2B2 2D2 2B2 B2 L2 B2",
    "2L2 2B2 2L2 L2 D2 L2 2L2 2B2 2L2 L2 D2 L2",
    "B2 2D2 B2 2B2 L2 2B2 B2 2D2 B2 2B2 L2 2B2",
    "L2 2B2 L2 2L2 D2 2L2 L2 2B2 L2 2L2 D2 2L2",
];
const WING_BASES: [&str; 1] = ["U R 2D R' U' R 2D' R'"];

/// Moves `from[k]` to `from[k + 1]`, the last to the first.
#[derive(Debug, Clone)]
pub(super) struct Cycle {
    pub slots: [u8; 3],
    pub moves: Vec<MoveId>,
}

#[derive(Debug, Default)]
pub(super) struct Library {
    pub x: Vec<Cycle>,
    pub t: Vec<Cycle>,
    pub wings: Vec<Cycle>,
}

/// How deep the conjugating setups go.
const SETUP_DEPTH: usize = 4;

impl Library {
    pub(super) fn build(model: &Model) -> Library {
        let wing_firsts: Vec<usize> = model.wings.iter().map(|w| w[0]).collect();
        let mut builds = [
            Build::new(model, &X_BASES, &model.x),
            Build::new(model, &T_BASES, &model.t),
            Build::new(model, &WING_BASES, &wing_firsts),
        ];
        let mut setup = Vec::new();
        visit(&mut setup, &mut |setup| {
            for build in &mut builds {
                build.see(model, setup);
            }
        });
        let [x, t, wings] = builds.map(Build::finish);
        Library { x, t, wings }
    }
}

type Permutation = [u8; STICKERS];

/// The sticker permutation of a sequence: sticker `i` ends at `result[i]`.
fn permutation(model: &Model, ids: &[MoveId]) -> Permutation {
    let mut perm: Permutation = std::array::from_fn(|i| i as u8);
    for &id in ids {
        let table = model.table(id);
        for slot in perm.iter_mut() {
            *slot = table[usize::from(*slot)];
        }
    }
    perm
}

const NONE: u8 = u8::MAX;

/// Collects the shortest sequence per slot cycle of one orbit.
struct Build {
    slot_of: [u8; STICKERS],
    slots: usize,
    /// The base commutators and their inverses.
    sequences: Vec<Vec<MoveId>>,
    /// For each base, the three stickers it cycles, in cycle order.
    supports: Vec<[usize; 3]>,
    best: Vec<Option<(usize, usize, Vec<MoveId>)>>,
}

impl Build {
    fn new(model: &Model, bases: &[&str], firsts: &[usize]) -> Build {
        let mut slot_of = [NONE; STICKERS];
        for (slot, &sticker) in firsts.iter().enumerate() {
            slot_of[sticker] = slot as u8;
        }
        let mut sequences: Vec<Vec<MoveId>> = Vec::new();
        for text in bases {
            let base = parse(text);
            sequences.push(invert(&base));
            sequences.push(base);
        }
        let supports = sequences
            .iter()
            .map(|sequence| {
                let perm = permutation(model, sequence);
                let moved: Vec<usize> = (0..STICKERS)
                    .filter(|&s| slot_of[s] != NONE && usize::from(perm[s]) != s)
                    .collect();
                assert_eq!(moved.len(), 3, "bases are pure 3-cycles of one orbit");
                let second = usize::from(perm[moved[0]]);
                [moved[0], second, usize::from(perm[second])]
            })
            .collect();
        let slots = firsts.len();
        Build {
            slot_of,
            slots,
            sequences,
            supports,
            best: vec![None; slots * slots * slots],
        }
    }

    /// Records what `setup`, then each base, then the setup undone, cycles.
    fn see(&mut self, model: &Model, setup: &[MoveId]) {
        let n = self.slots;
        for (index, support) in self.supports.iter().enumerate() {
            // The sticker that the setup carries onto `s` started at `before(s)`.
            let before = |s: usize| {
                setup.iter().rev().fold(s, |at, &id| {
                    usize::from(model.table(super::model::inverse(id))[at])
                })
            };
            let mapped = support.map(|s| self.slot_of[before(s)]);
            let start = (0..3).min_by_key(|&k| mapped[k]).expect("three slots");
            let key = (usize::from(mapped[start]) * n + usize::from(mapped[(start + 1) % 3])) * n
                + usize::from(mapped[(start + 2) % 3]);
            let length = 2 * setup.len() + self.sequences[index].len();
            if self.best[key].as_ref().is_none_or(|old| old.0 > length) {
                self.best[key] = Some((length, index, setup.to_vec()));
            }
        }
    }

    fn finish(self) -> Vec<Cycle> {
        let n = self.slots;
        let mut cycles: Vec<Cycle> = self
            .best
            .into_iter()
            .enumerate()
            .filter_map(|(key, entry)| {
                let (_, index, setup) = entry?;
                let mut moves = setup.clone();
                moves.extend(&self.sequences[index]);
                moves.extend(invert(&setup));
                let slots = [(key / (n * n)) as u8, (key / n % n) as u8, (key % n) as u8];
                Some(Cycle { slots, moves })
            })
            .collect();
        cycles.sort_by_key(|c| (c.moves.len(), c.slots));
        cycles
    }
}

/// Calls `f` with every setup up to [`SETUP_DEPTH`] moves; consecutive moves
/// never share a layer.
fn visit(setup: &mut Vec<MoveId>, f: &mut impl FnMut(&[MoveId])) {
    f(setup);
    if setup.len() == SETUP_DEPTH {
        return;
    }
    for id in 0..MOVES as MoveId {
        if setup.last().is_some_and(|&last| group(last) == group(id)) {
            continue;
        }
        setup.push(id);
        visit(setup, f);
        setup.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::Colours;
    use super::*;

    /// Replays every cycle on distinct sticker labels: exactly the three
    /// pieces move, and each lands where the cycle says.
    fn check(model: &Model, cycles: &[Cycle], slots: &[Vec<usize>]) {
        for cycle in cycles {
            let mut colours: Colours = std::array::from_fn(|i| i as u8);
            let before = colours;
            model.apply_all(&mut colours, &cycle.moves);
            let [a, b, c] = cycle.slots.map(|s| &slots[usize::from(s)]);
            let moved = (0..STICKERS).filter(|&i| colours[i] != before[i]).count();
            assert_eq!(moved, 3 * a.len(), "{cycle:?}");
            for k in 0..a.len() {
                assert_eq!(colours[b[k]], before[a[k]], "{cycle:?}");
                assert_eq!(colours[c[k]], before[b[k]], "{cycle:?}");
                assert_eq!(colours[a[k]], before[c[k]], "{cycle:?}");
            }
        }
    }

    #[test]
    fn every_cycle_is_a_pure_3_cycle_and_nearly_all_are_covered() {
        let model = Model::new();
        let library = Library::build(&model);
        // 24 * 23 * 22 / 3 = 4048 directed 3-cycles per orbit.
        assert!(library.x.len() >= 4000, "x {}", library.x.len());
        assert!(library.t.len() >= 3000, "t {}", library.t.len());
        assert!(library.wings.len() >= 4000, "wings {}", library.wings.len());
        let single = |stickers: &[usize]| stickers.iter().map(|&s| vec![s]).collect::<Vec<_>>();
        check(&model, &library.x, &single(&model.x));
        check(&model, &library.t, &single(&model.t));
        let wings: Vec<Vec<usize>> = model.wings.iter().map(|w| w.to_vec()).collect();
        check(&model, &library.wings, &wings);
    }
}
