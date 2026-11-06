// Comparison input for G1: solves the fixed scrambles of golden.txt with Cubing.js
// and prints the move counts and the time per solve as one JSON object.
//
//   node tools/wasm/cubing-moves.mjs <checkout with node_modules/cubing> [rounds]
//
// 3x3x3: experimentalSolve3x3x3IgnoringCenters; 2x2x2: experimentalSolve2x2x2.
// Moves are counted as written (R2 is one move). Every solution is applied to
// its scramble and must give a solved cube (whole-cube rotation and center
// orientation ignored); otherwise the script exits 1. Times are per call in
// Node, including the library's own overhead; "first" is the first call of
// each size in this process.
import fs from 'node:fs';
import { performance } from 'node:perf_hooks';
import { pathToFileURL } from 'node:url';

const root = process.argv[2];
const rounds = Number(process.argv[3] ?? 5);
if (!root) {
  console.error('usage: node tools/wasm/cubing-moves.mjs <checkout with node_modules/cubing> [rounds]');
  process.exit(2);
}
const lib = (name) => import(pathToFileURL(`${root}/node_modules/cubing/dist/lib/cubing/${name}/index.js`).href);
const { puzzles } = await lib('puzzles');
const { Alg } = await lib('alg');
const { experimentalSolve3x3x3IgnoringCenters, experimentalSolve2x2x2 } = await lib('search');

const lines = fs
  .readFileSync(new URL('./golden.txt', import.meta.url), 'utf8')
  .split('\n')
  .filter(Boolean)
  .map((line) => line.split('|'));
const kpuzzle = { 3: await puzzles['3x3x3'].kpuzzle(), 2: await puzzles['2x2x2'].kpuzzle() };
const solvers = { 3: experimentalSolve3x3x3IgnoringCenters, 2: experimentalSolve2x2x2 };

const percentile = (values, p) => {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.min(sorted.length - 1, Math.floor((p / 100) * sorted.length))];
};
const round = (x) => Math.round(x * 1000) / 1000;
const out = { rounds };
let unsolved = 0;
for (const size of [3, 2]) {
  const scrambles = lines.filter((line) => line[0] == size).map((line) => line[1]);
  const lengths = [];
  const times = [];
  let first;
  for (let r = 0; r < rounds; r++) {
    for (const [i, scramble] of scrambles.entries()) {
      const pattern = kpuzzle[size].defaultPattern().applyAlg(new Alg(scramble));
      const started = performance.now();
      const solution = await solvers[size](pattern);
      const millis = performance.now() - started;
      if (r === 0 && i === 0) first = millis;
      else times.push(millis);
      if (r === 0) {
        lengths.push([...solution.childAlgNodes()].length);
        if (!pattern.applyAlg(solution).experimentalIsSolved({ ignorePuzzleOrientation: true, ignoreCenterOrientation: true })) unsolved++;
      }
    }
  }
  out[`${size}x${size}`] = {
    scrambles: scrambles.length,
    mean_moves: round(lengths.reduce((a, b) => a + b, 0) / lengths.length),
    move_histogram: Object.fromEntries([...new Set(lengths)].sort((a, b) => a - b).map((k) => [k, lengths.filter((x) => x === k).length])),
    first_call_ms: round(first),
    later_calls: { n: times.length, median_ms: round(percentile(times, 50)), p90_ms: round(percentile(times, 90)), max_ms: round(Math.max(...times)) },
  };
}
out.unsolved = unsolved;
console.log(JSON.stringify(out, null, 2));
process.exit(unsolved === 0 ? 0 : 1);
