//! Repository tasks. `cargo xtask docs [--check]` keeps the command examples in `docs/`
//! true: it runs the commands of `docs/examples.toml` with the release `cubacle` binary
//! and rewrites the marked blocks of the pages. `cargo xtask wasm-deps` fails when the
//! normal dependency graph of the browser module holds `clap`, `sysinfo`, `toml` or `ureq`;
//! `cargo xtask wasm` builds the module from a clean tree and writes its manifest.

mod docs;
mod manifest;
mod wasm;
mod wasm_deps;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["docs"] => docs::run(false),
        ["docs", "--check"] => docs::run(true),
        ["wasm"] => wasm::run(),
        ["wasm-deps"] => wasm_deps::run(),
        _ => {
            eprintln!(
                "usage: cargo xtask docs [--check] | cargo xtask wasm-deps | cargo xtask wasm"
            );
            ExitCode::from(2)
        }
    }
}
