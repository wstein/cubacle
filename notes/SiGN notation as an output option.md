---
id: 20261023022802
aliases: []
tags: ["type/idea", "status/open", "category/open-items"]
---

Moves are written in WCA notation (`3Lw2`, `2R`, `Rw'`). SiGN notation (lowercase letters for wide turns, a leading number for the layer count) is the other common form on big cubes. An output option, for example `--notation wca|sign`, would translate at the writing step only; the engines and the metric (every move counts one) stay unchanged. WCA stays the default. The input parser reads only the WCA spellings (`Rw`, `3Rw`, `2R`); it rejects `r` and `3r` (checked with `cubacle facelets`), so SiGN would be an input question as well as an output one. The SiGN rules for depths above two need a check before any code.

## Links

- [[Open items]]
- [[Input notations]]

Source: owner decision of 2026-10-23, lines 1-1
