# 5x5x5 comparison harness

Compares `FiveSolver` with the external five-phase solver `cs0x7f/cube555` on
identical random cubes. This directory holds no code of that solver: the driver
only calls its public `Search.init()` and `solveReduction(cube, 0)`, and the
solver is built from a checkout you supply, outside this repository.

    tools/bench/run.sh /path/to/cube555 /tmp/bench 100

Needs Java 8 or newer, `make`, and BSD `/usr/bin/time -l` (macOS). The external
solver's tables (about 237 MB) are written into the work directory, never here.
The external answers are the reduction only (lowercase letters are two-layer
turns); the tool finishes each reduced cube with our 3x3x3 solver, so both
solvers share the same final stage, and replays the whole solution.
