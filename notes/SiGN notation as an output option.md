---
id: 20261023022802
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
Implemented: moves are read in WCA and SiGN spellings and written in either. Input detects the notation per token: a lowercase face letter (`r`, `3u2`) is a SiGN wide turn, every other spelling is shared (`R`, `2R`, `Rw`, `3Rw`). No spelling has two meanings, so nothing needs a tie-break. Output is `--notation wca|sign` on `solve` and `scramble` and a `notation` field of `/v1/solve` and `/v1/scramble` (default `wca`). The code is `src/notation/sign.rs`; it translates at the text boundary, the engines and the metric (every move counts one) are unchanged. Icon labels stay WCA.

Depths follow the SiGN page at mzrg.com and WCA Article 12a2: `2R` is the second layer only, `r` and `2r` the two outer layers, `3r` the three outer layers. Later, layer ranges, groups and commutators were added and the REST API got a `scramble` field; see [[Move input rules are WCA and SiGN]].

## Links

- [[Open items]]
- [[Input notations]]

Source: owner decision of 2026-10-23, lines 1-1
Source: src/notation/sign.rs, lines 1-9
Source: docs/cli.md, lines 315-349
