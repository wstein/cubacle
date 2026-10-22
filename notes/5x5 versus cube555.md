---
id: 20261022140000
aliases: []
tags: ["type/evidence", "status/reference", "category/engines"]
---
Measured on 100 identical random cubes (Mac14,10, 12 cores, release build, Java 21), every solution replayed with `Facelets::apply`. cube555 answers only the reduction; both solvers are given the same final stage (our Min2Phase hand-off), so totals are comparable. Lengths: cube555 reduction 51.5 mean (median 52, min 48, max 54, claimed 51 to 52) plus 21.6 for the 3x3x3 gives 73.1 total (70 to 76); our greedy engine 412.4 total (352 to 463, p90 441). The greedy engine is about 5.6 times longer, and Java solves in 0.87 s mean (median 0.66 s, max 5.6 s) against 1.3 ms for ours. Cold start: cube555 needs 50 s to build and write 237 MB of tables (claimed 240 MB), 1.8 s to load them afterwards; ours builds its 3-cycle library in 0.25 s. Peak resident memory: cube555 1.6 GB cold and 0.76 GB warm in the JVM (claimed at most 500 MB, not met), ours 5 MB. The conclusion is that only a multi-phase search closes the gap.

## Links

- [[5x5 reduction engine]]
- [[5x5 parity and limits]]

Source: tools/bench/run.sh, lines 1-21
Source: tools/bench/five_compare.rs, lines 1-10
Source: tools/bench/README.md, lines 1-14
