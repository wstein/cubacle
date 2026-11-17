//! The report counts the requested rewritten sequence.
use cubacle::notation::Notation;
use cubacle::puzzle::{MoveSequence, PuzzleKind};
use cubacle::ranges::Optimize;
use cubacle::report::SolveReport;
use std::time::Duration;

#[test]
fn moves_report_contains_optimized_exact_result() {
    let moves: MoveSequence = "R L R' L'".parse().unwrap();
    let report = SolveReport::of(
        PuzzleKind::Cube4,
        &moves,
        Notation::Wca,
        Some(Optimize::Moves),
        Duration::ZERO,
    );
    assert_eq!(report.length, 4);
    let compressed = report.compressed.unwrap();
    assert_eq!(compressed.text, "");
    assert_eq!(compressed.length, 0);
    assert_eq!(compressed.metrics.btm, 0);
}

#[test]
fn easy_inner_block_reaches_svg_as_one_action() {
    use cubacle::puzzle::{Face, Facelets, LayerMove, Turn};
    use cubacle::ranges::optimize;
    use cubacle::render::{icon, Action, Style};
    let kind = PuzzleKind::Cube4;
    let input: MoveSequence = "B2 F2".parse().unwrap();
    let result = optimize(kind, &input, Optimize::Easy);
    assert_eq!(result, vec![LayerMove::new(Face::F, 2, 3, Turn::Half)]);
    let cube = Facelets::solved(kind);
    let svg = |mv| {
        icon(&cube, &Action::block(mv), &Style::default())
            .unwrap()
            .to_svg()
    };
    let block = svg(result[0]);
    assert!(block.contains("2-3Fw2"));
    assert_ne!(block, svg(LayerMove::new(Face::F, 2, 2, Turn::Half)));
    assert_ne!(block, svg(LayerMove::new(Face::F, 3, 3, Turn::Half)));
}
