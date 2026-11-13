//! C ABI entry points of the WebAssembly facade (ABI 1 and 2).

use crate::solve::{solve_text, solve_text_budget, HEADER, STATUS_BAD_INPUT};

/// ABI version of this module. 1 is the scramble-to-solved C entry points
/// with the fixed default budget; 2 adds [`cubacle_solve_budget`] and keeps
/// every ABI 1 export unchanged.
#[no_mangle]
pub extern "C" fn cubacle_abi_version() -> u32 {
    2
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
    result_buffer(status, &text)
}

/// ABI 2: like [`cubacle_solve`] with a wall-clock budget of `budget_ms`
/// milliseconds (0 means the 2000 ms of ABI 1). A search that has not found a
/// solution when the budget runs out returns status 3 (timeout) instead of
/// blocking the host. `cubacle_solve` itself is unchanged.
///
/// # Safety
/// `input` must point to `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn cubacle_solve_budget(
    size: u32,
    input: *const u8,
    len: usize,
    max_moves: u32,
    budget_ms: u32,
) -> *mut u8 {
    // SAFETY: the caller guarantees `len` readable bytes at `input`.
    let bytes = unsafe { std::slice::from_raw_parts(input, len) };
    let (status, text) = match std::str::from_utf8(bytes) {
        Ok(scramble) => solve_text_budget(size, scramble, max_moves, budget_ms),
        Err(_) => (STATUS_BAD_INPUT, "scramble is not UTF-8".to_string()),
    };
    result_buffer(status, &text)
}

fn result_buffer(status: u8, text: &str) -> *mut u8 {
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
