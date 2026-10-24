//! Tests on invented lines in the table layout; no third-party data.

use super::*;
use crate::puzzle::{Facelets, PuzzleKind};

fn options(size: usize) -> ImportOptions {
    ImportOptions {
        size,
        name_prefix: "x-".to_owned(),
        origin: "synthetic".to_owned(),
        source: "ct".to_owned(),
    }
}

/// An invented line in the layout: 17 tab-separated fields.
fn row(code: &str, variant: u32, name: &str, moves: &str) -> String {
    let variant = variant.to_string();
    let f = [
        code, &variant, "Gruppe", "Group", "Unter", "Sub", "Muster", name, moves, "", "3*", "6*",
        "2", "", "nobody", "", "",
    ];
    f.join("\t")
}

#[test]
fn variants_merge_and_replay() {
    let text = [
        row("Z1.01", 1, "Pair", "R2 U2"),
        row("Z1.01", 2, "Pair", "U2 R2"),
        row("Z1.02", 1, "Inverse turns", "R- U'"),
    ]
    .join("\r\n");
    let got = import_tab(&text, &options(3), &read_plain);
    assert_eq!(
        got.report.problems,
        vec![ImportProblem {
            line: 2,
            message: "Z1.01: generator 2 builds another state".into()
        }]
    );
    assert_eq!(got.patterns.len(), 2);
    assert_eq!(got.patterns[1].generators()[0].moves, "R' U'");
    assert_eq!(got.patterns[0].name, "x-Pair");
    assert!(got.patterns[0].state(PuzzleKind::Cube3).is_some());
    assert!(got.patterns[0].state(PuzzleKind::Cube2).is_none());
}

#[test]
fn bad_lines_are_reported_not_fatal() {
    let text = [
        "short\tline".to_owned(),
        row("Z2.01", 1, "Letters", "R Q"),
        row("Z2.02", 1, "Wide", "Rw"),
        row("Z2.03", 1, "Empty", ""),
        row("Z2.04", 1, "Fine", "F2"),
    ]
    .join("\n");
    let got = import_tab(&text, &options(2), &read_plain);
    assert_eq!(got.report.lines, 5);
    assert_eq!(got.report.accepted, 1);
    let lines: Vec<usize> = got.report.problems.iter().map(|p| p.line).collect();
    assert_eq!(lines, vec![1, 2, 3, 4]);
}

#[test]
fn file_round_trip_and_identify() {
    let got = import_tab(&row("Z3.01", 1, "Slice", "R2 L2"), &options(3), &read_plain);
    let file = write_imported(&got.patterns, "synthetic");
    assert!(looks_imported(&file));
    let back = read_imported(&file, "synthetic").unwrap();
    assert_eq!(back[0].generators(), got.patterns[0].generators());
    // The same state turned (U2 D2 is R2 L2 seen from another side).
    let state = Facelets::solved(PuzzleKind::Cube3)
        .apply(&"U2 D2".parse().unwrap())
        .unwrap();
    assert_eq!(identify_imported(&back, &state).len(), 1);
    assert!(read_imported("a | b", "o").unwrap_err().contains("line 1"));
}

#[test]
fn catalogue_ids_groups_and_code_limit() {
    let text = [
        row("Z1.01", 7, "Pair", "R2 U2"),
        row("TOOLONG123", 1, "Long", "R2"),
    ]
    .join("\n");
    let got = import_tab(&text, &options(3), &read_plain);
    assert_eq!(got.report.problems.len(), 1);
    let p = &got.patterns[0];
    assert_eq!(p.catalogue, "ct:Z1.01");
    assert_eq!(p.group.len(), 3);
    assert_eq!(p.group[0].en, "Group");
    assert_eq!(p.provenance, "nobody");
    assert_eq!(p.generators()[0].number, 7);
    assert!(p.is_named("CT:z1.01"));
    let mut bad = options(3);
    bad.source = "CTX".into();
    assert_eq!(import_tab(&text, &bad, &read_plain).patterns.len(), 0);
}

#[test]
fn builtin_catalogue_ids_are_short_unique_and_grouped() {
    let mut seen = std::collections::HashSet::new();
    for p in all() {
        assert!(p.catalogue.len() <= 12, "{}", p.catalogue);
        assert!(p.catalogue.starts_with("cu:"));
        assert!(seen.insert(p.catalogue.clone()), "{}", p.catalogue);
        assert_eq!(p.group.len(), 1);
        assert_ne!(p.group[0].en, "Other patterns", "{}", p.id);
        let numbers: Vec<usize> = p.generators().iter().map(|g| g.number).collect();
        assert_eq!(numbers, (1..=numbers.len()).collect::<Vec<_>>());
    }
    assert!(find("cu:checker").is_some());
}

#[test]
fn latin1_text_decodes() {
    assert_eq!(decode_text(&[b'a', 0xE4]), "a\u{e4}");
    assert_eq!(decode_text("ä".as_bytes()), "ä");
}
