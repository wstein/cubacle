# Fixed-center picture drawing fixtures

The SVG and text files in this directory are generated drawings, not solver
outputs. `upright` has zero fixed-center twists; `turned` has one clockwise
quarter turn on U and F. Their Orbit64 V2 framed picture tokens are
`AAAAAAAAAAAAAA` and `AAAAAAAAAAAAGY`. Files cover each whole-cube view and the
x, y and z move icons, using the same color geometry as the existing renderer.

Regenerate from the repository root:

```sh
cargo run --release --example picture_render_fixtures -- tests/fixtures/picture-render
```

## Artwork license

Copyright © 2026 Werner Stein. Created with AI assistance (Codex, OpenAI).

SPDX-License-Identifier: CC-BY-SA-4.0

The SVG and text drawings in this directory are licensed under the
Creative Commons Attribution-ShareAlike 4.0 International License:
<https://creativecommons.org/licenses/by-sa/4.0/>.

You may share and adapt these drawings with attribution to Werner Stein,
a link to the license and an indication of changes. Adaptations must use the
same license. No additional restrictions may be imposed. These fixtures are
original generated drawings; no third-party artwork was adapted.
The generator and test code remain AGPL-3.0-or-later.
