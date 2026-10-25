//! `patterns identify --repair` on invented tables: the fix is printed, the
//! distance appears on the `closest:` entry, and without the flag nothing changes.

use std::path::{Path, PathBuf};
use std::process::Command;

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cubacle-repair-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn row(code: &str, name: &str, moves: &str) -> String {
    let f = [
        code,
        "1",
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

fn setup(name: &str) -> PathBuf {
    let dir = temp(name);
    let table = [
        row("T1.01", "Pair", "R2 U2"),
        row("T1.02", "Slice", "R2 L2"),
    ]
    .join("\n");
    std::fs::create_dir_all(dir.join("tables/3xT")).unwrap();
    std::fs::write(dir.join("tables/3xT/t.txt"), table).unwrap();
    std::fs::write(
        dir.join("c.toml"),
        "[patterns]\nfiles = [\"builtin\", \"tables/3xT/t.txt\"]\n",
    )
    .unwrap();
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
fn a_pattern_plus_one_move_is_repaired_and_the_distance_is_shown() {
    let dir = setup("after");
    let args = ["--scramble", "R2 U2 F", "--closest", "20"];
    let plain = identify(&dir, &args);
    assert!(!plain.contains("repair"), "{plain}");
    assert!(!plain.contains("away"), "{plain}");

    let with = identify(&dir, &[&args[..], &["--repair"]].concat());
    let fix = line(&with, "repair:");
    assert!(fix.contains("Pair (ct:T1.01)"), "{fix}");
    assert!(fix.ends_with("(1 block, 1 turn)"), "{fix}");
    assert!(
        line(&with, "closest:").contains("Pair (ct:T1.01) "),
        "{with}"
    );
    assert!(
        line(&with, "closest:").contains("%, 1 block away"),
        "{with}"
    );
    // Everything but the added distance and the repair line is unchanged.
    let strip = |t: &str| {
        t.lines()
            .filter(|l| !l.starts_with("repair"))
            .map(|l| l.replace(", 1 block away", ""))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(strip(&with), strip(&plain));
}

#[test]
fn extra_moves_before_the_pattern_and_the_closest_distance() {
    let dir = setup("before");
    let out = identify(
        &dir,
        &["--scramble", "F L R2 U2", "--closest", "20", "--repair=2"],
    );
    let fix = line(&out, "repair:");
    assert!(fix.contains("Pair (ct:T1.01)"), "{fix}");
    let closest = line(&out, "closest:");
    assert!(closest.contains("%, 2 blocks away"), "{closest}");
}

#[test]
fn a_state_far_from_every_pattern_gets_no_fix_and_no_distance() {
    let dir = setup("none");
    let out = identify(
        &dir,
        &[
            "--scramble",
            "R U F' L2 D B R' U2 F D' L B2 R F'",
            "--repair=3",
            "-v",
        ],
    );
    assert_eq!(line(&out, "repair:"), "repair: none within 3 moves");
    assert!(
        line(&out, "repair details:").contains("candidates"),
        "{out}"
    );
    assert!(!out.contains("away"), "{out}");
}

#[test]
fn the_time_limit_is_reported() {
    let dir = setup("time");
    let out = identify(
        &dir,
        &[
            "--size",
            "7",
            "--scramble",
            "3Rw U 3Fw' D 2L B2 3Uw",
            "--repair=7",
            "--max-time",
            "1",
        ],
    );
    assert!(
        line(&out, "repair:")
            .starts_with("repair: none found within 7 moves before the time limit"),
        "{out}"
    );
}
