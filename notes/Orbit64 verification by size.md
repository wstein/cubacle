---
id: 20261022042148
aliases: []
tags: ["type/evidence", "status/reference", "category/notation"]
---
Official Orbit64 vectors cover the 2x2x2 to 5x5x5 (20 each, all decode to the listed facelets and encode back to the exact token); the 6x6x6 and 7x7x7 have the solved cube and, in the format's test suite, two scrambled vectors each as orbit coordinates with tokens (`tests/fixtures/orbit64_orbits.txt`, which also lists the scrambled vectors of the smaller sizes); both directions match. Further scrambled states are checked by round trip.

## Links

- [[Orbit64 tokens]]
- [[Orbit64 sizes 6 and 7 verified]]

Source: tests/fixtures/orbit64.txt, lines 1-4
Source: src/notation/orbit64/tests.rs, lines 22-33
