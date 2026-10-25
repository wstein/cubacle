---
id: 20261025190000
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
`cubacle patterns update` (alias `download`) fetches the `http(s)` sources of the active config into `<cache>/downloads/`, with ureq (rustls, ring, built-in roots) as an optional dependency behind the default-on `fetch` feature; the server depends on cubacle without default features and so never links an HTTP client. curl is not an option because Windows has none to rely on. The user's config is the only source of URLs, and uncommenting an entry is the consent; `auto_fetch` (default false, set only by the user) downloads missing entries at the start of a patterns command and only warns on failure. A download goes to `NAME.part`, must parse to at least one pattern, then replaces the old file; ETag and Last-Modified sit in `NAME.meta` for a conditional request. Limits: https (plain http only to loopback, for the tests), 50 MB, 15 s connect, 120 s total, five redirects to https at most. `--offline` or `CUBACLE_OFFLINE=1` forbids the network entirely. Tests run a hand-written HTTP server on 127.0.0.1 and never the internet.

## Links

- [[Pattern files are listed in cubacle.toml and the default is the built-in library]]

Source: owner decision of 2026-10-25, lines 1-1
