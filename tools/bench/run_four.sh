#!/bin/sh
# Usage: tools/bench/run_four.sh TPR_CHECKOUT WORKDIR [COUNT] [SEED]
# Builds cs0x7f/TPR-4x4x4-Solver in WORKDIR (a copy, outside this repository),
# draws COUNT random cubes (default 1000) with its own generator, solves them
# with both solvers, replays every answer and prints the statistics.
set -eu
SRC=$1; WORK=$2; N=${3:-1000}; SEED=${4:-1}
HERE=$(cd "$(dirname "$0")" && pwd); REPO=$(cd "$HERE/../.." && pwd)
mkdir -p "$WORK"; cp -R "$SRC/." "$WORK/tpr"
(cd "$WORK/tpr" && sh make.sh >/dev/null 2>&1)
cp "$HERE/FourDriver.java" "$WORK/tpr/"
(cd "$WORK/tpr" && javac -cp threephase.jar -d . FourDriver.java)
(cd "$REPO" && cargo build --release --example four_compare)
cd "$WORK/tpr"
java -cp .:threephase.jar FourDriver gen "$N" "$SEED" > "$WORK/cubes.txt"
java -cp .:threephase.jar FourDriver < "$WORK/cubes.txt" > "$WORK/raw.out" 2> "$WORK/java.err"
# the Java initialisation also prints "depth<TAB>count" lines: keep the answers
awk -F'\t' 'NF >= 6' "$WORK/raw.out" > "$WORK/answers.tsv"
grep -a "INIT_MS" "$WORK/java.err"
echo "== Java"
"$REPO/target/release/examples/four_compare" external "$WORK/cubes.txt" "$WORK/answers.tsv"
echo "== Rust, $(printf %s "${CUBACLE_THREADS:-all}") thread(s)"
/usr/bin/time -l "$REPO/target/release/examples/four_compare" ours "$WORK/cubes.txt" "$WORK/ours.tsv" 2>&1 \
  | grep -v "^ *[0-9]* *\(page\|swap\|block\|messages\|signals\|volun\|invol\|vir\|ave\|shared\|unshared\|instr\|cycles\)"
echo "== Phases 1 to 3 against Java"
"$REPO/target/release/examples/four_compare" phases "$WORK/cubes.txt" "$WORK/answers.tsv" "$WORK/ours.tsv"
