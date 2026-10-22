---
id: 20261022120012
aliases: []
tags: ["type/design", "status/reference", "category/engines"]
---
A cold CLI call with a tiny time budget can fail while the table builds. `prepare()` avoids this by building the table up front. Embedding the table through build.rs was suggested but is not decided.

## Links

- [[2x2 timing]]
- [[2x2 embedded table idea]]

Source: docs/engines.md, lines 24-24
Source: src/engines/two/mod.rs, lines 433-436
Source: README.md, lines 37-37
Source: owner decision of 2026-10-21, lines 1-1
