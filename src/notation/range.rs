//! Layer ranges: `2-3r`, `3-5r2`, `2-4l'` (SiGN) and the hybrid `2-3Lw2`.
//!
//! A range names the layers `a` to `b` counted from a face. The reader expands
//! it into plain moves, so the move type and the engines never see a range:
//! turning layers `a..=b` is turning the outer `b` layers together and then
//! the outer `a - 1` layers back, e.g. `2-3r` is `3Rw R'`
//! (the outer layer alone is the plain `R`, not a wide turn).

use crate::puzzle::{Face, Move, Turn};

use super::sign::MovesError;

/// The plain moves of a range word, `None` if the word is not written as a
/// range (it has no `digits-digits` start), so another reader may take it.
pub(crate) fn expand(word: &str) -> Option<Result<Vec<Move>, MovesError>> {
    let (from, rest) = split_number(word)?;
    let rest = rest.strip_prefix('-')?;
    let (to, rest) = split_number(rest)?;
    Some(moves(word, from, to, rest))
}

fn split_number(text: &str) -> Option<(&str, &str)> {
    let digits = text.chars().take_while(char::is_ascii_digit).count();
    (digits > 0).then(|| text.split_at(digits))
}

fn moves(word: &str, from: &str, to: &str, tail: &str) -> Result<Vec<Move>, MovesError> {
    let fail = |reason| MovesError {
        token: word.to_owned(),
        reason,
    };
    let (Ok(from), Ok(to)) = (from.parse::<u8>(), to.parse::<u8>()) else {
        return Err(fail("a layer number is too large"));
    };
    if from == 0 {
        return Err(fail("layers are counted from 1"));
    }
    if from > to {
        return Err(fail(
            "a range runs from the outer layer inwards, as in 2-3r",
        ));
    }
    let mut chars = tail.chars();
    let letter = chars.next();
    let face = letter
        .and_then(|c| Face::from_letter(c.to_ascii_uppercase()))
        .ok_or_else(|| fail("a range needs a face letter, as in 2-3r"))?;
    let after = chars.as_str();
    // SiGN writes a lowercase letter; the hybrid writes Rw.
    let suffix = if letter.is_some_and(|c| c.is_ascii_lowercase()) {
        if after.starts_with('w') {
            return Err(fail("a lowercase letter already means a wide turn"));
        }
        after
    } else {
        after
            .strip_prefix('w')
            .ok_or_else(|| fail("a range needs a lowercase letter (2-3r) or Rw (2-3Rw)"))?
    };
    let turn = Turn::from_suffix(suffix).ok_or_else(|| fail("not a turn: use nothing, 2 or '"))?;
    Ok(layers(face, from, to, turn))
}

/// The layers `from..=to` of a face turned together.
pub(crate) fn layers(face: Face, from: u8, to: u8, turn: Turn) -> Vec<Move> {
    if from == to {
        return vec![Move::new(face, from, false, turn)];
    }
    // The outer `depth` layers: a plain turn for one, a wide turn for more.
    let outer = |depth: u8, turn: Turn| Move::new(face, depth, depth >= 2, turn);
    let mut moves = vec![outer(to, turn)];
    if from > 1 {
        moves.push(outer(from - 1, back(turn)));
    }
    moves
}

pub(crate) fn back(turn: Turn) -> Turn {
    match turn {
        Turn::Clockwise => Turn::CounterClockwise,
        Turn::CounterClockwise => Turn::Clockwise,
        Turn::Half => Turn::Half,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::parse_moves;
    use crate::puzzle::{Facelets, MoveSequence, PuzzleKind};

    fn expand_text(word: &str) -> String {
        MoveSequence::new(expand(word).expect("a range").expect("valid")).to_string()
    }

    #[test]
    fn ranges_expand_into_plain_moves() {
        assert_eq!(expand_text("2-3r"), "3Rw R'");
        assert_eq!(expand_text("3-5r"), "5Rw Rw'");
        assert_eq!(expand_text("2-4l2"), "4Lw2 L2");
        assert_eq!(expand_text("1-3r"), "3Rw");
        assert_eq!(expand_text("1-2u'"), "Uw'");
        assert_eq!(expand_text("2-3Lw2"), "3Lw2 L2");
        assert_eq!(expand_text("3-4Bw'"), "4Bw' Bw");
        assert_eq!(expand_text("2-2f"), "2F");
        assert_eq!(expand_text("1-1d"), "D");
    }

    fn solved(size: usize) -> Facelets {
        Facelets::solved(PuzzleKind::of_size(size).unwrap())
    }

    /// Turning layers a..=b is the same as turning each of them alone.
    #[test]
    fn a_range_turns_exactly_its_layers_on_every_size_and_face() {
        for size in 2..=7usize {
            for face in "URFDLB".chars() {
                for turn in ["", "2", "'"] {
                    for a in 1..size {
                        for b in a..size {
                            let singles: String =
                                (a..=b).map(|k| format!("{k}{face}{turn} ")).collect();
                            let want = solved(size).apply(&parse_moves(&singles).unwrap()).unwrap();
                            let lower = face.to_ascii_lowercase();
                            for word in [
                                format!("{a}-{b}{lower}{turn}"),
                                format!("{a}-{b}{face}w{turn}"),
                            ] {
                                let moves = parse_moves(&word).unwrap();
                                assert!(moves.len() <= 2, "{word}");
                                let got = solved(size).apply(&moves).unwrap();
                                assert_eq!(got, want, "{word} on {size}");
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn a_range_and_its_inverse_cancel() {
        let both = parse_moves("2-4r 2-4r'").unwrap();
        assert!(solved(5).apply(&both).unwrap().is_solved());
    }

    #[test]
    fn bad_ranges_say_why() {
        for (word, reason) in [
            ("0-2r", "counted from 1"),
            ("3-2r", "outer layer inwards"),
            ("2-3", "face letter"),
            ("2-3x", "face letter"),
            ("2-3R", "lowercase"),
            ("2-3rw", "wide"),
            ("2-3r4", "not a turn"),
            ("2-3r''", "not a turn"),
            ("2-300r", "too large"),
            ("2-3Rw3x", "not a turn"),
        ] {
            let error = expand(word).expect("a range").unwrap_err();
            assert!(error.reason.contains(reason), "{word}: {}", error.reason);
        }
        assert!(expand("R").is_none() && expand("2R").is_none() && expand("-3r").is_none());
        assert!(expand("2-").is_none());
    }
}
