//! The cases of the picture facelet differential and the answer of the Rust
//! reader to them. Shared by the example `orbit64_v2_picture_diff` and by
//! `tests/orbit64_v2/picture_diff.rs`.
//!
//! A case is a mutation of one vector of
//! `tests/fixtures/orbit64_v2_picture_facelets.txt`, drawn by a fixed seed, so
//! a run is the same on every machine. The generator is the same linear
//! congruential generator as the one in `tests/orbit64_v2/picture_errors.rs`.

use cubacle::notation::orbit64::v2::facelets::{decode, encode};
use cubacle::notation::orbit64::v2::V2Error;
use std::collections::BTreeMap;

pub const PICTURES: &str = include_str!("../../tests/fixtures/orbit64_v2_picture_facelets.txt");
pub const SLOTS: &str = include_str!("../../tests/fixtures/orbit64_v2_slots.txt");
pub const SAMPLE: &str = include_str!("../../tests/fixtures/orbit64_v2_picture_diff_cases.txt");
pub const SAMPLE_SEED: u64 = 1;
pub const SAMPLE_COUNT: usize = 200;

const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
const BAD: &[u8] = b"#@*?!$%&";
const KINDS: usize = 10;

struct Lcg(u64);

impl Lcg {
    fn next(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) % bound as u64) as usize
    }
}

/// One case: its index, the id of the vector it starts from and the facelets.
pub struct Case {
    pub index: usize,
    pub id: usize,
    pub facelets: String,
}

fn data_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
}

/// The vectors as (id, facelets).
fn rows() -> Vec<(usize, &'static str)> {
    data_lines(PICTURES)
        .map(|line| {
            let f: Vec<&str> = line.split('|').collect();
            (f[0].parse().unwrap(), f[2])
        })
        .collect()
}

/// The slots as (size, label) to facelet indexes.
fn slots() -> BTreeMap<(usize, String), Vec<usize>> {
    data_lines(SLOTS)
        .map(|line| {
            let f: Vec<&str> = line.split('|').collect();
            let facelets = f[2].split(' ').map(|x| x.parse().unwrap()).collect();
            ((f[0].parse().unwrap(), f[1].to_string()), facelets)
        })
        .collect()
}

fn size_of(facelets: &str) -> usize {
    let count = facelets.chars().filter(|&c| c != ' ').count();
    (count as f64 / 6.0).sqrt().round() as usize
}

/// The symbols in groups of one face, separated by spaces.
fn spaced(symbols: &[char], n: usize) -> String {
    let groups: Vec<String> = symbols
        .chunks(n * n)
        .map(|face| face.iter().collect())
        .collect();
    groups.join(" ")
}

fn make_case(
    rng: &mut Lcg,
    index: usize,
    base: (usize, &str),
    table: &BTreeMap<(usize, String), Vec<usize>>,
) -> Case {
    let (id, facelets) = base;
    let n = size_of(facelets);
    let mut s: Vec<char> = facelets.chars().filter(|&c| c != ' ').collect();
    let (i, j) = (rng.next(s.len()), rng.next(s.len()));
    let kind = index % KINDS;
    match kind {
        0 => s.swap(i, j),
        1 => s[i] = s[j],
        2 => s[i] = ALPHABET[rng.next(64)] as char,
        3 | 4 => {
            // Labels of corners (`C ..`) and midges (`M ..`) of this size, sorted.
            let corners: Vec<&String> = table
                .keys()
                .filter(|(m, l)| *m == n && (l.starts_with('C') || l.starts_with('M')))
                .map(|(_, l)| l)
                .collect();
            let a = &table[&(n, corners[rng.next(corners.len())].clone())];
            if kind == 3 {
                // A slot cycled: the piece twisted in place.
                let old: Vec<char> = a.iter().map(|&x| s[x]).collect();
                for (k, &x) in a.iter().enumerate() {
                    s[x] = old[(k + 1) % a.len()];
                }
            } else {
                // Two slots of the same length exchanged.
                let same: Vec<&Vec<usize>> = corners
                    .iter()
                    .map(|l| &table[&(n, (*l).clone())])
                    .filter(|t| t.len() == a.len())
                    .collect();
                let b = same[rng.next(same.len())];
                for (&x, &y) in a.iter().zip(b) {
                    s.swap(x, y);
                }
            }
        }
        5 => s.truncate(i),
        6 => s.insert(i, ALPHABET[rng.next(64)] as char),
        7 => s[i] = BAD[rng.next(BAD.len())] as char,
        8 => {
            // Spaces are ignored by the reader.
            let text = spaced(&s, n);
            let cut = rng.next(text.len());
            let gap = " ".repeat(1 + rng.next(3));
            return Case {
                index,
                id,
                facelets: format!("{}{gap}{}", &text[..cut], &text[cut..]),
            };
        }
        _ => {} // kind 9 keeps the vector as it is
    }
    Case {
        index,
        id,
        facelets: spaced(&s, n),
    }
}

pub fn make_cases(seed: u64, count: usize) -> Vec<Case> {
    let mut rng = Lcg(seed);
    let (table, base) = (slots(), rows());
    (0..count)
        .map(|i| {
            let b = base[rng.next(base.len())];
            make_case(&mut rng, i, b, &table)
        })
        .collect()
}

/// The case file: `index|entry id|facelets`.
pub fn cases_text(seed: u64, count: usize) -> String {
    let mut out = format!(
        "# Orbit64 V2 picture facelet cases: index | entry id | facelets.\n\
         # Generated by `cargo run --release --example orbit64_v2_picture_diff -- cases --seed {seed} --count {count}`;\n\
         # mutations of tests/fixtures/orbit64_v2_picture_facelets.txt; do not edit by hand.\n"
    );
    for case in make_cases(seed, count) {
        out.push_str(&format!("{}|{}|{}\n", case.index, case.id, case.facelets));
    }
    out
}

/// The cases of a case file, as (index text, id, facelets).
pub fn read_cases(text: &str) -> Vec<(&str, usize, &str)> {
    data_lines(text)
        .map(|line| {
            let mut f = line.splitn(3, '|');
            let (index, id, facelets) = (f.next().unwrap(), f.next().unwrap(), f.next().unwrap());
            (index, id.parse().unwrap(), facelets)
        })
        .collect()
}

/// The text of an error as Flix writes it: the reason without the wrapper
/// words that `V2Error` adds in its `Display` ("not a cube (...)").
fn reason(error: &V2Error) -> String {
    match error {
        V2Error::Invalid(why) | V2Error::Facelets(why) | V2Error::Unsupported(why) => why.clone(),
        other => other.to_string(),
    }
}

/// The answer line of the Rust reader: `index|ok|token|facelets` when `encode`
/// accepts the string and `decode` of that token answers `facelets`, otherwise
/// `index|err|text` with the error text of the failing call. The Flix program
/// writes the same form.
pub fn answer(index: &str, id: usize, text: &str) -> String {
    match encode(id, text) {
        Err(why) => format!("{index}|err|{}", reason(&why)),
        Ok(token) => match decode(&token) {
            Ok((_, back)) => format!("{index}|ok|{token}|{back}"),
            Err(why) => format!("{index}|err|{}", reason(&why)),
        },
    }
}
