use super::*;
use crate::puzzle::{Face, Move, Turn};

fn all(text: &str, size: usize) -> Vec<usize> {
    let kind = PuzzleKind::of_size(size).unwrap();
    Metric::ALL
        .iter()
        .map(|&metric| count_text(text, kind, metric).expect(text))
        .collect()
}

// Order: htm qtm stm etm obtm btm atm.
#[test]
fn plain_turns_on_the_3x3() {
    assert_eq!(all("R U R' U'", 3), [4, 4, 4, 4, 4, 4, 4]);
    assert_eq!(all("R2", 3), [1, 2, 1, 1, 1, 1, 1]);
    assert_eq!(all("R L'", 3), [2, 2, 2, 2, 2, 2, 1]);
    assert_eq!(all("R2 L2 U", 3), [3, 5, 3, 3, 3, 3, 2]);
}

#[test]
fn wide_and_middle_turns_on_the_3x3() {
    assert_eq!(all("Rw", 3), [1, 1, 2, 1, 1, 1, 1]);
    assert_eq!(all("r", 3), [1, 1, 2, 1, 1, 1, 1]);
    assert_eq!(all("M", 3), [2, 2, 1, 1, 2, 1, 1]);
    assert_eq!(all("M2", 3), [2, 4, 1, 1, 2, 1, 1]);
    assert_eq!(all("E S", 3), [4, 4, 2, 2, 4, 2, 2]);
}

#[test]
fn rotations_count_only_in_etm() {
    assert_eq!(all("x", 3), [0, 0, 0, 1, 0, 0, 0]);
    assert_eq!(all("x y2 R", 3), [1, 1, 1, 3, 1, 1, 1]);
    // A rotation about the axis of the surrounding turns joins their run.
    assert_eq!(all("R x R'", 3)[6], 1);
    assert_eq!(all("R y R'", 3)[6], 2);
}

#[test]
fn layer_ranges_and_inner_layers_on_the_5x5() {
    assert_eq!(all("2-3Rw", 5), [2, 2, 2, 1, 2, 1, 1]);
    assert_eq!(all("2-3r2", 5), [2, 4, 2, 1, 2, 1, 1]);
    assert_eq!(all("3Rw", 5), [1, 1, 3, 1, 1, 1, 1]);
    assert_eq!(all("2-4Rw", 5), [2, 2, 3, 1, 2, 1, 1]);
    assert_eq!(all("2R", 5), [2, 2, 1, 1, 2, 1, 1]);
    assert_eq!(all("4Rw", 5), [1, 1, 4, 1, 1, 1, 1]);
    assert_eq!(all("M E", 5), [4, 4, 2, 2, 4, 2, 2]);
}

#[test]
fn the_far_face_counts_from_its_own_side() {
    // 2L is layer 4 of 5, an inner layer; 4Lw reaches the far face.
    assert_eq!(all("2L", 5), [2, 2, 1, 1, 2, 1, 1]);
    assert_eq!(all("4Lw", 5), [1, 1, 4, 1, 1, 1, 1]);
    // R and 2-... on the same axis from both faces form one run.
    assert_eq!(all("R 2L 3Lw'", 5)[6], 1);
}

#[test]
fn text_that_is_not_flat_is_not_counted() {
    let kind = PuzzleKind::Cube3;
    for text in ["(R U)*2", "[R: U]", "R, U", "TR", "R4", "Mw", "3R"] {
        assert_eq!(count_text(text, kind, Metric::Htm), None, "{text}");
    }
    assert_eq!(count_text("M", PuzzleKind::Cube4, Metric::Htm), None);
}

#[test]
fn metric_names_round_trip() {
    for metric in Metric::ALL {
        assert_eq!(metric.to_string().parse::<Metric>().unwrap(), metric);
    }
    assert_eq!("FTM".parse::<Metric>().unwrap(), Metric::Htm);
    assert_eq!("ltm".parse::<Metric>().unwrap(), Metric::Stm);
    assert!("xyz".parse::<Metric>().is_err());
}

#[test]
fn summary_names_the_headline_metrics() {
    let seq: MoveSequence = "R U2 Rw".parse().unwrap();
    assert_eq!(summary(&seq, PuzzleKind::Cube3), "htm 3, qtm 4, stm 4");
    assert_eq!(
        summary(&seq, PuzzleKind::Cube4),
        "obtm 3, qtm 4, stm 4, btm 3"
    );
}

/// A repeatable sequence of valid moves for a cube, from a linear generator.
fn pseudo_random(kind: PuzzleKind, seed: u64, length: usize) -> MoveSequence {
    let n = u8::try_from(kind.size()).unwrap();
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
            let depth = 1 + u8::try_from(next(u64::from(n - 1))).unwrap();
            let wide = depth >= 2 && next(2) == 0;
            let turn = [Turn::Clockwise, Turn::Half, Turn::CounterClockwise][next(3) as usize];
            Move::new(face, depth, wide, turn)
        })
        .collect()
}

#[test]
fn relations_between_the_metrics_hold() {
    for size in 2..=7 {
        let kind = PuzzleKind::of_size(size).unwrap();
        for seed in 0..40 {
            let seq = pseudo_random(kind, seed, 30);
            let c = |metric| count(&seq, kind, metric);
            assert_eq!(c(Metric::Htm), c(Metric::Obtm));
            assert_eq!(c(Metric::Etm), c(Metric::Btm));
            assert_eq!(c(Metric::Etm), seq.len());
            assert!(c(Metric::Atm) <= c(Metric::Btm));
            assert!(c(Metric::Btm) <= c(Metric::Htm));
            assert!(c(Metric::Htm) <= c(Metric::Qtm));
            assert!(c(Metric::Btm) <= c(Metric::Stm));
            assert!(c(Metric::Qtm) <= 4 * c(Metric::Htm));
            // The same moves written down and counted as words agree.
            for (metric, expected) in count_all(&seq, kind) {
                assert_eq!(
                    count_text(&seq.to_string(), kind, metric),
                    Some(expected),
                    "{metric} of {seq} on {size}"
                );
            }
        }
    }
}
