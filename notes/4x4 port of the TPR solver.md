---
id: 20261022140009
aliases: []
tags: ["type/decision", "status/decided", "category/engines"]
---
The 4x4x4 is a Rust port of cs0x7f/TPR-4x4x4-Solver (Java, Chen Shuang), Tsai's reduction with steps 3 and 4 merged; it is built (src/engines/four) and replaces the earlier plan of an own search. The 3x3x3 jar is not ported; the injected Solver trait takes its place. Licence: the README offers GPLv3 and MIT; the owner took the MIT option, the NOTICE.md row and the MIT text came before any code, and every ported file names its Java origin. On 1000 identical cubes it matches the reference length (44.34 against 44.59 moves without the final rotation) and its first three phases are identical to the reference's.

## Links

- [[4x4 versus TPR]]
- [[4x4 phases and tables]]
- [[Licensing]]
- [[5x5 reduction engine]]

Source: docs/design/4x4-port.md, lines 16-65
Source: docs/design/4x4-port.md, lines 484-531
Source: NOTICE.md, lines 22-22
Source: NOTICE.md, lines 33-61
Source: owner decision of 2026-10-22, lines 1-1
