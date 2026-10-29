---
title: "Contributing"
description: "How to contribute to cubacle."
weight: 7
---

# Contributing

## Workflow

- Work on `develop` or on a short-lived branch from it. `main` mirrors the upstream repository; do not commit to it.
- Write commit messages in the [Conventional Commits](https://www.conventionalcommits.org/) format, for example `fix(server): bound shutdown` or `docs: add CLI guide`.
- Keep each commit atomic: one logical change per commit.

## Quality gates

Run these checks before you commit. All of them must pass.

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --release
```

## Command examples

Some command examples in `docs/` are generated. A block between `<!-- example:ID -->` and `<!-- /example -->` is rewritten from the command listed under that `id` in `docs/examples.toml`, which is run with the release `cubacle` binary. After a change that alters the output of such a command, run:

```sh
cargo xtask docs
```

`cargo xtask docs --check` changes nothing; it prints a diff and exits with status 1 when a block is stale. Add only commands whose output is the same on every run: the file lists the options (`stderr` to show standard error, `mask` for JSON numbers such as `elapsedMicros`, which are written as 0).

## Docs site

The pages in `docs/` are built with [Hugo](https://gohugo.io/) using the layouts in `site/`. The workflow in `.github/workflows/pages.yml` pins the Hugo version. To build locally with that same version, copy the config, layouts and pages into a scratch directory and run Hugo there:

```sh
tmp=$(mktemp -d)
cp site/hugo.toml "$tmp/hugo.toml"
cp -r site/layouts "$tmp/layouts"
cp -r docs "$tmp/content"
hugo --source "$tmp" --destination "$tmp/public"
```

Open `$tmp/public/index.html` to read the result, or run `hugo server --source "$tmp"` for a live preview. Internal links between pages are written as `page.md` and resolved by Hugo, so a broken link fails the build.

## License of contributions

cubacle is licensed under AGPL-3.0-or-later. See [Credits and license](credits.md).
