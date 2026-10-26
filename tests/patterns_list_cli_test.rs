//! `patterns list` and the options of the list.

use std::process::Command;

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        // never touch the real home or the developer's config
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "XDG_CONFIG_HOME",
            std::env::temp_dir().join("cubacle-test-xdg"),
        )
        .env_remove("CUBACLE_CONFIG")
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

#[test]
fn list_subcommand_prints_what_bare_patterns_prints() {
    for extra in [
        &["--size", "3"][..],
        &["--tree"],
        &["--sort", "patternness"],
    ] {
        let mut bare = vec!["patterns"];
        bare.extend_from_slice(extra);
        let mut list = vec!["patterns", "list"];
        list.extend_from_slice(extra);
        let (ok, bare_text, _) = cubacle(&bare);
        let (list_ok, list_text, _) = cubacle(&list);
        assert!(ok && list_ok);
        assert_eq!(bare_text, list_text, "{extra:?}");
    }
    let (_, before, _) = cubacle(&["patterns", "--size", "3", "list"]);
    let (_, after, _) = cubacle(&["patterns", "list", "--size", "3"]);
    assert_eq!(before, after);
}

#[test]
fn list_options_are_refused_by_the_other_subcommands() {
    let (ok, _, err) = cubacle(&["patterns", "show", "checkerboard", "--kind", "SYM"]);
    assert!(!ok);
    assert!(err.contains("--kind applies to the list only"), "{err}");
}
