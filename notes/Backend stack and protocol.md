---
id: 20261022120013
aliases: []
tags: ["type/decision", "status/decided", "category/backend"]
---
The backend uses axum and tokio. REST/JSON serves stateless calls and Server-Sent Events deliver live improving solutions. Planned: a client disconnect cancels the search, which needs engine cancellation (also planned).

## Links

- [[Backend concurrency]]
- [[Backend deployment and hardening]]
- [[Server review fixes]]

Source: crates/server/Cargo.toml, lines 20-21
Source: docs/api.md, lines 9-9
Source: owner decision of 2026-10-21, lines 1-1
