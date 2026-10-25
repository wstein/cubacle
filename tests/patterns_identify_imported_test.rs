//! `patterns identify` against the active library (built-in plus imported), on an
//! invented table.

use std::path::{Path, PathBuf};
use std::process::Command;

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cubacle-identify-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn row(code: &str, n: u32, name: &str, moves: &str) -> String {
    let n = n.to_string();
    let f = [
        code,
        &n,
        "Kat",
        "Category",
        "Unter",
        "Subgroup",
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
    ];
    f.join("\t")
}

/// A directory with a config listing the built-in library and one invented table.
fn setup(name: &str, with_table: bool) -> PathBuf {
    let dir = temp(name);
    let table = [
        row("T1.01", 1, "Pair", "R2 U2"),
        row("T1.02", 1, "Slice", "R2 L2"),
    ]
    .join("\n");
    std::fs::create_dir_all(dir.join("tables/3xT")).unwrap();
    std::fs::write(dir.join("tables/3xT/t.txt"), table).unwrap();
    let files = if with_table {
        "[\"builtin\", \"tables/3xT/t.txt\"]"
    } else {
        "[\"builtin\"]"
    };
    std::fs::write(dir.join("c.toml"), format!("[patterns]\nfiles = {files}\n")).unwrap();
    dir
}

fn identify(dir: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .arg("patterns")
        .arg("--config")
        .arg(dir.join("c.toml"))
        .arg("identify")
        .args(args)
        .current_dir(dir)
        .env("HOME", dir)
        .env("XDG_CONFIG_HOME", dir.join("xdg"))
        .env("CUBACLE_CONFIG", dir.join("c.toml"))
        .env("CUBACLE_CACHE_DIR", dir.join("cache"))
        .output()
        .expect("cubacle runs");
    assert!(out.status.success(), "{out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn line<'a>(text: &'a str, start: &str) -> &'a str {
    text.lines()
        .find(|l| l.starts_with(start))
        .unwrap_or_else(|| panic!("no {start} line in {text}"))
}

#[test]
fn an_imported_pattern_is_matched_up_to_symmetry_and_named_with_its_id() {
    let dir = setup("exact", true);
    let text = identify(&dir, &["-s", "U2 D2"]);
    assert!(
        line(&text, "pattern:").contains("Slice (ct:T1.02)"),
        "{text}"
    );
    assert!(
        line(&text, "closest:").contains("Slice (ct:T1.02) 100 %"),
        "{text}"
    );
}

#[test]
fn a_near_state_lists_the_imported_patterns_among_the_closest() {
    let dir = setup("near", true);
    let text = identify(&dir, &["-s", "R2 L2 U", "--closest", "40"]);
    let closest = line(&text, "closest:");
    assert!(closest.contains("Slice (ct:T1.02)"), "{text}");
    assert!(closest.contains("Pair (ct:T1.01)"), "{text}");
}

#[test]
fn the_number_of_closest_patterns_can_be_chosen() {
    let dir = setup("count", true);
    let text = identify(&dir, &["-s", "R2 L2 U", "--closest", "1"]);
    assert_eq!(line(&text, "closest:").matches(" %").count(), 1, "{text}");
    let text = identify(&dir, &["-s", "R2 L2 U"]);
    assert_eq!(line(&text, "closest:").matches(" %").count(), 3, "{text}");
}

#[test]
fn without_imports_nothing_imported_appears() {
    let dir = setup("plain", false);
    let text = identify(&dir, &["-s", "R2 L2 U"]);
    assert!(!text.contains("ct:"), "{text}");
    assert_eq!(line(&text, "closest:").matches(" %").count(), 3, "{text}");
}
