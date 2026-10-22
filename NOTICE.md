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

| Component | Upstream | Upstream licence | Terms of this work |
| --- | --- | --- | --- |
| 3x3 two-phase core (`src/engines/three/`) | cs0x7f/min2phase_rust | MIT | AGPL-3.0-or-later, MIT notice retained |
| Worker pool (`src/engines/three/pool.rs`), batch/parallel/pattern APIs | original work | — | AGPL-3.0-or-later |
| 5x5 five-phase search (`src/engines/five/`, ported from `src/*.java`; the parallel search, cache and solver adapter are new) | cs0x7f/cube555 (Chen Shuang) | GPL-3.0 or MIT, MIT taken | AGPL-3.0-or-later, MIT notice (2023) in `LICENSE-MIT` |
| Reduction building blocks (`src/reduction/`: `bfs.rs`, `hashed.rs`, `nibbles.rs`, `comb.rs`, `ida.rs`) and the table spaces (`src/engines/five/spaces.rs`) | `Util.PruningTable` (`bfs.rs`, `hashed.rs`, `nibbles.rs`), the coordinate helpers of `Util` (`comb.rs`) and `PhaseSearch` (`ida.rs`) of cs0x7f/cube555, ported and reorganised; the parallel build, cache (`cache.rs`) and parallel runner (`par.rs`, the parallel part of `ida.rs`) are new | MIT, same as above | AGPL-3.0-or-later, MIT notice (2023) in `LICENSE-MIT` |
| Orbit64 state codec (`src/notation/orbit64/`) | wstein/flix-orbit64 (Flix) | Apache-2.0, Copyright 2026 Werner Stein | AGPL-3.0-or-later; Rust re-implementation of the format and of the slot tables in `Orbit64.Net`, same author as cubacle; each file names its origin |

The port of `cs0x7f/cube555` (README: GPLv3 and MIT, both by Chen Shuang) takes the MIT
option. Each ported file starts with a header naming the Java file it comes from, and the
MIT notice is in `LICENSE-MIT` beside the one of min2phase_rust. `flix-cube-solvers` is
GPL-3.0 and `cube-solvers` is MIT; neither is used.

Ports added later (for example 4x4) record their upstream, licence and the option
taken in this table before they land.
