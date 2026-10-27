use std::time::{Duration, Instant};

use super::{position_keys, Alphabet, Kit, Table, Walker};
use crate::puzzle::{Facelets, PuzzleKind};

fn colors(cube: &Facelets) -> Vec<u8> {
    cube.stickers().iter().map(|f| f.index() as u8).collect()
}

/// The split walk (all cores, writing into slices) gives, entry for entry, the
/// vector of the serial walk, so the sorted table and its tie-breaks are the
/// same as before the walk was split.
#[test]
fn the_split_table_walk_equals_the_serial_walk() {
    for (kind, kit, depth) in [
        (PuzzleKind::Cube3, Kit::Outer, 3),
        (PuzzleKind::Cube4, Kit::Slices, 2),
        (PuzzleKind::Cube4, Kit::Blocks, 3),
    ] {
        let al = Alphabet::new(kind, kit);
        let start = colors(&Facelets::solved(kind));
        let keys = position_keys(start.len());
        let deadline = Instant::now() + Duration::from_secs(600);
        let expected = al.count_up_to(depth) as usize;
        let mut split_walker = Walker::new(&al, &keys, deadline).exact();
        let split = Table::split_entries(&mut split_walker, &start, depth, expected)
            .expect("the counted sizes add up");
        let mut serial_walker = Walker::new(&al, &keys, deadline).exact();
        let serial = Table::serial_entries(&mut serial_walker, &start, depth, expected)
            .expect("no deadline");
        assert_eq!(split.len(), expected, "{kind:?} {kit:?}");
        assert_eq!(split, serial, "{kind:?} {kit:?} depth {depth}");
    }
}

#[test]
fn the_counted_sizes_per_first_block_add_up() {
    let al = Alphabet::new(PuzzleKind::Cube4, Kit::Slices);
    for depth in 1..=4 {
        let total = al.sizes_from(depth).iter().sum::<usize>() + 1;
        assert_eq!(total as u64, al.count_up_to(depth), "depth {depth}");
    }
}
