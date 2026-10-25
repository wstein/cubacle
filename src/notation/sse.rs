//! Reading SSE (Superset ENG), the notation of Werner Randelshofer's cube
//! pages, from the specification text only.
//!
//! A token is a prefix letter, optional numbers, a face letter and a turn:
//! `T3R` (tier), `NL`, `N4R` (numbered layer), `N2-4R` (numbered range), `V3R`
//! (void), `M2R` (mid layers), `WR` (all layers between the faces), `SR`,
//! `S2R`, `S2-4R` (opposing slices) and `CR` (cube rotation). The turn is
//! nothing, `2`, `2'`, `'` or `-`. A bare face letter with `-` (`R-`) is `R'`.
//!
//! Every token expands to plain [`Move`]s at read time, so nothing downstream
//! knows SSE. A word is SSE when, after the prefix and its numbers, a face
//! letter follows; `M`, `M2`, `S'`, `S2` have none and stay WCA slices.
//! Which twists exist depends on the cube size (the six pages differ), see
//! [`Token::moves`]. Writing is in [`super::sse_write`].

use crate::puzzle::{Face, Move, Turn};

use super::range::{back, layers};
use super::sign::MovesError;

const PREFIXES: &str = "TNVMSWC";

/// The plain moves of an SSE word, `None` if the word is not SSE so another
/// reader may take it. `size` is the cube size when known; without it the
/// twists that depend on the size (`M W S C`) are refused.
pub(crate) fn expand(word: &str, size: Option<usize>) -> Option<Result<Vec<Move>, MovesError>> {
    let fail = |reason| MovesError {
        token: word.to_owned(),
        reason,
    };
    let mut chars = word.chars();
    if let (Some(face), Some('-'), None) = (
        chars.next().and_then(Face::from_letter),
        chars.next(),
        chars.next(),
    ) {
        return Some(Ok(vec![Move::new(face, 1, false, Turn::CounterClockwise)]));
    }
    let token = match scan(word)? {
        Ok(token) => token,
        Err(reason) => return Some(Err(fail(reason))),
    };
    Some(token.moves(size).map_err(fail))
}

struct Token {
    prefix: char,
    first: Option<u8>,
    last: Option<u8>,
    face: Face,
    turn: Turn,
}

fn split_digits(text: &str) -> (&str, &str) {
    let digits = text.chars().take_while(char::is_ascii_digit).count();
    text.split_at(digits)
}

fn scan(word: &str) -> Option<Result<Token, &'static str>> {
    let mut chars = word.chars();
    let prefix = chars.next().filter(|c| PREFIXES.contains(*c))?;
    let (first, rest) = split_digits(chars.as_str());
    let (last, rest) = match rest.strip_prefix('-') {
        Some(after) if !first.is_empty() && after.starts_with(|c: char| c.is_ascii_digit()) => {
            let (last, rest) = split_digits(after);
            (last, rest)
        }
        _ => ("", rest),
    };
    let mut chars = rest.chars();
    let Some(face) = chars.next().and_then(Face::from_letter) else {
        return (!last.is_empty()).then_some(Err("a range needs a face letter, as in N2-4R"));
    };
    let turn = match chars.as_str() {
        "" => Turn::Clockwise,
        "2" | "2'" => Turn::Half,
        "'" | "-" => Turn::CounterClockwise,
        _ => return Some(Err("not a turn: use nothing, 2, 2', ' or -")),
    };
    let number = |digits: &str| -> Result<Option<u8>, &'static str> {
        if digits.is_empty() {
            return Ok(None);
        }
        digits
            .parse()
            .map(Some)
            .map_err(|_| "a layer number is too large")
    };
    Some(match (number(first), number(last)) {
        (Ok(first), Ok(last)) => Ok(Token {
            prefix,
            first,
            last,
            face,
            turn,
        }),
        (Err(reason), _) | (_, Err(reason)) => Err(reason),
    })
}

fn opposite(face: Face) -> Face {
    Face::ALL[(face.index() + 3) % 6]
}

impl Token {
    /// The token as plain moves on a cube of `size` (`None`: not known).
    ///
    /// Per size, as on the pages: every size has face twists and `C`; `T`
    /// needs a layer left behind (so not on the 2x2); `M` and `S` start at
    /// the 3x3, `W` at the 4x4, `N`, `V` and ranges at the 5x5 (on smaller
    /// cubes the layers they could name are `M`, `W` or `T`).
    fn moves(&self, size: Option<usize>) -> Result<Vec<Move>, &'static str> {
        let (face, turn) = (self.face, self.turn);
        if self.last.is_some() && !matches!(self.prefix, 'N' | 'S') {
            return Err("only N and S take a range like 2-4");
        }
        let need = || size.ok_or("this twist depends on the cube size; give the cube");
        match self.prefix {
            'T' => {
                let depth = self.first.unwrap_or(2);
                if depth == 0 || size.is_some_and(|n| usize::from(depth) >= n) {
                    return Err(
                        "a tier must be at least 1 and leave a layer; use C to turn the whole cube",
                    );
                }
                Ok(layers(face, 1, depth, turn))
            }
            'N' => {
                if size.is_some_and(|n| n < 5) {
                    return Err("N twists exist from the 5x5 on; use M, W or T on smaller cubes");
                }
                let first = self.first.unwrap_or(2);
                block(face, first, self.last.unwrap_or(first), turn, size)
            }
            'V' => {
                if size.is_some_and(|n| n < 5) {
                    return Err("V twists exist from the 5x5 on");
                }
                let depth = self.first.unwrap_or(2);
                if depth == 0 || size.is_some_and(|n| usize::from(depth) + 2 > n) {
                    return Err("a void twist must be at least 1 and stop before the last layer");
                }
                block(face, 2, depth + 1, turn, size)
            }
            'M' => {
                let n = need()?;
                let count = self.first.unwrap_or(1);
                if n < 3 || count == 0 || usize::from(count) + 2 > n {
                    return Err("M turns 1 to n-2 layers in the middle; a 2x2 has none");
                }
                // The extra layer of an uneven split sits towards the face.
                let from = ((n - usize::from(count)) / 2 + 1) as u8;
                block(face, from, from + count - 1, turn, size)
            }
            'W' => {
                let n = need()?;
                if self.first.is_some() {
                    return Err("W takes no number: it turns all layers between the faces");
                }
                if n < 4 {
                    return Err("W exists from the 4x4 on; on a 3x3 use M");
                }
                block(face, 2, (n - 1) as u8, turn, size)
            }
            'S' => self.slices(need()?),
            _ => {
                let n = need()?;
                if self.first.is_some() {
                    return Err("C takes no number");
                }
                let wide = (n - 1) as u8;
                Ok(vec![
                    Move::new(face, wide, wide >= 2, turn),
                    Move::new(opposite(face), 1, false, back(turn)),
                ])
            }
        }
    }

    /// `S`, `S2`, `S2-4`: layers `a..=b` counted from the face and from the
    /// opposite face, all turned the same way round the axis. Where the two
    /// blocks meet they are one block (the middle layer turns once).
    fn slices(&self, n: usize) -> Result<Vec<Move>, &'static str> {
        let (from, to) = match self.last {
            Some(last) => (self.first.unwrap_or(1), last),
            None => (1, self.first.unwrap_or(1)),
        };
        if from == 0 || from > to || usize::from(to) > n {
            return Err("a slice runs from the outer layer inwards, as in S2-4R");
        }
        let mut turned = vec![false; n + 1];
        for position in usize::from(from)..=usize::from(to) {
            turned[position] = true;
            turned[n + 1 - position] = true;
        }
        let mut moves = Vec::new();
        let mut position = 1;
        while position <= n {
            if !turned[position] {
                position += 1;
                continue;
            }
            let start = position;
            while position <= n && turned[position] {
                position += 1;
            }
            // Layers counted from the face; a run reaching the last layer is
            // written from the other face by `block`.
            moves.extend(block(
                self.face,
                start as u8,
                (position - 1) as u8,
                self.turn,
                Some(n),
            )?);
        }
        Ok(moves)
    }
}

/// Layers `from..=to` counted from `face`. A block that reaches the last
/// layer is written from the opposite face, turning the other way round.
fn block(
    face: Face,
    from: u8,
    to: u8,
    turn: Turn,
    size: Option<usize>,
) -> Result<Vec<Move>, &'static str> {
    if from == 0 || from > to {
        return Err("a range runs from the outer layer inwards, as in N2-4R");
    }
    let Some(n) = size else {
        return Ok(layers(face, from, to, turn));
    };
    let n = u8::try_from(n).map_err(|_| "cube too large")?;
    if to > n {
        return Err("the layers reach beyond the cube");
    }
    if from == 1 && to == n {
        return Err("that turns every layer; use C to turn the whole cube");
    }
    if to == n {
        return Ok(layers(opposite(face), 1, n + 1 - from, back(turn)));
    }
    Ok(layers(face, from, to, turn))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::MoveSequence;

    fn show(word: &str, size: Option<usize>) -> String {
        match expand(word, size) {
            None => "not sse".to_owned(),
            Some(Ok(moves)) => MoveSequence::new(moves).to_string(),
            Some(Err(error)) => format!("error: {}", error.reason),
        }
    }

    #[test]
    fn tokens_expand_to_plain_moves() {
        let seven = Some(7);
        assert_eq!(show("TR", seven), "Rw");
        assert_eq!(show("T3R'", seven), "3Rw'");
        assert_eq!(show("NL", seven), "2L");
        assert_eq!(show("N4R2", seven), "4R2");
        assert_eq!(show("N2-4R", seven), "4Rw R'");
        assert_eq!(show("V3R", seven), "4Rw R'");
        assert_eq!(show("MR", seven), "4R");
        assert_eq!(show("WR", seven), "6Rw R'");
        assert_eq!(show("CR", seven), "6Rw L'");
        assert_eq!(show("R-", seven), "R'");
        assert_eq!(show("TR-", seven), "Rw'");
        assert_eq!(show("SR", seven), "R L'");
        // A half turn may be written 2' as it is in WCA; it is still a half turn.
        assert_eq!(show("TR2'", seven), "Rw2");
        assert_eq!(show("N3L2'", seven), "3L2");
    }

    #[test]
    fn words_without_a_face_letter_are_not_sse() {
        for word in ["M", "M2", "S'", "S2", "x", "Rw", "2R", "N", "N2", "T3"] {
            assert_eq!(show(word, Some(5)), "not sse", "{word}");
        }
    }

    #[test]
    fn a_block_to_the_last_layer_is_a_tier_of_the_other_face() {
        assert_eq!(show("N4-7R", Some(7)), "4Lw'");
        assert_eq!(show("N7R", Some(7)), "L'");
        assert_eq!(
            show("N3-8R", Some(7)),
            "error: the layers reach beyond the cube"
        );
    }
}
