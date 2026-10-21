---
id: 20261022120012
aliases: []
tags: ["type/design", "status/reference", "category/engines"]
---
A cold CLI call with a tiny time budget can fail while the table builds. `prepare()` avoids this by building the table up front. Embedding the table through build.rs was suggested but is not decided.

## Links

- [[2x2 timing]]
- [[2x2 embedded table idea]]

Source: project history of cubacle, lines 1-1
