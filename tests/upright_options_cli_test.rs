use std::process::Command;

fn refused(options: &[&str], option: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(["solve", "-s", "not-a-move", "--upright-centers"])
        .args(options)
        .output()
        .expect("cubacle runs");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{option}: {stderr}");
    assert!(
        stderr.contains(&format!("{option} does not work with --upright-centers")),
        "{option}: {stderr}"
    );
}

#[test]
fn upright_refuses_group() {
    refused(&["--group", "axis"], "--group");
}

#[test]
fn upright_refuses_every() {
    refused(&["--every", "2"], "--every");
}

#[test]
fn upright_refuses_optimize() {
    refused(&["--optimize", "ranges"], "--optimize");
}

#[test]
fn upright_refuses_depth() {
    refused(&["--depth", "2"], "--depth");
}

#[test]
fn upright_refuses_raw() {
    refused(&["--raw"], "--raw");
}

#[test]
fn upright_refuses_pattern_aware() {
    refused(&["--pattern-aware"], "--pattern-aware");
}

#[test]
fn upright_refuses_parallel() {
    refused(&["--parallel"], "--parallel");
}

#[test]
fn upright_refuses_nodes() {
    refused(&["--optimal", "--nodes", "10"], "--nodes");
}
