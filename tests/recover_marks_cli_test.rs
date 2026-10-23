use std::process::Command;

const ORIGINAL: &str = "R U2 F' D L2 B R' U' F2 D2 L B'";
/// The original cube after `B L' U2 B2 U2 B2 D2`: step 5 (`D'`) was skipped.
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
    let out = cubacle(&["recover", "-s", ORIGINAL, "--now", NOW]);
    assert!(out.contains("Recommended: recover"), "{out}");
    assert!(out.contains("Moves now: D2 B2 D'"), "{out}");
    assert!(!out.contains("Fresh solution:"), "{out}");
}

#[test]
fn a_fresh_route_is_labelled_fresh_solution() {
    let solution = "B L' U2 B2 D' U2 B2 D2 U' F2 D U' R2 U' R' B' L2 D' F U2 R'";
    let out = cubacle(&[
        "recover",
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
    let out = cubacle(&["recover", "-s", ORIGINAL, "--now", NOW, "--icons"]);
    let line = out
        .lines()
        .find(|l| l.starts_with("Stickers that differ: "))
        .unwrap_or_else(|| panic!("{out}"));
    let rest = line.trim_start_matches("Stickers that differ: ");
    let (count, rest) = rest.split_once(" of ").expect("of");
    let count: usize = count.parse().expect("number");
    assert!(count > 0 && rest.starts_with("54 "), "{line}");
    // the count line sits between the two pictures and the steps
    let at = out.find(line).unwrap();
    assert!(out.find("Is now").unwrap() < at && at < out.find("step 1 of").unwrap());
}
