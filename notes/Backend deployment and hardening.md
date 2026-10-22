---
id: 20261022120015
aliases: []
tags: ["type/decision", "status/decided", "category/backend"]
---
The server is fully stateless so several instances can run, and ships as a single container binary. Hardening is local-only for now (bind 127.0.0.1) with Cloudflare later. The browser frontend comes later, not now.

## Links

- [[Backend stack and protocol]]

Source: crates/server/src/lib.rs, lines 6-11
Source: docs/api.md, lines 27-27
Source: docs/api.md, lines 130-130
Source: owner decision of 2026-10-21, lines 1-1
