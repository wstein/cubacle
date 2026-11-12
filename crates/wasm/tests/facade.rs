//! The C-ABI facade, called the way a WebAssembly host calls it: through byte
//! buffers in linear memory.

use cubacle::puzzle::{Facelets, PuzzleKind};
use cubacle_wasm::{
    cubacle_abi_version, cubacle_alloc, cubacle_free, cubacle_result_len, cubacle_solve, HEADER,
    STATUS_BAD_INPUT, STATUS_OK,
};

/// Runs the facade on `scramble` and returns the status and the text.
fn call(size: u32, scramble: &str, max_moves: u32) -> (u8, String) {
    let bytes = scramble.as_bytes();
    let input = cubacle_alloc(bytes.len());
    // SAFETY: `input` was allocated with `bytes.len()` bytes just above.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), input, bytes.len()) };
    // SAFETY: `input` and its length describe the buffer filled above.
    let result = unsafe { cubacle_solve(size, input, bytes.len(), max_moves) };
    // SAFETY: the same allocation, released once.
    unsafe { cubacle_free(input, bytes.len()) };
    // SAFETY: `result` was just returned by `cubacle_solve`.
    let total = unsafe { cubacle_result_len(result) };
    // SAFETY: `result` points to `total` initialized bytes owned by the facade.
    let out = unsafe { std::slice::from_raw_parts(result, total) }.to_vec();
    // SAFETY: `result` came from `cubacle_solve` and is released once.
    unsafe { cubacle_free(result, total) };
    let text = String::from_utf8(out[HEADER..].to_vec()).expect("utf-8");
    (out[4], text)
}

#[test]
fn abi_version_is_one() {
    assert_eq!(cubacle_abi_version(), 1);
}

#[test]
fn solves_a_3x3_scramble_and_the_answer_replays() {
    let scramble = "R U R' U' F2 D L2 B' U2 R2 F D' L";
    let (status, text) = call(3, scramble, 25);
    assert_eq!(status, STATUS_OK, "{text}");
    let cube = Facelets::solved(PuzzleKind::Cube3)
        .apply(&scramble.parse().unwrap())
        .unwrap();
    assert!(cube.apply(&text.parse().unwrap()).unwrap().is_solved());
}

#[test]
fn solves_a_2x2_scramble_optimally_and_the_answer_replays() {
    let scramble = "R U R' U' F2 R";
    let (status, text) = call(2, scramble, 14);
    assert_eq!(status, STATUS_OK, "{text}");
    let cube = Facelets::solved(PuzzleKind::Cube2)
        .apply(&scramble.parse().unwrap())
        .unwrap();
    assert!(cube.apply(&text.parse().unwrap()).unwrap().is_solved());
}

#[test]
fn rejects_a_bad_size_and_bad_text_without_trapping() {
    assert_eq!(call(4, "R", 25).0, STATUS_BAD_INPUT);
    assert_eq!(call(3, "not a move", 25).0, STATUS_BAD_INPUT);
    assert_eq!(call(3, "\u{ff}\u{fe}", 25).0, STATUS_BAD_INPUT);
}
