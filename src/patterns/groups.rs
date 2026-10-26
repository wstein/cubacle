//! The small category tree of the built-in library (one level).

use super::Group;

/// The short catalog code of a built-in pattern id (`cu:<code>`).
pub(super) fn builtin_code(id: &str) -> &str {
    match id {
        "checkerboard" => "checker",
        "six-spots" => "sixspot",
        "six-center-spots" => "sixcentre",
        "four-spots" => "foursp",
        "four-center-spots" => "fourcen",
        "cube-in-cube" => "cubecube",
        "cube-in-cube-in-cube" => "ccc",
        "superflip" => "superflip",
        "centers-opposite" => "cenopp",
        "centers-cycled" => "cencyc",
        other => other,
    }
}

/// A code of 1 to 9 characters from letters, digits, `.` and `-`.
pub(super) fn valid_code(code: &str) -> bool {
    (1..=9).contains(&code.len())
        && code
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

/// A source tag: two lowercase letters.
pub(super) fn valid_source(source: &str) -> bool {
    source.len() == 2 && source.chars().all(|c| c.is_ascii_lowercase())
}

/// The category of a built-in pattern id.
pub(super) fn builtin_group(id: &str) -> Vec<Group> {
    let (de, en) = match id {
        "checkerboard" | "six-spots" | "six-center-spots" | "four-spots" | "four-center-spots" => {
            ("Einfache Muster", "Simple patterns")
        }
        "cube-in-cube" | "cube-in-cube-in-cube" => ("Würfel im Würfel", "Cube in a cube"),
        "superflip" => ("Flips", "Flips"),
        "centers-opposite" | "centers-cycled" => ("Zentren", "Centers"),
        _ => ("Weitere Muster", "Other patterns"),
    };
    vec![Group {
        de: de.to_owned(),
        en: en.to_owned(),
    }]
}
