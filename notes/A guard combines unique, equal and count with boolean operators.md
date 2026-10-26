---
id: 20261025140104
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
Planned: a guard is a condition line over the schema variables. unique(...) holds when all named variables differ, equal(...) when they are the same, and count(...) is the number of distinct colors among them, comparable with numbers. Conditions combine with `&&`, AND or and; `||`, OR or or; `!` or NOT; and compare with `==` and `!=`. Both spellings are decided so shell users and readers of prose get the form they expect. Example: unique(a1,b1,c1) for each face says the three ring colors of that face differ.

## Links

- [[A schema grid is written with lowercase variables, uppercase colors and don't-care dots]] defines the variables a guard talks about.
- [[An x star in a guard stands for any index number and not for a face]] shortens guards that repeat per face.
- [[A partial target is a map from slots to allowed colors]] is the plain target form; a guard goes beyond it by relating slots.

Source: owner decision of 2026-10-25, lines 1-1
