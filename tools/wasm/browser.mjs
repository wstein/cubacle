// G1 measurement of the browser WebAssembly facade (crates/wasm), run in Chromium.
//
//   node tools/wasm/browser.mjs <cubacle_wasm.wasm> <playwright-core module path> [runs]
//
// The module path is the directory (or entry file) of an installed `playwright-core`
// (or `playwright`) package; it is not a dependency of this repository. The script
// serves the .wasm over HTTP on 127.0.0.1 with cross-origin isolation (finer timer
// resolution), then loads a fresh page per cold run and prints one JSON object:
// compile (`WebAssembly.compileStreaming`), instantiate, first solve per size,
// warm time per solve, and linear memory. Exits 1 when an answer differs from
// golden.txt, a solve fails, or a figure is missing.
import fs from 'node:fs';
import http from 'node:http';
import { createRequire } from 'node:module';

const [wasmPath, playwrightPath, runsArg] = process.argv.slice(2);
if (!wasmPath || !playwrightPath) {
  console.error('usage: node tools/wasm/browser.mjs <cubacle_wasm.wasm> <playwright-core module path> [runs]');
  process.exit(2);
}
const runs = Number(runsArg ?? 7);
const require = createRequire(import.meta.url);
const { chromium } = require(require.resolve(playwrightPath, { paths: [process.cwd()] }));

const GOLDEN = fs
  .readFileSync(new URL('./golden.txt', import.meta.url), 'utf8')
  .split('\n')
  .filter((line) => line !== '')
  .map((line) => line.split('|'));
const wasmBytes = fs.readFileSync(wasmPath);

const server = http.createServer((req, res) => {
  const headers = {
    'Cross-Origin-Opener-Policy': 'same-origin',
    'Cross-Origin-Embedder-Policy': 'require-corp',
    'Cache-Control': 'no-store',
  };
  if (req.url === '/m.wasm') {
    res.writeHead(200, { ...headers, 'Content-Type': 'application/wasm' });
    res.end(wasmBytes);
  } else {
    res.writeHead(200, { ...headers, 'Content-Type': 'text/html' });
    res.end('<!doctype html><title>g1</title>');
  }
});
await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;

// Runs inside the page. One cold sequence in a fresh page, then the warm passes.
async function pageRun({ golden, rounds }) {
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  const MAX_MOVES = { 3: 25, 2: 14 };
  const result = { isolated: self.crossOriginIsolated, failures: [] };
  const t0 = performance.now();
  const module = await WebAssembly.compileStreaming(fetch('/m.wasm'));
  const t1 = performance.now();
  let memory;
  const instance = await WebAssembly.instantiate(module, {
    env: {
      cubacle_host_now: () => performance.now(),
      cubacle_host_random: (pointer, length) => crypto.getRandomValues(new Uint8Array(memory.buffer, pointer, length)),
    },
  });
  const t2 = performance.now();
  const exports = instance.exports;
  memory = exports.memory;
  result.compileMs = t1 - t0;
  result.instantiateMs = t2 - t1;
  result.memoryAfterInstantiate = memory.buffer.byteLength;
  const solve = (size, scramble) => {
    const input = encoder.encode(scramble);
    const pointer = exports.cubacle_alloc(input.length);
    new Uint8Array(memory.buffer, pointer, input.length).set(input);
    const started = performance.now();
    const out = exports.cubacle_solve(size, pointer, input.length, MAX_MOVES[size]);
    const millis = performance.now() - started;
    const total = exports.cubacle_result_len(out);
    const view = new Uint8Array(memory.buffer, out, total);
    const status = view[4];
    const text = decoder.decode(view.slice(5));
    exports.cubacle_free(out, total);
    exports.cubacle_free(pointer, input.length);
    const expected = golden.find((g) => g[0] == size && g[1] === scramble)?.[2];
    if (status !== 0 || text !== expected) result.failures.push(`${size}|${scramble}|${status}|${text}`);
    return { millis, length: text.trim() === '' ? 0 : text.trim().split(/\s+/).length };
  };
  result.firstSolve = {};
  result.warm = {};
  result.memoryAfterFirst = {};
  for (const size of [3, 2]) {
    const scrambles = golden.filter((g) => g[0] == size).map((g) => g[1]);
    result.firstSolve[size] = solve(size, scrambles[0]).millis;
    result.memoryAfterFirst[size] = memory.buffer.byteLength;
    const times = [];
    const lengths = [];
    const batchStart = performance.now();
    for (let round = 0; round < rounds; round++) {
      for (const scramble of scrambles) {
        const { millis, length } = solve(size, scramble);
        times.push(millis);
        if (round === 0) lengths.push(length);
      }
    }
    result.warm[size] = { times, batchMeanMs: (performance.now() - batchStart) / times.length, lengths };
  }
  result.memoryFinal = memory.buffer.byteLength;
  return result;
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

const browser = await chromium.launch();
const version = browser.version();
const all = [];
let failures = 0;
for (let i = 0; i < runs; i++) {
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.goto(origin + '/');
  const r = await page.evaluate(pageRun, { golden: GOLDEN, rounds: 10 });
  failures += r.failures.length;
  for (const f of r.failures) console.error('mismatch: ' + f);
  all.push(r);
  await context.close();
}
await browser.close();
server.close();

const col = (f) => all.map(f);
const out = {
  chromium: version,
  cross_origin_isolated: all.every((r) => r.isolated),
  runs,
  compile_streaming: stats(col((r) => r.compileMs)),
  instantiate: stats(col((r) => r.instantiateMs)),
  first_solve_3x3: stats(col((r) => r.firstSolve[3])),
  first_solve_2x2: stats(col((r) => r.firstSolve[2])),
  cold_3x3_compile_instantiate_first_solve: stats(col((r) => r.compileMs + r.instantiateMs + r.firstSolve[3])),
  warm_3x3_per_solve: { ...stats(all.flatMap((r) => r.warm[3].times)), batch_mean_ms: round(percentile(col((r) => r.warm[3].batchMeanMs), 50)) },
  warm_2x2_per_solve: { ...stats(all.flatMap((r) => r.warm[2].times)), batch_mean_ms: round(percentile(col((r) => r.warm[2].batchMeanMs), 50)) },
  mean_moves_3x3: round(all[0].warm[3].lengths.reduce((a, b) => a + b, 0) / all[0].warm[3].lengths.length),
  mean_moves_2x2: round(all[0].warm[2].lengths.reduce((a, b) => a + b, 0) / all[0].warm[2].lengths.length),
  memory_bytes: {
    after_instantiate: all[0].memoryAfterInstantiate,
    after_first_3x3: all[0].memoryAfterFirst[3],
    after_first_2x2: all[0].memoryAfterFirst[2],
    final: all[0].memoryFinal,
  },
  mismatches: failures,
};
console.log(JSON.stringify(out, null, 2));
process.exit(failures === 0 ? 0 : 1);
