---
title: "Synthetic face generator"
description: "Specification and description of the synthetic face generator: a deterministic renderer that turns a grid of sticker colors into a cube face photo."
weight: 40
---

# Synthetic face generator

Status: implemented in `src/synthetic/` (revision r1). The page describes what the code does and `tests/synthetic_test.rs` verifies it.

The generator renders one cube face as a `size x size` RGBA photo from a grid of sticker letters. The photo looks like a cropped camera shot: rounded stickers on a dark backdrop, a thin border, a vignette. A strip at the bottom carries the text `SYNTHETIC <PRESET>`, so the image is never mistaken for a camera photo. It is meant as test material for color recognition.

## Guarantee

The same binary with the same `(grid, preset, seed, size, options)` returns the same bytes. The random numbers come from an integer hash with wrapping arithmetic that is identical on every platform. The colors and the noise use `f64` functions of the Rust standard library (`powf`, `cbrt`, `atan2`, `ln`, `cos`), which can differ in the last bit between platforms; a few pixels may then differ by one level. The golden hashes in the tests are those of the platforms of the CI. Output may change between versions of cubacle when this page says so in the revision history.

## Interface

```rust
render_face(grid: &[Vec<char>], preset: Preset, seed: i32, size: usize, options: &Options)
    -> Result<Vec<u8>, SyntheticError>
```

| Input | Meaning |
| --- | --- |
| `grid` | Sticker letters, row-major, top row first, square, at least 1x1. `W Y O R G B` are known colors; any other letter is drawn gray. |
| `preset` | One of the eight presets below. `Preset::from_str` fails with `Unknown synthetic preset: <name>`. |
| `seed` | Random seed. |
| `size` | Side of the photo in pixels, at least 1. Recommended 64 to 2048. |
| `options` | All optional, see below. |

The result has `size * size * 4` bytes in RGBA order, row-major, top row first, index `(y * size + x) * 4`. Alpha is always 255. A grid that is empty or not square, and a size of 0, are errors (`SyntheticError`).

## Numbers

All arithmetic is IEEE-754 binary64. A float is stored into a pixel byte by clamping to 0..255 and rounding to the nearest whole number (NaN becomes 0). Every stage reads back the stored 8-bit pixels, never floats. `round` elsewhere is the nearest whole number as well.

## Random numbers

The generator has no state: every value is a function of `(seed, index)`.

```
z = ((seed as u32) << 32) xor (i * 0x9E3779B97F4A7C15)      wrapping 64-bit arithmetic
z = z + 0x9E3779B97F4A7C15, then the SplitMix64 finalizer of z
hash(seed, i) = the high 32 bits of z
unit(seed, i) = (hash(seed, i) and 0xFFFFFF) / 2^24                      in [0, 1)
gauss(seed, i) = sqrt(-2 ln(unit(seed, 2i) + 1e-9)) * cos(2 pi unit(seed, 2i + 1))
```

The index ranges in use are `0 .. 6 size^2` (noise, three values per pixel, doubled inside `gauss`), `1000 + k` (brightness jitter of sticker `k`), `5000 .. 5015` (glare), `6000 + 2k` and `6001 + 2k` (scatter). The option noise uses `seed + 7919`. A new option or preset takes a new index range, so that existing photos stay unchanged.

## Colors

| Name | Value |
| --- | --- |
| Palette `W` | (232, 232, 224) |
| Palette `Y` | (238, 206, 44) |
| Palette `O` | (238, 122, 34) |
| Palette `R` | (190, 36, 46) |
| Palette `G` | (34, 150, 74) |
| Palette `B` | (34, 84, 192) |
| Backdrop | (20, 20, 24) |
| Unknown sticker | (128, 128, 128) |
| Label | (170, 170, 176) |

Scatter spread per letter (hue sigma in degrees, chroma coefficient of variation): W 22.0 / 0.35, Y 2.5 / 0.045, O 1.2 / 0.034, R 1.4 / 0.043, G 0.7 / 0.063, B 1.0 / 0.08, any other letter 2.0 / 0.05. The table is keyed by the letter of the sticker, also when the palette overrides its color.

Every sticker color goes through OKLCH and back, also with scatter 0 (the round trip rounds to whole numbers). For sticker `k = row * n + col`, with `s` the scatter option:

```
(L, C, h) = oklch(base)
hue    = (h + gauss(seed, 6000 + 2k) * (hueSigma * s)) * pi / 180
chroma = max(0, C * (1 + gauss(seed, 6001 + 2k) * (chromaCV * s)))
color  = oklabToRgb(L, chroma cos(hue), chroma sin(hue))
```

The conversions are the OKLab matrices of Bjoern Ottosson with sRGB gamma; `oklabToRgb` rounds each channel to a whole number in 0..255.

## Drawing

With `margin = round(size * marginShare)`, `inner = size - 2 margin`, `cell = inner / n`, `gap = max(2, cell * gapShare)`, `radius = cell * radiusShare`, `half = cell / 2 - gap / 2` and `corner = half - radius`, every pixel `(x, y)` is drawn as follows.

1. `fx = x + 0.5 - margin`, `fy` likewise. Outside `[0, inner)` the pixel is the backdrop.
2. Inside, `col = floor(fx / cell)`, `row = floor(fy / cell)` (both at most `n - 1`), `dx = fx - (col + 0.5) cell`, `dy` likewise. The distance to the rounded square is `outside + min(max(qx, qy), 0) - radius` with `qx = |dx| - corner`, `qy = |dy| - corner` and `outside = sqrt(max(qx, 0)^2 + max(qy, 0)^2)`. The coverage is `clamp(0.5 - distance, 0, 1)`.
3. The sticker color is the scattered palette color. A logo on this sticker replaces it by the palette color of the logo inside the radius `logo.size * half` (default 0.5), with no scatter.
4. `shade = rim * slope * jitter` with `rim = 1 - 0.1 clamp(1 - depth / (0.08 cell), 0, 1)` (`depth = -distance`), `slope = 1 - 0.04 (dx + dy) / cell` and `jitter = 1 + (unit(seed, k + 1000) - 0.5) * 0.05 * lightness`.
5. `color = backdrop (1 - coverage) + base * shade * coverage`, multiplied by the vignette `1 - 0.28 (cx^2 + cy^2)` with `cx = x / size - 0.5`, `cy = y / size - 0.5`.

## Presets

| Preset | Effect |
| --- | --- |
| `clean` | none |
| `scatter` | none, but the scatter option defaults to 1 |
| `gradient` | `r * (0.7 + 0.6 tx)`, `b * (1.3 - 0.6 tx)`, `g` unchanged |
| `shadow` | diagonal band: `t = (tx + ty) / 2`, `inside = clamp((t - 0.4) / 0.03, 0, 1) * clamp((0.65 - t) / 0.03, 0, 1)`, all channels times `1 - 0.45 inside` |
| `dim-warm` | channels times (0.6, 0.5, 0.35), then noise with sigma 8 |
| `glare` | four elliptical highlights toward white |
| `noise` | noise with sigma 20 |
| `bad-camera` | box blur of radius `max(1, round(0.006 size))`, then noise with sigma 15 |

`tx = x / size`, `ty = y / size`. Noise adds `gauss(seed, 3k + c) * sigma` to channel `c` of pixel `k = y * size + x`. Glare spot `j = 0..3` has center `(unit(seed, 5000 + 4j), unit(seed, 5001 + 4j))` and radii `rx = 0.06 + 0.08 unit(seed, 5002 + 4j)`, `ry = 0.05 + 0.07 unit(seed, 5003 + 4j)`; with `q = ((tx - cx) / rx)^2 + ((ty - cy) / ry)^2` its weight is `0.7 (1 - q)` for `q < 1`, else 0; the largest weight wins and each channel becomes `c (1 - weight) + 255 weight`. The blur is a separable box blur, horizontal pass first, edges replicated, each pass reading the 8-bit result of the previous one.

## Options

| Option | Default | Effect |
| --- | --- | --- |
| `scatter` | 1 for `scatter`, else 0 | scale of the hue and chroma scatter |
| `lightness` | 1 | scale of the brightness jitter between stickers |
| `palette` | none | letters to colors, merged over the default palette; new letters allowed |
| `logo` | none | round patch on one sticker (`row`, `col`, `color`, optional `size`) |
| `brightness` | none | factor on every channel, applied after the preset |
| `blur` | none | box blur radius `round(blur)`, used when at least 1, after brightness |
| `noise` | none | noise with this sigma and `seed + 7919`, used when above 0, after blur |
| `geometry` | margin 0.035, gap 0.07, radius 0.14 | shares of `size` (margin) and of a sticker (gap, radius) |

## Label

The text is `SYNTHETIC ` plus the preset name in upper case. Glyphs are 5x7 pixels (`A B C D E G H I L M N O R S T W Y -`; a space or a letter without glyph only advances). With `m = round(size * marginShare)`, `scale = max(1, (m - 4) / 7)` (integer division) and `top = size - m + (m - 7 scale) / 2`, character `c` lights the pixel `(m + (6c + gx) scale + dx, top + gy scale + dy)` for each lit glyph pixel `(gx, gy)` and `dx, dy < scale`. Pixels beyond the right or bottom edge are clipped, so on a small photo the end of a long label is cut off.

## Processing order

1. Draw the face. 2. Apply the preset. 3. Brightness, blur, noise. 4. Label.

## Differences from the draft r0

The draft r0 came from the owner's ReScript prototype, which ran on JavaScript. cubacle does not reproduce its bytes; its 15 SHA-256 vectors do not apply. Changes in r1:

- The hash is a 64-bit integer mixer with wrapping arithmetic, not a double product with the JavaScript `ToInt32`. No libm emulation is attempted; the guarantee above says what holds.
- A float is stored by round-to-nearest (halves away from zero), not half to even, and `oklabToRgb` and the margin use the same rule instead of JavaScript's round-half-up.
- The label is clipped at the right edge instead of wrapping into the next row by flat index.
- The prototype had a ninth preset `real` with tone, shading and face-shift options; r1 keeps the eight presets of the draft and none of them.
- The sticker scatter is computed once per sticker, not once per pixel; the result is the same.
- Presets are an enum and the grid is a vector of `char` rows; the only error cases are an unknown preset name, a bad grid and a size of 0.

## Revision history

| Revision | Content |
| --- | --- |
| r0 | Draft from the owner's prototype (cube-assembler, `SyntheticFace.res`), with JavaScript-exact arithmetic and 15 SHA-256 vectors. |
| r1 | This page: the Rust implementation in `src/synthetic/`, with the differences above. Golden hashes pin the output of r1. |
