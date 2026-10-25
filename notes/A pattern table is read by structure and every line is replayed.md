---
id: 20261024140101
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
The importer for tab-separated pattern tables (the layout of CubeTwister's `PatternExportTab.txt`) was written from the shape of one small sample, never from a copy of the data. A line has 17 tab-separated fields: code, variant number, group and subgroup (German, English), name (German, English), the moves, counts, order, cycle notation, author and notes. Lines with one code are variants of one pattern. Moves are face letters with `2` and a trailing `-` or `'` for the inverse; the text is UTF-8 or, when that fails, Mac Roman (the exports come from a Mac, and Latin-1 turned the group separator `·`, byte 0xE1, into a letter that made most lines of the bigger folders unreadable). Every sequence is replayed on a solved cube of the folder's size (`2xPocket` is size 2); a line that cannot be read, does not apply or builds another state than its sibling variants is reported with its line number and skipped, never fatal. Verified only for the 2x2 folder: how the bigger folders write wide or slice moves is not known, so such lines surface as reported problems. Tests use invented lines. The data stays outside the repository and is kept in the user cache directory in our own `id | size | name | aliases | description | moves` file, where a pattern with no structural definition gets its state by replaying its generator and is identified by its canonical form.

## Links

- [[A pattern detector reads structure from the cubie state]]
- [[Pattern files are listed in cubacle.toml and the default is the built-in library]]

Source: owner request of 2026-10-24, lines 1-1
