---
id: 20261026213000
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
`solve --icons --optimize ranges` draws the compressed solution, and a range such as `2-3Rw` is one picture with one arrow, not one per layer. The icon code has an action for a block of layers: it lights exactly the layers the block turns, puts the arrow in the middle of the block like that of a wide move, and labels the picture with the WCA range (a plain bar is the hyphen glyph, the font data has none). The whole solution is compressed, as the text answer is, so the pictures are as many as its compressed words; a range that spans a solver phase border is drawn in the phase where it starts, and the phase headings and the move numbers count the drawn steps. Phase borders only group, fewer moves win. `moves --icons --optimize ranges` draws a typed expression the same way. `--group` stays refused with `--icons` and now says why. The default pictures are unchanged: without the option the steps are the plain moves, byte for byte.

## Links

- [[Move metrics count turns of contiguous blocks]]
- [[Icons carry the move number that recover takes]]

Source: src/steps/run.rs, lines 1-90
