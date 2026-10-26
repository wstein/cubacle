---
title: "Credits and license"
description: "People, projects and license."
weight: 8
---

# Credits and license

## People and projects

- **[Herbert Kociemba](https://kociemba.org/)** devised the two-phase algorithm and published it in his Cube Explorer, and with it made fast near-optimal solving of the 3x3x3 possible. Everything cubacle does for cubes of every size rests on his work: the 3x3x3 solver is a two-phase solver, and the reduction solvers for the larger cubes finish with it. Thank you for this great work.
- **Chen Shuang** is the author of [min2phase](https://github.com/cs0x7f/min2phase) and of its original Rust port, [min2phase_rust](https://github.com/cs0x7f/min2phase_rust). The 3x3x3 solver core of cubacle is derived from that port. Chen Shuang also wrote the Java solvers [TPR-4x4x4-Solver](https://github.com/cs0x7f/TPR-4x4x4-Solver) and [cube555](https://github.com/cs0x7f/cube555), from which the 4x4x4 and 5x5x5 engines are ported.
- **[Daniel Walton](https://github.com/dwalton76)** wrote [rubiks-cube-NxNxN-solver](https://github.com/dwalton76/rubiks-cube-NxNxN-solver), the staged reduction method for big cubes that the 6x6x6 and 7x7x7 engines are ported from, and which serves as the reference they are measured against. Thank you for this great work.

## License

cubacle is distributed under the GNU Affero General Public License, version 3 or (at your option) any later version (`AGPL-3.0-or-later`). The full text is in the `LICENSE` file of the repository.

The original min2phase_rust was published under the MIT License. That grant is retained unmodified in the `LICENSE-MIT` file. Its copyright and permission notice must stay with every copy of the code derived from it. MIT is compatible with the AGPL, so the combined work is offered under the AGPL, while the original portions remain available under their MIT terms.

| Component | Upstream | Upstream license | Terms of this work |
| --- | --- | --- | --- |
| 3x3 two-phase core | cs0x7f/min2phase_rust | MIT | AGPL-3.0-or-later, MIT notice retained |
| Worker pool, batch, parallel and pattern APIs | Original work | Not applicable | AGPL-3.0-or-later |
| 4x4x4 and 5x5x5 reduction engines | cs0x7f/TPR-4x4x4-Solver, cs0x7f/cube555 | MIT option taken | AGPL-3.0-or-later, MIT notice retained |
| 6x6x6 and 7x7x7 reduction engine | dwalton76/rubiks-cube-NxNxN-solver | MIT | AGPL-3.0-or-later, MIT notice retained |

The repository's `NOTICE.md` is the authoritative statement. If it differs from this page, follow `NOTICE.md`.
