---
id: 20261025140100
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
Planned, not implemented: `patterns find --template` will search for states that fit a schema grid and satisfy a guard, so that a family of patterns (for example every face with the same ring structure) is one question instead of many. It is decided that a concept paper and a spike come first. The parts are separate atoms so each can be judged alone: the grid, the colour scheme, the guard operators and the index wildcard.

## Links

- [[A schema grid is written with lowercase variables, uppercase colours and don't-care dots]] is the grid half of the question.
- [[A colour scheme in the schema header line turns uppercase letters into colours]] says how fixed colours are read.
- [[A guard combines unique, equal and count with boolean operators]] is the condition half.
- [[An x star in a guard stands for any index number and not for a face]] lets one guard cover every face.
- [[Don't-care targets follow eight owner decisions]] is the earlier decision that the schema's dots extend.
- [[One pattern search serves solve, the library and patterns find]] is the search this template mode would sit on.

Source: owner decision of 2026-10-25, lines 1-1
