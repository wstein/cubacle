---
id: 20261026031500
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
The owner decided that the identifier of a pattern is its canonical Orbit64 token, stored and printed together with the schema name `orbit64-Nxn-draft@1`, because the draft slot numbering may change and rename every token. The token is not a search key and never a file name. `cubacle canon` prints the bare token by default, so a script can use it, and the schema and size with `-v` or `--format json`. The REST server adds the same pair as the field `canonical` (`token`, `schema`) to `/v1/convert` and `/v1/random`, because both already describe a state and `convert` takes every input form; no endpoint was added, and the field is absent when the stickers are no cube. `canon` reads its input without the center check of `solve`, since a recolored cube is a valid input there.

## Links

- [[The canonical token is the Orbit64 token of the standard-colored member of the detector's class]]
- [[Orbit64 slot numbering caveat]]
- [[Input notations]]
- [[The REST responses always carry the move counts]]

Source: src/canon_cmd.rs, lines 29-52
Source: crates/server/src/canonical.rs, lines 1-33
