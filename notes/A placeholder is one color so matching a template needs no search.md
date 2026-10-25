---
id: 20261025200600
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
A template state matches when some assignment of colors to the placeholders makes every sticker agree and the guard true, but no search is needed to find it. A placeholder is one color on all its stickers, so the only candidate for its color is the color of its first sticker; one pass over the stickers either fixes the whole assignment or finds a disagreement, and the guard is evaluated once on the result. Searching and counting come later and enumerate assignments of colors, not matches. The name of a placeholder is the whole token (`a1` is a variable called a1, the number has no built-in meaning, in particular not a face), a `*` at the end of a name in the color constraints (`a*`) stands for the same index in all arguments of one term, as a conjunction over every index that occurs, and is expanded when the guard is parsed. Expansion, nesting depth and the length of the constraints are limited with clear errors so that no input makes the parser grow without bound or panic.

## Links

- [[Colour schemes]]
- [[Patterns are defined by structure and searched]]
- [[Don't-care stickers are written X or named as a subset]]

Source: src/notation/template/matching.rs, lines 1-45
Source: src/notation/template/resolve.rs, lines 1-60
