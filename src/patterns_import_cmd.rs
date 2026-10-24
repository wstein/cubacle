//! `patterns import`: read a tab-separated pattern table, replay every
//! sequence on a solved cube and write the verified patterns to the user's
//! data directory in our own format.

use std::path::{Path, PathBuf};

use clap::Args;
use cubacle::metrics::{count, Metric};
use cubacle::notation::parse_expression;
use cubacle::patterns::{
    cycles_of_moves, decode_text, import_tab, read_plain, write_imported, ImportOptions, ImportRow,
};
use cubacle::puzzle::{MoveSequence, PuzzleKind};

#[derive(Args)]
pub struct ImportArgs {
    /// A local pattern table (the `PatternExportTab.txt` layout). A URL is
    /// not fetched: download it yourself with `curl -o FILE URL` first
    file: String,
    /// Put in front of every imported id and name
    #[arg(long, default_value = "")]
    name_prefix: String,
    /// Two-letter catalogue tag of the ids written as `tag:code`
    #[arg(long, default_value = "ct")]
    source: String,
    /// Check and report only, including the counts each line claims; write nothing
    #[arg(long)]
    dry_run: bool,
    /// Write here instead of the data directory
    #[arg(long, value_name = "FILE")]
    out: Option<PathBuf>,
}

/// The directory for imported data: `CUBACLE_CACHE_DIR`, `$XDG_CACHE_HOME/cubacle`
/// or `~/.cache/cubacle`, plus `patterns` (as the table caches do).
pub fn default_data_dir() -> Result<PathBuf, String> {
    let var = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
    let base = if let Some(dir) = var("CUBACLE_CACHE_DIR") {
        PathBuf::from(dir)
    } else if let Some(dir) = var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|d| d.is_absolute())
    {
        dir.join("cubacle")
    } else if let Some(home) = var("HOME") {
        PathBuf::from(home).join(".cache").join("cubacle")
    } else {
        return Err("no cache directory: set CUBACLE_CACHE_DIR".to_owned());
    };
    Ok(base.join("patterns"))
}

/// The cube size named by a path or URL component such as `4xRevenge`.
pub fn size_from_name(text: &str) -> Option<usize> {
    text.split(['/', '\\']).rev().find_map(|part| {
        let (digits, rest) = part.split_once('x')?;
        let size: usize = digits.parse().ok()?;
        (!rest.is_empty() && (2..=7).contains(&size)).then_some(size)
    })
}

/// The move reader of the importer: the plain face-turn spelling first, then
/// our expression reader (groups, repeats, wide and slice turns).
pub fn reader(kind: PuzzleKind) -> impl Fn(&str) -> Result<MoveSequence, String> {
    move |text| {
        read_plain(text)
            .or_else(|plain| parse_expression(&dash_to_prime(text), kind).map_err(|_| plain))
    }
}

/// `-` after a move or a closing bracket is the inverse (`(R U)-`, `R2-`);
/// a dash before a digit is a layer range and stays.
fn dash_to_prime(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (i, &c) in chars.iter().enumerate() {
        let after_move = i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == ')');
        let before_digit = chars.get(i + 1).is_some_and(char::is_ascii_digit);
        if c == '-' && after_move && !before_digit {
            out.push('\'');
        } else {
            out.push(c);
        }
    }
    out
}

/// The counts a row claims against our own: block, face and quarter turns
/// and the order. One line per disagreement.
fn check_counts(rows: &[ImportRow], kind: PuzzleKind) -> Vec<String> {
    let mut out = Vec::new();
    for row in rows {
        let order = cycles_of_moves(kind, &row.moves)
            .ok()
            .and_then(|c| c.order)
            .map(|o| o.whole);
        let ours = [
            ("block turns", count(&row.moves, kind, Metric::Btm) as u128),
            ("face turns", count(&row.moves, kind, Metric::Htm) as u128),
            (
                "quarter turns",
                count(&row.moves, kind, Metric::Qtm) as u128,
            ),
        ];
        let mut mine: Vec<(&str, Option<u128>)> =
            ours.iter().map(|(n, v)| (*n, Some(*v))).collect();
        mine.push(("order", order));
        for (index, (name, value)) in mine.iter().enumerate() {
            let text = row.claimed[index].trim().trim_end_matches('*');
            let Ok(claimed) = text.parse::<u128>() else {
                continue;
            };
            if Some(claimed) != *value {
                let ours = value.map_or("?".to_owned(), |v| v.to_string());
                out.push(format!(
                    "line {} {}: claims {claimed} {name}, counted {ours}",
                    row.line, row.code
                ));
            }
        }
    }
    out
}

/// `size` is the global `--size`, else the size is read from a folder name.
pub fn run(args: ImportArgs, cache_dir: &Path, size: Option<usize>) -> Result<(), String> {
    if args.file.contains("://") {
        return Err(format!(
            "cubacle does not download; fetch it yourself with: curl -o FILE {}",
            args.file
        ));
    }
    let size = size.or_else(|| size_from_name(&args.file));
    let size = size.ok_or("cannot tell the cube size: pass --size N")?;
    let kind = crate::kind_of(size)?;
    let bytes = std::fs::read(&args.file).map_err(|e| format!("{}: {e}", args.file))?;
    let label = Path::new(&args.file)
        .file_name()
        .map_or(args.file.clone(), |n| n.to_string_lossy().into_owned());
    let options = ImportOptions {
        size,
        name_prefix: args.name_prefix,
        origin: label.clone(),
        source: args.source,
    };
    let got = import_tab(&decode_text(&bytes), &options, &reader(kind));
    let report = &got.report;
    for problem in &report.problems {
        eprintln!("{}:{}: {}", args.file, problem.line, problem.message);
    }
    println!(
        "{size}x{size}x{size}: {} lines, {} verified by replay, {} patterns, {} problems",
        report.lines,
        report.accepted,
        got.patterns.len(),
        report.problems.len()
    );
    if args.dry_run {
        let mismatches = check_counts(&got.rows, kind);
        for line in &mismatches {
            println!("{line}");
        }
        println!(
            "counts: {} of {} claimed values differ from ours",
            mismatches.len(),
            got.rows.len() * 4
        );
        return Ok(());
    }
    let target = args.out.unwrap_or_else(|| {
        let folder = Path::new(&args.file)
            .parent()
            .and_then(Path::file_name)
            .map_or(String::new(), |n| format!("{}-", n.to_string_lossy()));
        cache_dir.join("imported").join(format!("{folder}{label}"))
    });
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    std::fs::write(&target, write_imported(&got.patterns, &label))
        .map_err(|e| format!("{}: {e}", target.display()))?;
    println!("wrote {}", target.display());
    println!("list it under [patterns] files in cubacle.toml to use it");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_come_from_folder_names() {
        assert_eq!(size_from_name("db/3xRubik/File.txt"), Some(3));
        assert_eq!(size_from_name("https://h/x/7xCube/F.txt"), Some(7));
        assert_eq!(size_from_name("File.txt"), None);
        assert_eq!(size_from_name("9xBig/F"), None);
    }

    #[test]
    fn the_reader_falls_back_to_expressions() {
        let read = reader(PuzzleKind::Cube3);
        assert_eq!(read("R- U2").unwrap().to_string(), "R' U2");
        assert_eq!(read("(R U)2").unwrap().to_string(), "R U R U");
        assert_eq!(read("(R U)-").unwrap().to_string(), "U' R'");
        assert!(read("Q").is_err());
    }
}
