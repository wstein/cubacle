//! Browser WebAssembly facade of the 2x2x2 and 3x3x3 solvers.
//!
//! The module has no JavaScript glue: it exports a C ABI over linear memory,
//! so any host (a browser, a worker, Node) can call it with `WebAssembly`
//! alone. A call is: allocate the input with [`cubacle_alloc`], write a
//! scramble in standard notation, call [`cubacle_solve`], read the result
//! (a length, one status byte, then UTF-8 text), and release both buffers with
//! [`cubacle_free`].
//!
//! The module imports two functions from `env`. `cubacle_host_random` fills a
//! buffer with random bytes; the dependency `getrandom` needs it.
//! `cubacle_host_now` returns the clock in milliseconds; the solvers read it
//! through `cubacle::puzzle::Instant`, because `std::time::Instant` panics on
//! `wasm32-unknown-unknown`. See `docs/wasm.md`.

mod abi;
mod host;
mod solve;

pub use abi::{
    cubacle_abi_version, cubacle_alloc, cubacle_free, cubacle_result_len, cubacle_solve,
};
pub use solve::{
    solve_text, DEFAULT_BUDGET_MS, HEADER, STATUS_BAD_INPUT, STATUS_NO_SOLUTION, STATUS_OK,
};
