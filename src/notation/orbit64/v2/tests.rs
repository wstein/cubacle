use super::*;

/// The `Bits` column of the registry table in `FORMAT-V2.md`.
#[rustfmt::skip]
const BITS: [f64; 54] = [
    21.81, 26.39, 65.23, 69.81, 76.23, 80.81, 152.37, 156.96, 178.88, 183.47, 247.32, 251.91,
    311.34, 315.93, 386.00, 390.58, 492.03, 496.62, 532.47, 537.06, 702.53, 707.12, 722.67,
    727.26, 961.26, 965.85, 920.68, 925.26, 1249.79, 1254.38, 1162.41, 1166.99, 1586.56,
    1591.15, 1411.94, 1416.52, 1953.13, 1957.72, 1705.19, 1709.78, 2367.94, 2372.52, 2006.25,
    2010.84, 2812.55, 2817.13, 2351.04, 2355.62, 3305.39, 3309.98, 2703.63, 2708.21, 3828.04, 3832.62,
];

/// The entries whose offset is the state count of the entry before them in
/// the table, and the entry they follow.
const AFTER: [(usize, usize); 6] = [(11, 10), (17, 16), (21, 20), (25, 24), (39, 38), (53, 52)];

#[test]
fn the_registry_has_54_entries_with_the_bits_of_the_format() {
    assert_eq!(entries().len(), 54);
    for (entry, bits) in entries().iter().zip(BITS) {
        assert!(
            (entry.count.log2() - bits).abs() < 0.006,
            "entry {}: {} bits",
            entry.id,
            entry.count.log2()
        );
    }
}

#[test]
fn offsets_are_zero_except_after_a_shared_length() {
    for entry in entries() {
        match AFTER.iter().find(|&&(id, _)| id == entry.id) {
            Some(&(_, before)) => {
                assert_eq!(entries()[before].length, entry.length);
                assert_eq!(entry.offset, entries()[before].count, "entry {}", entry.id);
            }
            None => assert!(entry.offset.is_zero(), "entry {}", entry.id),
        }
    }
}

#[test]
fn the_small_cube_counts_are_the_known_ones() {
    let count = |id: usize| {
        let mut value = entries()[id].count.clone();
        let mut text = String::new();
        while !value.is_zero() {
            text.insert(0, char::from(b'0' + value.div_rem(10) as u8));
        }
        text
    };
    assert_eq!(count(0), "3674160");
    assert_eq!(count(1), "88179840");
    assert_eq!(count(2), "43252003274489856000");
    assert_eq!(count(3), "1038048078587756544000");
}

#[test]
fn the_fullest_length_is_42_with_both_5x5x5_color_entries() {
    let used = |length: usize| {
        let mut sum = Big::zero();
        for entry in entries().iter().filter(|e| e.length == length) {
            sum.add(&entry.count);
        }
        sum
    };
    let mut capacity = Big::one();
    for _ in 0..42 {
        capacity.mul_add(64, 0);
    }
    let ratio = used(42).log2() - capacity.log2();
    assert!((ratio.exp2() - 0.9772).abs() < 0.0001, "{ratio}");
}

#[test]
fn every_state_count_is_the_product_of_the_radices_of_its_digits() {
    for entry in entries() {
        let digits = digits::split(entry.n, entry.model, entry.pose, Big::zero());
        assert!(digits.iter().all(|&digit| digit == 0));
        let mut last = entry.count.clone();
        last.sub(&Big::one());
        let split = digits::split(entry.n, entry.model, entry.pose, last.clone());
        assert_eq!(digits::rank(entry.n, entry.model, entry.pose, &split), last);
    }
}

#[test]
fn the_highest_and_the_lowest_rank_of_every_entry_round_trip() {
    for entry in entries() {
        let mut last = entry.count.clone();
        last.sub(&Big::one());
        let split = digits::split(entry.n, entry.model, entry.pose, last);
        let cube = digits::from_digits(entry.n, entry.model, entry.pose, &split);
        let token = encode(entry.id, &cube).unwrap();
        let (found, back) = decode(&token).unwrap();
        assert_eq!((found.id, back), (entry.id, cube));
    }
}

#[test]
fn a_cube_below_the_2x2x2_has_no_solved_state_and_no_orbits() {
    for model in [Model::Color, Model::Picture] {
        assert_eq!(Cube::solved(0, model), None);
        assert_eq!(Cube::solved(1, model), None);
        assert!(Cube::solved(2, model).is_some());
        // Only a cube of fewer than two layers has none: the solved state is
        // defined beyond the registry's sizes too.
        assert!(Cube::solved(16, model).is_some());
    }
    for n in 0..2 {
        assert_eq!(cube::wing_orbits(n), 0);
        assert!(cube::center_orbits(n).is_empty());
    }
}

#[test]
fn the_orbit_counts_are_the_known_ones_for_every_size() {
    // Written out by hand for n = 2..=15: wing orbits floor((n - 2) / 2) and
    // center orbits floor((n - 2)^2 / 4), as in the layout table of FORMAT-V2.md.
    let wings = [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6];
    let centers = [0, 0, 1, 2, 4, 6, 9, 12, 16, 20, 25, 30, 36, 42];
    for (index, n) in (2..=15).enumerate() {
        assert_eq!(cube::wing_orbits(n), wings[index], "wings, n = {n}");
        assert_eq!(
            cube::center_orbits(n).len(),
            centers[index],
            "centers, n = {n}"
        );
    }
    let seven: Vec<_> = [(1, 1), (1, 2), (1, 3), (1, 4), (2, 2), (2, 3)].to_vec();
    assert_eq!(cube::center_orbits(7), seven);
}

#[test]
fn neighbors_of_a_range_are_not_tokens_of_the_entry() {
    // One past the 4-character range of entry 0 is no token at all.
    let mut end = entries()[0].count.clone();
    let token = end.to_base64(4).unwrap();
    assert_eq!(decode(&token), Err(V2Error::UnknownToken));
    end.sub(&Big::one());
    assert_eq!(decode(&end.to_base64(4).unwrap()).map(|(e, _)| e.id), Ok(0));
}
