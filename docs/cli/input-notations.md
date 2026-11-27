---
title: "Input notations"
description: "Orbit64 tokens, facelets, colors and color schemes."
weight: 5
---

# Input notations

## Input notations

Wherever a command takes a cube (`solve`, `convert`), you can write it in one of three notations. Spaces are allowed between any groups and need no quoting.

### Orbit64 tokens

An [Orbit64](https://github.com/wstein/flix-orbit64) token is a short string that encodes a cube state. Its length tells the cube size, so no flag is needed. This command solves a 3x3x3 token:

```sh
cubacle solve A4ib8aejxj4Y
```

This 2x2x2 token converts to facelets:

```sh
cubacle convert CFJnI
```

```text
UUUUBFRRFRFFDDDDLLLLRBBB
```

`convert --to token` writes a token instead. It accepts facelets, colors or a token:

```sh
cubacle convert --to token LFDRFLDBFURLBBLUURUDDFRB
cubacle convert --to token "$(cubacle facelets "R U 2R 2U' F" --size 6)"
```

```text
ASj79
A3DZCufPmdSpgSdAXL5x0iJAczTlcP4refzTMJv_VgBoTFSReBHZMvFpjDdnSJzhHA
```

Tokens exist for every size from 2x2x2 to 7x7x7. `solve` and `convert` read the color framed lengths in the table. A canonical-pose length (4, 11, 26, 65 or 89 characters) is not read as a token, so a move expression of that length stays a move expression. A token that begins with `-` is an option to the command unless it comes after `--`.

| Cube | Token length | Checked against |
| --- | ---: | --- |
| 2x2x2 | 5 | 20 official examples |
| 3x3x3 | 12 | 20 official examples, the format's compatibility vectors |
| 4x4x4 | 27 | 20 official examples |
| 5x5x5 | 42 | color framed V2; a 43-character class-`00` token is refused with "Orbit64 V1 is no longer supported; convert with an older version" |
| 6x6x6 | 66 | the official solved cube, and round trips only |
| 7x7x7 | 90 | the official solved cube, and round trips only |

"Round trips only" means the 6x6x6 and 7x7x7 slot numbering follows the rules of the format but has no official scrambled vector to compare with: tests scramble cubes with outer, wide and inner moves, and each time `encode` and `decode` agree and the token has the length above. A token made by another program for these sizes may use a different slot numbering; see the format's own caveat. A token with a wrong length, a character outside the base64url alphabet, a number above the state count, or (when encoding) pieces that no cube has, is refused with a one-sentence error. State tokens are Orbit64 V2 (`FORMAT-V2.md`). Move sequences and structured algorithms stay on the existing notation; the algorithm DAG draft is not read.

### Facelets (URFDLB)

Each sticker is named by the face it belongs to: `U`, `R`, `F`, `D`, `L` or `B`. Stickers are listed face by face in the order U, R, F, D, L, B. A face has 4 stickers on a 2x2x2 and 9 on a 3x3x3. The solved 3x3x3 is:

```text
UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
```

You can write facelets compactly or spaced in any grouping. These two commands solve the same cube:

```sh
cubacle solve FUFUUFFFF UUUURRURU RRRFFRRFR BDBBDDBBB DLDDLLDDD LBLBBLLLL
cubacle solve FUFUUFFFFUUUURRURURRRFFRRFRBDBBDDBBBDLDDLLDDDLBLBBLLLL
```

A 2x2x2 works the same way:

```sh
cubacle solve UUUU BFRR FRFF DDDD LLLL RBBB
```

```text
U R U' R F2 R' U R F2 R2
```

### Colors

Each sticker is a color letter: `W`, `R`, `G`, `Y`, `O` or `B`. Compact and spaced forms both work.

```sh
cubacle solve GWGWWGGGG WWWWRRWRW RRRGGRRGR BYBBYYBBB YOYYOOYYY OBOBBOOOO
```

This prints the same solution as the Orbit64 example above. On a cube with fixed centers (3x3x3), cubacle reads the scheme from the centers, so western and Japanese cubes need no flag.

### Color schemes

A cube without centers, such as the 2x2x2, needs a scheme to map colors to faces. Use `--scheme`.

| Scheme | U R F D L B | Opposite pairs |
| --- | --- | --- |
| `boy` (default) | W R G Y O B | white-yellow, green-blue, red-orange |
| `jp` | W R G B O Y | white-blue, green-yellow, red-orange |

You can also give six letters in URFDLB order, such as `--scheme WRGYOB`. The longer names `western`, `western-boy` (for `boy`) and `japanese`, `japanese-bl-wt` (for `jp`) work as aliases.

```sh
cubacle convert --scheme jp WWWW RRRR GGGG BBBB OOOO YYYY
cubacle convert --scheme WRGYOB WWWWRRRRGGGGYYYYOOOOBBBB
```

Both commands print:

```text
UUUURRRRFFFFDDDDLLLLBBBB
```
