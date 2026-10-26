---
id: 20261025095527
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
The owner decided on 2026-10-25 that solving and patterns are color based. Inner pieces of one color that look identical, such as the centers of the 4x4x4 to 7x7x7, are equal: two cubes that differ only by a swap of same-colored centers have the same Facelets, the same answer from `solve` and the same fingerprint, patternness and library match from `patterns identify`. There is no strict mode. A target on the 4x4x4 and larger may ask for a scrambled arrangement of same-colored centers (centers of one kind in two colors on a face): the route ends in a center fix-up and the replay compares colors, as everywhere ([[A center fix-up pass lets --to reach mixed centers]]). The test `tests/same_colour_centres_test.rs` builds such pairs with center-only sequences on the 4x4x4, 5x5x5 and 6x6x6.

## Links

- [[A target is solved as a relative state]]
- [[A partial target is a map from slots to allowed colors]]
- [[Orbit64 reads colors so equal-colored pieces need no representative]]

Source: tests/same_colour_centres_test.rs, lines 1-4
