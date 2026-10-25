---
id: 20261025170001
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
A pattern kind says which sort of regularity a state has, next to patternness (still the one score): SOLVED, FEW, MOSTLY, SYM, FACE, IRREG, first match wins. The measures are the defect of the best whole-cube turn (fixed share, symmetry of the defect, moved pieces), the arrangement and simplicity of the faces, and the two parts of patternness. Random states are an easy negative: they fail every criterion except on the 2x2x2. The thresholds per size are therefore set on short scrambles of 4 to 12 turns (`tools/bench/kind_rates.rs`), where SYM needs at least 4 symmetries (8 on the 2x2x2), FACE is stricter on the 2x2x2 and 3x3x3. A seventh kind, ORBIT (every piece orbit moved regularly, 4x4x4 and up), was removed on 2026-10-25: measured on 1110 to 1260 imported patterns per size it decided 0.3 to 1.6 % of them and was the only criterion that reached 90 for at most 0.4 %, while it fired on 6 to 16 % of the scrambles of 4 to 12 turns, and its perfect-score threshold could not be made stricter. The kinds describe structure and are not claimed to reproduce any collection's categories.

## Links

- [[Patternness is symmetry and uniformity, calibrated on random cubes and the library]]
- [[The detector explains patterns and never replaces the search]]

Source: owner decision of 2026-10-25, lines 1-1
