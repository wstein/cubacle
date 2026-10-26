---
title: "Photos of a cube"
description: "Write synthetic photos of the six faces of a cube as PNG files, to test color recognition."
weight: 13
---

# Photos of a cube

`cubacle photo` renders synthetic photos of a cube: rounded stickers on a dark backdrop, a vignette, and a strip that says `SYNTHETIC <PRESET>`. They are test material for color recognition, never to be mistaken for camera photos. The generator is described in the [specification](../specs/synthetic-face.md).

The cube is given like for the other commands (facelets, colors, an Orbit64 token, `--scramble` or a move expression; see [Input notations](input-notations.md)). Sizes 2x2x2 to 7x7x7 work. Colors are drawn from the color scheme of the input (`--scheme`, else the one read from the centers, else boy: white up, green front).

## One file per face

```sh
cubacle photo --scramble "R U" --out out --pixels 64
```

```text
out/U.png
out/R.png
out/F.png
out/D.png
out/L.png
out/B.png
```

The faces are written in the order U R F D L B, one path per line. `--out` is created when missing (default: the current directory). `--pixels` is the side of a face photo (16 to 4096, default 256); `--size` stays the cube size, as everywhere else.

## One contact sheet

```sh
cubacle photo --scramble "R U" --sheet --preset bad-camera --seed 9 --out out
```

```text
out/sheet.png
```

`--sheet` writes one image of 4 x 3 faces instead: the cross net with U above F and L F R B in a row, D below F. At the default size it is 1024 x 768 pixels.

## Lighting and options

`--preset` is one of `clean` (default), `scatter`, `gradient`, `shadow`, `dim-warm`, `glare`, `noise` and `bad-camera`. `--scatter`, `--noise`, `--blur` and `--brightness` set the options of the same names on top of the preset. A cube of another size works the same way:

```sh
cubacle photo --size 4 --scramble "Rw U" --out out4 --pixels 64 --preset glare
```

```text
out4/U.png
out4/R.png
out4/F.png
out4/D.png
out4/L.png
out4/B.png
```

## Shadows

The `shadow` preset puts a soft shadow on the face. `--shadow-strength` (how much light it takes, 0 to 1, default 0.15), `--shadow-shape` (`band`, `half`, `spot` or `corner`, default `band`) and `--shadow-softness` (width of the edge as a share of the photo, default 0.03) change it. Giving any of the three adds the shadow to any other preset as well:

```sh
cubacle photo --scramble "R U" --out out --pixels 64 --shadow-shape half --shadow-strength 0.3 --shadow-softness 0.2
cubacle photo --scramble "R U" --out out --pixels 64 --preset glare --shadow-shape corner
```

Both write the six face paths like above. The default strength of 0.15 is about what a soft real shadow does; a stronger one is a test for color recognition, not a realistic photo. An unknown shape is refused:

```sh
cubacle photo --scramble R --shadow-shape ring --out out
```

```text
error: invalid value 'ring' for '--shadow-shape <SHADOW_SHAPE>': Unknown shadow shape: ring

For more information, try '--help'.
```

## Unknown preset

An unknown preset is refused:

```sh
cubacle photo --scramble R --preset real --out out
```

```text
error: invalid value 'real' for '--preset <PRESET>': Unknown synthetic preset: real

For more information, try '--help'.
```

## Reproducible output

`--seed` (default 1) fixes the random part. The same cube, preset, seed and `--pixels` write byte-identical files with the same binary; face number `i` (U is 0) uses the seed plus `i`, so the six faces do not share one noise pattern. A different seed changes the files of every preset: even `clean` has a small brightness jitter per sticker that depends on it.

Across platforms the color math may differ in the last bit in a few pixels; see the guarantee in the specification.

## File format

The files are PNG, RGBA with 8 bits per channel, written without compression (zlib stored blocks), so a file is about as large as its pixels: 16 kB for a 64 pixel face, 3 MB for a sheet at the default size.
