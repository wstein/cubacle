---
id: 20261024133009
aliases: []
tags: ["type/evidence", "status/reference", "category/engines"]
---
Reading a 3x3 optimal solver with a sticker painter showed which ideas help us. Adopt: a restricted move set as the alphabet (a table built for the full alphabet stays a valid, weaker bound), enumeration of all solutions in canonical order with a stop count, a hard memory budget, cancel checked on every node, and a ranking by hand comfort. Do not adopt: the sticker array as the state, hashed pattern tables without stored keys and filled in a race (not deterministic, not compatible with fixed fixtures), greying the start state to express a partial target (it fixes one assignment and misses others), and a ranking that can put a longer solution first. Only ideas were taken; the code is GPL and none of it is used.

## Links

- [[A partial target is a map from slots to allowed colours]]
- [[Fingertrick ranking only orders solutions of the shortest length]]

Source: owner decision of 2026-10-24, lines 1-1
