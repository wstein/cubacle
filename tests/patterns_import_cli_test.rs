//! `patterns import` on a synthetic table (invented lines, not third-party data).

use std::path::PathBuf;
use std::process::Command;

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cubacle-import-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn row(code: &str, n: u32, name: &str, moves: &str, counts: [&str; 4]) -> String {
    let n = n.to_string();
    let f = [
        code,
        &n,
        "Gruppe",
        "Group",
        "Unter",
        "Sub",
        "Muster",
        name,
        moves,
        counts[0],
        counts[1],
        counts[2],
        counts[3],
        "",
        "nobody 2000",
        "",
        "",
    ];
    f.join("\t")
}

fn run(args: &[&str], cache: &PathBuf) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .env("CUBACLE_CACHE_DIR", cache)
        // never touch the real home: the first run writes a global config
        .env("HOME", std::env::temp_dir().join("cubacle-test-home"))
        .env(
            "XDG_CONFIG_HOME",
            std::env::temp_dir().join("cubacle-test-xdg"),
        )
        .output()
        .expect("cubacle runs");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn dry_run_reports_lines_and_count_mismatches() {
    let dir = temp("dry");
    let table = dir.join("3xTest").join("Table.txt");
    std::fs::create_dir_all(table.parent().unwrap()).unwrap();
    let text = [
        row("T1.01", 1, "Pair", "R2 U2", ["", "2*", "4*", "6"]),
        row("T1.02", 1, "Wrong counts", "R U", ["", "5", "9", "7"]),
        row("T1.03", 1, "Broken", "R Q", ["", "", "", ""]),
    ]
    .join("\n");
    std::fs::write(&table, text).unwrap();
    let cache = dir.join("cache");
    let (ok, out, err) = run(
        &["patterns", "import", table.to_str().unwrap(), "--dry-run"],
        &cache,
    );
    assert!(ok, "{err}");
    assert!(
        out.contains("3 lines, 2 verified by replay, 2 patterns, 1 problems"),
        "{out}"
    );
    assert!(err.contains(":3: T1.03"), "{err}");
    assert!(
        out.contains("line 2 T1.02: claims 5 face turns, counted 2"),
        "{out}"
    );
    assert!(out.contains("claims 7 order, counted 105"), "{out}");
    assert!(!out.contains("line 1 T1.01"), "{out}");
    assert!(out.contains("ct:T1.01\tpatternness "), "{out}");
    assert!(
        out.contains("patternness: mean ") && out.contains(" of 2 at least 95 %"),
        "{out}"
    );
    assert!(!cache.exists(), "a dry run writes nothing");
}

#[test]
fn import_writes_into_the_cache_dir_and_urls_are_refused() {
    let dir = temp("write");
    let table = dir.join("2xPocket-test.txt");
    std::fs::write(&table, row("T2.01", 1, "Half", "R2", ["", "", "", ""])).unwrap();
    let cache = dir.join("cache");
    let (ok, out, err) = run(
        &["patterns", "import", table.to_str().unwrap(), "--size", "2"],
        &cache,
    );
    assert!(ok, "{err}");
    assert!(out.contains("wrote "), "{out}");
    let written = std::fs::read_to_string(
        std::fs::read_dir(cache.join("patterns").join("imported"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path(),
    )
    .unwrap_or_default();
    assert!(written.contains("ct:T2.01"), "{written}");
    assert!(
        written.starts_with("#cubacle-patterns 1 patternness-v1\n"),
        "{written}"
    );
    let line = written.lines().find(|l| l.contains("ct:T2.01")).unwrap();
    let fields: Vec<&str> = line.split('\t').collect();
    assert_eq!(fields.len(), 11, "{line}");
    assert_eq!(fields[9], "100", "{line}");
    assert_eq!(fields[10], "R2", "{line}");
    let (ok, _, err) = run(
        &["patterns", "import", "https://example.org/3xA/F.txt"],
        &cache,
    );
    assert!(!ok);
    assert!(err.contains("curl"), "{err}");
}
