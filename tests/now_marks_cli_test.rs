use std::process::Command;

const ORIGINAL: &str = "R U2 F' D L2 B R' U' F2 D2 L B'";
/// The original cube after `B L' U2 B2 U2 B2 D2`: move 5 (`D'`) was skipped.
const NOW: &str = "FDDRURLDBRDBLRFFDUUBUUFBLBLBRDFDFBUFRUFULLDRURLDLBFRBL";

fn cubacle(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .expect("cubacle runs");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn the_way_back_is_labelled_moves_now() {
    let out = cubacle(&["solve", "-s", ORIGINAL, "--now", NOW]);
    assert!(out.contains("Recommended: recover"), "{out}");
    assert!(out.contains("Moves now: D2 B2 D'"), "{out}");
    assert!(!out.contains("Fresh solution:"), "{out}");
}

#[test]
fn a_fresh_route_is_labelled_fresh_solution() {
    let solution = "B L' U2 B2 D' U2 B2 D2 U' F2 D U' R2 U' R' B' L2 D' F U2 R'";
    let out = cubacle(&[
        "solve",
        "-s",
        ORIGINAL,
        "--now",
        NOW,
        "--solution",
        solution,
        "--depth",
        "0",
        "--one-mistake",
    ]);
    assert!(out.contains("Recommended: solve fresh"), "{out}");
    assert!(out.contains("\nFresh solution: "), "{out}");
    assert!(!out.contains("Moves now:"), "{out}");
}

#[test]
fn icons_count_the_differing_stickers() {
    let out = cubacle(&["solve", "-s", ORIGINAL, "--now", NOW, "--icons"]);
    let line = out
        .lines()
        .find(|l| l.starts_with("Stickers that differ: "))
        .unwrap_or_else(|| panic!("{out}"));
    let rest = line.trim_start_matches("Stickers that differ: ");
    let (count, rest) = rest.split_once(" of ").expect("of");
    let count: usize = count.parse().expect("number");
    assert!(count > 0 && rest.starts_with("54 "), "{line}");
    // the count line sits between the two pictures and the rollback moves
    let at = out.find(line).unwrap();
    assert!(out.find("Is now").unwrap() < at && at < out.find("move R1 of").unwrap());
}

#[test]
fn views_both_draws_two_corners_and_the_count_line_says_every_difference_is_marked() {
    let out = cubacle(&[
        "solve", "-s", ORIGINAL, "--now", NOW, "--icons", "--views", "both",
    ]);
    // three cubes (should be, is now, end state), each as a text picture from the URF and the DBL corner
    assert_eq!(out.matches("state, seen from URF").count(), 3, "{out}");
    assert_eq!(out.matches("state, seen from DBL").count(), 3, "{out}");
    let line = out
        .lines()
        .find(|l| l.starts_with("Stickers that differ: "))
        .unwrap();
    assert!(line.contains("all marked amber"), "{line}");
}

#[test]
fn the_folded_view_is_the_default_and_marks_every_difference() {
    let out = cubacle(&["solve", "-s", ORIGINAL, "--now", NOW, "--icons"]);
    assert_eq!(out.matches("state, folded view").count(), 3, "{out}");
    assert!(!out.contains("seen from"), "{out}");
    let line = out
        .lines()
        .find(|l| l.starts_with("Stickers that differ: "))
        .unwrap();
    assert!(line.contains("all marked amber"), "{line}");
}

#[test]
fn views_urf_draws_one_picture_and_admits_the_hidden_faces() {
    let args = ["solve", "-s", ORIGINAL, "--now", NOW, "--icons"];
    let out = cubacle(&[&args[..], &["--views", "urf"]].concat());
    assert_eq!(out.matches("state, seen from URF").count(), 3, "{out}");
    assert!(!out.contains("seen from DBL"), "{out}");
    let line = out
        .lines()
        .find(|l| l.starts_with("Stickers that differ: "))
        .unwrap();
    assert!(line.contains("visible faces only"), "{line}");
}

#[test]
fn views_net_and_all_draw_the_flat_net_and_count_every_difference_as_marked() {
    let args = ["solve", "-s", ORIGINAL, "--now", NOW, "--icons"];
    let net = cubacle(&[&args[..], &["--views", "net"]].concat());
    assert_eq!(net.matches("state, flat view").count(), 3, "{net}");
    assert!(!net.contains("seen from"), "{net}");
    let line = net
        .lines()
        .find(|l| l.starts_with("Stickers that differ: "))
        .unwrap();
    assert!(line.contains("all marked amber"), "{line}");
    let all = cubacle(&[&args[..], &["--views", "all"]].concat());
    assert_eq!(all.matches("state, flat view").count(), 3, "{all}");
    assert_eq!(all.matches("state, seen from URF").count(), 3, "{all}");
    assert_eq!(all.matches("state, seen from DBL").count(), 3, "{all}");
}
