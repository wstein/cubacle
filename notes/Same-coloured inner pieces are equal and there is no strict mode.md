---
id: 20261025095527
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
The owner decided on 2026-10-25 that solving and patterns are colour based. Inner pieces of one colour that look identical, such as the centres of the 4x4x4 to 7x7x7, are equal: two cubes that differ only by a swap of same-coloured centres have the same Facelets, the same answer from `solve` and the same fingerprint, patternness and library match from `patterns identify`. There is no strict mode. A target on the 4x4x4 and larger still cannot ask for a scrambled arrangement of same-coloured centres and is refused with `the target has centres of different colours`. The test `tests/same_colour_centres_test.rs` builds such pairs with centre-only sequences on the 4x4x4, 5x5x5 and 6x6x6.

## Links

- [[A target is solved as a relative state]]
- [[A partial target is a map from slots to allowed colours]]

Source: tests/same_colour_centres_test.rs, lines 1-4
