//! Differential harness for the Orbit64 V2 picture facelet reader.
//!
//! It compares the Rust reader of this repository with the Flix reference
//! (`Orbit64.V2.Facelets.encode` and `decode` of wstein/flix-orbit64) on
//! mutated picture facelet strings. The mutations are drawn from the vectors of
//! `tests/fixtures/orbit64_v2_picture_facelets.txt` by a fixed seed (see
//! `cases.rs`).
//!
//! Per case both sides write one line, `index|ok|token|facelets` when `encode`
//! accepts the string and `decode` of that token answers `facelets`, or
//! `index|err|text` with the error text of the failing call. The comparison is
//! line by line and exact, except that characters outside ASCII are written as
//! `?` on both sides, as Flix stdout may lose them. It compares the accepted
//! tokens, the facelets read back and the error texts of the generated
//! mutations; it does not compare any input that is not generated.
//!
//!   cases [--seed S] [--count N]       write the cases (`index|entry|facelets`)
//!   compare RUST_OUT FLIX_OUT          exit 1 on any differing line
//!   run [--seed S] [--count N] [--rust-out FILE] [--flix-dir DIR]
//!                                      Rust always; Flix only with --flix-dir
//!                                      (a flix-orbit64 checkout, needs the
//!                                      network once for its JDK)
//!   check-sample                       the committed sample is the output of
//!                                      `cases --seed 1 --count 200`
//!   provenance FLIX_DIR                the fixture's recorded origin is true
//!
//! `cases`, `check-sample` and `run` without `--flix-dir` need no Flix and no
//! network. A larger run is `run --seed 1 --count 9456 --flix-dir
//! PATH/TO/flix-orbit64`; it is not part of any automatic check.

mod cases;

use cases::{answer, cases_text, read_cases, PICTURES, SAMPLE, SAMPLE_COUNT, SAMPLE_SEED};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::{env, fs};

const PROVENANCE: &str =
    include_str!("../../tests/fixtures/orbit64_v2_picture_facelets.provenance");

/// The Flix program: a list of cases is a right-nested expression, which the Flix
/// parser reads recursively, so the cases are split into functions of `CHUNK`.
const FLIX_MAIN: &str = r#"
def answer(i: Int32, id: Int32, text: String): String =
    match Orbit64.V2.Facelets.encode(id, text) {
        case Err(why) => "${i}|err|${why}"
        case Ok(token) => match Orbit64.V2.Facelets.decode(token) {
            case Ok((_, back)) => "${i}|ok|${token}|${back}"
            case Err(why) => "${i}|err|${why}"
        }
    }

def main(): Unit \ IO =
    PictureDiffCases.all() |> List.forEach(match (i, id, text) -> println(answer(i, id, text)))

mod PictureDiffCases {
    pub def all(): List[(Int32, Int32, String)] =
@ALL@
@CHUNKS@
}
"#;

const CHUNK: usize = 200;

type Failure = String;

fn lines(text: &str) -> Vec<String> {
    text.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| {
            l.chars()
                .map(|c| if c.is_ascii() { c } else { '?' })
                .collect()
        })
        .collect()
}

/// Prints the first differences; returns the number of differing lines.
fn compare(rust: &str, flix: &str) -> usize {
    let (rust, flix) = (lines(rust), lines(flix));
    let total = rust.len().max(flix.len());
    let mut bad = 0;
    for k in 0..total {
        let a = rust.get(k).map_or("<missing>", String::as_str);
        let b = flix.get(k).map_or("<missing>", String::as_str);
        if a != b {
            bad += 1;
            if bad <= 5 {
                eprintln!("differs at line {k}:\n  rust: {a}\n  flix: {b}");
            }
        }
    }
    println!("compared {total} answer lines: {bad} disagreements");
    bad
}

/// The answers of the Rust reader to a case file, and (accepted, refused).
fn rust_answers(case_file: &str) -> (String, usize, usize) {
    let (mut out, mut accepted, mut refused) = (String::new(), 0, 0);
    for (index, id, facelets) in read_cases(case_file) {
        let line = answer(index, id, facelets);
        if line.split('|').nth(1) == Some("ok") {
            accepted += 1;
        } else {
            refused += 1;
        }
        out.push_str(&line);
        out.push('\n');
    }
    (out, accepted, refused)
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), Failure> {
    fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    for entry in fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        if name == ".git" {
            continue;
        }
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        let target = to.join(&name);
        if kind.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), &target).map_err(|e| format!("{}: {e}", target.display()))?;
        }
    }
    Ok(())
}

/// Runs the reference in a copy of the checkout; the checkout stays untouched.
/// The copy is deleted before this returns.
fn run_flix(case_file: &str, flix_dir: &Path) -> Result<String, Failure> {
    let work: PathBuf =
        env::temp_dir().join(format!("orbit64-picture-diff-{}", std::process::id()));
    let result = run_flix_in(case_file, flix_dir, &work);
    let _ = fs::remove_dir_all(&work);
    result
}

fn run_flix_in(case_file: &str, flix_dir: &Path, work: &Path) -> Result<String, Failure> {
    copy_tree(flix_dir, work)?;
    let cases = read_cases(case_file);
    let chunks: Vec<String> = cases
        .chunks(CHUNK)
        .enumerate()
        .map(|(k, part)| {
            let items: String = part
                .iter()
                .map(|(i, id, text)| format!("        ({i}, {id}, \"{text}\") :: \n"))
                .collect();
            format!("    def chunk{k}(): List[(Int32, Int32, String)] =\n{items}        Nil\n")
        })
        .collect();
    let all = (0..chunks.len())
        .map(|k| format!("        chunk{k}()"))
        .collect::<Vec<_>>()
        .join(" ::: \n");
    let all = if chunks.is_empty() {
        "        Nil".to_string()
    } else {
        all + "\n        ::: Nil"
    };
    let source = FLIX_MAIN
        .replace("@ALL@", &all)
        .replace("@CHUNKS@", &chunks.join("\n"));
    fs::write(work.join("src").join("PictureDiffMain.flix"), source).map_err(|e| e.to_string())?;
    let done = Command::new(work.join("flixw"))
        .arg("run")
        .current_dir(work)
        // Flix writes its answers to stdout: UTF-8 on both ends, whatever the locale.
        .env("LC_ALL", "C.UTF-8")
        .env(
            "JAVA_TOOL_OPTIONS",
            "-Dfile.encoding=UTF-8 -Dstdout.encoding=UTF-8",
        )
        .output()
        .map_err(|e| format!("flixw: {e}"))?;
    if !done.status.success() {
        return Err(format!(
            "flixw run failed:\n{}",
            String::from_utf8_lossy(&done.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&done.stdout).into_owned())
}

fn sha256(data: &[u8]) -> String {
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn recorded(key: &str) -> &'static str {
    PROVENANCE
        .lines()
        .find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
        .unwrap_or_else(|| panic!("no {key} in the provenance file"))
}

fn check_provenance(flix_dir: &str) -> Result<bool, Failure> {
    let spec = format!("{}:{}", recorded("flix_commit"), recorded("input_path"));
    let shown = Command::new("git")
        .args(["-C", flix_dir, "show", &spec])
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !shown.status.success() {
        return Err(format!(
            "git show {spec}: {}",
            String::from_utf8_lossy(&shown.stderr)
        ));
    }
    let mut problems = Vec::new();
    let input = sha256(&shown.stdout);
    if input != recorded("input_sha256") {
        problems.push(format!(
            "input sha256 is {input}, recorded {}",
            recorded("input_sha256")
        ));
    }
    let output = sha256(PICTURES.as_bytes());
    if output != recorded("output_sha256") {
        problems.push(format!(
            "fixture sha256 is {output}, recorded {}",
            recorded("output_sha256")
        ));
    }
    for p in &problems {
        eprintln!("{p}");
    }
    println!(
        "provenance: {}",
        if problems.is_empty() { "ok" } else { "differs" }
    );
    Ok(problems.is_empty())
}

/// The value of `--name` in the arguments.
fn option<'a>(args: &'a [String], name: &str) -> Result<Option<&'a str>, Failure> {
    match args.iter().position(|a| a == name) {
        None => Ok(None),
        Some(at) => args
            .get(at + 1)
            .map(|v| Some(v.as_str()))
            .ok_or_else(|| format!("{name} needs a value")),
    }
}

fn number<T: std::str::FromStr>(args: &[String], name: &str, default: T) -> Result<T, Failure> {
    match option(args, name)? {
        None => Ok(default),
        Some(v) => v.parse().map_err(|_| format!("{name}: not a number: {v}")),
    }
}

fn run(args: &[String]) -> Result<bool, Failure> {
    let cmd = args.first().map(String::as_str).unwrap_or("");
    let rest = args.get(1..).unwrap_or(&[]);
    match cmd {
        "cases" | "run" => {
            let known = ["--seed", "--count", "--rust-out", "--flix-dir"];
            if let Some(bad) = rest.iter().step_by(2).find(|a| !known.contains(&a.as_str())) {
                return Err(format!("unknown argument {bad}"));
            }
            let seed = number(rest, "--seed", SAMPLE_SEED)?;
            let count = number(rest, "--count", SAMPLE_COUNT)?;
            let text = cases_text(seed, count);
            if cmd == "cases" {
                print!("{text}");
                return Ok(true);
            }
            let (rust, accepted, refused) = rust_answers(&text);
            println!("rust: {} cases, {accepted} accepted, {refused} refused", accepted + refused);
            if let Some(path) = option(rest, "--rust-out")? {
                fs::write(path, &rust).map_err(|e| format!("{path}: {e}"))?;
            }
            let Some(dir) = option(rest, "--flix-dir")? else {
                println!("flix: not run (no --flix-dir)");
                return Ok(true);
            };
            let flix = run_flix(&text, Path::new(dir))?;
            Ok(compare(&rust, &flix) == 0)
        }
        "compare" if rest.len() == 2 => {
            let read = |p: &String| fs::read_to_string(p).map_err(|e| format!("{p}: {e}"));
            Ok(compare(&read(&rest[0])?, &read(&rest[1])?) == 0)
        }
        "check-sample" => {
            let same = SAMPLE == cases_text(SAMPLE_SEED, SAMPLE_COUNT);
            println!("sample: {}", if same { "ok" } else { "differs from the generator" });
            Ok(same)
        }
        "provenance" if rest.len() == 1 => check_provenance(&rest[0]),
        _ => Err("usage: orbit64_v2_picture_diff cases|compare|run|check-sample|provenance (see the header of tools/orbit64_v2_picture_diff/main.rs)".into()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(&args) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(why) => {
            eprintln!("{why}");
            ExitCode::from(2)
        }
    }
}
