---
id: 20261025160001
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
Every pattern has a catalog id `xx:code` of at most 12 characters: a two-letter source and a code of up to 9 characters from letters, digits, `.` and `-`. The built-ins use `cu:` with a readable code (`cu:checker`); imports keep the author's code (`ct:A200.01`) under the tag set by `--source`. `show` accepts the name, an alias, the short id or the catalog id, case-insensitive. Twelve characters keep the id column of the list and the tree narrow.

## Links

- [[Pattern files are listed in cubacle.toml and the default is the built-in library]]
- [[Imported patterns carry their patternness in a versioned TSV cache]]

Source: docs/patterns.md, lines 259-259
