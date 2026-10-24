//! Moves as written: one unit per word, so that `M`, `x` and `2-3Rw` keep
//! the single turn they stand for (the expression reader lowers them to
//! several layer moves, which would count more).

use super::unit::{size, Unit};
use crate::puzzle::{Face, PuzzleKind, Turn};

const GROUP_CHARS: &str = "()[]*,:";

/// The units of a flat sequence of WCA or SiGN words (with `M E S x y z` and
/// layer ranges), `None` when the text uses groups, repeats, commutators or
/// SSE words, or a word does not fit the cube. Words are separated by
/// whitespace or `·`.
pub(crate) fn units_of_text(text: &str, kind: PuzzleKind) -> Option<Vec<Unit>> {
    if text.contains(|c| GROUP_CHARS.contains(c)) {
        return None;
    }
    text.split(|c: char| c.is_whitespace() || c == '·')
        .filter(|word| !word.is_empty())
        .map(|word| unit_of_word(word, kind))
        .collect()
}

fn split_digits(text: &str) -> (&str, &str) {
    let digits = text.chars().take_while(char::is_ascii_digit).count();
    text.split_at(digits)
}

/// The face, whether the letter says "wide" (`r`, `Rw`) and the turn of
/// `R2`, `Rw'`, `r` from the text after the digits.
fn face_wide_turn(text: &str) -> Option<(Face, bool, Turn)> {
    let mut chars = text.chars();
    let letter = chars.next()?;
    let face = Face::from_letter(letter.to_ascii_uppercase())?;
    let tail = chars.as_str();
    let (wide, tail) = match tail.strip_prefix('w') {
        Some(_) if letter.is_ascii_lowercase() => return None,
        Some(stripped) => (true, stripped),
        None => (letter.is_ascii_lowercase(), tail),
    };
    Some((face, wide, Turn::from_suffix(tail)?))
}

fn unit_of_word(word: &str, kind: PuzzleKind) -> Option<Unit> {
    let n = size(kind);
    let (first, rest) = split_digits(word);
    if let Some(after) = rest.strip_prefix('-') {
        let (second, tail) = split_digits(after);
        let (from, to) = (first.parse::<u8>().ok()?, second.parse::<u8>().ok()?);
        let (face, wide, turn) = face_wide_turn(tail)?;
        let valid = wide && from >= 1 && from <= to && to < n;
        return valid.then(|| Unit::block(kind, face, from, to, turn));
    }
    let letter = rest.chars().next()?;
    if first.is_empty() && "MESxyz".contains(letter) {
        let half = Turn::from_suffix(&rest[1..])? == Turn::Half;
        return match letter {
            'x' => Some(Unit::rotation(kind, 1, half)),
            'y' => Some(Unit::rotation(kind, 0, half)),
            'z' => Some(Unit::rotation(kind, 2, half)),
            _ if n % 2 == 0 => None,
            _ => {
                let middle = n / 2 + 1;
                let axis = match letter {
                    'M' => 1,
                    'E' => 0,
                    _ => 2,
                };
                Some(Unit {
                    axis,
                    lo: middle,
                    hi: middle,
                    half,
                })
            }
        };
    }
    let (face, wide, turn) = face_wide_turn(rest)?;
    let depth = match (first.is_empty(), wide) {
        (false, _) => first.parse::<u8>().ok()?,
        (true, true) => 2,
        (true, false) => 1,
    };
    if depth == 0 || depth >= n || (wide && depth < 2) {
        return None;
    }
    let from = if wide { 1 } else { depth };
    Some(Unit::block(kind, face, from, depth, turn))
}
