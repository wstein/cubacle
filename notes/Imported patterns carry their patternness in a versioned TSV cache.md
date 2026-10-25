---
id: 20261025143000
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
The import cache is a tab-separated `.tsv` file, one line per generator, with the moves as the last field because they are the one free-length column. The first line is `#cubacle-patterns 1 patternness-v1`: the format version and the patternness formula version. The patternness is computed once at import. When the code's formula version differs, stored values are ignored and the file is rewritten on the next load, so a changed formula never shows stale numbers. Built-in patterns compute it live. `patterns --sort patternness` and `--min-patternness` use the highest value over the sizes shown.

## Links

- [[Patternness is symmetry and uniformity, calibrated on random cubes and the library]]
- [[Third-party pattern collections give names and facts, never move lists]]

Source: owner decision of 2026-10-25, lines 1-1
