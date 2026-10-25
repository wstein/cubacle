---
id: 20261025173000
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
A colour state is read as corners only when each corner shows all three colours, in turning order, of one real piece; the two side colours alone fit the mirror image of that piece too. The 5x5x5 engine once matched the side pair only, so a cube whose corners were mirrored (the up/down colour exchanged) was read as a valid corner permutation, although the corners of a reachable cube are always a rotation of the solved ones. Such a state is impossible and is refused with the corner sentence of the validity check. The CLI and the server were never exposed, because `notation::check` runs first; the engine check guards callers that use the engine directly.

## Links

- [[One check names an impossible state on every size]] - the shared validity rule set the engines repeat for direct callers
- [[5x5 reduction engine]] - the engine whose colour reading was fixed

Source: src/engines/five/cubie.rs, lines 208-300
Source: tests/mirror_corners_test.rs, lines 1-110
