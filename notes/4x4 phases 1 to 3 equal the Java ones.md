---
id: 20261022140010
aliases: []
tags: ["type/evidence", "status/reference", "category/engines"]
---
On 1000 identical cubes the moves before the 3x3x3 tail are identical to the Java solver's on all 1000, so the candidate queue, its tie order, the symmetry tables and the three searches are ported exactly. The whole answer without the rotation is identical on 665, because the tails come from different 3x3x3 solvers. The Java driver solves the first cube once unmeasured, because the first Edge3.getsym after new Search() reads a buffer that is not set up yet; the port does not copy that quirk. Tests keep 24 of the Java reductions as a fixture that must not be edited to pass.

## Links

- [[4x4 port of the TPR solver]]
- [[4x4 versus TPR]]

Source: tools/bench/four_compare.rs, lines 1-14
Source: tools/bench/FourDriver.java, lines 33-40
Source: tests/four_test.rs, lines 1-45
