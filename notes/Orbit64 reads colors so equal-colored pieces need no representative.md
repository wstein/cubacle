---
id: 20261025210100
aliases: []
tags: ["type/evidence", "status/reference", "category/notation"]
---
An Orbit64 token is a function of the stickers' colors alone, so a color state from the 4x4x4 on has exactly one token before any symmetry is looked at, and no representative among equal-colored pieces has to be chosen. A wing is read from the ordered color pair of its slot (24 slots, 24 pairs, orbits kept apart by depth) and a center orbit is stored as the colors it shows, so only centers are ambiguous as pieces. Two half-turn words with different pieces and equal colors, and the center-only swaps of the color tests, give one token on every size from the 4x4x4 to the 7x7x7; reading the colors back from a token returns the same cube on random states of all sizes.

## Links

- [[Orbit64 tokens]]
- [[Orbit64 state as mixed-radix digits]]
- [[Equal pieces are matched fixed points first]]
- [[Same-coloured inner pieces are equal and there is no strict mode]]
- [[The canonical token is the Orbit64 token of the standard-colored member of the detector's class]]

Source: src/notation/orbit64/pieces.rs, lines 108-138
Source: src/canonical/tests.rs, lines 205-280
