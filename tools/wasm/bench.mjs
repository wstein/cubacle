// G1 measurement of the browser WebAssembly facade (crates/wasm), run in Node.
//
//   node tools/wasm/bench.mjs <cubacle_wasm.wasm> [rounds]
//
// Prints one JSON object: file sizes (raw, gzip -9, brotli quality 11), cold
// time (new instance, tables built by the first solve) and warm time per
// solve on fixed scrambles, solution lengths, and linear memory. Exits 1 when
// a solve fails, a solution exceeds its move limit or differs from golden.txt.
import fs from 'node:fs';
import zlib from 'node:zlib';
import { performance } from 'node:perf_hooks';

const path = process.argv[2];
const rounds = Number(process.argv[3] ?? 20);
if (!path) {
  console.error('usage: node tools/wasm/bench.mjs <cubacle_wasm.wasm> [rounds]');
  process.exit(2);
}

// The fixed scrambles and their expected solutions: `size|scramble|solution`.
// The native test crates/wasm/tests/golden.rs replays every line, so a match
// here means the WebAssembly build answers exactly as the native build does.
const GOLDEN = fs
  .readFileSync(new URL('./golden.txt', import.meta.url), 'utf8')
  .split('\n')
  .filter((line) => line !== '')
  .map((line) => line.split('|'));
const SCRAMBLES = { 3: [], 2: [] };
const EXPECTED = new Map();
for (const [size, scramble, solution] of GOLDEN) {
  SCRAMBLES[size].push(scramble);
  EXPECTED.set(`${size}|${scramble}`, solution);
}
const MAX_MOVES = { 3: 25, 2: 14 };

const bytes = fs.readFileSync(path);
const sizes = {
  raw: bytes.length,
  gzip: zlib.gzipSync(bytes, { level: 9 }).length,
  brotli: zlib.brotliCompressSync(bytes, {
    params: { [zlib.constants.BROTLI_PARAM_QUALITY]: 11, [zlib.constants.BROTLI_PARAM_SIZE_HINT]: bytes.length },
  }).length,
};

const module = new WebAssembly.Module(bytes);
const encoder = new TextEncoder();
const decoder = new TextDecoder();

function instantiate() {
  let memory;
  const instance = new WebAssembly.Instance(module, {
    env: {
      cubacle_host_now: () => performance.now(),
      cubacle_host_random: (pointer, length) => {
        const view = new Uint8Array(memory.buffer, pointer, length);
        for (let i = 0; i < length; i++) view[i] = (Math.random() * 256) | 0;
      },
    },
  });
  memory = instance.exports.memory;
  return instance.exports;
}

let failures = 0;
function solve(exports, size, scramble) {
  const input = encoder.encode(scramble);
  const pointer = exports.cubacle_alloc(input.length);
  new Uint8Array(exports.memory.buffer, pointer, input.length).set(input);
  const started = performance.now();
  const result = exports.cubacle_solve(size, pointer, input.length, MAX_MOVES[size]);
  const millis = performance.now() - started;
  const total = exports.cubacle_result_len(result);
  const view = new Uint8Array(exports.memory.buffer, result, total);
  const status = view[4];
  const text = decoder.decode(view.slice(5));
  exports.cubacle_free(result, total);
  exports.cubacle_free(pointer, input.length);
  if (EXPECTED.has(`${size}|${scramble}`) && EXPECTED.get(`${size}|${scramble}`) !== text) {
    failures++;
    console.error(`size ${size} "${scramble}": got "${text}", expected "${EXPECTED.get(`${size}|${scramble}`)}"`);
  }
  const length = text.trim() === '' ? 0 : text.trim().split(/\s+/).length;
  if (status !== 0 || length > MAX_MOVES[size]) {
    failures++;
    console.error(`size ${size} "${scramble}": status ${status} "${text}"`);
  }
  return { millis, length };
}

const percentile = (values, p) => {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.min(sorted.length - 1, Math.floor((p / 100) * sorted.length))];
};
const round = (x) => Math.round(x * 1000) / 1000;
const stats = (values) => ({
  n: values.length,
  median_ms: round(percentile(values, 50)),
  p90_ms: round(percentile(values, 90)),
  max_ms: round(Math.max(...values)),
});

// Cold: a new instance per run; instantiate and the first solve (which builds the tables).
const cold = {};
for (const size of [3, 2]) {
  const instantiateMs = [];
  const firstSolveMs = [];
  for (let run = 0; run < 7; run++) {
    const t0 = performance.now();
    const exports = instantiate();
    instantiateMs.push(performance.now() - t0);
    firstSolveMs.push(solve(exports, size, SCRAMBLES[size][0]).millis);
  }
  cold[`${size}x${size}`] = {
    instantiate: stats(instantiateMs),
    first_solve_after_instantiate: stats(firstSolveMs),
  };
}

// Warm: one instance, tables built, `rounds` passes over the fixed scrambles.
const exports = instantiate();
const memory = { after_instantiate_bytes: exports.memory.buffer.byteLength };
const warm = {};
for (const size of [3, 2]) {
  solve(exports, size, SCRAMBLES[size][0]);
  memory[`after_first_solve_${size}x${size}_bytes`] = exports.memory.buffer.byteLength;
  const times = [];
  const lengths = [];
  for (let round = 0; round < rounds; round++) {
    for (const scramble of SCRAMBLES[size]) {
      const { millis, length } = solve(exports, size, scramble);
      times.push(millis);
      if (round === 0) lengths.push(length);
    }
  }
  warm[`${size}x${size}`] = {
    ...stats(times),
    scrambles: SCRAMBLES[size].length,
    mean_moves: round(lengths.reduce((a, b) => a + b, 0) / lengths.length),
    moves: lengths,
  };
}
memory.after_all_bytes = exports.memory.buffer.byteLength;
memory.note = 'WebAssembly linear memory never shrinks, so the final size is the peak.';

console.log(JSON.stringify({ node: process.version, file: path.split('/').pop(), sizes, cold, warm, memory }, null, 2));
process.exit(failures === 0 ? 0 : 1);
