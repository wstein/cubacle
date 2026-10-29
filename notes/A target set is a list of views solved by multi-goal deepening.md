---
id: 20261029113742
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
A 3x3x3 target set is a list of views: per sticker position the set of sticker identities that may show there, and a state meets the set when it meets one view. A free whole-cube orientation is one view per rotation, so it costs no moves and needs no second search. The answer comes from an iterative deepening whose bound is the maximum of two pruning tables, one over all corners and one over up to six edges, each seeded from every projected state that some view accepts and so never above the true distance. The answer is therefore proven shortest, or the search says how deep it proved before a time, move or memory limit stopped it. It runs only on request (`--optimal` with open stickers, or `--any-orientation`), because building the corner table costs about two seconds.

## Links

- [[A partial target is a map from slots to allowed colors]]
- [[Partial targets are solved as a goal-set search]]
- [[The calendar cube needs labeled stickers and an open target and is a later extension]]

Source: docs/design/dont-care-targets.md, lines 391-415
