---
id: 20261022120015
aliases: []
tags: ["type/decision", "status/decided", "category/backend"]
---
The server is fully stateless so several instances can run, and ships as a single container binary. Hardening is local-only for now (bind 127.0.0.1) with Cloudflare later. The browser frontend comes later, not now.

## Links

- [[Backend stack and protocol]]

Source: project history of cubacle, lines 1-1
