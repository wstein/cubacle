//! Origin: Rust tests of the slot tables behind the facelet bridge. The
//! expected tables are `FORMAT-V2-SLOTS.md` of flix-orbit64 (Apache-2.0,
//! Werner Stein), the normative slot convention for `n = 2..=15`, copied to
//! `tests/fixtures/orbit64_v2_slots.txt` by `tools/orbit64_v2_slots_fixture.jl`.

use crate::notation::orbit64::layout::Layout;

const SLOTS: &str = include_str!("../../../../../tests/fixtures/orbit64_v2_slots.txt");

/// The facelets of every slot line of the spec as `(n, label, facelets)`.
fn spec() -> Vec<(usize, &'static str, Vec<usize>)> {
    SLOTS
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| {
            let mut fields = line.split('|');
            let n = fields.next().unwrap().parse().unwrap();
            let label = fields.next().unwrap();
            let facelets = fields
                .next()
                .unwrap()
                .split(' ')
                .map(|f| f.parse().unwrap())
                .collect();
            (n, label, facelets)
        })
        .collect()
}

/// The slot lines `Layout` produces for a size, in the order of the spec.
fn computed(n: usize) -> Vec<(String, Vec<usize>)> {
    let layout = Layout::new(n);
    let mut lines = Vec::new();
    let mut add = |label: String, facelets: &[usize]| lines.push((label, facelets.to_vec()));
    for (slot, facelets) in layout.corners.iter().enumerate() {
        add(format!("C {slot}"), facelets);
    }
    for (slot, facelets) in layout.midges.iter().enumerate() {
        add(format!("M {slot}"), facelets);
    }
    for (depth, table) in layout.wings.iter().enumerate() {
        for (slot, facelets) in table.iter().enumerate() {
            add(format!("W{} {slot}", depth + 1), facelets);
        }
    }
    // Center orbits are named Z[row,col] by their lowest cell, in this order.
    let last = n.saturating_sub(1);
    let names = (1..last).flat_map(|r| (r..last - r).map(move |c| format!("Z[{r},{c}]")));
    for (name, facelets) in names.zip(&layout.centers) {
        add(name, facelets);
    }
    lines
}

#[test]
fn the_slot_tables_are_those_of_format_v2_slots_for_n_2_to_15() {
    let spec = spec();
    for n in 2..=15 {
        let expected: Vec<_> = spec
            .iter()
            .filter(|(size, _, _)| *size == n)
            .map(|(_, label, facelets)| (label.to_string(), facelets.clone()))
            .collect();
        assert!(!expected.is_empty(), "{n}x{n}x{n} missing from the spec");
        assert_eq!(computed(n), expected, "{n}x{n}x{n}");
    }
    assert_eq!(spec.len(), 1407);
}

#[test]
fn the_fixed_centers_are_the_middle_facelets_of_odd_cubes() {
    for n in 2..=15 {
        let expected: Vec<usize> = if n % 2 == 1 {
            (0..6).map(|face| face * n * n + (n * n - 1) / 2).collect()
        } else {
            Vec::new()
        };
        assert_eq!(Layout::new(n).fixed, expected, "{n}");
    }
}

#[test]
fn every_facelet_belongs_to_exactly_one_slot_for_n_2_to_15() {
    for n in 2..=15 {
        let layout = Layout::new(n);
        let mut all: Vec<usize> = layout.corners.iter().flatten().copied().collect();
        all.extend(layout.midges.iter().flatten());
        all.extend(layout.wings.iter().flatten().flatten());
        all.extend(layout.centers.iter().flatten());
        all.extend(&layout.fixed);
        all.sort_unstable();
        assert_eq!(all, (0..6 * n * n).collect::<Vec<_>>(), "{n}x{n}x{n}");
    }
}
