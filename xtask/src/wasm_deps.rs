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
        .filter(|name| package_names(tree).any(|package| package == *name))
        .collect()
}

/// First token of each line. That is the package name only when `cargo tree`
/// was run with `--prefix none`; a drawn tree puts the name after a box character.
fn package_names(tree: &str) -> impl Iterator<Item = &str> {
    tree.lines()
        .filter_map(|line| line.split_whitespace().next())
}

/// Refuse a graph that holds a forbidden crate, and refuse output that does not
/// even name `cubacle`. The second case is the drawn tree: the parser would
/// otherwise see no package names and report success.
fn check_tree(tree: &str) -> Result<(), String> {
    if !package_names(tree).any(|name| name == "cubacle") {
        return Err("cargo tree output has no package named cubacle".to_string());
    }
    let found = forbidden_in(tree);
    if found.is_empty() {
        Ok(())
    } else {
        Err(format!("the cubacle-wasm graph holds {}", found.join(", ")))
    }
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
    let tree = String::from_utf8_lossy(&output.stdout);
    match check_tree(&tree) {
        Ok(()) => {
            println!(
                "wasm-deps: none of {} in the cubacle-wasm graph",
                FORBIDDEN.join(", ")
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("wasm-deps: {error}");
            ExitCode::FAILURE
        }
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

    #[test]
    fn box_drawing_output_is_rejected_even_when_it_mentions_clap() {
        // Without `--prefix none`, cargo tree draws the graph. The package
        // name is not the first token, so a parser that only reads that token
        // sees neither cubacle nor clap and would otherwise report a clean graph.
        let tree = "\
cubacle-wasm v0.1.0 (/r/crates/wasm)
└── cubacle v0.1.0 (/r)
    └── clap v4.6.7
";
        let error = check_tree(tree).expect_err("box drawing is not a package list");
        assert!(error.contains("cubacle"), "{error}");
        assert!(forbidden_in(tree).is_empty());
    }

    #[test]
    fn a_prefix_none_tree_names_cubacle() {
        assert!(check_tree(CLEAN).is_ok());
    }
}
