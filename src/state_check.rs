//! The check for impossible cube states, as the commands that read a state use it.

use cubacle::notation::check;
use cubacle::puzzle::{Face, Facelets, PuzzleKind};

/// `Ok` for a state that turns can reach; otherwise the error text: the case
/// named, and a `hint:` line where the repair is certain.
pub(crate) fn require(cube: &Facelets) -> Result<(), String> {
    check(cube).map_err(|reason| {
        let hint = reason
            .hint()
            .map_or_else(String::new, |hint| format!("\nhint: {hint}"));
        format!("the cube state is impossible ({reason}){hint}")
    })
}

/// Which colour has too many stickers and which too few, for a cube written
/// as face letters whose colour counts are wrong.
pub(crate) fn count_hint(text: &str) -> Option<String> {
    let letters: Vec<Face> = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| Face::from_letter(c.to_ascii_uppercase()))
        .collect::<Option<_>>()?;
    let kind = PuzzleKind::of_facelet_count(letters.len())?;
    let each = kind.face_len();
    let mut parts = Vec::new();
    for face in Face::ALL {
        let found = letters.iter().filter(|&&f| f == face).count();
        let plural = |n: usize| if n == 1 { "sticker" } else { "stickers" };
        if found > each {
            let extra = found - each;
            parts.push(format!(
                "colour {} has {extra} {} too many",
                face.letter(),
                plural(extra)
            ));
        } else if found < each {
            let missing = each - found;
            parts.push(format!(
                "colour {} has {missing} {} too few",
                face.letter(),
                plural(missing)
            ));
        }
    }
    (!parts.is_empty()).then(|| format!("hint: {}", parts.join(", ")))
}
