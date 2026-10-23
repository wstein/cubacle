//! The per-case lines and the aggregate tables.

use std::collections::BTreeMap;

use crate::cases::Case;
use crate::data::PilotCube;
use crate::run::{Outcome, Verdict};

pub struct Result<'a> {
    pub case: &'a Case,
    pub size: usize,
    pub outcome: Outcome,
}

pub fn case_line(r: &Result, pilot: &PilotCube) {
    let o = &r.outcome;
    let mistakes: Vec<String> = r
        .case
        .mistakes
        .iter()
        .map(|m| format!("{}@{}(-{})", m.kind.name(), m.step, m.distance))
        .collect();
    let show = |v: Option<usize>| v.map_or("-".to_owned(), |v| v.to_string());
    println!(
        "{:>2}.{} {}x{} len {:>3} stop {:>3} [{}] -> {} kinds={} stage={} {:.0}ms | route {} fresh {} -> {} {} (fresh took {:.1}s){}",
        r.case.cube + 1,
        r.case.number + 1,
        r.size,
        r.size,
        pilot.solution.len(),
        r.case.stop,
        mistakes.join(" "),
        o.scoped.verdict.name(),
        o.scoped.kinds_agree,
        o.scoped.stage,
        o.scoped.time.as_secs_f64() * 1000.0,
        show(o.scoped.route_total),
        show(o.fresh_len),
        o.advice,
        show(o.advised),
        o.fresh_time.as_secs_f64(),
        match &o.plain {
            Some(p) => format!(
                " | no stop: {} stage={} {:.0}ms",
                p.verdict.name(),
                p.stage,
                p.time.as_secs_f64() * 1000.0
            ),
            None => String::new(),
        }
    );
}

fn mean(values: impl Iterator<Item = f64>) -> String {
    let v: Vec<f64> = values.collect();
    if v.is_empty() {
        "-".to_owned()
    } else {
        format!("{:.1}", v.iter().sum::<f64>() / v.len() as f64)
    }
}

fn header(first: &str) {
    println!(
        "{first:<10} {:>3} {:>5} {:>7} {:>5} {:>5} {:>7} {:>8} {:>8} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7}",
        "n", "full", "partial", "wrong", "none", "onpath", "kinds", "ms avg", "ms max", "fresh", "advised", "saved", "rec<fr", "rec=fr"
    );
}

fn row(label: &str, group: &[&Result]) {
    let count = |v: Verdict| {
        group
            .iter()
            .filter(|r| r.outcome.scoped.verdict == v)
            .count()
    };
    let found: Vec<_> = group
        .iter()
        .filter(|r| matches!(r.outcome.scoped.verdict, Verdict::Full | Verdict::Partial))
        .collect();
    let kinds = found
        .iter()
        .filter(|r| r.outcome.scoped.kinds_agree)
        .count();
    let ms = |r: &&Result| r.outcome.scoped.time.as_secs_f64() * 1000.0;
    let max = group.iter().map(ms).fold(0.0, f64::max);
    let both = |r: &&Result| r.outcome.fresh_len.zip(r.outcome.advised);
    let shorter = group
        .iter()
        .filter(|r| {
            r.outcome
                .scoped
                .route_total
                .zip(r.outcome.fresh_len)
                .is_some_and(|(a, f)| a < f)
        })
        .count();
    let equal = group
        .iter()
        .filter(|r| {
            r.outcome
                .scoped
                .route_total
                .zip(r.outcome.fresh_len)
                .is_some_and(|(a, f)| a == f)
        })
        .count();
    println!(
        "{label:<10} {:>3} {:>5} {:>7} {:>5} {:>5} {:>7} {:>8} {:>8} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7}",
        group.len(),
        count(Verdict::Full),
        count(Verdict::Partial),
        count(Verdict::Wrong),
        count(Verdict::NoExplanation),
        count(Verdict::OnPath),
        format!("{kinds}/{}", found.len()),
        mean(group.iter().map(ms)),
        format!("{max:.0}"),
        mean(group.iter().filter_map(|r| both(r).map(|(f, _)| f as f64))),
        mean(group.iter().filter_map(|r| both(r).map(|(_, a)| a as f64))),
        mean(group.iter().filter_map(|r| both(r).map(|(f, a)| f as f64 - a as f64))),
        shorter,
        equal,
    );
}

fn table(title: &str, key: impl Fn(&Result) -> String, results: &[Result]) {
    println!("\n{title}");
    header("");
    let mut groups: BTreeMap<String, Vec<&Result>> = BTreeMap::new();
    for r in results {
        groups.entry(key(r)).or_default().push(r);
    }
    for (label, group) in &groups {
        row(label, group);
    }
    row("all", &results.iter().collect::<Vec<_>>());
}

pub fn tables(results: &[Result]) {
    println!("\nColumns: verdict of the explanation against the injected mistakes (full = all");
    println!("injected steps named and nothing else; partial = some; wrong = other steps;");
    println!("none = near/unexplained), kinds = named steps whose kind agrees (of full+partial),");
    println!("search ms, mean fresh length, mean moves still to make with the advice,");
    println!("saved = fresh minus advised, rec<fr / rec=fr = best route shorter than / as long");
    println!("as the fresh solve.");
    table(
        "By number of mistakes",
        |r| r.case.mistakes.len().to_string(),
        results,
    );
    table(
        "By distance of the farthest mistake from the stop",
        |r| format!("{:>3}", r.case.mistakes[0].distance),
        results,
    );
    table("By cube", |r| format!("{}x{}", r.size, r.size), results);
    println!("\nStage that answered (with the stop)");
    let mut stages: BTreeMap<&str, Vec<&Result>> = BTreeMap::new();
    for r in results {
        stages.entry(r.outcome.scoped.stage).or_default().push(r);
    }
    for (stage, group) in &stages {
        let full = group
            .iter()
            .filter(|r| r.outcome.scoped.verdict == Verdict::Full)
            .count();
        println!("  {stage:<50} {:>3} cases, {full} full", group.len());
    }
    println!("\nWith the stop against without it (the cases where both ran)");
    let both: Vec<&Result> = results
        .iter()
        .filter(|r| r.outcome.plain.is_some())
        .collect();
    for verdict in Verdict::ALL {
        let with = both
            .iter()
            .filter(|r| r.outcome.scoped.verdict == verdict)
            .count();
        let without = both
            .iter()
            .filter(|r| {
                r.outcome
                    .plain
                    .as_ref()
                    .is_some_and(|p| p.verdict == verdict)
            })
            .count();
        println!(
            "  {:<8} with stop {with:>2}  without {without:>2}",
            verdict.name()
        );
    }
    let ms = |f: &dyn Fn(&Result) -> f64| mean(both.iter().map(|r| f(r)));
    println!(
        "  mean search ms: with stop {}, without {}",
        ms(&|r| r.outcome.scoped.time.as_secs_f64() * 1000.0),
        ms(&|r| r
            .outcome
            .plain
            .as_ref()
            .map_or(0.0, |p| p.time.as_secs_f64() * 1000.0)),
    );
    let timeouts = results
        .iter()
        .filter(|r| r.outcome.scoped.timed_out)
        .count();
    println!("  searches that hit the time limit (with stop): {timeouts}");
}
