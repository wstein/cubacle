---
id: 20261025170000
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
A word that fails as one move may be several moves without spaces. It is cut after a quote that is followed by a letter or digit, after a 2 that follows a face letter and precedes an SSE prefix letter, and after the face letter of an SSE move that is followed by a capital letter. Every piece must parse, or the word keeps its original error, and a word that already parses is never cut, so no accepted input changes. `F2R` is not cut: it is `F2 R` or `F 2R`, and the parser does not guess. `2'` is read as a half turn in SSE as it is in WCA.

## Links

- [[Third-party pattern collections give names and facts, never move lists]]

Source: owner decision of 2026-10-25, lines 1-1
