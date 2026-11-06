//! `tools/wasm/golden.txt` holds the fixed scrambles of the G1 measurement and
//! the solutions the WebAssembly build gave. The native facade must give the
//! same text, and every solution must replay to the solved cube; the Node
//! benchmark compares the WebAssembly build with the same file.

use cubacle::puzzle::{Facelets, PuzzleKind};
use cubacle_wasm::{solve_text, STATUS_OK};

#[test]
fn native_answers_equal_the_golden_file_and_replay() {
    let golden = include_str!("../../../tools/wasm/golden.txt");
    let mut lines = 0;
    for line in golden.lines().filter(|line| !line.is_empty()) {
        let parts: Vec<&str> = line.split('|').collect();
        assert_eq!(parts.len(), 3, "{line}");
        let size: u32 = parts[0].parse().unwrap();
        let max = if size == 3 { 25 } else { 14 };
        let (status, text) = solve_text(size, parts[1], max);
        assert_eq!(status, STATUS_OK, "{line}");
        assert_eq!(text, parts[2], "{line}");
        let kind = if size == 3 {
            PuzzleKind::Cube3
        } else {
            PuzzleKind::Cube2
        };
        let cube = Facelets::solved(kind)
            .apply(&parts[1].parse().unwrap())
            .unwrap();
        assert!(cube.apply(&text.parse().unwrap()).unwrap().is_solved());
        lines += 1;
    }
    assert_eq!(lines, 24);
}
