---
id: 20261022120016
aliases: []
tags: ["type/evidence", "status/done", "category/backend"]
---
The server review led to: a header-read timeout and bounded graceful shutdown, validated limits, a required JSON content type (415 otherwise), JSON error bodies, and 501 for sizes without an engine.

## Links

- [[Backend stack and protocol]]

Source: docs/api.md, lines 92-111
Source: docs/api.md, lines 122-122
Source: crates/server/src/lib.rs, lines 505-512
