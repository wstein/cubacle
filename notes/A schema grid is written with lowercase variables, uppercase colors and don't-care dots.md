---
id: 20261025140102
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
Planned: a schema is a grid of tokens, one face per line or faces separated by `/`; tokens are separated by comma, space or `/`. A lowercase letter with an optional number (a, b1) is a symbolic color variable that may take any color; an uppercase letter is a fixed color; `.` and `?` are don't care. Variables make the schema say how stickers relate (these two are equal) without naming the color, which is why one schema covers all colorings of a pattern. Example, 5x5, one face (25 tokens, rows in order): `a b b b a b b c b b b c U c b b b c b b a b b b a`, with the face index appended to every variable.

## Links

- [[Don't-care stickers are written X or named as a subset]] fixes X as the don't-care in cube strings; the schema keeps `.` and `?` because it is a separate grid notation.
- [[A guard combines unique, equal and count with boolean operators]] gives the variables their conditions.

Source: owner decision of 2026-10-25, lines 1-1
