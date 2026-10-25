---
id: 20261025170001
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
A pattern kind says which sort of regularity a state has, next to patternness (still the one score): SOLVED, FEW, MOSTLY, SYM, FACE, ORBIT, IRREG, first match wins. The measures are the defect of the best whole-cube turn (fixed share, symmetry of the defect, moved pieces), the arrangement and simplicity of the faces, and the two parts of patternness. Random states are an easy negative: they fail every criterion except on the 2x2x2. The thresholds per size are therefore set on short scrambles of 4 to 12 turns (`tools/bench/kind_rates.rs`), where SYM needs at least 4 symmetries (8 on the 2x2x2), FACE is stricter on the 2x2x2 and 3x3x3, and ORBIT needs a perfect score and never decides on the 2x2x2 and 3x3x3. ORBIT stays the weakest kind: it fires on 4 to 17 % of short scrambles of the big cubes and on no library pattern. The kinds describe structure and are not claimed to reproduce any collection's categories.

## Links

- [[Patternness is symmetry and uniformity, calibrated on random cubes and the library]]
- [[The detector explains patterns and never replaces the search]]

Source: owner decision of 2026-10-25, lines 1-1
