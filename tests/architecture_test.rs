//! Enforces the dependency rules in ARCHITECTURE.md by reading the sources:
//! `puzzle <- notation <- engines <- apps`, engines never reach into each
//! other, and only the 3x3 engine uses the worker pool.
//!
//! Only non-test code is checked: unit tests may cross-check engines against
//! each other (a test module is the tail of its file, from `#[cfg(test)]`).

use std::fs;
use std::path::{Path, PathBuf};

/// (source directory, module paths it may name through `crate::`)
const RULES: [(&str, &[&str]); 5] = [
    ("src/puzzle", &["puzzle"]),
    ("src/notation", &["puzzle", "notation"]),
    ("src/generate", &["puzzle", "generate"]),
    ("src/engines/three", &["puzzle", "engines::three"]),
    ("src/engines/two", &["puzzle", "notation", "engines::two"]),
];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("source directory") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// The module paths named after `crate::`, e.g. `engines::three`.
fn crate_paths(source: &str) -> Vec<String> {
    let production = source.split("#[cfg(test)]").next().unwrap_or(source);
    let mut paths = Vec::new();
    for (start, _) in production.match_indices("crate::") {
        let rest = &production[start + "crate::".len()..];
        let mut words = rest.split("::").map(|part| {
            part.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
        });
        let first: String = words.next().expect("a path").collect();
        let path = if first == "engines" {
            let second: String = words.next().map(Iterator::collect).unwrap_or_default();
            format!("engines::{second}")
        } else {
            first
        };
        paths.push(path);
    }
    paths
}

#[test]
fn modules_only_use_what_the_dependency_rules_allow() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (dir, allowed) in RULES {
        let mut files = Vec::new();
        rust_files(&root.join(dir), &mut files);
        assert!(!files.is_empty(), "{dir} has no sources");
        for file in files {
            let source = fs::read_to_string(&file).expect("readable source");
            for path in crate_paths(&source) {
                assert!(
                    allowed.contains(&path.as_str()),
                    "{} uses crate::{path}, which {dir} may not depend on",
                    file.display()
                );
            }
        }
    }
}

#[test]
fn the_worker_pool_stays_private_to_the_3x3_engine() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join("src/engines/three/pool.rs").exists());
    assert!(!root.join("src/pool.rs").exists());
    let engine = fs::read_to_string(root.join("src/engines/three/mod.rs")).expect("mod.rs");
    assert!(engine.contains("\nmod pool;"), "pool must stay private");
}
