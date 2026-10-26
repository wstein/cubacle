//! The local-table part of `kind_rates`: patternness, kinds and recognition of
//! the patterns of tab-separated tables the caller has on disk. No table is
//! part of this repository; the paths come from the environment variable
//! `CUBACLE_LOCAL_TABLES` (files separated by `:`, the cube size is read from
//! a name such as `4xRevenge`) and only aggregate numbers are printed.

use std::path::PathBuf;

use cubacle::notation::parse_expression;
use cubacle::patterns::{
    self, decode_bytes, detect_with, import_tab, read_plain, ImportOptions, Measures, Pattern,
    PatternKind,
};
use cubacle::puzzle::{Facelets, PuzzleKind};

/// The imported patterns of one table and what could not be read.
pub struct Table {
    pub size: usize,
    pub lines: usize,
    pub unreadable: usize,
    pub patterns: Vec<Pattern>,
}

fn size_of(path: &str) -> Option<usize> {
    path.split('/').rev().find_map(|part| {
        let (digits, rest) = part.split_once('x')?;
        let size: usize = digits.parse().ok()?;
        (!rest.is_empty() && (2..=7).contains(&size)).then_some(size)
    })
}

/// `-` after a move or a closing bracket is the inverse; before a digit it
/// is a layer range.
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

/// The tables named by `CUBACLE_LOCAL_TABLES`, read like `patterns import`.
pub fn tables() -> Vec<Table> {
    let Some(list) = std::env::var_os("CUBACLE_LOCAL_TABLES") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for path in std::env::split_paths(&list).collect::<Vec<PathBuf>>() {
        let name = path.to_string_lossy().into_owned();
        let Some(size) = size_of(&name) else {
            eprintln!("{name}: no cube size in the name, skipped");
            continue;
        };
        let kind = PuzzleKind::of_size(size).expect("size 2 to 7");
        let Ok(bytes) = std::fs::read(&path) else {
            eprintln!("{name}: cannot read, skipped");
            continue;
        };
        let (text, _) = decode_bytes(&bytes);
        let options = ImportOptions {
            size,
            name_prefix: String::new(),
            origin: "local".to_owned(),
            source: "ct".to_owned(),
        };
        let read = move |t: &str| {
            read_plain(t)
                .or_else(|plain| parse_expression(&dash_to_prime(t), kind).map_err(|_| plain))
        };
        let got = import_tab(&text, &options, &read);
        out.push(Table {
            size,
            lines: got.report.lines,
            unreadable: got.report.problems.len(),
            patterns: got.patterns,
        });
    }
    out.sort_by_key(|t| t.size);
    out
}

/// Whether a pattern belongs to the library part (about 70 %), by a hash of
/// its id; the rest is held out.
fn in_library(id: &str) -> bool {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in id.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    (h >> 20) % 10 < 7
}

fn pct(count: usize, total: usize) -> f64 {
    if total == 0 {
        f64::NAN
    } else {
        100.0 * count as f64 / total as f64
    }
}

const BANDS: [(&str, u8, u8); 8] = [
    ("100", 100, 100),
    ("98-99", 98, 99),
    ("95-97", 95, 97),
    ("90-94", 90, 94),
    ("80-89", 80, 89),
    ("60-79", 60, 79),
    ("40-59", 40, 59),
    ("<40", 0, 39),
];

fn median(sorted: &[u8]) -> f64 {
    match sorted.len() {
        0 => f64::NAN,
        n if n % 2 == 1 => f64::from(sorted[n / 2]),
        n => (f64::from(sorted[n / 2 - 1]) + f64::from(sorted[n / 2])) / 2.0,
    }
}

fn print_scores(label: &str, scores: &mut [u8]) {
    scores.sort_unstable();
    let n = scores.len();
    let mean = scores.iter().map(|&s| f64::from(s)).sum::<f64>() / n as f64;
    let cells: Vec<String> = BANDS
        .iter()
        .map(|&(_, lo, hi)| {
            format!(
                "{:.1}",
                pct(scores.iter().filter(|&&s| s >= lo && s <= hi).count(), n)
            )
        })
        .collect();
    println!(
        "{label}\t{n}\t{}\t{mean:.1}\t{:.1}\t{:.1}\t{:.1}",
        cells.join("\t"),
        median(scores),
        pct(scores.iter().filter(|&&s| s >= 95).count(), n),
        pct(scores.iter().filter(|&&s| s >= 90).count(), n)
    );
}

/// Patternness bands, kinds and the acceptance goal per size.
pub fn report_measures(tables: &[Table]) {
    println!("\n== patternness bands (% of patterns)");
    let heads: Vec<&str> = BANDS.iter().map(|b| b.0).collect();
    println!(
        "size/set\tpatterns\t{}\tmean\tmedian\t>=95 %\t>=90 %",
        heads.join("\t")
    );
    let mut kind_lines = Vec::new();
    for table in tables {
        let kind = PuzzleKind::of_size(table.size).expect("size 2 to 7");
        let built: Vec<Measures> = patterns::all()
            .iter()
            .filter_map(|p| p.state(kind))
            .map(|s| patterns::measures(&s))
            .collect();
        let imported: Vec<Measures> = table
            .patterns
            .iter()
            .filter_map(|p| p.state(kind))
            .map(|s| patterns::measures(&s))
            .collect();
        print_scores(
            &format!("{} built-in", table.size),
            &mut built.iter().map(|m| m.patternness).collect::<Vec<_>>(),
        );
        print_scores(
            &format!("{} imported", table.size),
            &mut imported.iter().map(|m| m.patternness).collect::<Vec<_>>(),
        );
        let mut all: Vec<u8> = built
            .iter()
            .chain(&imported)
            .map(|m| m.patternness)
            .collect();
        print_scores(&format!("{} both", table.size), &mut all);
        println!(
            "{}\tlines {}, unreadable lines {} ({:.1} %)",
            table.size,
            table.lines,
            table.unreadable,
            pct(table.unreadable, table.lines)
        );
        kind_lines.push(kind_report(table.size, &imported));
    }
    println!("\n== kinds among the imported patterns (% of patterns) and the acceptance goal");
    println!("size\tpatterns\tSOLVED\tFEW\tMOSTLY\tSYM\tFACE\tIRREG\tpatternness>=90\tkind not IRREG\tboth ways");
    for line in kind_lines {
        println!("{line}");
    }
}

fn kind_report(size: usize, set: &[Measures]) -> String {
    let n = set.len();
    let share = |k: PatternKind| pct(set.iter().filter(|m| m.kind() == k).count(), n);
    let kinds: Vec<String> = PatternKind::ALL
        .iter()
        .map(|&k| format!("{:.1}", share(k)))
        .collect();
    let high = |m: &Measures| m.patternness >= 90;
    let some_kind = |m: &Measures| m.kind() != PatternKind::Irreg;
    format!(
        "{size}\t{n}\t{}\t{:.1}\t{:.1}\t{:.1}",
        kinds.join("\t"),
        pct(set.iter().filter(|m| high(m)).count(), n),
        pct(set.iter().filter(|m| some_kind(m)).count(), n),
        pct(set.iter().filter(|m| high(m) || some_kind(m)).count(), n)
    )
}

struct Spread(Vec<u8>);

impl Spread {
    fn line(&mut self, label: &str) {
        self.0.sort_unstable();
        let n = self.0.len();
        let at = |lo: u8| pct(self.0.iter().filter(|&&s| s >= lo).count(), n);
        println!(
            "{label}\t{n}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{}\t{}",
            at(100),
            at(95),
            at(90),
            at(80),
            at(70),
            median(&self.0),
            self.0.first().copied().unwrap_or(0)
        );
    }
}

/// Recognition: held-out imported patterns against built-in plus the library
/// share of the imported ones, and the baseline of random scrambles.
pub fn report_identify(tables: &[Table], scramble: &mut dyn FnMut(PuzzleKind, usize) -> Facelets) {
    println!(
        "\n== identify: held-out patterns (30 %) against built-in + 70 % of the imported ones"
    );
    println!("size/set\tstates\t=100 %\t>=95 %\t>=90 %\t>=80 %\t>=70 %\tmedian\tlowest");
    for table in tables {
        let kind = PuzzleKind::of_size(table.size).expect("size 2 to 7");
        let library: Vec<Pattern> = table
            .patterns
            .iter()
            .filter(|p| in_library(&p.catalog))
            .cloned()
            .collect();
        let held: Vec<&Pattern> = table
            .patterns
            .iter()
            .filter(|p| !in_library(&p.catalog))
            .collect();
        let mut exact = 0;
        let mut best = Spread(Vec::new());
        let mut best_unmatched = Spread(Vec::new());
        for p in &held {
            let Some(state) = p.state(kind) else { continue };
            let report = detect_with(&state, &library, 1);
            let percent = report.closest.first().map_or(0, |c| c.percent);
            best.0.push(percent);
            if report.matches.is_empty() {
                best_unmatched.0.push(percent);
            } else {
                exact += 1;
            }
        }
        println!(
            "{}\tlibrary {} imported + built-in, held out {}, exact match {} ({:.1} %)",
            table.size,
            library.len(),
            held.len(),
            exact,
            pct(exact, held.len())
        );
        best.line(&format!("{} best closest, all held out", table.size));
        best_unmatched.line(&format!("{} best closest, no exact match", table.size));
        for length in [8, 12] {
            let mut spread = Spread(
                (0..50)
                    .map(|_| {
                        let state = scramble(kind, length);
                        detect_with(&state, &library, 1)
                            .closest
                            .first()
                            .map_or(0, |c| c.percent)
                    })
                    .collect(),
            );
            spread.line(&format!("{} baseline, {length}-move scrambles", table.size));
        }
    }
}
