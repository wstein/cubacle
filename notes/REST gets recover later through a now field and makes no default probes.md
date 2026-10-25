---
id: 20261025160004
aliases: []
tags: ["type/decision", "status/decided", "category/backend"]
---
Owner decision of 2026-10-25: `/v1/solve` gets recover later as a `now` field of the request, not as a second endpoint. The server makes no default 3000 probes and uses only the `minProbes` of the request, so a plain request keeps the first answer while the command line improves by default. This settles the server half of the probe note.

## Links

- [[The plain 3x3x3 solve makes 3000 probes and -m keeps them]]
- [[The REST responses always carry the move counts]]

Source: docs/api.md, lines 37-119
