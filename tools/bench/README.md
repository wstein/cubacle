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

Our side reads or builds its tables in the cache directory (`CUBACLE_CACHE_DIR`,
else `$XDG_CACHE_HOME/cubacle`, else `~/.cache/cubacle`) and honours `CUBACLE_THREADS`; the `ours` mode prints
`prepare_ms` (table load or build) and then the solve times and total moves.
Run it once with the cache directory empty for the cold start, again for the
warm one, and with `CUBACLE_THREADS=1` for the single-thread numbers.

# 4x4x4 comparison harness

`run_four.sh` compares `FourSolver` with the external solver
`cs0x7f/TPR-4x4x4-Solver` (the Java original of the port) on identical random
cubes drawn by its generator. This directory holds no code of that solver:
`FourDriver.java` calls only its public `Search.init()` and `solution()` (and
reads the phase lengths by reflection); the solver is built from a checkout
you supply, outside this repository.

    tools/bench/run_four.sh /path/to/TPR-4x4x4-Solver /tmp/bench4 1000 1

Needs Java 8 or newer and BSD `/usr/bin/time -l` (macOS). Every answer of both
solvers is replayed with `Facelets::apply`; the Java rotation letters (`x`,
`y`, `z`) are turned into wide turns first. The tool reports the lengths with
and without the final rotation, the time per cube, and how many cubes have the
same phases 1 to 3 as Java. Run it with `CUBACLE_THREADS=1` for the
single-thread numbers; the Java solver is single-threaded.
