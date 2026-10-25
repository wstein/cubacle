---
id: 20261025110000
aliases: []
tags: ["type/decision", "status/decided", "category/project"]
---
The big public pattern collection is used as a local data basis for proofs: hit rates, patternness and kinds, and pattern-aware solving are measured against it, because our own library holds only a handful of patterns per size. It is never on by default. A new installation lists only the built-in library; the user adds the source files by hand in `~/.config/cubacle/config.toml`, where the raw URLs stand commented out. The files live outside the repository, nothing of them is committed, bundled or fetched by code, and the tests in the repository do not depend on them. Measurements and docs carry aggregates, pattern ids and category names only, never sequences or his text. An experiment reports the built-in library and the built-in plus imported collection side by side, with the imported patterns split into library entries and held-out test states so that recognition is not trivial.

## Links

- [[Third-party pattern collections give names and facts, never move lists]]
- [[Pattern files are listed in cubacle.toml and the default is the built-in library]]

Source: owner decision of 2026-10-25, lines 1-1
