//! Browser WebAssembly facade of the 2x2x2 and 3x3x3 solvers.
//!
//! The module has no JavaScript glue: it exports a C ABI over linear memory,
//! so any host (a browser, a worker, Node) can call it with `WebAssembly`
//! alone. A call is: allocate the input with [`cubacle_alloc`], write a
//! scramble in standard notation, call [`cubacle_solve`], read the result
//! (a length, one status byte, then UTF-8 text), and release both buffers with
//! [`cubacle_free`].
//!
//! The one host import is `cubacle_host_random` (module `env`), which fills a
//! buffer with random bytes; the dependency `getrandom` needs it. The solvers
//! read the clock through `std::time::Instant`, which the
//! `wasm32-unknown-unknown` target does not implement; the clock calls in the
//! engines are the measured blocker described in `docs/wasm.md`.

use cubacle::engines::three::Min2PhaseSolver;
use cubacle::engines::two::PocketSolver;
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind, SolveOptions, Solver};

/// Bytes before the text of a result: the total length and the status.
pub const HEADER: usize = 5;

/// The result text is the solution.
pub const STATUS_OK: u8 = 0;
/// The size or the scramble text is not accepted; the text says why.
pub const STATUS_BAD_INPUT: u8 = 1;
/// The solver found no solution; the text says why.
pub const STATUS_NO_SOLUTION: u8 = 2;

#[doc(hidden)]
pub fn solve_text(size: u32, scramble: &str, max_moves: u32) -> (u8, String) {
    let kind = match size {
        2 => PuzzleKind::Cube2,
        3 => PuzzleKind::Cube3,
        other => return (STATUS_BAD_INPUT, format!("size {other} is not 2 or 3")),
    };
    let moves: MoveSequence = match scramble.parse() {
        Ok(moves) => moves,
        Err(error) => return (STATUS_BAD_INPUT, format!("scramble: {error}")),
    };
    let cube = match Facelets::solved(kind).apply(&moves) {
        Ok(cube) => cube,
        Err(error) => return (STATUS_BAD_INPUT, format!("scramble: {error}")),
    };
    let options = SolveOptions::default().with_max_moves(max_moves.min(255) as u8);
    let result = match kind {
        PuzzleKind::Cube2 => PocketSolver::new().solve(&cube, &options),
        _ => Min2PhaseSolver::shared().solve(&cube, &options),
    };
    match result.solution() {
        Some(solution) => (STATUS_OK, solution.to_string()),
        None => (
            STATUS_NO_SOLUTION,
            result.message.unwrap_or_else(|| "no solution".to_string()),
        ),
    }
}

/// Allocates `len` bytes in linear memory for the host to fill.
#[no_mangle]
pub extern "C" fn cubacle_alloc(len: usize) -> *mut u8 {
    let mut buffer = vec![0u8; len].into_boxed_slice();
    let pointer = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    pointer
}

/// Releases a buffer from [`cubacle_alloc`] or [`cubacle_solve`].
///
/// # Safety
/// `pointer` and `len` must be exactly a pointer and length obtained from this
/// module (for a result, the length from [`cubacle_result_len`]), released once.
#[no_mangle]
pub unsafe extern "C" fn cubacle_free(pointer: *mut u8, len: usize) {
    if !pointer.is_null() {
        // SAFETY: the caller passes a live allocation of `len` bytes from `cubacle_alloc`.
        drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(pointer, len)) });
    }
}

/// Solves `scramble` (UTF-8, `len` bytes at `input`) on a cube of `size` 2 or
/// 3 with at most `max_moves` moves. Returns a buffer of [`cubacle_result_len`]
/// bytes: four bytes of total length (little endian), the status byte, then
/// the text.
///
/// # Safety
/// `input` must point to `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn cubacle_solve(
    size: u32,
    input: *const u8,
    len: usize,
    max_moves: u32,
) -> *mut u8 {
    // SAFETY: the caller guarantees `len` readable bytes at `input`.
    let bytes = unsafe { std::slice::from_raw_parts(input, len) };
    let (status, text) = match std::str::from_utf8(bytes) {
        Ok(scramble) => solve_text(size, scramble, max_moves),
        Err(_) => (STATUS_BAD_INPUT, "scramble is not UTF-8".to_string()),
    };
    let total = HEADER + text.len();
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.push(status);
    out.extend_from_slice(text.as_bytes());
    let mut buffer = out.into_boxed_slice();
    let pointer = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    pointer
}

/// The byte length of a buffer returned by [`cubacle_solve`], read from its header.
///
/// # Safety
/// `result` must be a pointer returned by [`cubacle_solve`] and not yet released.
#[no_mangle]
pub unsafe extern "C" fn cubacle_result_len(result: *const u8) -> usize {
    // SAFETY: the header is the first four bytes of the live buffer.
    let header = unsafe { std::slice::from_raw_parts(result, 4) };
    u32::from_le_bytes([header[0], header[1], header[2], header[3]]) as usize
}

#[cfg(all(target_arch = "wasm32", getrandom_backend = "custom"))]
mod host_random {
    #[link(wasm_import_module = "env")]
    extern "C" {
        fn cubacle_host_random(pointer: *mut u8, len: usize);
    }

    /// The `getrandom` custom backend: the host fills the buffer.
    ///
    /// # Safety
    /// Called by `getrandom` with a writable buffer.
    #[no_mangle]
    unsafe extern "Rust" fn __getrandom_v03_custom(
        dest: *mut u8,
        len: usize,
    ) -> Result<(), getrandom::Error> {
        // SAFETY: `dest` is writable for `len` bytes, as `getrandom` guarantees.
        unsafe { cubacle_host_random(dest, len) };
        Ok(())
    }
}
