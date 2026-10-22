---
id: 20261022140000
aliases: []
tags: ["type/evidence", "status/reference", "category/engines"]
---
Measured on 100 identical random cubes (12 cores, release build), every solution replayed. cube555 answers only the reduction, so both solvers get the same 3x3x3 hand-off. Total moves are equal: mean 73.1, median 73, p90 75 for both, with 1 or 12 threads. Mean solve time is 865 ms for the Java reference, 92 ms with 12 threads and 444 ms with one. Table build is about 50 s for Java, 3.3 s on 12 threads and 18.3 s on one; reading them again takes about 1 s. Peak memory is 0.76 GB for Java warm and 327 to 335 MB for ours, above the 300 MB aim.

## Links

- [[5x5 reduction engine]]
- [[5x5 memory above target]]

Source: tools/bench/run.sh, lines 1-21
Source: tools/bench/five_compare.rs, lines 1-10
Source: docs/engines.md, lines 50-60
