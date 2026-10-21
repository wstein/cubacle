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

## Licence of contributions

cubacle is licensed under AGPL-3.0-or-later. See [Credits and licence](credits.md).
