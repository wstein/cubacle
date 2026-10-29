//! The `docs` task: run the examples, rewrite or check the marked blocks.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// One entry of `docs/examples.toml`.
struct Example {
    id: String,
    /// Page below `docs/`, for example `cli/solve.md`.
    page: String,
    /// Arguments of `cubacle`.
    args: Vec<String>,
    /// Fence language of the output block, default `text`.
    lang: String,
    /// Show standard error (before standard output) too.
    stderr: bool,
    /// JSON number keys whose value is replaced by 0 (they change from run to run).
    mask: Vec<String>,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

pub(crate) fn run(check: bool) -> ExitCode {
    match task(check) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

/// Returns false when `--check` found a stale block.
fn task(check: bool) -> Result<bool, String> {
    let root = root();
    let text = std::fs::read_to_string(root.join("docs/examples.toml"))
        .map_err(|e| format!("docs/examples.toml: {e}"))?;
    let examples = parse(&text)?;
    let bin = build(&root)?;
    let mut pages: Vec<&str> = examples.iter().map(|e| e.page.as_str()).collect();
    pages.sort();
    pages.dedup();
    let mut fresh = true;
    for page in pages {
        let path = root.join("docs").join(page);
        let old = std::fs::read_to_string(&path).map_err(|e| format!("{page}: {e}"))?;
        let mut new = old.clone();
        for ex in examples.iter().filter(|e| e.page == page) {
            let body = block(ex, &execute(&bin, ex)?);
            new = replace(&new, &ex.id, &body).map_err(|e| format!("{page}: {e}"))?;
        }
        if new == old {
            continue;
        }
        if check {
            fresh = false;
            eprintln!("docs/{page} is stale:");
            eprint!("{}", diff(&old, &new));
        } else {
            std::fs::write(&path, new).map_err(|e| format!("{page}: {e}"))?;
            println!("updated docs/{page}");
        }
    }
    if check && !fresh {
        eprintln!("run `cargo xtask docs` and commit the result");
    }
    Ok(fresh)
}

/// Builds the release binary (or takes `CUBACLE_BIN`) and returns its path.
fn build(root: &Path) -> Result<PathBuf, String> {
    if let Some(p) = std::env::var_os("CUBACLE_BIN") {
        return Ok(PathBuf::from(p));
    }
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let status = Command::new(cargo)
        .args(["build", "--release", "--bin", "cubacle"])
        .current_dir(root)
        .status()
        .map_err(|e| format!("cargo build: {e}"))?;
    if !status.success() {
        return Err("cargo build --release --bin cubacle failed".into());
    }
    let target = std::env::var_os("CARGO_TARGET_DIR").map_or(root.join("target"), PathBuf::from);
    Ok(target.join("release/cubacle"))
}

fn parse(text: &str) -> Result<Vec<Example>, String> {
    let table: toml::Table = text
        .parse()
        .map_err(|e| format!("docs/examples.toml: {e}"))?;
    let list = table
        .get("example")
        .and_then(|v| v.as_array())
        .ok_or("docs/examples.toml: no [[example]] entries")?;
    let strings = |t: &toml::Table, key: &str| -> Vec<String> {
        let items = t.get(key).and_then(|v| v.as_array());
        items
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    };
    let mut out: Vec<Example> = Vec::new();
    for item in list {
        let t = item
            .as_table()
            .ok_or("docs/examples.toml: [[example]] is not a table")?;
        let get = |k: &str| t.get(k).and_then(|v| v.as_str()).map(String::from);
        let id = get("id").ok_or("docs/examples.toml: an example has no id")?;
        let page = get("page").ok_or_else(|| format!("example {id}: no page"))?;
        if out.iter().any(|e| e.id == id) {
            return Err(format!("example {id} is listed twice"));
        }
        out.push(Example {
            args: strings(t, "args"),
            lang: get("lang").unwrap_or_else(|| "text".into()),
            stderr: t.get("stderr").and_then(|v| v.as_bool()).unwrap_or(false),
            mask: strings(t, "mask"),
            id,
            page,
        });
    }
    Ok(out)
}

fn execute(bin: &Path, ex: &Example) -> Result<String, String> {
    let out = Command::new(bin)
        .args(&ex.args)
        .output()
        .map_err(|e| format!("{}: {e}", bin.display()))?;
    if !out.status.success() {
        return Err(format!(
            "example {}: the command failed ({})",
            ex.id, out.status
        ));
    }
    let mut text = String::new();
    if ex.stderr {
        text.push_str(&String::from_utf8_lossy(&out.stderr));
    }
    text.push_str(&String::from_utf8_lossy(&out.stdout));
    for key in &ex.mask {
        text = mask_number(&text, key);
    }
    Ok(text)
}

/// Replaces the digits after `"key":` by 0.
fn mask_number(text: &str, key: &str) -> String {
    let needle = format!("\"{key}\":");
    let mut out = String::new();
    let mut rest = text;
    while let Some(i) = rest.find(&needle) {
        let end = i + needle.len();
        out.push_str(&rest[..end]);
        let tail = &rest[end..];
        let digits = tail.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 {
            out.push('0');
        }
        rest = &tail[digits..];
    }
    out.push_str(rest);
    out
}

fn quote(arg: &str) -> String {
    let plain = |c: char| c.is_ascii_alphanumeric() || "_@%+=:,./-".contains(c);
    if !arg.is_empty() && arg.chars().all(plain) {
        arg.to_string()
    } else if arg.contains(['"', '$', '`', '\\']) {
        format!("'{}'", arg.replace('\'', "'\\''"))
    } else {
        format!("\"{arg}\"")
    }
}

/// The text between the markers: the command and its output.
fn block(ex: &Example, output: &str) -> String {
    let cmd: Vec<String> = std::iter::once("cubacle".to_string())
        .chain(ex.args.iter().map(|a| quote(a)))
        .collect();
    format!(
        "\n```sh\n{}\n```\n\n```{}\n{}\n```\n",
        cmd.join(" "),
        ex.lang,
        output.trim_end()
    )
}

/// Replaces the body of the block `id` in `page`.
fn replace(page: &str, id: &str, body: &str) -> Result<String, String> {
    let open = format!("<!-- example:{id} -->");
    let close = "<!-- /example -->";
    let start = page
        .find(&open)
        .ok_or_else(|| format!("marker {open} not found"))?
        + open.len();
    let len = page[start..]
        .find(close)
        .ok_or_else(|| format!("{open} has no {close}"))?;
    Ok(format!(
        "{}{}{}",
        &page[..start],
        body,
        &page[start + len..]
    ))
}

/// A line diff of the changed middle part: `-` old lines, `+` new lines.
fn diff(old: &str, new: &str) -> String {
    let (a, b): (Vec<&str>, Vec<&str>) = (old.lines().collect(), new.lines().collect());
    let head = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let tail = a[head..]
        .iter()
        .rev()
        .zip(b[head..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let mut out = String::new();
    for l in &a[head..a.len() - tail] {
        out.push_str(&format!("-{l}\n"));
    }
    for l in &b[head..b.len() - tail] {
        out.push_str(&format!("+{l}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ex(args: &[&str]) -> Example {
        Example {
            id: "x".into(),
            page: "p.md".into(),
            args: args.iter().map(|s| s.to_string()).collect(),
            lang: "text".into(),
            stderr: false,
            mask: vec![],
        }
    }

    #[test]
    fn masks_numbers() {
        let s = r#"{"a":1,"elapsedMicros":4821,"b":2}"#;
        assert_eq!(
            mask_number(s, "elapsedMicros"),
            r#"{"a":1,"elapsedMicros":0,"b":2}"#
        );
    }

    #[test]
    fn quotes_arguments() {
        let b = block(&ex(&["solve", "R U", "-s"]), "out\n");
        assert_eq!(
            b,
            "\n```sh\ncubacle solve \"R U\" -s\n```\n\n```text\nout\n```\n"
        );
    }

    #[test]
    fn replaces_only_the_block() {
        let page = "a\n<!-- example:x -->\nold\n<!-- /example -->\nb\n";
        let got = replace(page, "x", "\nnew\n").unwrap();
        assert_eq!(got, "a\n<!-- example:x -->\nnew\n<!-- /example -->\nb\n");
        assert!(replace(page, "y", "").is_err());
    }

    #[test]
    fn diff_shows_changed_lines() {
        assert_eq!(diff("a\nb\nc\n", "a\nx\nc\n"), "-b\n+x\n");
    }

    #[test]
    fn parses_entries() {
        let t = "[[example]]\nid = \"a\"\npage = \"p.md\"\nargs = [\"solve\"]\n";
        let v = parse(t).unwrap();
        assert_eq!(
            (v[0].id.as_str(), v[0].lang.as_str(), v[0].stderr),
            ("a", "text", false)
        );
    }
}
