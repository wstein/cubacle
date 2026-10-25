---
id: 20261025230000
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
The lines of an imported pattern table claim block, face and quarter turns, and our counting differs from the table's on prefixed words, so a sixth to a fifth of the claims looked wrong (2100 to 2750 per size). The rule was found by testing hypotheses on the whole collection, not by reading single lines: a prefixed word (`M`, `N`, `S`, `V`, `W`) is one block turn whatever it expands to, a whole-cube rotation (`C`) counts none, a plain or tier (`T`) turn is one face turn and an inner block two (as in ours), and quarter turns are the face turns, doubled for a half turn. With it 99.9 % of the claims of the six folders agree (38 of about 42000 differ, which look like errors in the file). Our counting is not changed, only the comparison in `patterns import --dry-run`, which reads the move text for it (`sheet_counts`). The claimed order is still compared with ours and differs on 2 % of the 3x3x3 lines and 11 to 15 % of the bigger ones; no rule for it was derived. Tests use invented sequences.

## Links

- [[A pattern table is read by structure and every line is replayed]]

Source: owner decision of 2026-10-25, lines 1-1
