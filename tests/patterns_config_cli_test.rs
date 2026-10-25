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
fn the_first_run_creates_the_global_config_and_fetches_nothing() {
    let dir = temp("first");
    let global = dir.join(".config/cubacle/config.toml");
    assert!(!global.exists());
    let (ok, out, err) = run(&dir, &["patterns"]);
    assert!(ok, "{err}");
    assert!(out.contains("checkerboard"));
    assert!(!out.contains("imported"), "{out}");
    assert!(
        err.contains("created") && err.contains("config.toml"),
        "{err}"
    );
    let text = std::fs::read_to_string(&global).unwrap();
    assert!(text.contains("\"builtin\""));
    assert!(text.contains("# \"https://raw.githubusercontent.com/wrandelshofer/CubeTwister/"));
    assert!(text.contains("7xVCube7"));
    assert!(!dir.join(".cache").exists(), "nothing is downloaded");
    // The second run reuses it silently, and `sources` names it.
    let (ok, _, err) = run(&dir, &["patterns"]);
    assert!(ok && !err.contains("created"), "{err}");
    let (_, out, _) = run(&dir, &["patterns", "sources"]);
    assert!(
        out.contains("config: ") && out.contains("config.toml"),
        "{out}"
    );
    assert!(out.contains("builtin: the built-in library"), "{out}");
}

#[test]
fn xdg_config_home_decides_where_the_global_config_goes() {
    let dir = temp("xdg");
    let xdg = dir.join("xdg");
    let out = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(["patterns", "sources"])
        .current_dir(&dir)
        .env("HOME", &dir)
        .env("XDG_CONFIG_HOME", &xdg)
        .env_remove("CUBACLE_CONFIG")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(xdg.join("cubacle/config.toml").is_file());
    assert!(!dir.join(".config").exists());
}

#[test]
fn an_unwritable_location_is_said_and_the_run_continues() {
    let dir = temp("nowrite");
    // HOME is a file, so ~/.config cannot be created.
    let home = dir.join("home-file");
    std::fs::write(&home, "x").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(["patterns"])
        .current_dir(&dir)
        .env("HOME", &home)
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("CUBACLE_CONFIG")
        .output()
        .unwrap();
    assert!(out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("could not create"), "{err}");
    assert!(String::from_utf8_lossy(&out.stdout).contains("checkerboard"));
}

#[test]
fn an_explicit_or_local_config_creates_nothing() {
    let (dir, _) = setup("local");
    let (ok, _, err) = run(&dir, &["patterns"]);
    assert!(ok && !err.contains("created"), "{err}");
    assert!(!dir.join(".config").exists());
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
