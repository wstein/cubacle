//! Moves written together without a space: `R'U'`, `MF2R`, `NBL`.
//!
//! A word that does not parse as one move is cut at the places where a move
//! must end, and every piece has to parse on its own, otherwise the word is
//! refused as before. The places are lexical, never a guess:
//!
//! 1. after a `'` that is followed by a letter or a digit (`R'U`, `R'2U`);
//! 2. after a `2` that follows a face letter and is followed by an SSE prefix
//!    letter (`F2NR` is `F2 NR`). `F2R` is not cut: it could be `F2 R` or
//!    `F 2R` (a layer move), so it stays refused, as does `R22`;
//! 3. after the face letter of an SSE move (a prefix letter `T N V M S W C`,
//!    optional numbers, a face letter) that is followed by an upper-case
//!    letter (`NBL` is `NB L`, `TRU` is `TR U`).
//!
//! A word that already parses is never cut, so no input that was accepted
//! changes. `-` never ends a move here, because it also spells a range.

const PREFIXES: &str = "TNVMSWC";
const FACES: &str = "URFDLB";

/// The byte offsets and texts of the pieces of `word`, `None` if no place
/// qualifies.
pub(super) fn pieces(word: &str) -> Option<Vec<(usize, &str)>> {
    let bytes = word.as_bytes();
    let mut cuts = Vec::new();
    for i in 1..bytes.len() {
        let (before, next) = (bytes[i - 1] as char, bytes[i] as char);
        let after_quote = before == '\'' && (next.is_ascii_alphabetic() || next.is_ascii_digit());
        let after_half = before == '2'
            && i >= 2
            && FACES.contains(bytes[i - 2] as char)
            && PREFIXES.contains(next);
        let after_sse_face =
            FACES.contains(before) && next.is_ascii_uppercase() && sse_prefix_before(&word[..i]);
        if after_quote || after_half || after_sse_face {
            cuts.push(i);
        }
    }
    if cuts.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    let mut from = 0;
    for cut in cuts.into_iter().chain([word.len()]) {
        out.push((from, &word[from..cut]));
        from = cut;
    }
    Some(out)
}

/// Whether `piece` (up to and including a face letter) is an SSE move head:
/// a prefix letter, numbers or a range, and the face letter.
fn sse_prefix_before(head: &str) -> bool {
    // The longest suffix that begins with a prefix letter and has only
    // digits and `-` between that letter and the face letter.
    let chars: Vec<char> = head.chars().collect();
    let Some((&face, middle)) = chars.split_last() else {
        return false;
    };
    if !FACES.contains(face) {
        return false;
    }
    let digits = middle
        .iter()
        .rev()
        .take_while(|c| c.is_ascii_digit() || **c == '-')
        .count();
    middle
        .len()
        .checked_sub(digits + 1)
        .is_some_and(|at| PREFIXES.contains(middle[at]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cut(word: &str) -> Vec<&str> {
        pieces(word).map_or_else(|| vec![word], |p| p.into_iter().map(|(_, t)| t).collect())
    }

    #[test]
    fn cuts_follow_the_three_rules() {
        assert_eq!(cut("R'U'"), ["R'", "U'"]);
        assert_eq!(cut("R'2U"), ["R'", "2U"]);
        assert_eq!(cut("MF2NR'"), ["MF2", "NR'"]);
        assert_eq!(cut("NBL"), ["NB", "L"]);
        assert_eq!(cut("N2-4RU"), ["N2-4R", "U"]);
        assert_eq!(cut("TL'NR"), ["TL'", "NR"]);
    }

    #[test]
    fn nothing_else_is_cut() {
        for word in [
            "R22", "MF2R'", "F2R", "R2", "RU", "Rw", "TL2'", "N2-4R", "R-U", "2R'",
        ] {
            assert_eq!(cut(word), [word], "{word}");
        }
    }
}
