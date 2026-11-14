// The front-goal measurement (see front-measure.mjs) in headless Chromium.
//
//   node tools/wasm/front-measure-browser.mjs <module.wasm> <playwright-core path> [--cases N|all] [--time MS]
//
// Same TSV as front-measure.mjs without the VmHWM column (the browser process is
// not this one). Each case runs in a new instance on a fresh page, served from
// 127.0.0.1 with cross-origin isolation. Exit 1 on a status other than ok or a
// length that differs from the fixture.
import fs from 'node:fs';
import http from 'node:http';
import { createRequire } from 'node:module';

const args = process.argv.slice(2);
const [wasmPath, playwrightPath] = args;
const opt = (name, dflt) => {
  const i = args.indexOf(`--${name}`);
  return i < 0 ? dflt : args[i + 1];
};
if (!wasmPath || !playwrightPath) {
  console.error('usage: node tools/wasm/front-measure-browser.mjs <module.wasm> <playwright-core path> [--cases N|all] [--time MS]');
  process.exit(2);
}
const perPrint = opt('cases', '8') === 'all' ? 20 : Number(opt('cases', '8'));
const timeMs = Number(opt('time', 120000));
const require = createRequire(import.meta.url);
const { chromium } = require(require.resolve(playwrightPath, { paths: [process.cwd()] }));

const fixture = fs
  .readFileSync(new URL('../../tests/fixtures/front_goal_calendar.tsv', import.meta.url), 'utf8')
  .split('\n')
  .filter((l) => l !== '' && !l.startsWith('#'))
  .map((l) => l.split('\t'));
const counts = new Map();
const cases = fixture.filter((c) => {
  const n = counts.get(c[0]) ?? 0;
  counts.set(c[0], n + 1);
  return n < perPrint;
});

const wasmBytes = fs.readFileSync(wasmPath);
const server = http.createServer((req, res) => {
  const h = { 'Cross-Origin-Opener-Policy': 'same-origin', 'Cross-Origin-Embedder-Policy': 'require-corp' };
  if (req.url === '/m.wasm') {
    res.writeHead(200, { ...h, 'Content-Type': 'application/wasm' });
    res.end(wasmBytes);
  } else {
    res.writeHead(200, { ...h, 'Content-Type': 'text/html' });
    res.end('<!doctype html><title>m</title>');
  }
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const origin = `http://127.0.0.1:${server.address().port}`;
const browser = await chromium.launch();
console.log('print\tstep\texpected\tgot\tstatus\tnodes\tinst_ms\twall_ms\ttables_ms\tsearch_ms\tmem_mb\tisolated');
let bad = 0;
for (const c of cases) {
  const input = `${c[5]}\t${c[6]}\t${c[7]}\t${c[4].split('|')[4] === '' ? 0 : 1}`;
  const page = await (await browser.newContext()).newPage();
  await page.goto(origin);
  const r = await page.evaluate(
    async ({ input, timeMs }) => {
      const module = await WebAssembly.compileStreaming(fetch('/m.wasm'));
      let memory;
      let t0 = performance.now();
      const x = (
        await WebAssembly.instantiate(module, {
          env: {
            cubacle_host_now: () => performance.now(),
            cubacle_host_random: (p, n) => crypto.getRandomValues(new Uint8Array(memory.buffer, p, n)),
          },
        })
      ).exports;
      memory = x.memory;
      const inst = performance.now() - t0;
      const b = new TextEncoder().encode(input);
      const p = x.fm_alloc(b.length);
      new Uint8Array(memory.buffer, p, b.length).set(b);
      t0 = performance.now();
      const res = x.fm_solve(p, b.length, timeMs, 0, 0);
      const wall = performance.now() - t0;
      const n = x.fm_result_len(res);
      const text = new TextDecoder().decode(new Uint8Array(memory.buffer, res + 4, n - 4));
      return { text, inst, wall, mem: memory.buffer.byteLength / 1048576, isolated: self.crossOriginIsolated };
    },
    { input, timeMs },
  );
  await page.context().close();
  const [status, moves, , nodes, tablesUs] = r.text.split('\t');
  const got = status === 'ok' ? moves.split(' ').filter(Boolean).length : -1;
  if (status !== 'ok' || got !== Number(c[3])) bad++;
  const tables = Number(tablesUs) / 1000;
  console.log([c[0], c[1], c[3], got, status, nodes, r.inst.toFixed(2), r.wall.toFixed(0), tables.toFixed(0), (r.wall - tables).toFixed(0), r.mem.toFixed(1), r.isolated].join('\t'));
}
await browser.close();
server.close();
process.exit(bad ? 1 : 0);
