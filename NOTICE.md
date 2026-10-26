# Licensing and provenance

cubacle is distributed under the **GNU Affero General Public License,
version 3 or (at your option) any later version** (`AGPL-3.0-or-later`).
The full text is in [`LICENSE`](LICENSE).

## Third-party origin

The 3x3 solver core is a Rust port of [min2phase](https://github.com/cs0x7f/min2phase)
by Chen Shuang, originally published as
[min2phase_rust](https://github.com/cs0x7f/min2phase_rust) under the MIT License.
That grant is retained unmodified in [`LICENSE-MIT`](LICENSE-MIT); its copyright
and permission notice must stay with every copy of the code derived from it.
MIT is compatible with AGPL, so the combined work is offered under the AGPL
while the original portions remain available under their MIT terms.

| Component | Upstream | Upstream license | Terms of this work |
| --- | --- | --- | --- |
| 3x3 two-phase core (`src/engines/three/`) | cs0x7f/min2phase_rust | MIT | AGPL-3.0-or-later, MIT notice retained |
| Worker pool (`src/engines/three/pool.rs`), batch/parallel/pattern APIs | original work | — | AGPL-3.0-or-later |
| 5x5 five-phase search (`src/engines/five/`, ported from `src/*.java`; the parallel search, cache and solver adapter are new) | cs0x7f/cube555 (Chen Shuang) | GPL-3.0 or MIT, MIT taken | AGPL-3.0-or-later, MIT notice (2023) in `LICENSE-MIT` |
| Reduction building blocks (`src/reduction/`: `bfs.rs`, `hashed.rs`, `nibbles.rs`, `comb.rs`, `ida.rs`) and the table spaces (`src/engines/five/spaces.rs`) | `Util.PruningTable` (`bfs.rs`, `hashed.rs`, `nibbles.rs`), the coordinate helpers of `Util` (`comb.rs`) and `PhaseSearch` (`ida.rs`) of cs0x7f/cube555, ported and reorganized; the parallel build, cache (`cache.rs`) and parallel runner (`par.rs`, the parallel part of `ida.rs`) are new | MIT, same as above | AGPL-3.0-or-later, MIT notice (2023) in `LICENSE-MIT` |
| 4x4 reduction engine (`src/engines/four/`, ported from `src/*.java`; the parallel search, the 3x3 hand-off through the `Solver` trait and the validation are new) | [cs0x7f/TPR-4x4x4-Solver](https://github.com/cs0x7f/TPR-4x4x4-Solver), Java | MIT or GPL-3.0-or-later, both offered in its README | MIT option taken; AGPL-3.0-or-later with the MIT notice below retained |
| 6x6 and 7x7 reduction engine (`src/engines/dwalton/`, ported from the Python and C sources of the reference solver; the table generation, cache, parallel search and the 3x3 hand-off through the `Solver` trait are new) | [dwalton76/rubiks-cube-NxNxN-solver](https://github.com/dwalton76/rubiks-cube-NxNxN-solver), Python and C | MIT, Copyright (c) 2019 Daniel Walton | AGPL-3.0-or-later with the MIT notice (2019) in `LICENSE-MIT` retained; each ported file names its origin |
| Orbit64 state codec (`src/notation/orbit64/`) | wstein/flix-orbit64 (Flix) | Apache-2.0, Copyright 2026 Werner Stein | AGPL-3.0-or-later; Rust re-implementation of the format and of the slot tables in `Orbit64.Net`, same author as cubacle; each file names its origin |
| Canonical form vectors (`tests/canonical_vectors.txt`, `src/canonical/`) | wstein/flix-orbit64 v0.9.0 (Flix), `test/TestCanonicalVectors.flix`, `test/TestCanonical.flix` | Apache-2.0, Copyright 2026 Werner Stein | AGPL-3.0-or-later; test data copied from the owner's own library, the Rust code follows its reference `Orbit64.Canonical` |
| Synthetic face generator (`docs/specs/synthetic-face.md`, `src/synthetic/`) | wstein/cube-assembler, `docs/synthetic-face-generator-spec.md` and `src/core/vision/SyntheticFace.res` (ReScript prototype) | MIT | AGPL-3.0-or-later; the draft is copied with American spelling; the Rust code follows its frame points and deviates where the page says so |

The port of `cs0x7f/cube555` (README: GPLv3 and MIT, both by Chen Shuang) takes the MIT
option. Each ported file starts with a header naming the Java file it comes from, and the
MIT notice is in `LICENSE-MIT` beside the one of min2phase_rust. `flix-cube-solvers` is
GPL-3.0 and `cube-solvers` is MIT; neither is used.

Ports added later record their upstream, license and the option
taken in this table before they land.

## TPR-4x4x4-Solver (MIT option)

The 4x4x4 engine is a Rust port of the three-phase reduction solver
`cs0x7f/TPR-4x4x4-Solver`, which implements Tsai's 4x4x4 algorithm. Its README
offers the code under the GPLv3 or under the MIT license; this work takes the
MIT option. Two of its source files (`Search.java`, `MainProgram.java`) also carry
a 2014 header that names only the GPL-3.0-or-later. The 3x3x3 part of that solver
is not ported: the injected `Solver` trait replaces it. Every ported file
names the Java file it derives from and keeps this notice:

    Copyright (c) 2023 Chen Shuang

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.

## Label glyph outlines (SIL Open Font License 1.1)

The move labels and the `2x` mark of the icons can be drawn from glyph outlines
of Quicksand Bold, converted to polygons and embedded as Rust data
(`src/render/glyphs/quicksand.rs`, written by `tools/embed_glyphs.py`). No font
file is shipped and no font is loaded at run time. Only the characters
`0123456789URFDLBwx'` are embedded. Under the OFL 1.1 these embedded outlines
are derived glyph data, not a font program: they are not offered or named as a
font, and the Reserved Font Name "Quicksand" is not used for any product. The
file names its origin; the license text is kept in
[`licenses/OFL-Quicksand.txt`](licenses/OFL-Quicksand.txt).

| Font | Upstream | License | Copyright line | License text |
| --- | --- | --- | --- | --- |
| Quicksand (Bold, instance of the variable font) | <https://github.com/andrew-paglinawan/QuicksandFamily>, taken from [google/fonts](https://github.com/google/fonts) at commit `45e3d5269e4a17d823f6d84470bc084dc5ae46f6`, `ofl/quicksand/Quicksand[wght].ttf` | OFL-1.1, Reserved Font Name "Quicksand" | Copyright 2011 The Quicksand Project Authors | [`licenses/OFL-Quicksand.txt`](licenses/OFL-Quicksand.txt) |
