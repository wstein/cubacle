//! Writing SSE: the canonical spelling per turn, and read-back on every size.

use cubacle::notation::{parse_expression, Notation};
use cubacle::puzzle::{Face, Facelets, LayerMove, Move, MoveSequence, PuzzleKind, Turn};
use cubacle::ranges::{compress, expand, write};

const TURNS: [Turn; 3] = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise];

fn kind(size: usize) -> PuzzleKind {
    PuzzleKind::of_size(size).unwrap()
}

fn apply(size: usize, moves: &MoveSequence) -> Facelets {
    Facelets::solved(kind(size)).apply(moves).unwrap()
}

fn sse(size: usize, text: &str) -> String {
    Notation::Sse.write(kind(size), &text.parse().unwrap())
}

fn block(size: usize, face: Face, from: u8, to: u8, turn: Turn) -> String {
    cubacle::ranges::write_move(
        Notation::Sse,
        kind(size),
        &LayerMove::new(face, from, to, turn),
    )
}

/// A small fixed-seed generator, so the tests are the same on every run.
struct Rng(u64);

impl Rng {
    fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % bound as u64) as usize
    }
}

#[test]
fn canonical_spellings() {
    // the same layers on each size, the shortest page spelling
    assert_eq!(sse(2, "R U' F2"), "R U' F2");
    assert_eq!(sse(3, "R Rw 2R' 2L2"), "R TR MR' ML2");
    assert_eq!(sse(4, "Rw 3Rw 2R 3R'"), "TR T3R MR ML");
    assert_eq!(sse(5, "2R 3R 4R' Rw 4Rw"), "NR MR N4R' TR T4R");
    assert_eq!(sse(6, "3R 4R 5U"), "MR N4R N5U");
    assert_eq!(sse(7, "4R 3R' Rw2 5Fw"), "MR N3R' TR2 T5F");
}

#[test]
fn blocks_follow_the_order_of_the_rules() {
    use Face::{L, R, U};
    let q = Turn::Clockwise;
    assert_eq!(block(7, R, 2, 3, q), "VR");
    assert_eq!(block(7, R, 2, 4, q), "V3R");
    assert_eq!(block(7, R, 2, 5, Turn::Half), "V4R2");
    assert_eq!(block(7, R, 2, 6, q), "WR");
    assert_eq!(block(7, R, 3, 5, q), "M3R");
    assert_eq!(block(7, R, 3, 4, q), "M2R");
    assert_eq!(block(7, U, 4, 6, Turn::CounterClockwise), "N4-6U'");
    assert_eq!(block(7, L, 3, 6, q), "N3-6L");
    assert_eq!(block(6, R, 2, 3, q), "VR");
    assert_eq!(block(6, R, 3, 4, q), "M2R");
    assert_eq!(block(6, R, 2, 5, q), "WR");
    assert_eq!(block(6, R, 2, 4, q), "V3R");
    assert_eq!(block(6, R, 3, 5, q), "N3-5R");
    assert_eq!(block(5, R, 2, 4, q), "WR");
    assert_eq!(block(5, R, 2, 3, q), "VR");
    assert_eq!(block(4, R, 2, 3, q), "WR");
    assert_eq!(block(4, R, 3, 3, q), "ML'");
    // a block to the last layer is the tier of the other face, turned back
    let last = LayerMove::new(R, 4, 7, q);
    assert_eq!(
        cubacle::ranges::write_move(Notation::Sse, kind(7), &last),
        "T4L'"
    );
    assert_eq!(block(7, R, 1, 7, q), "CR");
}

/// Every move that fits a size is written and read back as the same move,
/// except the 4x4's third layer: it has no `N`, so it is written from the
/// other face as the same turn (`3R` is `ML'`).
#[test]
fn every_move_of_every_size_reads_back() {
    for size in 2..=7usize {
        for face in Face::ALL {
            for turn in TURNS {
                for depth in 1..size as u8 {
                    for wide in [false, true] {
                        if wide && depth < 2 {
                            continue;
                        }
                        let mv = Move::new(face, depth, wide, turn);
                        let text = Notation::Sse.write_move(kind(size), &mv);
                        let back = parse_expression(&text, kind(size))
                            .unwrap_or_else(|e| panic!("{mv} -> {text} on {size}: {e}"));
                        let original = MoveSequence::new(vec![mv]);
                        assert_eq!(apply(size, &back), apply(size, &original), "{mv} / {text}");
                        if !(size == 4 && !wide && depth == 3) {
                            assert_eq!(back, original, "{mv} -> {text} on {size}");
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn random_sequences_round_trip_on_every_size() {
    let mut rng = Rng(0x5eed_1234_abcd_ef01);
    for size in 2..=7usize {
        for _ in 0..200 {
            let moves: Vec<Move> = (0..30)
                .map(|_| {
                    let depth = 1 + rng.below(size - 1) as u8;
                    let wide = depth >= 2 && rng.below(2) == 0;
                    Move::new(Face::ALL[rng.below(6)], depth, wide, TURNS[rng.below(3)])
                })
                .collect();
            let sequence = MoveSequence::new(moves);
            let text = Notation::Sse.write(kind(size), &sequence);
            let back = parse_expression(&text, kind(size)).unwrap();
            assert_eq!(apply(size, &back), apply(size, &sequence), "{text}");
            // a second write is the same text: the spelling is canonical
            assert_eq!(Notation::Sse.write(kind(size), &back), text);
        }
    }
}

#[test]
fn compressed_ranges_round_trip_on_every_size() {
    let mut rng = Rng(0x0dd5_eed5_1357_9bdf);
    for size in 4..=7usize {
        for _ in 0..200 {
            let moves: Vec<Move> = (0..24)
                .map(|_| {
                    let depth = 1 + rng.below(size - 1) as u8;
                    let wide = depth >= 2 && rng.below(2) == 0;
                    let axis = rng.below(3);
                    let face = Face::ALL[axis + 3 * rng.below(2)];
                    Move::new(face, depth, wide, TURNS[rng.below(3)])
                })
                .collect();
            let packed = compress(kind(size), &MoveSequence::new(moves.clone()));
            let text = write(Notation::Sse, kind(size), &packed);
            let back = parse_expression(&text, kind(size))
                .unwrap_or_else(|e| panic!("{text} on {size}: {e}"));
            let want = apply(size, &MoveSequence::new(moves));
            assert_eq!(apply(size, &back), want, "{text}");
            assert_eq!(apply(size, &expand(&packed)), want);
        }
    }
}

#[test]
fn a_range_in_each_notation_is_the_same_turn() {
    let packed = compress(kind(7), &"2R 3R 4R".parse().unwrap());
    assert_eq!(write(Notation::Wca, kind(7), &packed), "2-4Rw");
    assert_eq!(write(Notation::Sign, kind(7), &packed), "2-4r");
    assert_eq!(write(Notation::Sse, kind(7), &packed), "V3R");
}
