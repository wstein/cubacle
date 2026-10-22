---
title: "Move icons"
description: "Draw moves as small pictures for the terminal, the web and print."
weight: 4
---

# Move icons

`cubacle icon` draws a move on a cube as a small picture, and `cubacle solve --icons` shows every step of a solution that way. The same code is available as the `cubacle::render` module, which returns SVG, RGBA pixels or Kitty graphics escapes for a frontend or a PDF export.

## What an icon shows

An icon shows the Front, Up and Right faces of the cube in an oblique view, with every sticker of the N×N×N grid (2 to 7) in the real colours of the cube state before the move.

- **Moving layers** keep their colours at full brightness and everything else is darkened to 45 percent. With `--grey`, the moving layers turn grey instead.
- **A red arrow with a white halo** lies on the face where the moving layer is visible, along the whole layer. A "2" badge on the arrow marks a half turn.
- **A curved cyan arrow** over the whole cube marks the rotations `x`, `y` and `z`.
- **The notation** is written beside the cube.

| Move | Arrow lies on | Direction |
| --- | --- | --- |
| `R`, `Rw`, `2R` | the Front face, over the turning column | up |
| `L` | the Front face, left column | down |
| `U` | the Front face, top row | left |
| `D` | the Front face, bottom row | right |
| `F` | the Right face, front column | down |
| `B` | the Right face, back column | up |

A prime (`'`) reverses the direction. The direction is derived from the rotation itself and is tested against `Facelets::apply`; it is not drawn by eye.

### Notation in captions

The caption is the canonical notation of the move. `Rw` and `Rw2` turn the two outer layers, `2R` and `2R2` turn only the second layer, and `3Rw` turns the three outer layers. Lowercase `r` is not accepted. The arrow is centred on the turning layers, so a wide move has its arrow between its layers.

## Draw an icon

```sh
cubacle icon R "U'" Rw2 --cube 4
```

Each move is drawn on the cube state left by the moves before it. On a terminal without graphics, the output is text; on a pipe it is plain text without escape sequences:

```text
R ↑
   w w W
  w w W
 w w W
g g G R R R
g g G R R R
g g G R R R
```

Moving stickers are uppercase and the others lowercase.

| Option | Meaning |
| --- | --- |
| `--format auto\|svg\|rgba\|kitty\|text` | The output format. `auto` (the default) uses Kitty graphics on a capable terminal and text otherwise. |
| `--size N` | Icon height in pixels, 96 by default. The width is 1.5 times the height. |
| `--cube N` | Cube size, 2 to 7, 3 by default. |
| `--out FILE` | Write to a file instead of standard output. |
| `--scheme boy\|jp` | Colour scheme, `boy` by default; six letters in `URFDLB` order also work. |
| `--from SCRAMBLE` | Start from the cube that this scramble produces. |
| `--grey` | Grey the moving layers instead of darkening the rest. |

`svg` and `rgba` hold one icon, so they take one move. `rgba` writes `width × height × 4` bytes of straight (not premultiplied) RGBA. Graphics escapes are never written to a standard output that is not a terminal, and raw pixels are never written to a terminal; use `--out FILE` in those cases.

```sh
cubacle icon "Rw'" --cube 5 --format svg --out rw.svg
```

## Solve with icons

```sh
cubacle solve --scramble "R U" --icons
```

Each step prints `step i of n: MOVE` and then the icon of the cube before that step. Without `--icons`, the output of `solve` does not change. The colours come from `--scheme` (`boy` by default).

## Terminal support

`cubacle` writes Kitty graphics when standard output is a terminal and the environment names a terminal that supports the [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/): `KITTY_WINDOW_ID` is set, `TERM` is `xterm-kitty`, or `TERM_PROGRAM` is `ghostty` or `WezTerm`. Other terminals get coloured text, or plain text when `NO_COLOR` is set, `TERM` is `dumb` or standard output is not a terminal.

The Kitty output transmits raw RGBA (`a=T,f=32`) in chunks of at most 4096 base64 bytes, with `m=1` on every chunk but the last, and asks the terminal to scale the image to a fixed number of cells (`c` and `r`) so that the layout is predictable. This is covered by unit tests of the encoding; it has not been tried in a live Kitty, Ghostty or WezTerm window.

## Use the module

```rust
use cubacle::puzzle::{Facelets, PuzzleKind};
use cubacle::render::{icon, Action, Style};

let state = Facelets::solved(PuzzleKind::Cube5);
let action: Action = "2R".parse().expect("a move");
let drawn = icon(&state, &action, &Style::default()).expect("2R exists on a 5x5");
let svg = drawn.to_svg();
let pixels = drawn.to_rgba(128);
let escapes = pixels.to_kitty(None);
assert!(svg.starts_with("<svg") && escapes.starts_with("\x1b_G"));
```

`render` depends only on `puzzle`, does no I/O and keeps no state. The SVG is built from filled polygons with two-decimal numbers, so the same input always gives the same text. The `Style` sets the palette, the shading and a target size in pixels; the size only widens gaps and arrows so that icons at 64 to 128 pixels stay crisp, and the SVG scales freely.

Examples of the SVG output are in `tests/fixtures/icons/`.

## Limits

- PDF export, animation and a web frontend are not part of this module yet.
- Inner slices use the straight arrow; there is no curved arrow for them.
- Geometry and arrow directions are tested for 2×2×2 to 5×5×5 (geometry) and 3×3×3 to 5×5×5 (arrows against `Facelets::apply`). The 6×6×6 and 7×7×7 use the same code and are only tested for shape counts and a stable SVG.
