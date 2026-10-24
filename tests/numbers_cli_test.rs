//! `--numbers on|off`: move numbers in the corner of the icons.

use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn kitty(command: &str, rest: &[&str], numbers: Option<&str>) -> String {
    let mut args = vec![command];
    args.extend(rest);
    args.extend(["--icons", "--graphics", "kitty", "--columns", "4"]);
    if let Some(mode) = numbers {
        args.extend(["--numbers", mode]);
    }
    let (ok, out, err) = cubacle(&args);
    assert!(ok, "{err}");
    out
}

#[test]
fn moves_icons_have_numbers_unless_off() {
    let rest = ["R", "U", "F'", "D2", "--size", "3"];
    let default = kitty("moves", &rest, None);
    let on = kitty("moves", &rest, Some("on"));
    let off = kitty("moves", &rest, Some("off"));
    assert_eq!(default, on);
    assert_ne!(on, off);
}

#[test]
fn solve_icons_have_numbers_unless_off() {
    let rest = ["-s", "R U F' D2"];
    let on = kitty("solve", &rest, None);
    let off = kitty("solve", &rest, Some("off"));
    assert_ne!(on, off);
    assert_eq!(on.lines().count(), off.lines().count());
}

#[test]
fn text_pictures_and_plain_output_do_not_change() {
    let (_, plain, _) = cubacle(&["solve", "-s", "R U F' D2"]);
    let (_, text_on, _) = cubacle(&["solve", "-s", "R U F' D2", "--icons"]);
    let (_, text_off, _) = cubacle(&["solve", "-s", "R U F' D2", "--icons", "--numbers", "off"]);
    assert_eq!(text_on, text_off);
    assert!(text_on.contains("move 1 of 4: "));
    assert!(!plain.contains("move 1 of"));
    let (ok, _, _) = cubacle(&["solve", "-s", "R", "--numbers", "off"]);
    assert!(!ok, "--numbers needs --icons");
}
