//! The counts of the source sheets, read from the text of a move sequence.
//!
//! The sheets of an imported table state three counts per line (block turns,
//! face turns, quarter turns). Their convention differs from ours in two
//! ways, found by testing it against a whole collection (about 99.9 % of the
//! claims agree):
//!
//! - a prefixed word is one turn, whatever it expands to: `M`, `N`, `S`, `V`
//!   and `W` are one block turn (we count the layers they move) and a whole
//!   cube rotation (`C`) counts 0;
//! - a face turn is a turn of an outer block (a plain face letter, or a tier
//!   `T`), worth 1; an inner block is worth 2, as in ours. A quarter turn
//!   count is the face turn count, doubled for a half turn.
//!
//! Our own counts do not change; this reader only lets the importer compare a
//! claim in the convention it was written in.

/// Block, face and quarter turns of a move text as the sheets count them, or
/// `None` when the text is not a sequence of SSE words, groups and repeats.
pub fn sheet_counts(text: &str) -> Option<[usize; 3]> {
    let chars: Vec<char> = text.chars().collect();
    let mut at = 0;
    let counts = sequence(&chars, &mut at, 0)?;
    (at == chars.len()).then_some(counts)
}

fn sequence(chars: &[char], at: &mut usize, depth: usize) -> Option<[usize; 3]> {
    let mut total = [0usize; 3];
    while let Some(&c) = chars.get(*at) {
        match c {
            c if c.is_whitespace() || c == '\u{b7}' => *at += 1,
            '(' => {
                *at += 1;
                let inner = sequence(chars, at, depth + 1)?;
                if chars.get(*at) != Some(&')') {
                    return None;
                }
                *at += 1;
                let mut times = 0usize;
                let mut digits = false;
                while let Some(d) = chars.get(*at).and_then(|c| c.to_digit(10)) {
                    times = times.checked_mul(10)?.checked_add(d as usize)?;
                    digits = true;
                    *at += 1;
                }
                if matches!(chars.get(*at), Some('-' | '\'')) {
                    *at += 1;
                }
                let times = if digits { times } else { 1 };
                for (sum, part) in total.iter_mut().zip(inner) {
                    *sum += part.checked_mul(times)?;
                }
            }
            ')' => {
                return (depth > 0).then_some(total);
            }
            _ => {
                let word = word(chars, at)?;
                for (sum, part) in total.iter_mut().zip(word) {
                    *sum += part;
                }
            }
        }
    }
    (depth == 0).then_some(total)
}

/// One word: prefix letter, layer numbers, face letter, turn.
fn word(chars: &[char], at: &mut usize) -> Option<[usize; 3]> {
    let prefix = match chars.get(*at) {
        Some(&p) if "TNVMSWC".contains(p) => {
            *at += 1;
            Some(p)
        }
        _ => None,
    };
    let mut numbered = false;
    while chars.get(*at).is_some_and(|c| c.is_ascii_digit()) {
        numbered = true;
        *at += 1;
    }
    if numbered
        && chars.get(*at) == Some(&'-')
        && chars.get(*at + 1).is_some_and(|c| c.is_ascii_digit())
    {
        *at += 1;
        while chars.get(*at).is_some_and(|c| c.is_ascii_digit()) {
            *at += 1;
        }
    }
    if !chars.get(*at).is_some_and(|c| "URFDLB".contains(*c)) || (numbered && prefix.is_none()) {
        return None;
    }
    *at += 1;
    let half = chars.get(*at) == Some(&'2');
    if half {
        *at += 1;
    }
    if matches!(chars.get(*at), Some('-' | '\'')) {
        *at += 1;
    }
    Some(match prefix {
        Some('C') => [0, 0, 0],
        None | Some('T') => [1, 1, if half { 2 } else { 1 }],
        Some(_) => [1, 2, if half { 4 } else { 2 }],
    })
}

#[cfg(test)]
mod tests {
    use super::sheet_counts;

    #[test]
    fn plain_turns_count_as_in_every_metric() {
        assert_eq!(sheet_counts("R U R' U'"), Some([4, 4, 4]));
        assert_eq!(sheet_counts("R2 U- F2-"), Some([3, 3, 5]));
    }

    #[test]
    fn a_prefixed_word_is_one_block_turn_and_a_rotation_none() {
        assert_eq!(sheet_counts("MR2 MD"), Some([2, 4, 6]));
        assert_eq!(sheet_counts("CD2 R"), Some([1, 1, 1]));
        assert_eq!(sheet_counts("T3R2 NU V3F S2L N2-4D"), Some([5, 9, 10]));
        assert_eq!(sheet_counts("R2MD L"), Some([3, 4, 5]));
    }

    #[test]
    fn groups_repeat_and_invert() {
        assert_eq!(sheet_counts("(R U)2"), Some([4, 4, 4]));
        assert_eq!(sheet_counts("(R2 MU)- \u{b7} L"), Some([3, 4, 5]));
        assert_eq!(sheet_counts("((R U)2 MU)2"), Some([10, 12, 12]));
    }

    #[test]
    fn what_is_not_a_sequence_gives_none() {
        for text in [
            "Q",
            "2R",
            "(R U",
            "R U)",
            "R 3",
            "(R U)99999999999999999999",
        ] {
            assert_eq!(sheet_counts(text), None, "{text}");
        }
    }
}
