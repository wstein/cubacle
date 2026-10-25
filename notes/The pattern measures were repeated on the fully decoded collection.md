---
id: 20261025230001
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
After the importer learned Mac Roman, the measures of patternness, kinds and recognition were repeated on all readable lines of the six tables: 110 patterns for the 2x2x2 and 1119 to 1262 for the other sizes (before, a quarter of the lines). Patternness at least 95 holds for 74, 75, 79, 86, 75 and 89 % of the patterns (2x2x2 to 7x7x7), at least 90 for 74, 86, 87, 94, 83 and 94 %, the median is 100 everywhere. The acceptance goal, most patterns reaching 90 in patternness or in a kind other than IRREG, holds for every size: 92.7, 93.4, 99.0, 99.9, 99.5 and 99.6 %. The kind ORBIT decided 0.3 to 1.6 % of the patterns and was alone in reaching 90 for at most 0.4 %, but fired on 6 to 16 % of scrambles of 4 to 12 turns, so it was removed instead of made stricter (its threshold was already perfect). For the held-out 30 % of the patterns, `identify` finds an exact match for 73, 46, 35, 28, 24 and 16 %, the closest pattern agrees on 90 % of the stickers or more for 65 to 80 % of them from the 3x3x3 up, and scrambles of 8 or 12 turns stay at a median of 43 to 56 %, so 90 % is a strong hint. The tables stay outside the repository; only these aggregates are kept, measured with `tools/bench/kind_rates.rs local`.

## Links

- [[Patternness is symmetry and uniformity, calibrated on random cubes and the library]]
- [[Pattern kinds name what is regular and are calibrated on short scrambles]]

Source: owner request of 2026-10-25, lines 1-1
