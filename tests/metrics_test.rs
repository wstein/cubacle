//! Metrics of compressed (layer range) solutions against the plain ones.

use cubacle::metrics::{count, count_layer_moves, Metric};
use cubacle::puzzle::{Face, Move, MoveSequence, PuzzleKind, Turn};
use cubacle::ranges::compress;

/// A repeatable sequence of valid moves, from a linear generator.
fn pseudo_random(kind: PuzzleKind, seed: u64, length: usize) -> MoveSequence {
    let n = u64::try_from(kind.size()).unwrap();
    let mut state = seed;
    let mut next = move |bound: u64| {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 33) % bound
    };
    (0..length)
        .map(|_| {
            let face = Face::ALL[next(6) as usize];
            let depth = 1 + u8::try_from(next(n - 1)).unwrap();
            let wide = depth >= 2 && next(2) == 0;
            let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise][next(3) as usize];
            Move::new(face, depth, wide, turn)
        })
        .collect()
}

#[test]
fn compressing_into_ranges_never_raises_the_axial_or_block_count() {
    // Ranges merge turns of one axis, and a run that cancels lets its
    // neighbours join: both counts can only fall.
    for size in 4..=7 {
        let kind = PuzzleKind::of_size(size).unwrap();
        for seed in 0..40 {
            let seq = pseudo_random(kind, seed, 30);
            let packed = compress(kind, &seq);
            for metric in [Metric::Atm, Metric::Btm] {
                assert!(
                    count_layer_moves(&packed, kind, metric) <= count(&seq, kind, metric),
                    "{metric} of {seq}"
                );
            }
        }
    }
}
