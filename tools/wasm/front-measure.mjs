// Measurement of the front-goal solver (crates/wasm-front-measure) in Node.
//
//   node tools/wasm/front-measure.mjs <module.wasm> [--cases N|all] [--time MS] [--nodes N] [--print P] [--reuse]
//
// One TSV line per case on stdout: print, step, expected_htm, got_htm, status,
// nodes, instantiate_ms, wall_ms (whole call), tables_ms (reported by the
// solver), search_ms (wall - tables), memory_mb (WebAssembly.Memory after the
// call), rss_hwm_mb (VmHWM of this process, Linux only). Each case runs in a
// new instance (cold), except with --reuse, where one instance serves all
// cases (shows whether memory grows between calls). A summary goes to stderr.
// Exit 1 when a status is not ok or a length differs from the fixture.
import fs from 'node:fs';
import zlib from 'node:zlib';
import { performance } from 'node:perf_hooks';

const args = process.argv.slice(2);
const path = args[0];
const opt = (name, dflt) => {
  const i = args.indexOf(`--${name}`);
  return i < 0 ? dflt : args[i + 1];
};
if (!path) {
  console.error('usage: node tools/wasm/front-measure.mjs <module.wasm> [--cases N|all] [--time MS] [--nodes N] [--print P] [--reuse]');
  process.exit(2);
}
const perPrint = opt('cases', '8') === 'all' ? 20 : Number(opt('cases', '8'));
const timeMs = Number(opt('time', 120000));
const maxNodes = Number(opt('nodes', 0));
const only = opt('print', null);
const reuse = args.includes('--reuse');

const bytes = fs.readFileSync(path);
const raw = bytes.length;
const gz = zlib.gzipSync(bytes, { level: 9 }).length;
const br = zlib.brotliCompressSync(bytes, {
  params: { [zlib.constants.BROTLI_PARAM_QUALITY]: 11, [zlib.constants.BROTLI_PARAM_SIZE_HINT]: raw },
}).length;
console.error(`module bytes raw=${raw} gzip9=${gz} brotli11=${br}`);

const fixture = fs
  .readFileSync(new URL('../../tests/fixtures/front_goal_calendar.tsv', import.meta.url), 'utf8')
  .split('\n')
  .filter((l) => l !== '' && !l.startsWith('#'))
  .map((l) => l.split('\t'));
const byPrint = new Map();
for (const c of fixture) {
  if (only && c[0] !== only) continue;
  if (!byPrint.has(c[0])) byPrint.set(c[0], []);
  byPrint.get(c[0]).push(c);
}

const module = new WebAssembly.Module(bytes);
const enc = new TextEncoder();
const dec = new TextDecoder();

function instantiate() {
  let memory;
  const instance = new WebAssembly.Instance(module, {
    env: {
      cubacle_host_now: () => performance.now(),
      cubacle_host_random: (p, n) => {
        const v = new Uint8Array(memory.buffer, p, n);
        for (let i = 0; i < n; i++) v[i] = (Math.random() * 256) | 0;
      },
    },
  });
  memory = instance.exports.memory;
  return instance.exports;
}

function call(x, input) {
  const b = enc.encode(input);
  const p = x.fm_alloc(b.length);
  new Uint8Array(x.memory.buffer, p, b.length).set(b);
  const r = x.fm_solve(p, b.length, timeMs, maxNodes, 0);
  const n = x.fm_result_len(r);
  const text = dec.decode(new Uint8Array(x.memory.buffer, r + 4, n - 4));
  x.fm_free(r, n);
  x.fm_free(p, b.length);
  return text.split('\t');
}

function hwmMb() {
  try {
    const m = /VmHWM:\s+(\d+) kB/.exec(fs.readFileSync('/proc/self/status', 'utf8'));
    return m ? (Number(m[1]) / 1024).toFixed(1) : 'NA';
  } catch {
    return 'NA';
  }
}

console.log('print\tstep\texpected\tgot\tstatus\tnodes\tinst_ms\twall_ms\ttables_ms\tsearch_ms\tmem_mb\trss_hwm_mb');
let shared = reuse ? instantiate() : null;
let bad = 0;
const rows = [];
for (const [print, list] of byPrint) {
  for (const c of list.slice(0, perPrint)) {
    const input = `${c[5]}\t${c[6]}\t${c[7]}\t${c[4].split('|')[4] === '' ? 0 : 1}`;
    let t0 = performance.now();
    const x = shared ?? instantiate();
    const inst = performance.now() - t0;
    t0 = performance.now();
    const [status, moves, rotation, nodes, tablesUs] = call(x, input);
    const wall = performance.now() - t0;
    const got = status === 'ok' ? moves.split(' ').filter(Boolean).length : -1;
    if (status !== 'ok' || got !== Number(c[3])) bad++;
    const tables = Number(tablesUs) / 1000;
    const mem = x.memory.buffer.byteLength / 1048576;
    const row = [print, c[1], c[3], got, status, nodes, inst.toFixed(2), wall.toFixed(0), tables.toFixed(0), (wall - tables).toFixed(0), mem.toFixed(1), hwmMb()];
    rows.push(row);
    console.log(row.join('\t'));
  }
}
const med = (a) => [...a].sort((p, q) => p - q)[Math.floor(a.length / 2)];
const col = (i) => rows.map((r) => Number(r[i]));
console.error(
  `cases=${rows.length} bad=${bad} tables_ms median=${med(col(8))} max=${Math.max(...col(8))} search_ms median=${med(col(9))} max=${Math.max(...col(9))} mem_mb max=${Math.max(...col(10))}`,
);
process.exit(bad ? 1 : 0);
