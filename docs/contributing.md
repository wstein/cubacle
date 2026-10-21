---
title: Contributing
---

# Contributing

## Workflow

- Work on a branch, not directly on `main`.
- Write commit messages in the [Conventional Commits](https://www.conventionalcommits.org/) format, for example `fix(server): bound shutdown` or `docs: add CLI guide`.
- Keep each commit atomic: one logical change per commit.

## Quality gates

Run these checks before you commit. All of them must pass.

```sh
cargo fmt --check
cargo clippy --release --all-targets -- -D warnings
cargo test --release --all-targets
```

To include the server tests, run the workspace form of the test command:

```sh
cargo test --workspace --release
```

## Licence of contributions

cubacle is licensed under AGPL-3.0-or-later. See [Credits and licence](credits.md).
