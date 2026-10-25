//! `patterns identify` prints the kind; `patterns --kind` filters the list.

use std::process::{Command, Output};

fn patterns(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .arg("patterns")
        .args(args)
        // never touch the real home: the first run writes a global config
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "XDG_CONFIG_HOME",
            std::env::temp_dir().join("cubacle-test-xdg"),
        )
        .output()
        .expect("cubacle runs")
}

fn text(args: &[&str]) -> String {
    let output = patterns(args);
    assert!(output.status.success(), "{args:?}");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn identify_prints_the_kind_and_its_measures() {
    let out = text(&["identify", "R U R' U'"]);
    assert!(out.contains("kind: IRREG"), "{out}");
    assert!(out.contains("measures: fixed 0.78"), "{out}");
    assert!(out.contains("moved 7 of 26 pieces"), "{out}");
    // patternness stays the first score
    assert!(out.contains("patternness:"), "{out}");
}

#[test]
fn identify_names_the_kind_of_a_solved_cube_and_a_turn() {
    assert!(text(&["identify", "R R'"]).contains("kind: SOLVED"));
    assert!(text(&["identify", "R"]).contains("kind: MOSTLY"));
}

#[test]
fn the_kind_filter_lists_the_matching_library_patterns_with_a_column() {
    let out = text(&["--kind", "SYM", "--size", "3"]);
    assert!(
        out.starts_with("id\tname\tsizes (outer/range moves)\tkind\n"),
        "{out}"
    );
    assert!(out.contains("checkerboard\tCheckerboard\t3:"), "{out}");
    assert!(out.lines().skip(1).all(|l| l.ends_with("\tSYM")), "{out}");
    assert!(!out.contains("six-spots"), "{out}");
}

#[test]
fn the_kind_filter_ignores_case_and_composes_with_the_other_filters() {
    let out = text(&["--kind", "mostly", "--size", "5", "--sort", "patternness"]);
    assert!(out.contains("six-spots"), "{out}");
    assert!(out.contains("patternness"), "{out}");
    let none = text(&["--kind", "irreg"]);
    assert_eq!(none.lines().count(), 1, "{none}");
}

#[test]
fn an_unknown_kind_is_refused_with_the_list() {
    let output = patterns(&["--kind", "bogus"]);
    assert!(!output.status.success());
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(
        err.contains("SOLVED, FEW, MOSTLY, SYM, FACE, ORBIT, IRREG"),
        "{err}"
    );
}

/// An invented 3x3x3 table with a quarter turn, a half-turn pair and a
/// loose sequence, in a directory of its own with a config.
fn imported_setup() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("cubacle-kind-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("tables/3xK")).unwrap();
    let row = |code: &str, name: &str, moves: &str| {
        [
            code,
            "1",
            "Kat",
            "Category",
            "Unter",
            "Sub",
            "Muster",
            name,
            moves,
            "",
            "",
            "",
            "",
            "",
            "nobody 2000",
            "",
            "",
        ]
        .join("\t")
    };
    let text = [
        row("K1.01", "Quarter", "R"),
        row("K1.02", "Pair", "R2 L2"),
        row("K1.03", "Loose", "R U R' U'"),
    ]
    .join("\n");
    std::fs::write(dir.join("tables/3xK/k.txt"), text).unwrap();
    std::fs::write(
        dir.join("cubacle.toml"),
        "[patterns]\nfiles = [\"tables/3xK/k.txt\"]\n",
    )
    .unwrap();
    dir
}

#[test]
fn the_kind_filter_applies_to_imported_patterns() {
    let dir = imported_setup();
    let list = |kind: &str| {
        let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
            .args(["patterns", "--kind", kind])
            .current_dir(&dir)
            .env("HOME", &dir)
            .env_remove("CUBACLE_CONFIG")
            .env_remove("XDG_CONFIG_HOME")
            .output()
            .expect("cubacle runs");
        assert!(output.status.success());
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    let irregular = list("IRREG");
    assert!(irregular.contains("Loose"), "{irregular}");
    assert!(!irregular.contains("Quarter"), "{irregular}");
    let mostly = list("MOSTLY");
    assert!(mostly.contains("Quarter"), "{mostly}");
    assert!(mostly.contains("imported from"), "{mostly}");
}
