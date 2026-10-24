---
title: "Move icons"
description: "Draw moves as small pictures for the terminal, the web and print."
weight: 4
---

# Move icons

A move icon (a "minicube": a small cube picture with an arrow, also called an arrow diagram) shows one move on the cube. `cubacle moves --icons` draws moves as such pictures, and `cubacle solve --icons` shows every step of a solution that way. The same code is available as the `cubacle::render` module, which returns SVG, RGBA pixels or Kitty graphics escapes for a frontend or a PDF export.

## What an icon shows

An icon shows the Front, Up and Right faces of the cube in a deep oblique view, with every flat, bright sticker of the N×N×N grid (2 to 7) in the real colours of the cube state before the move.

- **Moving layers** keep their colours at full brightness and everything else is darkened: to 25 percent on a dark background and to 40 percent on a light one (see `--background`). With `--grey`, the moving layers turn grey instead.
- **A soft violet arrow with a white outline** lies on the face where the moving layer is visible, along the layer with a margin of 12 percent of the face width at both ends. It has a round tail and a head about twice as long as wide with rounded corners. Its size stays the same on the canvas for a given cube size: 4.0 canvas units wide on the 2x2x2, narrowing to 2.8 on the 7x7x7. On the Right face it is foreshortened like the face.
- **An upright violet "2x"** with a white outline, the same height as the notation, marks a half turn. It sits right of a vertical arrow and above a horizontal one.
- **A curved violet arrow** over the whole cube marks the rotations `x`, `y` and `z`.
- **The notation** is written below the cube in Quicksand Bold, always in the same size and on the same baseline. It is light on a dark background and dark on a light one.

| Move | Arrow lies on | Direction |
| --- | --- | --- |
| `R`, `Rw`, `2R` | the Front face, over the turning column | up |
| `L` | the Front face, left column | down |
| `U` | the Front face, top row | left |
| `D` | the Front face, bottom row | right |
| `F` | the Right face, front column | down |
| `B` | the Right face, back column | up |

A prime (`'`) reverses the direction. The direction is derived from the rotation itself and is tested against `Facelets::apply`; it is not drawn by eye.

### Notation

The notation below the cube is the canonical notation of the move. `Rw` and `Rw2` turn the two outer layers, `2R` and `2R2` turn only the second layer, and `3Rw` turns the three outer layers. Lowercase SiGN spellings such as `r` are read as input (see [Move notations](cli.md#move-notations)); the label is always the WCA one. The arrow is centred on the turning layers, so a wide move has its arrow between its layers.

## Draw moves

```sh
cubacle moves R "U'" Rw2 --size 4 --icons
```

The moves are read like those of `convert` and `solve` (groups, commutators, slices, rotations, layer ranges), and `--size` is the cube size. With `--icons` and no `--format` or `--out`, the output is the step pictures of `solve --icons`: each move is drawn on the cube left by the moves before it (starting from the solved cube or `--from`), under a line `step i of n: MOVE`. On a terminal without graphics, the output is text; on a pipe it is plain text without escape sequences:

```text
step 1 of 2: R
R ↑
   w w W
  w w W
 w w W
g g G R R R
g g G R R R
g g G R R R
```

Moving stickers are uppercase and the others lowercase.

An explicit `--format` or `--out` writes the icons themselves, one per move or rotation, without step lines; each is drawn on its own start cube, as in a move sheet, unless `--cumulative`. All options below need `--icons`.

| Option | Meaning |
| --- | --- |
| `--format auto\|svg\|rgba\|kitty\|text` | The output format. `auto` (the default) shows step pictures (Kitty graphics on a capable terminal, text otherwise); with `--out` it writes SVG. |
| `--icon-size PIXELS` | Icon height in pixels. Default: 32 per cube layer, at least 96 (96 for the 2x2x2 and 3x3x3, 128 for the 4x4x4, 224 for the 7x7x7), so stickers stay readable on larger cubes. The canvas is 114 by 109.2 units, so the width is about 1.04 times the height (100 by 96 pixels by default). |
| `--size N` | Cube size, 2 to 7, 3 by default. |
| `--out FILE` | Write to a file instead of standard output. |
| `--scheme boy\|jp` | Colour scheme, `boy` by default; six letters in `URFDLB` order also work. |
| `--from SCRAMBLE` | Start from the cube that this scramble produces. |
| `--background dark\|light` | What the icon is seen on, `dark` by default. It sets the dimming (25 or 40 percent) and the colour of the notation (light or dark). |
| `--opaque` | Paint a solid background of the `--background` colour (`#20242b` for dark, white for light) behind SVG, RGBA and Kitty output. The default is transparent, so the picture takes the colours of the terminal or page. |
| `--cumulative` | Draw each move on the cube left by the moves before it. |
| `--grey` | Grey the moving layers instead of darkening the rest. |

`svg` and `rgba` hold one icon, so they take one move. `rgba` writes `width × height × 4` bytes of straight (not premultiplied) RGBA. Graphics escapes are never written to a standard output that is not a terminal, and raw pixels are never written to a terminal; use `--out FILE` in those cases.

```sh
cubacle moves "Rw'" --size 5 --icons --format svg --out rw.svg
```

### Hand units

A `·` between moves ends a hand unit, and `( ... )` makes a block of units (see [Hand units](cli.md#hand-units)). With `·` or `(` in the input, `moves --icons` draws each unit as icons placed close together, units separated by a larger gap, a thin bracket with the unit number above each unit, and a thicker outer bracket above each parenthesised block (two levels; deeper parentheses are flattened). The label under every icon stays the move. Rows are as wide as the terminal (or `--columns` icons) and break only between units; a unit wider than a row is cut into rows as the last resort, and its heading says `(continued)`. The headings read `group 3 of 9` or `groups 3-6 of 9` instead of `step 5 of 22`. Input without `·` or `(` is drawn as before, and `--group none` draws any input as before.

On a terminal without graphics the first line is the moves with ` · ` between the units (a block in parentheses), then a heading and the pictures of each unit. This is the real output of `cubacle moves --size 2 --icons --graphics plain "R U · F'"`:

```text
R U · F'
group 1 of 2: R U
R ↑
  w W
 w W
g G R R
g G R R

U ←
  W G
 W G
G Y R R
g y r r

group 2 of 2: F'
F' ↑
  w w
 G G
R R W b
G Y R r
```

## Solve with icons

```sh
cubacle solve --scramble "R U" --icons
```

On a Kitty-graphics terminal the solution is shown in phases (see below) and the icons come in rows, as many side by side as fit the width of the terminal, and each row has a line above it such as `steps 9-16 of 128`. A row is one image, with 8 pixels between the icons, and the background stays transparent. The last line is the plain move string. Without graphics (a pipe, `NO_COLOR`, `TERM=dumb` or another terminal), each step prints `step i of n: MOVE` and then its text picture, as before. A quarter turn with a half turn right before and right after it, the easiest move to do as a half turn, gets the note `<-- quarter turn, not a half turn` after its move, as in `step 9 of 21: F   <-- quarter turn, not a half turn`. An optional compile-time feature, off by default, also draws an amber frame around the icon of such a move in the Kitty rows (see below). Without `--icons`, the output of `solve` does not change. The colours come from `--scheme` (`boy` by default).

| Option | Meaning |
| --- | --- |
| `--icon-size PIXELS` | Icon height; all icons have the same size. Default: 32 per cube layer, at least 96. |
| `--columns N` | Icons per row. Default: as many as fit. Wide rows are cut off by the terminal, so a large `N` is your own choice. |
| `--background dark\|light` | What the icons are seen on. Default: asked from the terminal, dark when it does not answer. |
| `--views both\|urf\|net\|all` | Pictures of a whole cube: the two corners (`both`, default), URF only (`urf`), the flat net (`net`) or the net and the two corners (`all`). See [Two views of a state](#two-views-of-a-state) and [The flat net](#the-flat-net). |

### Phases

The solution is cut into phases, and each phase has a heading line, its rows of icons and then a picture of the cube after it. A row never spans two phases.

```text
Phase 1: centres (25 moves)
steps 1-10 of 44
(a row of 10 icons)
steps 11-20 of 44
(a row of 10 icons)
steps 21-25 of 44
(a row of 5 icons)
State after phase 1 - compare it with your cube
(a framed picture of the cube)
State after phase 1 as token: (an Orbit64 token)
Phase 2: edges (nothing to do)
Phase 3: 3x3 stage (19 moves)
...
```

The state picture has no arrow, no notation and no dimming: every sticker is in full colour inside a thin frame, so that it cannot be mistaken for a move. Compare it with your own cube before you go on.

For the 4x4x4 to 7x7x7 the phases are the stages of the engine that solved the cube: the engine notes where each stage ends and the program carries those ends through the rewriting of the answer, so a phase ends where the engine's stage ended (matched by the state of the cube; when the rewriting leaves no such state, at the closest one). For the 2x2x2 and 3x3x3 the phases are read from the cube: the solution is replayed and a phase ends at the first move after which its condition holds for the rest of the solution. A phase with no moves keeps its number and its heading reads `nothing to do`, with no icons and no picture. A phase that no answer of an engine ever needs is not in its list: the big cubes have no `edges` phase, because their engines pair the wings together with the centres.

| Cube | Phases |
| --- | --- |
| 2x2x2 | `solve` |
| 3x3x3 | `to the G1 subgroup` (corners and edges oriented, the four middle-layer edges in the middle layer: the Kociemba domino state), `solve` |
| 4x4x4 | `centres staged` (each pair of opposite colours has its centres on one axis), `centres sorted and wings split` (the centres sorted far enough for the last phase and every wing pair in the half-slot split it needs), `centres and wings paired` (the cube is now a 3x3x3), `3x3 stage` |
| 5x5x5 | `centres staged` (the same, in two searches), `wings staged` (the wings in slots where they can be paired), `centres and wings paired` (a 3x3x3), `3x3 stage` |
| 6x6x6, 7x7x7 | `left/right centres staged`, `up/down centres staged`, `centres prepared`, `inner wings and centres`, `outer wings and centres` (a 3x3x3), `3x3 stage` |

The 6x6x6 and 7x7x7 engine follows dwalton76/rubiks-cube-NxNxN-solver, whose many small steps are grouped into the six phases like this:

| Phase | 6x6x6 steps | 7x7x7 steps |
| --- | --- | --- |
| `left/right centres staged` | inner x-centres staged and obliques paired; centres staged | inner centres staged; obliques paired; centres staged |
| `up/down centres staged` | the same for up and down | the same for up and down |
| `centres prepared` | the two last centre steps ("reducible") | the three last centre steps (bars, daisy) |
| `inner wings and centres` | the 4x4x4 engine on the inner wings and centres | the 5x5x5 engine on the inner wings and centres |
| `outer wings and centres` | the 5x5x5 engine on the outer wings and centres | the 5x5x5 engine on the outer wings and centres |
| `3x3 stage` | the 3x3x3 solver | the 3x3x3 solver |

Measured on three random 7x7x7 cubes (183 to 190 moves) the phases take 30 to 33, 28 to 31, 30 to 33, 35 to 40, 32 to 42 and 21 moves, so none dominates. At the end of `inner wings and centres` the centres inside the outermost ring are one colour per face; at the end of `outer wings and centres` all centres are solved and all wings are paired. With `--verbose`, the phases are also written to standard error, for every cube and with or without `--icons`:

```text
phase 1: centres staged, moves 1-19
phase 2: wings staged, moves 20-25
phase 3: centres and wings paired, moves 26-51
phase 4: 3x3 stage, moves 52-70
```

Without graphics there are no phase headings: the text output is one picture per step, as before, and the token line of a phase follows the last step of that phase.

### Quarter-turn frame

The amber frame around a lone quarter turn in the Kitty rows is an optional compile-time feature, off by default, because it may confuse more than it helps. The text note stays in every build. To get the frame, build with the feature:

```sh
cargo build --release --features quarter-frame
```

There is no command-line option or environment variable for it.

### Two views of a state

A move icon shows the cube from the Up-Right-Front corner (URF), so the Left, Back and Down faces are hidden. The picture of a whole state (`State after phase N - compare it with your cube` in `solve --icons`, `Should be` and `Is now` in `recover --icons`) is therefore drawn twice, side by side in one image: the URF view on the left and the view from the opposite Down-Left-Back corner (DBL) on the right. Together they show every sticker of the six faces exactly once. Move icons stay URF only.

The four side faces form one belt from left to right, every face upright (its U edge on top): F and R in the left picture, then B and L in the right one. The right picture is the cube seen from below: B is the large face in front, L the narrow side on its right, and D lies under B. Physically: hold the cube with U up, turn it around its vertical axis (`y2`) so that B faces you and L is on your right, then bend down and look up at it from below. A turn of the cube about the vertical axis keeps U up, so B and L stay upright; seen from underneath, D appears below them. (Turning the cube over, for example with `x2`, would put the side faces upside down, which is why the second view is not drawn that way.) Tests check this against the facelet layout: every sticker of D, L and B lands on a visible face of the `y2`-turned cube, the two views cover each sticker once, the side faces are drawn in the order F, R, B, L with their U edge on top, and every amber mark lies on its sticker.

On a Kitty terminal the two pictures are one image. When that image is wider than the terminal, or `--columns 1` is given, the two views are stacked, one image each. With `--views urf` only the URF picture is drawn (`net` and `all` add the flat net, below). Without graphics `solve --icons` prints no state picture (only the token line), and `recover --icons` prints each state as two text pictures, one below the other, each with a heading that names its corner (`state, seen from URF: ...`, `state, seen from DBL, turned so that B is in front: ...`; the second shows B and L side by side with D below them); `--views urf` leaves out the second.

### The flat net

`--views net` draws a whole cube as the classic cross, with no projection: all six faces are equally large squares, each seen from outside with U up, and every sticker appears exactly once.

```text
      U
   L  F  R  B
      D
```

U lies above F and D below F; L, F, R and B form one row. U is drawn with F at its bottom edge and D with F at its top edge, so every edge shared by two faces that touch in the picture continues across the gap. Rings, stripes and point symmetry of a pattern read directly from it, which the corner views cannot show, and it works for every size from 2x2x2 to 7x7x7. A 5x5x5 at the default height of 160 pixels is an image of 211 by 160 pixels (the height is `--icon-size`, as for the other pictures).

`--views all` draws the net first, then the two corners, in one image when they fit the terminal width and `--columns` allows it, otherwise stacked one image each (the same rule as for `both`). For a 3x3x3 at the default height the three pictures are one image of 343 by 96 pixels in a terminal 1600 pixels wide, and three images of 127, 100 and 100 pixels width in one of 240 pixels. Amber marks of `recover --icons` sit on the stickers that differ, as in the corner views; as the net shows every face, each differing sticker is marked once, and the line `Stickers that differ` says so. Tests check each of the 6 n x n stickers is drawn once, with its colour, at its place, that stickers of one cubie meet across the gap on touching faces, and that every differing sticker has exactly one mark.

Without graphics the net is a letter grid, with a heading and one space between faces more than between stickers. This is the text of `recover --icons --views net` for the example of [Recover with icons](#recover-with-icons), as run:

```text
state, flat view: U above F, L F R B in a row, D below
       B Y O
       R W R
       Y R B
R Y G  O B O  W W G  W B W
G O O  W G B  O R W  G B O
B Y W  R B O  Y R R  Y O R
       G W B
       G Y G
       Y Y G
```

### Checkpoints

`solve --icons` writes the cube as an [Orbit64 token](cli.md) (the text `convert --to token` writes) at these places: the line `Start state: TOKEN` once at the top, and the line `State after phase N as token: TOKEN` after each phase that has moves, below the state picture on a graphics terminal and after its last step otherwise. A phase with nothing to do has no line. The last token is the solved cube. Keep the start token; after a phase, the token of your own cube can be compared with the printed one. Plain `solve` without `--icons` prints no token.

### How the terminal is asked

Before drawing, `solve --icons` asks the terminal for two things, only when standard input and output are a terminal with Kitty graphics:

- **Width.** `TIOCGWINSZ` gives the columns and, in most terminals, the size in pixels. When the pixel size is 0, the reply to `CSI 14 t` (text area in pixels) is used, then `CSI 16 t` (cell size) times the columns, and last `columns * 8` pixels (an assumed cell width; `COLUMNS` or 80 for the columns). The same sources give the cell height that moves the cursor below a row, with 16 pixels assumed.
- **Background.** The reply to `OSC 11` (`ESC ] 11 ; ? ESC \`, answered as `rgb:RRRR/GGGG/BBBB`) is judged by its brightness (0.2126 R + 0.7152 G + 0.0722 B above 0.5 is light). Without `--background` and without a reply the background is dark. With `--background`, the colour is not asked.

The queries go through `/dev/tty` in a short raw mode, and the saved settings are restored on every path, also on an error. A DA1 query (`CSI c`) is sent last: a terminal that does not know the other queries still answers it, so the program does not wait. If nothing answers, the wait is 100 ms.

Inside `tmux` and `screen` (`TMUX` or `STY` is set, or `TERM` starts with `screen` or `tmux`) nothing is asked, because they would swallow the queries and the images need a passthrough that is not implemented. The defaults apply there: the width from `TIOCGWINSZ` or `columns * 8`, and a dark background.

## Recover with icons

`recover` takes the same options as `solve`, with the same names and meaning (`--icons`, `--icon-size`, `--background`, `--columns`, `--views`; all but the first need `--icons`):

```sh
cubacle recover -s "R U2 F' D L2 B R' U' F2 D2 L B'" --now FDDRURLDBRDBLRFFDUUBUUFBLBLBRDFDFBUFRUFULLDRURLDLBFRBL --at-move 7 --sure-until 3 --icons
```

The usual text of `recover` comes first. After it:

1. When a mistake or a nearest cube was found, two pictures of the whole cube: `Should be, after step N (the last correct one)`, the cube of the printed solution after the last correct move, and `Is now`, the cube you typed. Each has its Orbit64 token below it. Compare the two to see what differs: on a Kitty terminal every sticker of `Is now` whose colour differs from `Should be` carries a small amber square, in the view (URF or DBL) that shows its face, so all of them are marked. In every mode the line `Stickers that differ: N of M` counts them over all faces. With `--views urf` the hidden faces are not drawn, so only the stickers of U, R and F are marked and the line says so; `net` and `all` mark every one.
2. The moves to do now (the way back, or the fresh solution when that is recommended), one picture per step as in `solve --icons`, with the quarter-turn note (and the optional frame). Phase headings and `State after phase N as token` lines appear only when the moves are a fresh solution that reports solver stages; a way back has none. In `recover` (not in `solve`) a run of phases with nothing to do is one heading, `Phases 1-5: nothing to do`; a single empty phase keeps its own line.

Without graphics the pictures are the text pictures of `solve --icons`:

```text
Should be, after step 4 (the last correct one)
state, seen from URF: U up, F towards you, R on your right
   B Y O
  R W R
 Y R B
O B O W W G
W G B O R W
R B O Y R R

state, seen from DBL, turned so that B is in front: B, L, D below
W B W R Y G
G B O G O O
Y O R B Y W
 G Y Y
  G Y G
   B W G

token: Enl7Pqxeueao
Is now
state, seen from URF: U up, F towards you, R on your right
   G Y Y
  R W R
 O Y B
W B W R Y B
W G B O R G
O B O G Y W

state, seen from DBL, turned so that B is in front: B, L, D below
R O Y R W G
O B G W O O
R B O Y R W
 G W B
  G Y G
   Y R B

token: FpkdfDU1GuT4
Stickers that differ: 31 of 54 (all marked amber in graphics rows, in the view that shows them).
step 1 of 3: D2
D2 → (half turn)
...
```

`recover --fresh --icons` draws the fresh solution only. When the way back is for a cube that has to be turned as a whole first, there are no step pictures. On a Kitty terminal the two state pictures are stacked, each under its label.

## Terminal support

`cubacle` writes Kitty graphics when standard output is a terminal and the environment names a terminal that supports the [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/): `KITTY_WINDOW_ID` is set, `TERM` is `xterm-kitty`, or `TERM_PROGRAM` is `ghostty` or `WezTerm`. Other terminals get coloured text, or plain text when `NO_COLOR` is set, `TERM` is `dumb` or standard output is not a terminal.

The Kitty output transmits raw RGBA (`a=T,f=32`) in chunks of at most 4096 base64 bytes, with `m=1` on every chunk but the last. It first erases the terminal rows that the image covers, steps back up, draws the image with `C=1` (the cursor does not move) and then steps down below it, so older text cannot show through and later output does not overlap. The icon background is transparent by default, so it takes the colours of a terminal and does not drift against other backgrounds. `--opaque` paints a solid backdrop of the `--background` colour instead (`moves --icons` only). In `solve --icons` the image is drawn at its own pixel size, without `r`, so a row is as wide as computed; the number of rows to erase is its height divided by the cell height, rounded up. For `moves --icons` with `--format kitty`, only the height in rows (`r`) is sent, from an assumed 16 pixels per row. See [How the terminal is asked](#how-the-terminal-is-asked) for the width and the background. The encoding is covered by unit tests; the drawing was seen in a Kitty-protocol terminal by the owner.

## Forcing the graphics and terminals

`--graphics auto|kitty|ansi|plain` (on `moves`, `solve` and `recover`, with `--icons`) and the variable `CUBACLE_GRAPHICS` with the same values choose how the pictures are drawn. The flag wins over the variable, and the variable over `auto`. An invalid value is refused with the accepted ones listed (`[possible values: auto, kitty, ansi, plain]`).

- `auto` (the default) decides in this order. Standard output not a terminal, `TERM=dumb` or `NO_COLOR` give plain text. The variables listed under [Terminal support](#terminal-support) give Kitty graphics. Otherwise the terminal is asked once (below), and coloured text is drawn only when it does not answer.
- `kitty`, `ansi` and `plain` are used as given, also when standard output is not a terminal: `--graphics kitty` writes the escape sequences into a pipe or a file.

**The question.** Terminals that support the Kitty protocol without being named by a variable (for example `herdr`, which answers `kitty icat --detect-support` with `stream`) are found by asking. In one write the program sends `ESC _ G i=31,q=1 ESC \` (no payload and no action, so nothing is stored), the query of the protocol specification `ESC _ G i=31,s=1,v=1,a=q,t=d,f=24;AAAA ESC \` and, as the sentinel, `ESC [ c` (DA1). Any graphics reply that carries `i=31` counts, an error reply as well as `OK`, because an error shows that the terminal parsed the command. The replies seen: kitty answers `Gi=31;EINVAL:Zero width/height not allowed` and `Gi=31;OK`, `herdr` answers `Gi=31;EINVAL: dimensions required`. The sentinel's answer comes after the others, so the program reads until it arrives and nothing is left in the input of the shell; with no answer it stops after 200 ms. A DA1 answer without a graphics reply means no support. The question is asked only for `auto`, only when the variables left coloured text as the result, only when standard input and output are terminals and only once per process. The tty is in a short raw mode that is restored on every path.

I ran the program against a fake terminal on a pty (a script that answers what the program writes): with the kitty replies and DA1 the output was Kitty images (0.28 s in all, including the width query); with only DA1 coloured text, at once; with no answer at all coloured text after 0.21 s, and the program exited normally.

**tmux and screen.** Inside `tmux` and `screen` (`TMUX` or `STY` is set, or `TERM` starts with `screen` or `tmux`) nothing is asked, as for the width and the background; the images would need a passthrough that is not implemented. On the pty the program wrote no query there. `--graphics kitty` still forces the images, which show only where the multiplexer passes them on.

**In the shell configuration.** To fix the choice for a terminal that is not found, put the variable in the startup file of the shell:

```sh
# ~/.bashrc or ~/.zshrc
export CUBACLE_GRAPHICS=kitty
# fish: set -Ux CUBACLE_GRAPHICS kitty
```

`--graphics auto` on the command line undoes it for one run.

## Use the module

```rust
use cubacle::puzzle::{Facelets, PuzzleKind};
use cubacle::render::{icon, Action, KittyPlacement, Style};

let state = Facelets::solved(PuzzleKind::Cube5);
let action: Action = "2R".parse().expect("a move");
let drawn = icon(&state, &action, &Style::default()).expect("2R exists on a 5x5");
let svg = drawn.to_svg();
let pixels = drawn.to_rgba(128);
let escapes = pixels.to_kitty(&KittyPlacement::default());
assert!(svg.starts_with("<svg") && escapes.starts_with("\x1b_G"));
```

`state_icon(&state, &style)` draws a cube state alone: no arrow, no notation, no dimming, in a thin frame, with the size of a move icon. `Raster::hstack(&rasters, gap)` joins rasters side by side, as `solve --icons` does for a row.

`render` depends only on `puzzle`, does no I/O and keeps no state. The SVG is built from filled polygons with two-decimal numbers, so the same input always gives the same text. The `Style` sets the palette, the shading, the `Backdrop` (dark or light), an optional solid background and a target size in pixels; the size only widens gaps and arrows so that icons at 64 to 128 pixels stay crisp, and the SVG scales freely.

The notation and the `2x` are glyph outlines of Quicksand Bold, converted to polygons and embedded in the program, so SVG, pixels and Kitty graphics look the same and no font is needed. The font is under the SIL Open Font License; see `NOTICE.md` in the repository.

Examples of the SVG output are in `tests/fixtures/icons/`.

## Limits

- PDF export, animation and a web frontend are not part of this module yet.
- Inner slices use the straight arrow; there is no curved arrow for them.
- Geometry and arrow directions are tested for 2×2×2 to 5×5×5 (geometry) and 3×3×3 to 5×5×5 (arrows against `Facelets::apply`). The 6×6×6 and 7×7×7 use the same code and are only tested for shape counts and a stable SVG.
