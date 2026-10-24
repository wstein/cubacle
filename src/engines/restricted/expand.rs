//! Solving with a big alphabet by rewriting the answer of the default engine.
//!
//! An alphabet that makes every face turn of the face-turn metric within a
//! few of its own letters (the quarter turns are the example: a half turn is
//! two of them) cannot be searched in reasonable time, because the shortest
//! solutions have twenty letters and more. Such a cube is solved by the
//! default engine and every face turn is replaced by its shortest word in the
//! alphabet. The result is a solution in the alphabet, not a shortest one, and
//! the solver says so.

use std::collections::hash_map::Entry;
use std::collections::HashMap;

use super::letters::{identity, then, Letters, Perm};
use crate::puzzle::{Face, Move, MoveSequence, Turn};

/// The longest word that stands for a face turn.
const LONGEST_WORD: usize = 3;

/// The 18 face turns in the order face (U R F D L B) times turn (cw, half, ccw).
pub(super) fn face_turns() -> Vec<Move> {
    let mut all = Vec::new();
    for face in Face::ALL {
        for turn in [Turn::Clockwise, Turn::Half, Turn::CounterClockwise] {
            all.push(Move::new(face, 1, false, turn));
        }
    }
    all
}

/// The word (letter numbers) for each of the 18 face turns, when the alphabet
/// has one of at most [`LONGEST_WORD`] letters for each.
pub(super) fn words(letters: &Letters) -> Option<Vec<Vec<usize>>> {
    let mut known: HashMap<Perm, Vec<usize>> = HashMap::new();
    known.insert(identity(), Vec::new());
    let mut layer: Vec<(Perm, Vec<usize>)> = vec![(identity(), Vec::new())];
    for _ in 0..LONGEST_WORD {
        let mut next = Vec::new();
        for (perm, word) in &layer {
            for (index, letter) in letters.letters.iter().enumerate() {
                let product = then(perm, &letter.src);
                if let Entry::Vacant(slot) = known.entry(product) {
                    let mut longer = word.clone();
                    longer.push(index);
                    slot.insert(longer.clone());
                    next.push((product, longer));
                }
            }
        }
        layer = next;
    }
    face_turns()
        .iter()
        .map(|mv| known.get(&super::letters::perm_of(mv)).cloned())
        .collect()
}

/// `solution` (face turns) written in the letters, adjacent letters that make
/// one letter or nothing merged.
pub(super) fn rewrite(
    letters: &Letters,
    words: &[Vec<usize>],
    solution: &MoveSequence,
) -> Option<Vec<usize>> {
    let faces = face_turns();
    let mut out: Vec<usize> = Vec::new();
    for mv in solution.moves() {
        let at = faces.iter().position(|f| f == mv)?;
        for &letter in &words[at] {
            out.push(letter);
            merge(letters, &mut out);
        }
    }
    Some(out)
}

/// Replaces the last two letters by their one letter, or removes them, while
/// that is possible.
fn merge(letters: &Letters, out: &mut Vec<usize>) {
    while out.len() >= 2 {
        let (a, b) = (out[out.len() - 2], out[out.len() - 1]);
        let product = then(&letters.letters[a].src, &letters.letters[b].src);
        if product == identity() {
            out.truncate(out.len() - 2);
        } else if let Some(c) = letters.letters.iter().position(|l| l.src == product) {
            out.truncate(out.len() - 2);
            out.push(c);
        } else {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::parse_alphabet;
    use crate::puzzle::PuzzleKind;

    fn letters(set: &str) -> Letters {
        Letters::new(&parse_alphabet(set, PuzzleKind::Cube3).unwrap()).unwrap()
    }

    #[test]
    fn the_quarter_turns_make_every_face_turn() {
        let l = letters("qtm");
        let w = words(&l).expect("every face turn is a word of quarter turns");
        assert_eq!(w.len(), 18);
        // A half turn is two quarter turns, a quarter turn is one letter.
        assert_eq!(w[0].len(), 1);
        assert_eq!(w[1].len(), 2);
    }

    #[test]
    fn two_generators_are_no_dense_alphabet() {
        assert!(words(&letters("<R,U>")).is_none());
        assert!(words(&letters("half-turns")).is_none());
    }

    #[test]
    fn merging_cancels_and_joins() {
        let l = letters("R R2 R' U");
        let mut out = vec![0, 0];
        merge(&l, &mut out);
        assert_eq!(out, vec![1], "R R is R2");
        let mut out = vec![0, 2];
        merge(&l, &mut out);
        assert!(out.is_empty(), "R R' is nothing");
        let mut out = vec![0, 3];
        merge(&l, &mut out);
        assert_eq!(out, vec![0, 3]);
    }
}
