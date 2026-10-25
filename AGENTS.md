# Agent and contributor guide

Rules for anyone, human or automated, who changes this repository.

## Project

cubacle is a solver and generator for Rubik's cubes (2x2, 3x3, 4x4 and 5x5 now, other
sizes planned), with a lightweight CLI and a stateless REST backend
(`crates/server`). Documentation is in `docs/` (Hugo, published from `develop`);
decision notes are in `notes/` (ChartScribe format) and are raw material, not
user documentation.

## Git

- History is linear. Rebase onto `develop` and fast-forward; never create merge commits.
- `main` mirrors the upstream repository (`cs0x7f/min2phase_rust`). Do not commit to it.
  Work on `develop` or on short-lived branches from it.
- Commits are conventional (`feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`,
  `ci`, `chore`; `!` for breaking changes), atomic and focused: one concern per commit,
  each commit builds and passes the checks.
- Commits written with AI assistance carry a `Co-Authored-By:` trailer naming the model.
- Do not push, tag or change repository settings unless the owner asks.
- Move code with `git mv` in its own commit, before changing it, so history follows.

## Checks (run before every commit)

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --release
```

Tests that record solver output (`tests/fixtures/`) must not be edited to make a
change pass; a change that alters solver output needs an explicit decision.

## Code

- Keep the search code unchanged unless the task is about the search.
- Default visibility is `pub(crate)`. Expose only what is meant to be the API.
- Dependencies point one way: puzzle types, then notation, then engines, then the
  applications (CLI, server). Engines do not use each other's internals; a
  reduction engine (the 4x4 and 5x5 now, 6x6 and up later) uses the 3x3 solver only through the
  `Solver` trait, with an injected instance, never through a global.
- File size: no source file over 1000 lines, ever. From about 500 lines, plan a split
  along a real responsibility. Most files should stay far smaller.
- Compute-heavy work never runs on async threads in the server.
- The server is stateless and binds to `127.0.0.1` by default.

## Documentation and notes

- Describe verified behavior only; run the commands you document.
- Write American English everywhere: texts, docs, notes, comments, identifiers, JSON
  and file keys, CLI options and output strings (color, center, neighbor, optimize,
  license, labeled). British spellings are mistakes. Rename without alias or fallback;
  the project is alpha, breaking is fine. `tests/fixtures` stay untouched except where
  the owner orders a change.
- Notes stay short, one idea each. Check whether docs pages need updating when notes change.

## License

AGPL-3.0-or-later. The original MIT grant of Chen Shuang stays in `LICENSE-MIT`;
provenance of third-party code is recorded in `NOTICE.md` before it lands.

## Dependencies and Rust version

Keep dependencies on their latest releases. `rust-version` is the lowest Rust version
that is measured to build the workspace (the `msrv` CI job), not a wish; raise it
deliberately when a dependency update requires it.
