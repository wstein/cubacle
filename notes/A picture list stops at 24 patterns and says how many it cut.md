---
id: 20261026021500
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
`patterns --icons` draws one picture per listed pattern, and a library with imported tables lists hundreds, so the pictures stop after 24 unless `--limit N` says otherwise; the plain list without `--icons` keeps showing every row. Standard error always says what was cut (`showing the first 24 of 110 patterns; --limit 110 shows all of them`), because a silent cut would look like a library with fewer patterns. The picture options are the ones of `solve --icons`, made global in their struct so that `patterns list --icons` takes them after the subcommand; the list options (`--tree`, `--sort`, `--kind`, `--min-patternness`, `--limit`) are global for the same reason, and the other subcommands refuse them instead of ignoring them. `--tree` stays text and says so. `identify` prints `canonical: TOKEN` below its `pattern:` lines, the place where the class is named, and `show` reads an Orbit64 token by looking up the library pattern it is.

## Links

- [[Pattern output puts icons first and the tree is the overview]]
- [[The canonical token is the Orbit64 token of the standard-colored member of the detector's class]]

Source: docs/icons.md, lines 316-360
