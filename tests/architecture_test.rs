//! Enforces the dependency rules in ARCHITECTURE.md by reading the sources:
//! `puzzle <- notation <- engines <- apps`, `render` and `generate` use only
//! `puzzle`, and engines never reach into each other.
//!
//! The scan resolves `crate::`, `super::` and `self::` paths (also inside
//! grouped `use crate::{..}` trees, with any spacing) to absolute module paths
//! and checks the owning module of each. Comments, string literals and
//! `#[cfg(test)]` items are ignored, but nothing else: code after a test
//! module is still checked, because only the test item itself is removed.

use std::fs;
use std::path::{Path, PathBuf};

/// (source directory, modules it may name), keyed as `puzzle` or `engines::three`.
const RULES: [(&str, &[&str]); 13] = [
    ("src/puzzle", &["puzzle"]),
    ("src/notation", &["puzzle", "notation"]),
    ("src/generate", &["puzzle", "generate"]),
    ("src/phases", &["puzzle", "phases"]),
    ("src/render", &["puzzle", "render"]),
    ("src/recover", &["puzzle", "recover"]),
    ("src/ranges", &["puzzle", "notation", "ranges"]),
    ("src/reduction", &["puzzle", "reduction"]),
    ("src/engines/three", &["puzzle", "engines::three"]),
    ("src/engines/two", &["puzzle", "notation", "engines::two"]),
    (
        "src/engines/five",
        &["puzzle", "reduction", "engines::five"],
    ),
    (
        "src/engines/four",
        &["puzzle", "reduction", "engines::four"],
    ),
    (
        "src/engines/dwalton",
        &["puzzle", "notation", "reduction", "engines::dwalton"],
    ),
];

/// Modules the applications may name through `cubacle::`.
const APP_ALLOWED: &[&str] = &[
    "puzzle",
    "notation",
    "generate",
    "phases",
    "render",
    "recover",
    "ranges",
    "engines::three",
    "engines::two",
    "engines::five",
    "engines::dwalton",
    "engines::four",
    "engines::pattern",
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

/// Removes comments and string literals, keeping everything else (and the
/// line structure irrelevant to the scan).
fn strip_comments_and_strings(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < chars.len() {
        match (chars[i], chars.get(i + 1)) {
            ('/', Some('/')) => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            ('/', Some('*')) => {
                let mut depth = 0;
                while i < chars.len() {
                    if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                        depth += 1;
                        i += 2;
                    } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                        depth -= 1;
                        i += 2;
                        if depth == 0 {
                            break;
                        }
                    } else {
                        i += 1;
                    }
                }
            }
            ('"', _) => {
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    i += if chars[i] == '\\' { 2 } else { 1 };
                }
                i += 1;
                out.push_str("\"\"");
            }
            (c, _) => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

/// Removes every item marked `#[cfg(test)]` (a braced block or a `;` item).
fn strip_test_items(source: &str) -> String {
    let mut text = source.to_owned();
    while let Some(start) = find_attr(&text, "#[cfg(test)]") {
        let after = start + "#[cfg(test)]".len();
        let rest = &text[after..];
        let end = match (rest.find('{'), rest.find(';')) {
            (Some(open), semi) if semi.is_none_or(|s| open < s) => {
                let mut depth = 0;
                let mut close = rest.len();
                for (offset, c) in rest[open..].char_indices() {
                    match c {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                close = open + offset + 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                close
            }
            (_, Some(semi)) => semi + 1,
            _ => rest.len(),
        };
        text.replace_range(start..after + end, "");
    }
    text
}

fn find_attr(text: &str, attr: &str) -> Option<usize> {
    // Tolerate spaces inside the attribute: `#[cfg( test )]`.
    let squeezed = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    let target = squeezed(attr);
    let mut from = 0;
    while let Some(pos) = text[from..].find("#[") {
        let start = from + pos;
        let close = text[start..].find(']')? + start + 1;
        if squeezed(&text[start..close]) == target {
            return Some(start);
        }
        from = start + 2;
    }
    None
}

/// A parsed path tree element: the idents of one path, possibly with a group.
fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Reads one `a::b::{c, d::e}` path starting at `pos`; pushes every full path.
fn read_tree(chars: &[char], pos: &mut usize, prefix: &[String], out: &mut Vec<Vec<String>>) {
    let skip = |pos: &mut usize| {
        while *pos < chars.len() && chars[*pos].is_whitespace() {
            *pos += 1;
        }
    };
    let mut path = prefix.to_vec();
    loop {
        skip(pos);
        if *pos < chars.len() && chars[*pos] == '{' {
            *pos += 1;
            loop {
                skip(pos);
                if *pos >= chars.len() {
                    return;
                }
                if chars[*pos] == '}' {
                    *pos += 1;
                    return;
                }
                if chars[*pos] == ',' {
                    *pos += 1;
                    continue;
                }
                let before = *pos;
                read_tree(chars, pos, &path, out);
                if *pos == before {
                    *pos += 1;
                }
            }
        }
        let start = *pos;
        while *pos < chars.len() && is_ident(chars[*pos]) {
            *pos += 1;
        }
        if *pos == start {
            if path.len() > prefix.len() {
                out.push(path);
            }
            return;
        }
        path.push(chars[start..*pos].iter().collect());
        skip(pos);
        if chars.get(*pos) == Some(&':') && chars.get(*pos + 1) == Some(&':') {
            *pos += 2;
        } else {
            // `Face as Alias`: the alias is a name, not a path.
            let word_end = *pos + 2;
            if chars.get(*pos) == Some(&'a')
                && chars.get(*pos + 1) == Some(&'s')
                && chars.get(word_end).is_some_and(|c| c.is_whitespace())
            {
                *pos = word_end;
                skip(pos);
                while *pos < chars.len() && is_ident(chars[*pos]) {
                    *pos += 1;
                }
            }
            out.push(path);
            return;
        }
    }
}

/// Every `crate`/`super`/`self` rooted path in `source`, as absolute module
/// segments. `module` is the module the source belongs to.
fn absolute_paths(source: &str, module: &[String]) -> Vec<Vec<String>> {
    let cleaned = strip_test_items(&strip_comments_and_strings(source));
    let chars: Vec<char> = cleaned.chars().collect();
    let mut found = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let word_start = i == 0 || !is_ident(chars[i - 1]);
        let root = ["crate", "super", "self"]
            .into_iter()
            .find(|w| word_start && chars[i..].iter().take(w.len()).collect::<String>() == *w);
        let Some(root) = root else {
            i += 1;
            continue;
        };
        let mut pos = i + root.len();
        if pos < chars.len() && is_ident(chars[pos]) {
            i += 1;
            continue;
        }
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }
        if chars.get(pos) != Some(&':') || chars.get(pos + 1) != Some(&':') {
            i = pos.max(i + 1);
            continue;
        }
        pos += 2;
        let mut trees = Vec::new();
        read_tree(&chars, &mut pos, &[root.to_owned()], &mut trees);
        for tree in trees {
            found.push(resolve(&tree, module));
        }
        i = pos.max(i + 1);
    }
    found
}

/// Resolves `crate::..`, `self::..` and `super::..` against `module`.
fn resolve(path: &[String], module: &[String]) -> Vec<String> {
    let mut base: Vec<String> = match path[0].as_str() {
        "crate" => Vec::new(),
        _ => module.to_vec(),
    };
    for segment in &path[1..] {
        match segment.as_str() {
            "super" => {
                base.pop();
            }
            "self" => {}
            other => base.push(other.to_owned()),
        }
    }
    if path[0] == "super" {
        // The leading `super` itself: one step up from the file's module.
        base = {
            let mut up = module.to_vec();
            up.pop();
            let mut rest: Vec<String> = Vec::new();
            for segment in &path[1..] {
                match segment.as_str() {
                    "super" => {
                        up.pop();
                    }
                    other => rest.push(other.to_owned()),
                }
            }
            up.extend(rest);
            up
        };
    }
    base
}

/// The owning module key of an absolute path: `puzzle`, `engines::three`, ...
fn key(path: &[String]) -> String {
    match path {
        [first, second, ..] if first == "engines" => format!("engines::{second}"),
        [first, ..] => first.clone(),
        [] => String::new(),
    }
}

/// The module path of a source file under `src/`.
fn module_of(file: &Path, src: &Path) -> Vec<String> {
    let relative = file.strip_prefix(src).expect("under src");
    let mut parts: Vec<String> = relative
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let last = parts.pop().expect("file name");
    let stem = last.trim_end_matches(".rs");
    if stem != "mod" && stem != "lib" && stem != "main" {
        parts.push(stem.to_owned());
    }
    parts
}

fn violations(source: &str, module: &[String], allowed: &[&str]) -> Vec<String> {
    absolute_paths(source, module)
        .into_iter()
        .map(|path| key(&path))
        .filter(|key| !allowed.contains(&key.as_str()))
        .collect()
}

#[test]
fn modules_only_use_what_the_dependency_rules_allow() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    for (dir, allowed) in RULES {
        let mut files = Vec::new();
        rust_files(&root.join(dir), &mut files);
        assert!(!files.is_empty(), "{dir} has no sources");
        for file in files {
            // A `#[cfg(test)]` module in its own file (it may use a real 3x3x3
            // solver where the engine itself takes any `dyn Solver`).
            if file.ends_with("five/solver/tests.rs") || file.ends_with("four/solver_tests.rs") {
                continue;
            }
            let source = fs::read_to_string(&file).expect("readable source");
            let module = module_of(&file, &src);
            let bad = violations(&source, &module, allowed);
            assert!(
                bad.is_empty(),
                "{} names {bad:?}, which {dir} may not depend on",
                file.display()
            );
        }
    }
}

#[test]
fn lib_rs_only_declares_modules() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib = strip_comments_and_strings(&fs::read_to_string(root.join("src/lib.rs")).unwrap());
    for line in lib.lines().map(str::trim).filter(|l| !l.is_empty()) {
        assert!(
            line.starts_with("pub mod ")
                || line.starts_with("mod ")
                || line.starts_with("pub use "),
            "lib.rs must stay thin, found: {line}"
        );
    }
    assert!(
        !lib.contains("mod pool"),
        "the pool belongs to engines::three"
    );
}

#[test]
fn the_worker_pool_stays_private_to_the_3x3_engine() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join("src/engines/three/pool.rs").exists());
    assert!(!root.join("src/pool.rs").exists());
    let engine = fs::read_to_string(root.join("src/engines/three/mod.rs")).expect("mod.rs");
    assert!(engine.contains("\nmod pool;"), "pool must stay private");
    // Nobody outside the 3x3 engine may name it, however they spell the path.
    let src = root.join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    for file in files {
        let module = module_of(&file, &src);
        if module.starts_with(&["engines".to_owned(), "three".to_owned()]) {
            continue;
        }
        let source = fs::read_to_string(&file).unwrap();
        for path in absolute_paths(&source, &module) {
            assert!(
                !path.iter().any(|segment| segment == "pool"),
                "{} reaches the pool through {path:?}",
                file.display()
            );
        }
    }
}

/// The shared reduction code keeps no global state either: tables, caches and
/// thread counts belong to the instance that uses them.
#[test]
fn the_reduction_code_keeps_no_global_state() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("src/reduction"), &mut files);
    assert!(!files.is_empty());
    for file in files {
        let source = strip_test_items(&strip_comments_and_strings(
            &fs::read_to_string(&file).unwrap(),
        ));
        assert!(
            !has_global_state(&source),
            "{} keeps global state; the caller owns it",
            file.display()
        );
    }
}

/// `static` items, `thread_local!` and `lazy_static`; `'static` lifetimes are fine.
fn has_global_state(source: &str) -> bool {
    source.replace("'static", "").contains("static ")
        || source.contains("thread_local")
        || source.contains("lazy_static")
}

#[test]
fn the_global_state_check_bites() {
    assert!(has_global_state("static X: u32 = 1;"));
    assert!(has_global_state("thread_local! { }"));
    assert!(has_global_state("lazy_static! { }"));
    assert!(!has_global_state("fn f(x: &'static str) {}"));
}

#[test]
fn the_5x5_engine_reaches_the_3x3_solver_only_through_the_trait() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let mut files = Vec::new();
    rust_files(&root.join("src/engines/five"), &mut files);
    assert!(!files.is_empty());
    for file in files {
        // `solver/tests.rs` is a `#[cfg(test)]` module that stubs the hand-off.
        if file.ends_with("solver/tests.rs") {
            continue;
        }
        let source = strip_test_items(&strip_comments_and_strings(
            &fs::read_to_string(&file).unwrap(),
        ));
        assert!(
            !source.contains("Min2PhaseSolver") && !source.contains("three::"),
            "{} names the 3x3 engine; inject a `dyn Solver` instead",
            file.display()
        );
        assert!(
            !has_global_state(&source),
            "{} keeps global state; the solver and its tables belong to an instance",
            file.display()
        );
        let _ = module_of(&file, &src);
    }
    let solver = fs::read_to_string(root.join("src/engines/five/solver.rs")).unwrap();
    assert!(solver.contains("Arc<dyn Solver + Send + Sync>"));
}

#[test]
fn the_4x4_engine_reaches_the_3x3_solver_only_through_the_trait() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("src/engines/four"), &mut files);
    assert!(!files.is_empty());
    for file in files {
        // `solver_tests.rs` is a `#[cfg(test)]` module that may use the real
        // 3x3x3 solver and stubs for the hand-off.
        if file.ends_with("solver_tests.rs") {
            continue;
        }
        let source = strip_test_items(&strip_comments_and_strings(
            &fs::read_to_string(&file).unwrap(),
        ));
        assert!(
            !source.contains("Min2PhaseSolver") && !source.contains("three::"),
            "{} names the 3x3 engine; inject a `dyn Solver` instead",
            file.display()
        );
        assert!(
            !has_global_state(&source),
            "{} keeps global state; the solver and its tables belong to an instance",
            file.display()
        );
    }
    let solver = fs::read_to_string(root.join("src/engines/four/solver.rs")).unwrap();
    assert!(solver.contains("Arc<dyn Solver + Send + Sync>"));
}

#[test]
fn applications_use_only_the_public_modules() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in [
        "src/main.rs",
        "src/icons.rs",
        "src/steps.rs",
        "src/rows.rs",
        "src/terminal/mod.rs",
        "crates/server/src/lib.rs",
        "crates/server/src/main.rs",
    ] {
        let source = strip_test_items(&strip_comments_and_strings(
            &fs::read_to_string(root.join(file)).unwrap(),
        ));
        let chars: Vec<char> = source.chars().collect();
        let mut pos = 0;
        let text: String = chars.iter().collect();
        while let Some(found) = text[pos..].find("cubacle") {
            let start = pos + found;
            pos = start + "cubacle".len();
            let before_ok = start == 0 || !is_ident(chars[start - 1]);
            let mut p = pos;
            while chars.get(p).is_some_and(|c| c.is_whitespace()) {
                p += 1;
            }
            if !before_ok || chars.get(p) != Some(&':') || chars.get(p + 1) != Some(&':') {
                continue;
            }
            p += 2;
            let mut trees = Vec::new();
            read_tree(&chars, &mut p, &["crate".to_owned()], &mut trees);
            for tree in trees {
                let path = resolve(&tree, &[]);
                assert!(
                    APP_ALLOWED.contains(&key(&path).as_str()),
                    "{file} names cubacle::{path:?}"
                );
                // An engine exports its solver from the module root; its
                // sub-modules (pool, tables, ...) are internal.
                assert!(
                    path.first().is_none_or(|first| first != "engines") || path.len() <= 3,
                    "{file} reaches into an engine through cubacle::{path:?}"
                );
            }
        }
    }
}

#[cfg(test)]
mod checker {
    //! The scan itself must catch what it claims to catch.
    use super::*;

    fn module(path: &str) -> Vec<String> {
        path.split("::").map(str::to_owned).collect()
    }

    fn bad(source: &str, at: &str, allowed: &[&str]) -> Vec<String> {
        violations(source, &module(at), allowed)
    }

    const THREE: &[&str] = &["puzzle", "engines::three"];

    #[test]
    fn super_super_into_a_sibling_engine() {
        let found = bad(
            "use super::super::two::PocketSolver;",
            "engines::three::engine",
            THREE,
        );
        assert_eq!(found, ["engines::two"]);
    }

    #[test]
    fn code_after_a_test_module_is_still_checked() {
        let source = "#[cfg(test)]\nmod tests { use crate::generate; }\nuse crate::notation::Foo;";
        assert_eq!(bad(source, "engines::three", THREE), ["notation"]);
        let only_tests = "#[cfg(test)]\nmod tests { use crate::generate; }";
        assert!(bad(only_tests, "engines::three", THREE).is_empty());
    }

    #[test]
    fn spaced_paths_are_resolved() {
        assert_eq!(
            bad(
                "use crate ::\n  generate::scramble;",
                "engines::three",
                THREE
            ),
            ["generate"]
        );
        assert_eq!(
            bad(
                "let x = crate  ::  notation :: parse_cube;",
                "engines::three",
                THREE
            ),
            ["notation"]
        );
    }

    #[test]
    fn grouped_imports_are_read_per_item() {
        assert!(bad("use crate::{puzzle::Face};", "engines::three", THREE).is_empty());
        assert!(bad(
            "use crate::{puzzle::Face as F, puzzle::*};",
            "engines::three",
            THREE
        )
        .is_empty());
        assert!(bad(
            "use crate::{puzzle::{Face, Move}, engines::three::Foo};",
            "engines::three",
            THREE
        )
        .is_empty());
        assert_eq!(
            bad(
                "use crate::{puzzle::Face, generate::scramble, engines::two::P};",
                "engines::three",
                THREE
            ),
            ["generate", "engines::two"]
        );
    }

    #[test]
    fn comments_and_strings_do_not_count() {
        let source =
            "// use crate::generate;\n/// [`crate::notation`]\nlet s = \"crate::generate::x\";";
        assert!(bad(source, "engines::three", THREE).is_empty());
    }

    #[test]
    fn in_module_super_paths_are_fine() {
        assert!(bad(
            "use super::cubie::*; use super::*;",
            "engines::three::engine",
            THREE
        )
        .is_empty());
    }
}
