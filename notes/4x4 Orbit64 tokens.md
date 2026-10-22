---
id: 20261022140008
aliases: []
tags: ["type/question", "status/open", "category/open-items"]
---
The Orbit64 format puts the 4x4x4 in 27 characters and 156.96 bits, which equals 8! 3^7 24! (24!/(4!)^6), that is the usual 7.4e45 positions times 24, with the frame always canonical on even cubes. Its one reference vector is a valid cube the reference solver solves in 45 moves. Slot numbering of wings and centres and the centre ranking are not verified; decoding is a separate later commit accepted only against that vector, and more vectors are requested.

## Links

- [[Orbit64 tokens]]
- [[Orbit64 slot numbering caveat]]

Source: docs/design/4x4-port.md, lines 267-283
Source: src/notation/orbit64.rs, lines 7-8
