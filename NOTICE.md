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
| 5x5 reduction engine (`src/engines/five/`) | original work, written from scratch | — | AGPL-3.0-or-later |

The 5x5 engine contains no third-party code and was written without reading other
solvers' sources. The licences of the related solvers were checked and none of their code
was copied or translated: `cs0x7f/cube555` and `flix-cube-solvers` are GPL-3.0 and
`cube-solvers` is MIT. The techniques used (reduction, commutator 3-cycles conjugated by
setup moves) are textbook methods.

Ports added later (for example 4x4) record their upstream, licence and the option
taken in this table before they land.
