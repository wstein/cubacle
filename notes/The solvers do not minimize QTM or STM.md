---
id: 20261024170001
aliases: []
tags: ["type/evidence", "status/reference", "category/engines"]
---
Measured on fixed pseudo-random scrambles: the 3x3x3 default answer averages 20.70 HTM and 30.90 QTM (18.30 and 25.40 with `--improve -b 300`), and `solve --moves qtm` is only the default answer rewritten in quarter turns (25 moves where `--improve` found 11). Only the 2x2x2 is proven optimal, in HTM. The 4x4x4 and larger count plain moves of their own alphabet. `solve` therefore prints the metric counts as facts about the answer and never says shortest in QTM or STM.

## Links

- [[Move metrics count turns of contiguous blocks]]
- [[Restricted solves search the alphabet's own letters]]

Source: docs/metrics.md, lines 111-126
