---
id: 20261022140003
aliases: []
tags: ["type/design", "status/reference", "category/engines"]
---
The reference has no parity algorithms: a parity bit sits in the phase 2 and 3 centre coordinates, flipped by inner-slice quarter turns, and combined with the edge and corner parity, so the search ends only in a cube whose corners and paired edges are a valid 3x3x3. The port keeps this, so neither a single flipped paired edge nor two swapped edges needs a fixed algorithm; the move tables are the only thing to verify by replay. Loops must be bounded, as the reference can spin on an impossible cube.

## Links

- [[5x5 parity and limits]]
- [[4x4 phases and tables]]

Source: docs/design/4x4-port.md, lines 205-234
Source: src/engines/five/reduce.rs, lines 1-9
