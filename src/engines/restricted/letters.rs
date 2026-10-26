//! The letters of the alphabet as sticker permutations, and the relations
//! between them that the search prunes with.
//!
//! The 3x3x3 is searched as 54 sticker colors. A move is the permutation
//! `src` (`new[i] = old[src[i]]`), so the engine needs no model of cubies and
//! any move the cube has (turns of faces, wide turns, slices) is a letter.
//! The relations are computed from the permutations, not assumed from the 18
//! moves of the face-turn metric: a pair of letters is *redundant* when the
//! two moves make the identity or one letter, and two letters *commute* when
//! their products agree in both orders.

use crate::puzzle::{destinations, Alphabet, Move, PuzzleKind};

/// Stickers of the 3x3x3.
pub(super) const N: usize = 54;

pub(super) type State = [u8; N];
pub(super) type Perm = [u8; N];

/// The most letters a search can tell apart (one bit each in a mask).
pub(super) const MOST_LETTERS: usize = 64;

pub(super) struct Letter {
    pub mv: Move,
    /// `new[i] = old[src[i]]`
    pub src: Perm,
    /// The permutation that undoes it.
    pub inv: Perm,
}

pub(super) struct Letters {
    pub letters: Vec<Letter>,
    /// `commute[a]`: the letters whose product with `a` is the same in both orders.
    pub commute: Vec<u64>,
    /// `before[a]`: the letters with a smaller number than `a`.
    pub before: Vec<u64>,
    /// `redundant[a]`: the letters that must not follow `a` because the pair
    /// makes the identity or one letter.
    pub redundant: Vec<u64>,
}

pub(super) fn identity() -> Perm {
    let mut perm = [0u8; N];
    for (i, slot) in perm.iter_mut().enumerate() {
        *slot = i as u8;
    }
    perm
}

pub(super) fn apply(src: &Perm, state: &State, out: &mut State) {
    for (slot, &s) in out.iter_mut().zip(src.iter()) {
        *slot = state[usize::from(s)];
    }
}

/// `first` then `second`: `new[i] = old[first[second[i]]]`.
pub(super) fn then(first: &Perm, second: &Perm) -> Perm {
    let mut out = [0u8; N];
    for (slot, &s) in out.iter_mut().zip(second.iter()) {
        *slot = first[usize::from(s)];
    }
    out
}

fn invert(perm: &Perm) -> Perm {
    let mut out = [0u8; N];
    for (i, &s) in perm.iter().enumerate() {
        out[usize::from(s)] = i as u8;
    }
    out
}

pub(super) fn perm_of(mv: &Move) -> Perm {
    let dest = destinations(PuzzleKind::Cube3, mv);
    let mut src = [0u8; N];
    for (i, &d) in dest.iter().enumerate() {
        src[d] = i as u8;
    }
    src
}

impl Letters {
    pub(super) fn new(alphabet: &Alphabet) -> Result<Letters, String> {
        if alphabet.len() > MOST_LETTERS {
            return Err(format!(
                "the move set has {} moves, at most {MOST_LETTERS} are supported",
                alphabet.len()
            ));
        }
        let letters: Vec<Letter> = alphabet
            .moves()
            .iter()
            .map(|mv| {
                let src = perm_of(mv);
                Letter {
                    mv: *mv,
                    inv: invert(&src),
                    src,
                }
            })
            .collect();
        let count = letters.len();
        let mut commute = vec![0u64; count];
        let mut before = vec![0u64; count];
        let mut redundant = vec![0u64; count];
        let none = identity();
        for a in 0..count {
            before[a] = (1u64 << a) - 1;
            for b in 0..count {
                let ab = then(&letters[a].src, &letters[b].src);
                if a != b && ab == then(&letters[b].src, &letters[a].src) {
                    commute[a] |= 1 << b;
                }
                if ab == none || letters.iter().any(|c| c.src == ab) {
                    redundant[a] |= 1 << b;
                }
            }
        }
        Ok(Letters {
            letters,
            commute,
            before,
            redundant,
        })
    }

    pub(super) fn len(&self) -> usize {
        self.letters.len()
    }

    /// The letters that may follow a path that ends in `last`, given the
    /// commuting letters `held` that must not follow (see [`Letters::step`]).
    pub(super) fn allowed(&self, last: Option<usize>, held: u64) -> u64 {
        let all = if self.len() == 64 {
            u64::MAX
        } else {
            (1u64 << self.len()) - 1
        };
        match last {
            None => all,
            Some(x) => all & !(held | self.redundant[x]),
        }
    }

    /// The commuting letters that must not follow once `x` is appended: those
    /// that commute with `x` and come before it, or were held back already and
    /// commute with `x`. It keeps one order of commuting moves.
    pub(super) fn step(&self, x: usize, held: u64) -> u64 {
        self.commute[x] & (self.before[x] | held)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::parse_alphabet;

    fn letters(text: &str) -> Letters {
        Letters::new(&parse_alphabet(text, PuzzleKind::Cube3).unwrap()).unwrap()
    }

    #[test]
    fn relations_of_the_face_turns_come_from_the_permutations() {
        let l = letters("R R2 R' L U");
        // R then R2 is R', a letter; R then R' is the identity; R then L is nothing.
        assert_ne!(l.redundant[0] & (1 << 1), 0);
        assert_ne!(l.redundant[0] & (1 << 2), 0);
        assert_eq!(l.redundant[0] & (1 << 3), 0);
        // R and L commute, R and U do not.
        assert_ne!(l.commute[0] & (1 << 3), 0);
        assert_eq!(l.commute[0] & (1 << 4), 0);
    }

    #[test]
    fn a_pair_is_only_redundant_when_the_alphabet_holds_the_result() {
        // Without R2 and R', two quarter turns are a real move.
        let l = letters("R U");
        assert_eq!(l.redundant[0] & (1 << 0), 0);
        let both = letters("R R2");
        // R R = R2 is a letter.
        assert_ne!(both.redundant[0] & 1, 0);
    }

    #[test]
    fn opposite_slices_and_faces_commute_across_the_axis() {
        let l = letters("U D 2U U2");
        // U and D, U and the slice 2U, commute.
        assert_ne!(l.commute[0] & (1 << 1), 0);
        assert_ne!(l.commute[0] & (1 << 2), 0);
    }

    #[test]
    fn every_move_of_the_3x3x3_fits_in_a_mask() {
        // 3 axes, 5 blocks of layers, 3 turns: 45 different moves at most.
        let mut all = Vec::new();
        for face in crate::puzzle::Face::ALL {
            for depth in 1..=2 {
                for wide in [false, true] {
                    for turn in [
                        crate::puzzle::Turn::Clockwise,
                        crate::puzzle::Turn::Half,
                        crate::puzzle::Turn::CounterClockwise,
                    ] {
                        all.push(Move::new(face, depth, wide, turn));
                    }
                }
            }
        }
        let alphabet = Alphabet::new(PuzzleKind::Cube3, "everything", all);
        assert!(alphabet.len() <= MOST_LETTERS);
        assert!(Letters::new(&alphabet).is_ok());
    }
}
