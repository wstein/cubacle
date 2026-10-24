//! `moves --icons` ends with the state after the last move.

use std::process::Command;

fn cubacle(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .expect("cubacle runs");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn moves_icons_end_with_the_end_state() {
    let out = cubacle(&["moves", "R", "U", "--icons", "--graphics", "plain"]);
    let token = cubacle(&["convert", "--to", "token", "-s", "R U"]);
    let end = out.lines().find(|l| l.starts_with("End state: ")).unwrap();
    assert_eq!(end, format!("End state: {}", token.trim_end()));
    assert!(out.find("move 2 of 2").unwrap() < out.find(end).unwrap());
}

#[test]
fn moves_icons_draw_the_end_picture_on_graphics_terminals() {
    let out = cubacle(&[
        "moves",
        "R",
        "U",
        "--icons",
        "--graphics",
        "kitty",
        "--columns",
        "4",
    ]);
    // two move icons in one row, then the end picture
    assert_eq!(out.matches("\x1b_Ga=T").count(), 2, "{out:?}");
    assert!(out.find("End state: ").unwrap() < out.rfind("\x1b_Ga=T").unwrap());
}
