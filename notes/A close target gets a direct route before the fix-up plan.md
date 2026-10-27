---
id: 20261027120000
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
`centers::plan` first runs a meet-in-the-middle search (`within::search`, the single inner layers in the alphabet, at most 6 moves, 100 MB) between start and target. A route it finds is the answer: the plan then has the start itself as its goal and the route as its tail, stage `direct route`. Reason: for a target a few moves from the start, the uniform goal plus fix-up cost 40 to 170 moves more than the moves between the two states (5x5x5 median +42, 7x7x7 351 against 183 at three slices). Six moves, not seven: on the 7x7x7 six take 0.2 s and 9 MB when nothing is found, seven take 6 s and 700 MB. The search is bounded by moves and memory, not by a node counter or the clock (a 20 s stop is only a safety), so the answer is the same on every machine. The route is shortest in its alphabet, which holds every outer block and single layer an engine move can be, so the engine route is not compared with it. Farther targets keep the fix-up plan unchanged.

## Links

- [[A center fix-up pass lets --to reach mixed centers]]
- [[The end of the engine's center sorting is shortened by a beam search]]

Source: src/centers/direct.rs, lines 1-40
