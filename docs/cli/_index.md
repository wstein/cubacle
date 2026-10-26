---
title: "Command-line guide"
description: "Use the cubacle command-line tool."
weight: 3
---

# Command-line guide

The `cubacle` tool has eleven subcommands: `solve` (also the way back after a mistake, with `--now`), `convert`, `canon`, `scramble`, `random`, `facelets`, `moves`, `order`, `photo` (see [Photos of a cube](photo.md)), `template` (see "Check a template file") and `patterns` (see [Patterns](patterns.md); its actions are `list` (the same as `patterns` alone), `show`, `find`, `identify` for the structure of a state and `cycles` for its part cycles and order, `import` for pattern tables, `update` (alias `download`) to download the URL sources of the config, `sources` for the pattern files in use and `--tree` for the overview by category; `--icons` draws pictures of the list, `show`, `find` and `identify`, see [Patterns with icons](icons.md#patterns-with-icons); see "Importing pattern files"). Run `cubacle <command> --help` for the full option list.

## Pages

- [Solve a cube](solve.md): Solve a cube, with targets and shorter or shortest solutions.
- [Solve options](solve-options.md): Within, move counts, known patterns, restricted move sets, listing solutions and failures.
- [Solve a 4x4x4 or 5x5x5](large-cubes.md): Solving the 4x4x4 and 5x5x5 from the command line.
- [Recover a mistake](recover.md): Rescan the cube after a wrong move and solve from there.
- [Input notations](input-notations.md): Orbit64 tokens, facelets, colors and color schemes.
- [Check a template file](template.md): Check a template file of target stickers.
- [Convert a cube](convert.md): Convert a cube between notations.
- [Generate scrambles and cubes](generate.md): Generate random scrambles and cubes.
- [Apply a scramble](moves.md): Apply a scramble: groups, commutators, hand units and slices.
- [Canonical token](canon.md): The canonical Orbit64 token, one identifier per pattern.
- [Order of a move sequence](order.md): The order of a move sequence.
- [Photos of a cube](photo.md): Write synthetic photos of the faces as PNG files.
- [Draw moves](draw.md): Draw moves in the terminal.
- [Move notations](move-notations.md): WCA, SiGN and SSE move notations.
