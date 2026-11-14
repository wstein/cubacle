//! Measurement harness: `cubacle::engines::three::solve_front_goal` behind a C
//! ABI over linear memory, to measure it as WebAssembly. Not a contract: no
//! version, no stable names; it adds nothing to cubacle itself.
//!
//! Input (UTF-8): `facelets TAB twists TAB accepted TAB center_up`, the columns
//! 6 to 8 of tests/fixtures/front_goal_calendar.tsv, then `1` or `0`.
//! Output (UTF-8, tab separated): `status TAB moves TAB rotation TAB nodes TAB
//! tables_us`, status one of `ok`, `budget`, `noroute`, `invalid`.
//! The module imports `cubacle_host_now` (clock, ms) and `cubacle_host_random`.

mod host;

use std::time::Duration;

use cubacle::engines::three::{solve_front_goal, FrontFailure, FrontGoal, FrontLimits};
use cubacle::puzzle::{CenterTwists, Facelets, PuzzleKind};

/// Runs one case. `time_ms` 0 = the search default, `max_nodes` 0 = no cap,
/// `max_moves` 0 = the search default.
pub fn solve_text(input: &str, time_ms: u32, max_nodes: u64, max_moves: u32) -> String {
    let bad = |why: &str| format!("invalid\t{why}\t\t0\t0");
    let f: Vec<&str> = input.split('\t').collect();
    if f.len() != 4 {
        return bad("expected four tab separated fields");
    }
    let Ok(start) = Facelets::parse(f[0]) else {
        return bad("facelets");
    };
    let twists: Result<Vec<u8>, _> = f[1].split(',').map(str::parse).collect();
    let Ok(twists) = twists else {
        return bad("twists");
    };
    let Ok(twists): Result<[u8; 6], _> = twists.try_into() else {
        return bad("twists");
    };
    let Ok(twists) = CenterTwists::new(PuzzleKind::Cube3, twists) else {
        return bad("twists");
    };
    let groups: Result<Vec<Vec<u8>>, _> = f[2]
        .split(' ')
        .map(|g| g.split(',').map(str::parse).collect())
        .collect();
    let Ok(groups) = groups else {
        return bad("accepted");
    };
    let classes: Vec<&[u8]> = groups.iter().map(Vec::as_slice).collect();
    let Ok(goal) = FrontGoal::new(&classes, f[3] == "1") else {
        return bad("goal");
    };
    let mut limits = FrontLimits::default();
    if time_ms > 0 {
        limits = limits.with_time(Duration::from_millis(u64::from(time_ms)));
    }
    if max_nodes > 0 {
        limits = limits.with_max_nodes(max_nodes);
    }
    if max_moves > 0 {
        limits = limits.with_max_moves(max_moves.min(255) as u8);
    }
    match solve_front_goal(&start, &twists, &goal, &limits) {
        Ok(r) => format!(
            "ok\t{}\t{}\t{}\t{}",
            r.moves,
            r.rotation,
            r.nodes,
            r.tables.as_micros()
        ),
        Err(FrontFailure::BudgetExhausted { nodes }) => format!("budget\t\t\t{nodes}\t0"),
        Err(FrontFailure::NoRoute(m)) => format!("noroute\t{m}\t\t0\t0"),
        Err(FrontFailure::InvalidInput(m)) => format!("invalid\t{m}\t\t0\t0"),
    }
}

/// Allocates `len` bytes for the host to fill.
#[no_mangle]
pub extern "C" fn fm_alloc(len: usize) -> *mut u8 {
    let mut b = vec![0u8; len].into_boxed_slice();
    let p = b.as_mut_ptr();
    std::mem::forget(b);
    p
}

/// Releases a buffer from `fm_alloc` or `fm_solve`.
///
/// # Safety
/// Exactly a pointer and length obtained from this module, released once.
#[no_mangle]
pub unsafe extern "C" fn fm_free(p: *mut u8, len: usize) {
    if !p.is_null() {
        // SAFETY: a live allocation of `len` bytes from this module.
        drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(p, len)) });
    }
}

/// Solves one case; returns a buffer of four length bytes (little endian,
/// total) then the output text.
///
/// # Safety
/// `input` points to `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn fm_solve(
    input: *const u8,
    len: usize,
    time_ms: u32,
    max_nodes: f64,
    max_moves: u32,
) -> *mut u8 {
    // SAFETY: `len` readable bytes at `input`.
    let bytes = unsafe { std::slice::from_raw_parts(input, len) };
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => solve_text(s, time_ms, max_nodes as u64, max_moves),
        Err(_) => "invalid\tnot UTF-8\t\t0\t0".to_string(),
    };
    let total = 4 + text.len();
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(text.as_bytes());
    let mut b = out.into_boxed_slice();
    let p = b.as_mut_ptr();
    std::mem::forget(b);
    p
}

/// The byte length of a result buffer.
///
/// # Safety
/// `r` is a live result of `fm_solve`.
#[no_mangle]
pub unsafe extern "C" fn fm_result_len(r: *const u8) -> usize {
    // SAFETY: the header is the first four bytes.
    let h = unsafe { std::slice::from_raw_parts(r, 4) };
    u32::from_le_bytes([h[0], h[1], h[2], h[3]]) as usize
}
