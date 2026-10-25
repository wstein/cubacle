use std::process::Command;

fn check(name: &str, text: &str) -> (bool, String, String) {
    let path = format!("{}/{name}.cubacle", env!("CARGO_TARGET_TMPDIR"));
    std::fs::write(&path, text).expect("writes the .cubacle file");
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(["template", "check", &path])
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .replace(&path, "FILE"),
        String::from_utf8_lossy(&output.stderr)
            .trim()
            .replace(&path, "FILE"),
    )
}

/// Six faces of nine stickers each, face k all `a<k>`.
fn faces() -> String {
    (1..=6)
        .map(|k| vec![format!("a{k}"); 9].join(" "))
        .collect::<Vec<_>>()
        .join(" /\n  ")
}

#[test]
fn a_good_file_prints_one_ok_line() {
    let text = format!(
        "# one color per face\npattern-name: six-faces\ncube-size: 3\nsticker-grid:\n  {}\ncolor-constraints: unique(a*)\n",
        faces()
    );
    let (ok, out, err) = check("template_ok", &text);
    assert!(ok, "{err}");
    assert_eq!(out, "ok: six-faces, size 3, 6 variables, guard ok");
    assert_eq!(err, "");
}

#[test]
fn a_file_without_color_constraints_says_so() {
    let text = format!(
        "pattern-name: any\ncube-size: 3\nsticker-grid:\n  {}\n",
        faces()
    );
    let (ok, out, _) = check("template_noguard", &text);
    assert!(ok);
    assert_eq!(out, "ok: any, size 3, 6 variables, no guard");
}

#[test]
fn a_bad_guard_names_file_line_and_column() {
    let text = format!(
        "pattern-name: bad\ncube-size: 3\nsticker-grid:\n  {}\ncolor-constraints: unique(a1, zz)\n",
        faces()
    );
    let (ok, out, err) = check("template_bad", &text);
    assert!(!ok);
    assert_eq!(out, "");
    assert_eq!(
        err,
        "error: FILE:10:31: variable \"zz\" does not occur in the schema"
    );
}

#[test]
fn a_missing_file_is_an_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(["template", "check", "/no/such/file.cubacle"])
        .output()
        .expect("cubacle runs");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .starts_with("error: /no/such/file.cubacle: cannot read"));
}
