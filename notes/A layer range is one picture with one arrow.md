---
id: 20261026213000
aliases: []
tags: ["type/decision", "status/decided", "category/notation"]
---
`solve --icons --optimize ranges` draws the compressed solution, and a range such as `2-3Rw` is one picture with one arrow, not one per layer. The icon code has an action for a block of layers: it lights exactly the layers the block turns, puts the arrow in the middle of the block like that of a wide move, and labels the picture with the WCA range (a plain bar is the hyphen glyph, the font data has none). The ranges are formed inside each solver phase, so the phase headings and the move numbers count the drawn steps and no step crosses a phase border; a compression of the whole solution can find more, and the text answer without dots still compresses the whole. `--group` stays refused with `--icons` and now says why. The default pictures are unchanged: without the option the steps are the plain moves, byte for byte.

## Links

- [[Move metrics count turns of contiguous blocks]]
- [[Icons carry the move number that recover takes]]

Source: src/steps/run.rs, lines 1-90
