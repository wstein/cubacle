//! `--to NAME`: a pattern of the library (built-in and config, never a
//! download) as the target of a solve.

use cubacle::patterns::Pattern;
use cubacle::puzzle::{Facelets, PuzzleKind};

use crate::patterns_config::Library;

/// Case, spaces, hyphens and underscores do not matter in a name.
fn squash(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Whether `text` could be meant as a name: words, spaces and hyphens only.
pub(crate) fn looks_like_name(text: &str) -> bool {
    let text = text.trim();
    text.chars().filter(|c| c.is_alphabetic()).count() >= 3
        && text
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | ':' | '.'))
}

fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (diagonal + usize::from(ca != *cb))
                .min(row[j] + 1)
                .min(above + 1);
            diagonal = above;
        }
    }
    row[b.len()]
}

/// The pattern ids whose names are close to `text`, best first.
fn close_names(text: &str, library: &Library) -> Vec<String> {
    let key = squash(text);
    let mut scored: Vec<(usize, String)> = Vec::new();
    for pattern in library.all() {
        let best = std::iter::once(pattern.name.as_str())
            .chain(std::iter::once(pattern.id.as_str()))
            .chain(pattern.aliases.iter().map(String::as_str))
            .map(|name| {
                let name = squash(name);
                if !key.is_empty() && (name.contains(&key) || key.contains(&name)) {
                    0
                } else {
                    distance(&key, &name)
                }
            })
            .min()
            .unwrap_or(usize::MAX);
        if best <= 2.max(key.len() / 4) && !scored.iter().any(|(_, id)| *id == pattern.id) {
            scored.push((best, pattern.id.clone()));
        }
    }
    scored.sort();
    scored.into_iter().take(5).map(|(_, id)| id).collect()
}

/// The note added to the error of a text that is no cube and no expression,
/// when the text may have been a pattern name.
pub(crate) fn unknown_note(text: &str, library: &Library) -> String {
    let close = close_names(text, library);
    let list = "`cubacle patterns list` shows every pattern name";
    if close.is_empty() {
        format!("\nno pattern is called {text:?}; {list}")
    } else {
        format!(
            "\nno pattern is called {text:?}; close: {}; {list}",
            close.join(", ")
        )
    }
}

/// The target state of the library pattern `text` on `kind`: `None` when no
/// pattern has that name, `Some(Err(..))` when it exists but not for that
/// size, or the name is ambiguous.
pub(crate) fn library_target(
    text: &str,
    kind: PuzzleKind,
    library: &Library,
) -> Option<Result<Facelets, String>> {
    let named: Vec<&Pattern> = library.all().filter(|p| p.is_named(text)).collect();
    if named.is_empty() {
        return None;
    }
    let size = kind.size();
    let states: Vec<(&Pattern, Facelets)> = named
        .iter()
        .filter_map(|p| p.state(kind).map(|state| (*p, state)))
        .collect();
    let Some((_, state)) = states.first() else {
        let mut sizes: Vec<usize> = named.iter().flat_map(|p| p.sizes()).collect();
        sizes.sort_unstable();
        sizes.dedup();
        let list: Vec<String> = sizes.iter().map(|s| format!("{s}x{s}x{s}")).collect();
        return Some(Err(format!(
            "the pattern {text:?} has no state on a {size}x{size}x{size}{}",
            if list.is_empty() {
                String::new()
            } else {
                format!(" (it is defined for {})", list.join(", "))
            }
        )));
    };
    if states.iter().any(|(_, other)| other != state) {
        let ids: Vec<&str> = states.iter().map(|(p, _)| p.catalog.as_str()).collect();
        return Some(Err(format!(
            "the name {text:?} is ambiguous on a {size}x{size}x{size}; use one of {}",
            ids.join(", ")
        )));
    }
    Some(Ok(state.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builtin() -> Library {
        Library {
            builtin: true,
            ..Library::default()
        }
    }

    #[test]
    fn names_resolve_with_case_and_hyphen_variants() {
        let library = builtin();
        let kind = PuzzleKind::Cube3;
        let spots = library_target("six-spots", kind, &library)
            .unwrap()
            .unwrap();
        for text in ["Six Spots", "SIX_SPOTS", "six spots", "6 spots", "six dots"] {
            assert_eq!(
                library_target(text, kind, &library).unwrap().unwrap(),
                spots
            );
        }
        let generator = cubacle::patterns::find("six-spots")
            .unwrap()
            .generator(3, None)
            .unwrap()
            .sequence()
            .unwrap();
        assert_eq!(Facelets::solved(kind).apply(&generator).unwrap(), spots);
    }

    #[test]
    fn an_unknown_name_is_none_and_gets_close_names() {
        let library = builtin();
        assert!(library_target("six-spotz", PuzzleKind::Cube3, &library).is_none());
        let note = unknown_note("six-spotz", &library);
        assert!(
            note.contains("six-spots") && note.contains("patterns list"),
            "{note}"
        );
        assert!(unknown_note("zzzzzz", &library).contains("patterns list"));
    }

    #[test]
    fn a_pattern_without_a_state_on_the_size_says_so() {
        let library = builtin();
        let error = library_target("six-spots", PuzzleKind::Cube2, &library)
            .unwrap()
            .unwrap_err();
        assert!(error.contains("2x2x2"), "{error}");
    }

    #[test]
    fn two_entries_with_one_name_and_different_states_are_ambiguous() {
        let table = "#cubacle-patterns 1 patternness-v1\n#source test\n\
            twin-a\tt:1\t3\t1\tTwin\t\t\t\t\t-\tR2\n\
            twin-b\tt:2\t3\t1\tTwin\t\t\t\t\t-\tU2\n";
        let library = Library {
            extra: cubacle::patterns::read_imported(table, "test").unwrap(),
            ..Library::default()
        };
        let error = library_target("twin", PuzzleKind::Cube3, &library)
            .unwrap()
            .unwrap_err();
        assert!(
            error.contains("ambiguous") && error.contains("t:1, t:2"),
            "{error}"
        );
        // The same state under one name is not ambiguous.
        let same = table.replace("U2", "R2");
        let library = Library {
            extra: cubacle::patterns::read_imported(&same, "test").unwrap(),
            ..Library::default()
        };
        assert!(library_target("twin", PuzzleKind::Cube3, &library)
            .unwrap()
            .is_ok());
    }

    #[test]
    fn move_expressions_do_not_look_like_names() {
        assert!(!looks_like_name("R U"));
        assert!(!looks_like_name("F2 B2"));
        assert!(looks_like_name("six-spotz"));
    }
}
