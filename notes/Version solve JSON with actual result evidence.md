---
id: 20261029225001
aliases: []
tags: ["type/decision", "status/decided", "category/backend"]
---
From v0.2.0 solve JSON defaults to format 2. Format 1 remains selectable with its original
bytes. Frame and HTM proof metadata follow the actual search result, not requested flags;
composition does not inherit a shortest-path claim. The CLI and HTTP server share one writer.

## Links

- [[2x2 solver]]

Source: src/report/mod.rs, lines 1-260
Source: src/puzzle/proof.rs, lines 1-38
Source: owner decision of 2026-10-29, lines 1-1
