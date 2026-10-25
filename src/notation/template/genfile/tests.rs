use super::*;
use crate::puzzle::Facelets;

const TINY: &str = "\
# a comment
pattern-name: tiny
cube-size: 2
sticker-grid:
  a1 a1 a1 a1 / a2 a2 a2 a2 / a3 a3 a3 a3
  a4 a4 a4 a4 / a5 a5 a5 a5 / a6 a6 a6 a6
color-constraints: unique(a*)   # trailing comment
";

fn error(text: &str) -> String {
    GenFile::parse(text).unwrap_err().to_string()
}

#[test]
fn a_complete_file() {
    let text = "\
pattern-name: faces-2
pattern-title: One color per face
pattern-aliases: faces, six-colors
cube-size: 2
color-scheme: western
sticker-grid: a1 a1 a1 a1 / a2 a2 a2 a2 / a3 a3 a3 a3 / a4 a4 a4 a4 / a5 a5 a5 a5 / a6 a6 a6 a6
color-constraints: unique(a*)
move-alphabet: outer
search-limits: depth 8
pattern-tags: easy, faces
";
    let file = GenFile::parse(text).unwrap();
    assert_eq!(file.name, "faces-2");
    assert_eq!(file.title.as_deref(), Some("One color per face"));
    assert_eq!(file.aliases, ["faces", "six-colors"]);
    assert_eq!(file.tags, ["easy", "faces"]);
    assert_eq!(file.alphabet.as_deref(), Some("outer"));
    assert_eq!(file.limits.as_deref(), Some("depth 8"));
    assert_eq!(file.scheme.to_string(), "WRGYOB");
    assert_eq!(file.templates.len(), 1);
    assert_eq!(file.templates[0].variables().len(), 6);
    assert!(file.templates[0].matches(&Facelets::solved(PuzzleKind::Cube2)));
}

#[test]
fn continuation_lines_and_comments() {
    let file = GenFile::parse(TINY).unwrap();
    assert_eq!(file.name, "tiny");
    assert_eq!(file.guard, "unique(a*)");
    assert_eq!(file.scheme.to_string(), "URFDLB");
    assert!(file.templates[0].matches(&Facelets::solved(PuzzleKind::Cube2)));
}

#[test]
fn several_sizes_give_one_template_each() {
    let text = "pattern-name: x\ncube-size: 2, 2, 3\nsticker-grid: ".to_owned()
        + &(1..=6)
            .map(|k| vec![format!("a{k}"); 4].join(" "))
            .collect::<Vec<_>>()
            .join(" / ");
    // The 3x3 needs nine stickers per face, so the list fails on its second size.
    assert_eq!(
        error(&text),
        "line 3, column 27: face 1 has 4 stickers, expected 9 (3x3)"
    );
    let two = text.replace("2, 2, 3", "2, 2");
    assert_eq!(GenFile::parse(&two).unwrap().templates.len(), 1);
}

#[test]
fn errors_name_the_line_and_column_in_the_file() {
    let broken = |from: &str, to: &str| error(&TINY.replace(from, to));
    assert_eq!(
        broken("pattern-name: tiny\n", ""),
        "line 1, column 1: missing required key \"pattern-name\""
    );
    assert_eq!(
        broken("cube-size: 2\n", "cube-size: 2\nfoo: 1\n"),
        "line 4, column 1: unknown key \"foo\", expected one of pattern-name, pattern-title, pattern-aliases, cube-size, color-scheme, color-letters, sticker-grid, color-constraints, move-alphabet, search-limits, pattern-tags, required-symmetry, required-antisymmetry, sequence-order, movable-orbits, fixed-orbits, format-version"
    );
    assert_eq!(
        broken("pattern-name: tiny", "pattern-name: Bad_Id"),
        "line 2, column 15: \"Bad_Id\" is not an id, expected lowercase letters, digits and \"-\", not starting with \"-\""
    );
    assert_eq!(
        broken("cube-size: 2", "cube-size: 9"),
        "line 3, column 12: \"9\" is not a cube size, expected 2 to 7"
    );
    assert_eq!(
        broken("  a4 a4 a4 a4 /", "  a4 a4 a4 /"),
        "line 6, column 12: face 4 has 3 stickers, expected 4 (2x2)"
    );
    assert_eq!(
        broken("unique(a*)", "unique(a1, zz)"),
        "line 7, column 31: variable \"zz\" does not occur in the schema"
    );
    assert_eq!(
        broken("cube-size: 2", "cube-size: 2\ncube-size: 3"),
        "line 4, column 1: the key \"cube-size\" is given twice"
    );
    assert_eq!(
        broken("pattern-name: tiny", "name tiny"),
        "line 2, column 10: expected \"key: value\""
    );
    assert_eq!(
        broken("# a comment", "  stray"),
        "line 1, column 3: a continuation line needs a key line before it"
    );
    assert_eq!(
        broken("cube-size: 2", "cube-size: 2\ncolor-scheme: WWWWWW"),
        "line 4, column 15: the letter 'W' is used twice, expected URFDLB, western (boy), japanese or six distinct letters"
    );
    assert_eq!(
        error("pattern-name: x\ncube-size: 2\nsticker-grid:\n"),
        "line 3, column 14: the sticker grid is empty"
    );
}

#[test]
fn a_blank_line_ends_a_value() {
    let text = TINY.replace("  a4 a4", "\n  a4 a4");
    assert_eq!(
        error(&text),
        "line 7, column 3: a continuation line needs a key line before it"
    );
}

#[test]
fn garbage_never_panics() {
    for text in [
        "",
        ":",
        "::::",
        "pattern-name:",
        "\n\n\n",
        "sticker-grid:\n  \n  /",
        "cube-size: 2,,,\npattern-name: a\nsticker-grid: /",
        "#",
        "\r\n\r\n",
        "pattern-name: é\ncube-size: ☃",
    ] {
        let _ = GenFile::parse(text);
    }
}
