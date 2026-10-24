---
id: 20261024133008
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
A target with don't-care stickers is stored as a map from position to an optional colour and tested directly on the current state; the start is never greyed. Per slot it becomes a set of allowed piece and twist pairs, and the heuristic is the maximum of several relaxed pattern tables over the slots that have a cared sticker, seeded from every allowed assignment so the bound never exceeds the true distance. With nothing to look up the bound is zero, which is correct and slow, so the search gets a depth and node limit and says so. On 4x4 and up the same colour target removes the need to tell equal centres apart, which is today's limit of --to.

## Links

- [[A target is solved as a relative state]]
- [[Don't-care stickers are written X or named as a subset]]

Source: owner decision of 2026-10-24, lines 1-1
