//! Repository tasks. `cargo xtask docs [--check]` keeps the command examples in `docs/`
//! true: it runs the commands of `docs/examples.toml` with the release `cubacle` binary
//! and rewrites the marked blocks of the pages.

mod docs;

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
        _ => {
            eprintln!("usage: cargo xtask docs [--check]");
            ExitCode::from(2)
        }
    }
}
