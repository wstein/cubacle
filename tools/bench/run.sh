#!/bin/sh
# Usage: tools/bench/run.sh CUBE555_CHECKOUT WORKDIR [COUNT]
# Builds the external solver in WORKDIR (a copy, outside this repository),
# draws COUNT random 5x5x5 cubes (default 100), and compares both solvers.
set -eu
SRC=$1; WORK=$2; N=${3:-100}
HERE=$(cd "$(dirname "$0")" && pwd); REPO=$(cd "$HERE/../.." && pwd)
mkdir -p "$WORK"; cp -R "$SRC/." "$WORK/c555"
(cd "$WORK/c555" && mkdir -p dist && make build >/dev/null 2>&1)
cp "$HERE/Cube555Driver.java" "$WORK/c555/"
(cd "$WORK/c555" && javac -d dist -cp dist/cube555.jar Cube555Driver.java)
(cd "$REPO" && cargo build --release --example five_compare)
for i in $(seq "$N"); do "$REPO/target/release/cubacle" random --size 5; done > "$WORK/cubes.txt"
cd "$WORK/c555"
# first run writes the tables (cold start); the second one reads them (warm start)
/usr/bin/time -l java -cp dist:dist/cube555.jar Cube555Driver < /dev/null 2> "$WORK/cold.err" || true
/usr/bin/time -l java -cp dist:dist/cube555.jar Cube555Driver < "$WORK/cubes.txt" > "$WORK/raw.out" 2> "$WORK/warm.err"
grep -aP '^\d+\t' "$WORK/raw.out" | sed 's/ *$//' > "$WORK/answers.tsv"
grep -a "INIT_MS\|maximum resident" "$WORK/cold.err" "$WORK/warm.err"
"$REPO/target/release/examples/five_compare" external "$WORK/cubes.txt" "$WORK/answers.tsv"
/usr/bin/time -l "$REPO/target/release/examples/five_compare" ours "$WORK/cubes.txt" 2>&1 | grep -v "^ *[0-9]* *\(page\|swap\|block\|messages\|signals\|volun\|invol\|vir\|ave\|shared\|unshared\|instr\|cycles\)"
