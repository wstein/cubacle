---
id: 20261024090000
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
SSE (Superset ENG) is read and written beside WCA and SiGN, on all sizes 2x2 to 7x7. Reading is automatic per token by the one expression reader; a word is SSE when a prefix letter `T N V M S W C`, optional numbers and then a face letter follow (`TR`, `T3R`, `NL`, `N2-4R`, `VL`, `MR`, `SR`, `WR`, `CR`). `M`, `M2`, `S'`, `S2` and `x y z` have no face letter and stay WCA. Writing is `--notation sse` and the `notation` field `sse`; the writer needs the cube size, so `Notation::write` takes the `PuzzleKind`.

Only the notation as the six pages describe it was used, read 2026-10-23: no code, no text and no table of the site was copied, so nothing is added to NOTICE.md. The size pages differ in which twists exist: the 2x2 has faces and `C`; the 3x3 adds `T`, `M`, `S`; the 4x4 adds depth for `T` and `M`, `W` and slice ranges but has no `N` or `V`; the 5x5, 6x6 and 7x7 add `N`, numbered ranges and `V`. Permutation and orientation cycles of the pages are out of scope and are rejected.

Decisions the pages leave open: an uneven middle (`M` on a 4x4 or `M3R` on a 6x6) puts the extra layer towards the named face; `S2R` means layers 1 and 2 of both faces and overlapping ends of a slice range turn once; the pages' "equivalent" rewrites (`S2-4F2` as `SF2`) differ by a cube rotation and are not applied; `V3R` and `N2-4R` name the same layers, and the writer keeps the shorter, in the order tier, `W`, `V`, `M`, `N`, range. Every SSE token expands to plain moves at read time; tests turn each twist layer by layer on sizes 2 to 7 and read every written move back.

## Links

- [[Move input rules are WCA and SiGN]]
- [[SiGN notation as an output option]]

Source: https://www.randelshofer.ch/rubik/pocket/doc/supersetENG_2x2.html, lines 1-1
Source: https://www.randelshofer.ch/rubik/patterns/doc/supersetENG_3x3.html, lines 1-1
Source: https://www.randelshofer.ch/rubik/revenge/doc/supersetENG_4x4.html, lines 1-1
Source: https://www.randelshofer.ch/rubik/professor/doc/supersetENG_5x5.html, lines 1-1
Source: https://www.randelshofer.ch/rubik/vcube6/doc/supersetENG_6x6.html, lines 1-1
Source: https://www.randelshofer.ch/rubik/vcube7/doc/supersetENG_7x7.html, lines 1-1
Source: owner task of 2026-10-24, lines 1-1
Source: src/notation/sse.rs, lines 1-14
Source: src/notation/sse_write.rs, lines 1-22
Source: docs/cli.md, lines 573-647
