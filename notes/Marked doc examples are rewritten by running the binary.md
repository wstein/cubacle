---
id: 20261029111952
aliases: []
tags: ["type/process", "status/decided", "category/process"]
---
`cargo xtask docs` runs the commands listed in `docs/examples.toml` with the release `cubacle` binary and rewrites the text between `<!-- example:ID -->` and `<!-- /example -->` in the pages; `--check` prints a diff and exits 1 for a stale block. Only deterministic commands belong in the list: standard output by default, standard error on request, and JSON number keys such as `elapsedMicros` masked to 0. Curl examples against the server and the stderr filter rule of the CLI pages are not covered yet.

## Links

- [[Feature workflow]]

Source: xtask/src/docs.rs, lines 1-20
Source: docs/examples.toml, lines 1-11
