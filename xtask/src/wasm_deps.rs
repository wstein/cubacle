//! `cargo xtask wasm-deps`: the browser module must not carry the command line, the
//! operating system probe, the configuration reader or the network client.

use std::process::{Command, ExitCode};

/// Crates that must not be in the normal dependency graph of `cubacle-wasm`.
pub const FORBIDDEN: [&str; 4] = ["clap", "sysinfo", "toml", "ureq"];

/// The forbidden crates named in the output of `cargo tree --prefix none` (one
/// `name version ...` line per package), sorted and without repeats.
pub fn forbidden_in(tree: &str) -> Vec<&'static str> {
    FORBIDDEN
        .iter()
        .copied()
        .filter(|name| {
            tree.lines()
                .any(|line| line.split_whitespace().next() == Some(name))
        })
        .collect()
}

pub fn run() -> ExitCode {
    let output = Command::new("cargo")
        .args([
            "tree",
            "-p",
            "cubacle-wasm",
            "--target",
            "wasm32-unknown-unknown",
            "-e",
            "normal",
            "--prefix",
            "none",
        ])
        .output();
    let output = match output {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            eprintln!(
                "cargo tree failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return ExitCode::from(2);
        }
        Err(error) => {
            eprintln!("cannot run cargo tree: {error}");
            return ExitCode::from(2);
        }
    };
    let found = forbidden_in(&String::from_utf8_lossy(&output.stdout));
    if found.is_empty() {
        println!(
            "wasm-deps: none of {} in the cubacle-wasm graph",
            FORBIDDEN.join(", ")
        );
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "wasm-deps: the cubacle-wasm graph holds {}",
            found.join(", ")
        );
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLEAN: &str = "cubacle-wasm v0.1.0 (/r/crates/wasm)\ncubacle v0.1.0 (/r)\nserde v1.0.228\ntoml_datetime v0.7.0\nclapper v1.0.0\n";

    #[test]
    fn a_clean_graph_passes() {
        assert!(forbidden_in(CLEAN).is_empty());
    }

    #[test]
    fn every_forbidden_name_is_found_by_exact_name() {
        for name in FORBIDDEN {
            let tree = format!("{CLEAN}{name} v1.2.3\n");
            assert_eq!(forbidden_in(&tree), vec![name]);
        }
    }

    #[test]
    fn a_longer_name_with_the_same_prefix_is_not_a_match() {
        assert!(
            forbidden_in("clap_builder v4.6.0\ntoml_edit v0.2.0\nsysinfo-extra v1\n").is_empty()
        );
    }
}
