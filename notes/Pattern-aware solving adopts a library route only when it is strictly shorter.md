---
id: 20261025191500
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
`solve --pattern-aware` looks the cube up in the active pattern library (built-in plus the files of the config) and answers with the library route only when it has strictly fewer moves than the solver's answer; the solver still runs, so the answer is never longer and the default output without the flag does not change. Fewer moves for the person beat compute time, and whether the flag becomes the default is the owner's decision.

The lookup is exact. Every generator of the library, for every symmetry of the cube (24 turns and their 24 mirrors), is stored as a sticker colouring with the colours renamed to follow their faces; a state is one hash lookup (a few microseconds; building the map for the built-in library takes about a millisecond). The route is the generator carried over to that position (faces mapped, turn direction flipped under a mirror) and inverted, and it is replayed before it is used. A `kind != IRREG` pre-filter was measured and dropped: it missed 12 to 15 % of the hits and costs more than the lookup. With `--to`, the relative state R and the state of the swapped call (R inverse) are both looked up.

Routes keep to the solver's alphabet: face turns on the 3x3x3, plus outer blocks (`Rw`) from the 4x4x4 up, single inner layers only with `--slices`. Otherwise a three-move slice route would be counted as shorter while it is longer in half-turn moves. The 2x2x2 solver is optimal, so the flag does nothing there.

## Links

- [[Pattern shortcut finds hand-built cubes]]
- [[A target is solved as a relative state]]
- [[Pattern files are listed in cubacle.toml and the default is the built-in library]]

Source: src/patterns/route.rs, lines 1-10
Source: src/pattern_aware_cmd.rs, lines 1-8
