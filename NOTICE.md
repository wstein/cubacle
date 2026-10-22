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
| 5x5 greedy 3-cycle reduction (`src/engines/five/`, today) | original work, written from scratch | — | AGPL-3.0-or-later |
| 5x5 five-phase search (`src/engines/five/`, planned port, not yet landed) | cs0x7f/cube555 (Chen Shuang) | GPL-3.0 or MIT, MIT taken | AGPL-3.0-or-later, MIT notice retained |

The greedy 5x5 engine in the tree today contains no third-party code and was written
without reading other solvers' sources (commutator 3-cycles conjugated by setup moves are
textbook methods). The planned port of `cs0x7f/cube555` (README: GPLv3 and MIT, both by
Chen Shuang) takes the MIT option; its files will carry an attribution header naming the
Java origin, and `LICENSE-MIT` stays with them. `flix-cube-solvers` is GPL-3.0 and
`cube-solvers` is MIT; neither is used.

Ports added later (for example 4x4) record their upstream, licence and the option
taken in this table before they land.
