//! `patterns --icons`: pictures for the list, show, find and identify.

use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "XDG_CONFIG_HOME",
            std::env::temp_dir().join("cubacle-test-xdg"),
        )
        .env_remove("CUBACLE_CONFIG")
        .env_remove("CUBACLE_GRAPHICS")
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

/// Every picture of a graphics terminal starts with this escape.
const KITTY: &str = "\x1b_Ga=T";

#[test]
fn the_list_without_icons_is_unchanged() {
    let (_, bare, _) = cubacle(&["patterns", "--size", "3"]);
    let (ok, with, err) = cubacle(&["patterns", "list", "--size", "3", "--limit", "100"]);
    assert!(ok, "{err}");
    assert_eq!(bare, with);
    assert!(err.is_empty(), "{err}");
}

#[test]
fn list_icons_draws_one_plain_picture_per_pattern_on_a_pipe() {
    let (ok, out, err) = cubacle(&["patterns", "--icons", "--size", "3", "--limit", "3"]);
    assert!(ok, "{err}");
    assert!(!out.contains('\x1b'), "{out}");
    assert_eq!(out.matches("3x3x3\n").count(), 3, "{out}");
    // the rows of the plain list are still there, one per picture
    let (_, plain, _) = cubacle(&["patterns", "--size", "3"]);
    for row in plain.lines().take(4) {
        assert!(out.contains(row), "{row}");
    }
    assert!(err.contains("showing the first 3 of "), "{err}");
}

#[test]
fn list_icons_say_when_they_cut_nothing_by_staying_quiet() {
    let (ok, _, err) = cubacle(&[
        "patterns", "list", "--icons", "--size", "3", "--limit", "50",
    ]);
    assert!(ok);
    assert!(err.is_empty(), "{err}");
}

#[test]
fn list_icons_follow_the_filters_and_the_graphics_option() {
    let (ok, out, err) = cubacle(&[
        "patterns",
        "list",
        "--icons",
        "--graphics",
        "kitty",
        "--kind",
        "SYM",
        "--size",
        "3",
        "--limit",
        "2",
    ]);
    assert!(ok, "{err}");
    assert!(out.starts_with("id\tname\tsizes"), "{out}");
    assert!(out.contains("\tSYM"), "{out}");
    assert_eq!(out.matches(KITTY).count(), out.matches("\tSYM").count());
    assert!(out.matches(KITTY).count() >= 1, "{out}");
    let (ok, ansi, _) = cubacle(&[
        "patterns",
        "--icons",
        "--graphics",
        "ansi",
        "--size",
        "3",
        "--limit",
        "1",
    ]);
    assert!(ok);
    assert!(ansi.contains("\x1b["), "{ansi:?}");
    assert!(!ansi.contains(KITTY));
}

#[test]
fn the_tree_stays_text_and_says_so() {
    let (_, tree, _) = cubacle(&["patterns", "--tree"]);
    let (ok, out, err) = cubacle(&["patterns", "--tree", "--icons", "--limit", "1"]);
    assert!(ok);
    assert_eq!(out, tree);
    assert!(err.contains("--tree is text only"), "{err}");
}

#[test]
fn picture_options_need_icons_and_only_some_commands_take_icons() {
    let (ok, _, err) = cubacle(&["patterns", "--icon-size", "40"]);
    assert!(!ok);
    assert!(err.contains("--icons"), "{err}");
    let (ok, _, err) = cubacle(&["patterns", "cycles", "R", "--icons"]);
    assert!(!ok);
    assert!(err.contains("--icons applies to"), "{err}");
}

#[test]
fn identify_icons_draw_the_cube_and_the_closest_patterns_labeled() {
    let (_, text, _) = cubacle(&["patterns", "identify", "R2 L2 U2 D2 F2 B2"]);
    let (ok, out, err) = cubacle(&[
        "patterns",
        "identify",
        "R2 L2 U2 D2 F2 B2",
        "--closest",
        "2",
        "--icons",
        "--graphics",
        "plain",
    ]);
    assert!(ok, "{err}");
    let (_, short, _) = cubacle(&[
        "patterns",
        "identify",
        "R2 L2 U2 D2 F2 B2",
        "--closest",
        "2",
    ]);
    assert!(out.starts_with(&short), "the text comes first, unchanged");
    assert!(!text.is_empty());
    assert!(out.contains("\nyour cube\n"), "{out}");
    assert!(
        out.contains("\nclosest: Checkerboard (checkerboard) 100 %\n"),
        "{out}"
    );
    let drawn = out.find("\nyour cube\n").unwrap();
    assert!(out[drawn..].contains("\nclosest: Checkerboard (checkerboard) 100 %\n"));
    let (_, kitty, _) = cubacle(&[
        "patterns",
        "identify",
        "R2 L2 U2 D2 F2 B2",
        "--closest",
        "2",
        "--icons",
        "--graphics",
        "kitty",
    ]);
    assert_eq!(kitty.matches(KITTY).count(), 3, "{kitty:?}");
}

#[test]
fn identify_icons_take_an_orbit64_token() {
    let (_, token, _) = cubacle(&["convert", "--to", "token", "R2 L2 U2 D2 F2 B2"]);
    let (ok, out, err) = cubacle(&["patterns", "identify", token.trim(), "--icons"]);
    assert!(ok, "{err}");
    assert!(out.contains("\nyour cube\n"), "{out}");
    assert!(!out.contains('\x1b'));
}

#[test]
fn show_icons_draw_the_pattern_and_the_moves_of_every_printed_generator() {
    let (ok, out, err) = cubacle(&[
        "patterns",
        "show",
        "checkerboard",
        "--size",
        "3",
        "--icons",
        "--graphics",
        "plain",
    ]);
    assert!(ok, "{err}");
    assert!(out.contains("3x3x3 pattern\n"), "{out}");
    assert!(out.contains("move 1 of 6"), "{out}");
    assert!(out.contains("move 6 of 6"), "{out}");
    // the outer and the range generator are printed, each with its own icons
    assert!(out.contains("move 1 of 3: 2U2"), "{out}");
    assert_eq!(out.matches("move 1 of ").count(), 2, "{out}");
    let (ok, kitty, _) = cubacle(&[
        "patterns",
        "show",
        "checkerboard",
        "--size",
        "3",
        "--icons",
        "--graphics",
        "kitty",
        "--columns",
        "3",
    ]);
    assert!(ok);
    // the pattern, then two rows for six moves and one row for three
    assert_eq!(kitty.matches(KITTY).count(), 1 + 2 + 1, "{kitty:?}");
}

#[test]
fn show_icons_respect_the_size() {
    let (ok, out, err) = cubacle(&["patterns", "show", "checkerboard", "--icons"]);
    assert!(ok, "{err}");
    assert!(out.contains("3x3x3 pattern\n"), "{out}");
    let (ok, out, _) = cubacle(&["patterns", "show", "checkerboard", "--size", "4", "--icons"]);
    assert!(ok);
    assert!(out.contains("4x4x4 pattern\n"), "{out}");
    assert!(!out.contains("3x3x3 pattern"), "{out}");
}

#[test]
fn find_icons_draw_the_found_moves_and_the_state_they_reach() {
    let (ok, out, err) = cubacle(&["patterns", "find", "R2 L2", "--icons"]);
    assert!(ok, "{err}");
    assert!(out.contains("L2 R2\n"), "{out}");
    assert!(out.contains("move 1 of 2: L2"), "{out}");
    assert!(out.contains("move 2 of 2: R2"), "{out}");
    assert!(out.contains("End state: "), "{out}");
    assert!(!out.contains('\x1b'));
    let (ok, plain, _) = cubacle(&["patterns", "find", "R2 L2"]);
    assert!(ok);
    assert_eq!(plain, "L2 R2\n");
    let (ok, kitty, _) = cubacle(&[
        "patterns",
        "find",
        "R2 L2",
        "--icons",
        "--graphics",
        "kitty",
        "--columns",
        "4",
    ]);
    assert!(ok);
    // two move icons in a row, then the end picture
    assert_eq!(kitty.matches(KITTY).count(), 2, "{kitty:?}");
}
