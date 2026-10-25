//! `cubacle.toml` and imported patterns in `patterns` list, tree, show and identify,
//! on a synthetic table (invented lines, not third-party data).

use std::path::{Path, PathBuf};
use std::process::Command;

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cubacle-config-{name}-{}", std::process::id()));
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

/// A directory with a config and one 3x3 table.
fn setup(name: &str) -> (PathBuf, PathBuf) {
    let dir = temp(name);
    std::fs::create_dir_all(dir.join("tables/3xT")).unwrap();
    let text = [
        row("T1.01", 1, "Pair", "R2 U2"),
        row("T1.01", 2, "Pair", "R2 U2 D2 D2"),
        row("T1.02", 1, "Slice", "R2 L2"),
    ]
    .join("\n");
    std::fs::write(dir.join("tables/3xT/t.txt"), text).unwrap();
    let config = dir.join("cubacle.toml");
    std::fs::write(
        &config,
        "[patterns]\nfiles = [\"builtin\", \"tables/3xT/t.txt\"]\n",
    )
    .unwrap();
    (dir, config)
}

fn run(dir: &Path, args: &[&str]) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .current_dir(dir)
        .env("HOME", dir)
        .env_remove("CUBACLE_CONFIG")
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("cubacle runs");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn without_a_config_only_the_builtin_library_is_listed() {
    let dir = temp("none");
    let (ok, out, _) = run(&dir, &["patterns"]);
    assert!(ok);
    assert!(out.contains("checkerboard"));
    assert!(!out.contains("imported"), "{out}");
    let (_, out, _) = run(&dir, &["patterns", "sources"]);
    assert!(out.contains("config: none"), "{out}");
}

#[test]
fn imported_patterns_are_listed_shown_and_identified() {
    let (dir, _) = setup("full");
    let (ok, out, err) = run(&dir, &["patterns", "--size", "3"]);
    assert!(ok, "{err}");
    assert!(
        out.contains("t1-01\tPair\t3:2/2\timported from t.txt"),
        "{out}"
    );
    let (_, out, _) = run(&dir, &["patterns", "show", "ct:t1.01#2"]);
    assert!(out.contains("Pair (ct:T1.01)"), "{out}");
    assert!(out.contains("group: Category > Subgroup"), "{out}");
    assert!(out.contains("imported from t.txt"), "{out}");
    assert!(out.contains("#2: 4 moves"), "{out}");
    assert!(!out.contains("#1"), "{out}");
    let (ok, out, _) = run(&dir, &["patterns", "show", "ct:T1.01"]);
    assert!(ok && out.contains("#1") && out.contains("#2"), "{out}");
    let (_, out, _) = run(&dir, &["patterns", "identify", "--scramble", "U2 D2"]);
    assert!(
        out.contains("imported: ct:T1.02 Slice (imported from t.txt)"),
        "{out}"
    );
}

#[test]
fn the_tree_groups_by_category() {
    let (dir, _) = setup("tree");
    let (ok, out, err) = run(&dir, &["patterns", "--tree", "--size", "3"]);
    assert!(ok, "{err}");
    assert!(
        out.contains("Simple patterns\n  cu:checker\tCheckerboard\t"),
        "{out}"
    );
    assert!(
        out.contains("Category\n  Subgroup\n    ct:T1.01\tPair\t2 generators\t3:2"),
        "{out}"
    );
}

#[test]
fn config_errors_name_file_and_line_and_a_bad_entry_is_a_warning() {
    let dir = temp("bad");
    std::fs::write(dir.join("cubacle.toml"), "[patterns]\n\nfiles = [\n").unwrap();
    let (ok, _, err) = run(&dir, &["patterns"]);
    assert!(!ok);
    assert!(err.contains("cubacle.toml:"), "{err}");
    std::fs::write(
        dir.join("cubacle.toml"),
        "[patterns]\nfiles = [\"builtin\", \"missing/3xA/x.txt\", \"https://h/3xA/y.txt\"]\n",
    )
    .unwrap();
    let (ok, out, err) = run(&dir, &["patterns"]);
    assert!(ok, "{err}");
    assert!(out.contains("checkerboard"));
    assert!(err.contains("warning: missing/3xA/x.txt"), "{err}");
    assert!(err.contains("curl -o"), "{err}");
}

#[test]
fn patterns_sort_and_filter_by_patternness() {
    let (dir, _) = setup("score");
    let (ok, out, err) = run(&dir, &["patterns", "--size", "3", "--sort", "patternness"]);
    assert!(ok, "{err}");
    assert!(
        out.starts_with("id\tname\tsizes (outer/range moves)\tpatternness\n"),
        "{out}"
    );
    assert!(
        out.contains("t1-01\tPair\t3:2/2\t87 %\timported from t.txt"),
        "{out}"
    );
    let (_, plain, _) = run(&dir, &["patterns", "--size", "3"]);
    assert!(!plain.contains("patternness"), "{plain}");
    let (ok, none, _) = run(
        &dir,
        &["patterns", "--size", "3", "--min-patternness", "101"],
    );
    assert!(!ok || !none.contains("t1-01"), "{none}");
    let (ok, some, _) = run(
        &dir,
        &[
            "patterns",
            "--size",
            "3",
            "--min-patternness",
            "95",
            "--tree",
        ],
    );
    assert!(ok);
    assert!(
        some.contains("ct:T1.02\tSlice\t1 generator\t3:2\t100 %"),
        "{some}"
    );
    let (_, shown, _) = run(&dir, &["patterns", "show", "ct:T1.01"]);
    assert!(shown.contains("3x3x3 patternness: 87 %"), "{shown}");
}
