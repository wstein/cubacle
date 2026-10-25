---
id: 20261025210000
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
A cube that is a library pattern plus a few moves is found by a meet in the middle over block moves (single layers and outer two- and three-layer blocks, the layers 2-3), from both sides. A table holds every block sequence of up to da blocks from the state (or from the solved cube), keyed by a hash that ignores color names; each candidate pattern walks db blocks in each of its 24 orientations, and a hash match gives the extra moves, verified by replay. Sticker similarity is no distance (a state four wide-block moves from a pattern agrees on 39 % of its stickers), and a single-layer alphabet missed that case, so wide blocks are part of the alphabet. Candidates are ordered by differing stickers and cut by a safe displacement bound; the search is off unless asked for (`--repair`) because it takes seconds to a minute, and it reports a time-out honestly.

## Links

- [[The detector explains patterns and never replaces the search]]
- [[Shorter solutions beat compute time]]

Source: src/patterns/repair/mod.rs, lines 1-25
