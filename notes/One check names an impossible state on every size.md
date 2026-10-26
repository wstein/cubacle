---
id: 20261025110001
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
`notation::check` returns a typed `Impossible` reason for any state of the 2x2x2 to the 7x7x7; it reuses the Orbit64 piece reader, so one rule set decides what a cube can be. The engines (2x2x2, 4x4x4, 5x5x5) and the commands that read a state call it and print the same words; `solve`, `convert`, `solve --now`, `--to` and `patterns find` refuse an impossible state, while `patterns identify` and `patterns cycles` only warn on standard error, because patterns on illegally assembled cubes are legitimate. A hint (`turn one corner back`, `flip one edge back`, `swap two edges`, or the colors that have too many or too few stickers) is printed only where the repair is certain.

## Links

- [[Orbit64 state as mixed-radix digits]]
- [[Input notations]]

Source: src/notation/impossible.rs, lines 1-110
Source: src/state_check.rs, lines 1-50
