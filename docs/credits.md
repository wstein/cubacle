---
title: "Credits and licence"
description: "People, projects and licence."
weight: 8
---

# Credits and licence

## People and projects

- **[Herbert Kociemba](https://kociemba.org/)** devised the two-phase algorithm and published it in his Cube Explorer, and with it made fast near-optimal solving of the 3x3x3 possible. Everything cubacle does for cubes of every size rests on his work: the 3x3x3 solver is a two-phase solver, and the reduction solvers for the larger cubes finish with it. Thank you for this great work.
- **Werner Stein** is the author of cubacle.
- **Chen Shuang** is the author of [min2phase](https://github.com/cs0x7f/min2phase) and of its original Rust port, [min2phase_rust](https://github.com/cs0x7f/min2phase_rust). The 3x3x3 solver core of cubacle is derived from that port.
- **[flix-orbit64](https://github.com/wstein/flix-orbit64)** defines the Orbit64 format that cubacle reads and writes. The codec is a Rust re-implementation of that Apache-2.0 project by the same author (see `NOTICE.md`).

## Licence

cubacle is distributed under the GNU Affero General Public License, version 3 or (at your option) any later version (`AGPL-3.0-or-later`). The full text is in the `LICENSE` file of the repository.

The original min2phase_rust was published under the MIT License. That grant is retained unmodified in the `LICENSE-MIT` file. Its copyright and permission notice must stay with every copy of the code derived from it. MIT is compatible with the AGPL, so the combined work is offered under the AGPL, while the original portions remain available under their MIT terms.

| Component | Upstream | Upstream licence | Terms of this work |
| --- | --- | --- | --- |
| 3x3 two-phase core | cs0x7f/min2phase_rust | MIT | AGPL-3.0-or-later, MIT notice retained |
| Worker pool, batch, parallel and pattern APIs | Original work | Not applicable | AGPL-3.0-or-later |

The repository's `NOTICE.md` is the authoritative statement. If it differs from this page, follow `NOTICE.md`.
