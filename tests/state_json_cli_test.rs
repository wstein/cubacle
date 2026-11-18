//! `solve --json` carries the start state in every format as the object
//! `state`, after the keys of the server body.

use std::process::Command;

fn ok(args: &[&str]) -> (String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "XDG_CONFIG_HOME",
            std::env::temp_dir().join("cubacle-test-xdg"),
        )
        .env_remove("CUBACLE_CONFIG")
        .output()
        .expect("cubacle runs");
    assert!(output.status.success(), "{args:?}");
    (
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

/// A scramble with a whole-cube rotation, so that the frame matters.
const SCRAMBLE: &str = "R U x y2 F'";

#[test]
fn json_carries_the_state_in_every_format_after_the_server_keys() {
    let (stdout, _) = ok(&["solve", "--size", "3", "-s", SCRAMBLE, "--fast", "--json"]);
    let json = &stdout;
    let at = json.find(",\"state\":").expect("a state object");
    let object = &json[at + ",\"state\":".len()..json.len() - 1];
    let token2 = ok(&["convert", "--size", "3", SCRAMBLE, "--to", "token2"]).0;
    let token = ok(&["convert", "--size", "3", SCRAMBLE, "--to", "token"]).0;
    let facelets = ok(&["convert", "--size", "3", SCRAMBLE]).0;
    let colors = ok(&["convert", "--size", "3", SCRAMBLE, "--to", "color"]).0;
    assert_eq!(
        object,
        format!(
            "{{\"orbit64v2\":\"{token2}\",\"orbit64\":\"{token}\",\"facelets\":\"{facelets}\",\"color\":\"{colors}\"}}"
        )
    );
    // the keys before it are those of the server body, unchanged
    assert!(json[..at].ends_with("\"lowerBound\":null"), "{json}");
    // --state-format acts on the text line only, never on the JSON object
    let quiet = ok(&[
        "solve",
        "--size",
        "3",
        "-s",
        SCRAMBLE,
        "--fast",
        "--json",
        "--state-format",
        "none",
    ]);
    assert!(quiet.0.contains(&format!("\"orbit64v2\":\"{token2}\"")));
    assert!(!quiet.1.contains("state:"));
}
