---
id: 20261024180002
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
Home needs a colour frame and a flip needs an axis priority, and neither is canonical, so the fingerprint reads the pieces in every frame that fits (the centres on an odd cube, every best matching of colours to faces on an even one) and every one of six axis orders, and keeps the least result. A turned, mirrored or recoloured cube then gives the same code, which the tests check on all 48 symmetries. Corners and the middle wings of odd cubes can be told apart and give a cycle type; other wings and centres only give counts of displaced and flipped pieces, because equal pieces have no identity in the colours.

## Links

- [[A pattern detector reads structure from the cubie state]]
- [[The detector explains patterns and never replaces the search]]
- [[The canonical token is the Orbit64 token of the standard-colored member of the detector's class]]

Source: src/patterns/detector/fingerprint.rs, lines 1-60
