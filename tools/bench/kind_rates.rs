//! Measuring tool for the pattern kinds: how often each criterion fires on
//! states that are not patterns, and on the pattern library.
//!
//! `kind_rates [PER_LENGTH [RANDOM]]` builds, per cube size 2 to 7, a
//! deterministic set of
//!
//! - short scrambles: PER_LENGTH (default 30) random turns of random layers
//!   for each length 4 to 12,
//! - "random" states: RANDOM (default 200) scrambles of 150 turns, which for
//!   these measures are as irregular as a uniformly random state and, unlike
//!   `generate::random_state`, repeat from run to run,
//! - the built-in patterns that exist for the size,
//!
//! and prints, per size and per criterion threshold, the share of each set that
//! passes. A criterion that passes many short scrambles cannot be a decision.
//! It uses the library only; no solver is run.
//!
//! `kind_rates local [SCRAMBLES]` instead reads the tables named by the
//! environment variable `CUBACLE_LOCAL_TABLES` (files separated by `:`, the
//! size taken from a name such as `4xRevenge`; nothing is bundled) and prints
//! only aggregates: patternness bands, kind shares, the acceptance goal and
//! recognition of held-out patterns, see `kind_local.rs`.

mod kind_local;

use std::env;

use cubacle::patterns::{self, Measures, PatternKind};
use cubacle::puzzle::{Facelets, PuzzleKind};

/// A small deterministic generator (xorshift64).
struct Rng(u64);

impl Rng {
    fn next(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

fn scrambled(kind: PuzzleKind, rng: &mut Rng, length: usize) -> Facelets {
    let mut text = Vec::new();
    for _ in 0..length {
        let face = ["U", "R", "F", "D", "L", "B"][rng.next(6)];
        let depth = 1 + rng.next(kind.size() - 1);
        let suffix = ["", "'", "2"][rng.next(3)];
        let layer = if depth == 1 {
            String::new()
        } else {
            depth.to_string()
        };
        text.push(format!("{layer}{face}{suffix}"));
    }
    Facelets::solved(kind)
        .apply(&text.join(" ").parse().expect("valid moves"))
        .expect("moves exist on this cube")
}

type Test = (&'static str, f64, fn(&Measures, f64) -> bool);

const TESTS: [Test; 23] = [
    ("symmetry >=", 0.5, |m, t| m.symmetry >= t),
    ("symmetry >=", 0.7, |m, t| m.symmetry >= t),
    ("symmetry >=", 0.9, |m, t| m.symmetry >= t),
    ("symmetry order >=", 4.0, |m, t| {
        m.symmetry_order as f64 >= t
    }),
    ("symmetry order >=", 8.0, |m, t| {
        m.symmetry_order as f64 >= t
    }),
    ("symmetry order >=", 16.0, |m, t| {
        m.symmetry_order as f64 >= t
    }),
    ("faces >=", 0.5, |m, t| m.faces >= t),
    ("faces >=", 0.67, |m, t| m.faces >= t),
    ("faces >=", 0.83, |m, t| m.faces >= t),
    ("faces >=", 1.0, |m, t| m.faces >= t),
    ("simple faces >=", 0.5, |m, t| m.simple_faces >= t),
    ("simple faces >=", 0.67, |m, t| m.simple_faces >= t),
    ("simple faces >=", 0.83, |m, t| m.simple_faces >= t),
    ("simple faces >=", 1.0, |m, t| m.simple_faces >= t),
    ("orbit >=", 0.8, |m, t| m.orbit >= t),
    ("orbit >=", 0.9, |m, t| m.orbit >= t),
    ("orbit >=", 0.95, |m, t| m.orbit >= t),
    ("orbit >=", 1.0, |m, t| m.orbit >= t),
    ("fixed >= 0.6 & mask symmetry >=", 0.5, |m, t| {
        m.fixed >= 0.6 && m.mask_symmetry >= t
    }),
    ("moved pieces <= 3 or share <=", 0.15, |m, t| {
        m.moved <= 3 || m.moved_share() <= t
    }),
    ("patternness >=", 70.0, |m, t| f64::from(m.patternness) >= t),
    ("patternness >=", 80.0, |m, t| f64::from(m.patternness) >= t),
    ("patternness >=", 90.0, |m, t| f64::from(m.patternness) >= t),
];

fn share(set: &[Measures], pass: impl Fn(&Measures) -> bool) -> f64 {
    if set.is_empty() {
        return f64::NAN;
    }
    100.0 * set.iter().filter(|m| pass(m)).count() as f64 / set.len() as f64
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.get(1).map(String::as_str) == Some("local") {
        let tables = kind_local::tables();
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        kind_local::report_measures(&tables);
        kind_local::report_identify(&tables, &mut |kind, length| {
            scrambled(kind, &mut rng, length)
        });
        return;
    }
    let per_length: usize = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(30);
    let random: usize = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(200);
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    for size in 2..=7usize {
        let kind = PuzzleKind::of_size(size).expect("size 2 to 7");
        let randoms: Vec<Measures> = (0..random)
            .map(|_| patterns::measures(&scrambled(kind, &mut rng, 150)))
            .collect();
        let short: Vec<Measures> = (4..=12)
            .flat_map(|length| (0..per_length).map(move |_| length))
            .map(|length| patterns::measures(&scrambled(kind, &mut rng, length)))
            .collect();
        // The scrambles no decision of FEW or MOSTLY already explains.
        let rest: Vec<Measures> = short
            .iter()
            .filter(|m| !matches!(m.kind(), PatternKind::Few | PatternKind::Mostly))
            .cloned()
            .collect();
        let library: Vec<Measures> = patterns::all()
            .iter()
            .filter_map(|p| p.state(kind))
            .map(|state| patterns::measures(&state))
            .collect();
        println!(
            "\n{size}x{size}x{size}: {} random, {} short scrambles ({} beyond FEW/MOSTLY), {} library patterns",
            randoms.len(),
            short.len(),
            rest.len(),
            library.len()
        );
        println!("criterion\trandom %\tscrambles %\tbeyond FEW/MOSTLY %\tlibrary %");
        for (name, at, test) in TESTS {
            let pass = |m: &Measures| test(m, at);
            println!(
                "{name} {at}\t{:.1}\t{:.1}\t{:.1}\t{:.1}",
                share(&randoms, pass),
                share(&short, pass),
                share(&rest, pass),
                share(&library, pass)
            );
        }
        println!("kind (first match)\trandom %\tscrambles %\tlibrary %");
        for k in PatternKind::ALL {
            let is = |m: &Measures| m.kind() == k;
            println!(
                "{}\t{:.1}\t{:.1}\t{:.1}",
                k.name(),
                share(&randoms, is),
                share(&short, is),
                share(&library, is)
            );
        }
    }
}
