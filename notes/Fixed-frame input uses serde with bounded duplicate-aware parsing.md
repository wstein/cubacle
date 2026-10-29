---
id: 20261029213001
aliases: []
tags: ["type/design", "status/decided", "category/architecture"]
---
The fixed-frame proof design's owner-approved D2 adds `serde` and `serde_json` with only `std` to the root crate and moves `sha2` from development to runtime dependencies. All three already exist in the lockfile and are MIT or Apache-2.0. The input reader belongs to the application layer; puzzle types and solver engines do not parse JSON. It bounds physical request lines to 16 MiB, nesting to 64 containers and numeric tokens to 128 bytes, rejects duplicate keys at every depth, and preserves unsigned 64-bit seeds. The later typed request decoder must reject unknown fields in closed schema objects while retaining the sticker extension boundary.

This first foundation does not expose a new CLI command or solver. Its private module has an explicit dead-code expectation until command and server wiring follows in the next stages. The existing solver routes and recorded output remain unchanged.

Source: owner-approved fixed-frame design D2 in Cubacle PR #6, lines 1-1
