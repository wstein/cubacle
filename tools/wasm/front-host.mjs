// Host test of the ABI 3 front-goal export of cubacle_wasm.wasm in Node.
//
//   node tools/wasm/front-host.mjs <cubacle_wasm.wasm> [--cases N|all] [--mutate NAME]
//
// Runs the 80 cases of tests/fixtures/front_goal_calendar.tsv (20 steps of each
// of four prints) in one instance, checks the length against the fixture (the
// wstein/cubistry#484 optimum), at most two rotation tokens, and replays every
// route with its own cube model (written here, not taken from the solver):
// after the moves and the rotation each front position shows an accepted
// sticker. The center-up rule is not replayed here (the native test
// crates/wasm/tests/front.rs does); it is sent as in the fixture. Also checks
// the version, limit and error answers. Prints one TSV line per case on stdout
// and a summary on stderr; exit 1 on any failure.
//
// --mutate changes the request before the call, to show that a check fails for
// its own reason: version, drop-rotation (the answer's rotation is dropped before the replay),
// truncate-accepted, flip-center-up, ignore-limits (the limits of the limit checks are sent as 0, the defaults).
import fs from 'node:fs';
import { performance } from 'node:perf_hooks';

const args = process.argv.slice(2);
const path = args[0];
const opt = (name, dflt) => {
  const i = args.indexOf(`--${name}`);
  return i < 0 ? dflt : args[i + 1];
};
if (!path) {
  console.error('usage: node tools/wasm/front-host.mjs <cubacle_wasm.wasm> [--cases N|all] [--mutate NAME]');
  process.exit(2);
}
const perPrint = opt('cases', 'all') === 'all' ? 20 : Number(opt('cases', 'all'));
const mutate = opt('mutate', '');

const fixture = fs
  .readFileSync(new URL('../../tests/fixtures/front_goal_calendar.tsv', import.meta.url), 'utf8')
  .split('\n')
  .filter((l) => l !== '' && !l.startsWith('#'))
  .map((l) => l.split('\t'));
const cases = [];
const counts = new Map();
for (const c of fixture) {
  const n = counts.get(c[0]) ?? 0;
  if (n >= perPrint) continue;
  counts.set(c[0], n + 1);
  cases.push({
    print: c[0], step: c[1], expected: Number(c[3]),
    centerUp: c[4].split('|')[4] !== '', facelets: c[5], twists: c[6], accepted: c[7],
  });
}

// ---- cube model: facelets 0..53 in U R F D L B order, 9 each ----
const FACE = 'URFDLB';
// Position and normal of facelet i (x right, y up, z toward the front).
function geometry(i) {
  const face = Math.floor(i / 9), r = Math.floor((i % 9) / 3), c = i % 3;
  switch (face) {
    case 0: return { p: [c - 1, 1, r - 1], n: [0, 1, 0] };
    case 1: return { p: [1, 1 - r, 1 - c], n: [1, 0, 0] };
    case 2: return { p: [c - 1, 1 - r, 1], n: [0, 0, 1] };
    case 3: return { p: [c - 1, -1, 1 - r], n: [0, -1, 0] };
    case 4: return { p: [-1, 1 - r, c - 1], n: [-1, 0, 0] };
    default: return { p: [1 - c, 1 - r, -1], n: [0, 0, -1] };
  }
}
const geo = Array.from({ length: 54 }, (_, i) => geometry(i));
const key = (p, n) => p.join(',') + '|' + n.join(',');
const index = new Map(geo.map((g, i) => [key(g.p, g.n), i]));
// Rotation by -90 degrees (clockwise seen from the tip) about the axis `a` (a unit vector).
function turn(v, a) {
  // v' = a x v ... for a clockwise quarter turn about a: v' = (a . v) a - (a x v)
  const d = a[0] * v[0] + a[1] * v[1] + a[2] * v[2];
  const x = [a[1] * v[2] - a[2] * v[1], a[2] * v[0] - a[0] * v[2], a[0] * v[1] - a[1] * v[0]];
  return [0, 1, 2].map((k) => d * a[k] - x[k] + 0);
}
const AXIS = { U: [0, 1, 0], R: [1, 0, 0], F: [0, 0, 1], D: [0, -1, 0], L: [-1, 0, 0], B: [0, 0, -1] };
// perm[i] = where the sticker at i goes for one clockwise quarter turn; layer = (g) => in the layer
function permFor(axis, inLayer) {
  const perm = Array.from({ length: 54 }, (_, i) => i);
  geo.forEach((g, i) => {
    if (!inLayer(g.p)) return;
    perm[i] = index.get(key(turn(g.p, axis).map(Math.round), turn(g.n, axis).map(Math.round)));
  });
  return perm;
}
const dot = (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const FACE_PERM = {};
for (const f of FACE) FACE_PERM[f] = permFor(AXIS[f], (p) => dot(p, AXIS[f]) === 1);
// Whole-cube rotations follow R (x), U (y), F (z).
const ROT_PERM = { x: permFor(AXIS.R, () => true), y: permFor(AXIS.U, () => true), z: permFor(AXIS.F, () => true) };

function applyPerm(state, perm, times) {
  let s = state;
  for (let t = 0; t < times; t++) {
    const next = s.slice();
    for (let i = 0; i < 54; i++) next[perm[i]] = s[i];
    s = next;
  }
  return s;
}
function applyWord(state, word) {
  let s = state;
  for (const tok of word.split(/\s+/).filter(Boolean)) {
    const base = tok[0];
    const times = tok.endsWith("'") ? 3 : tok.endsWith('2') ? 2 : 1;
    const perm = FACE_PERM[base] ?? ROT_PERM[base];
    if (!perm) throw new Error(`unknown token ${tok}`);
    s = applyPerm(s, perm, times);
  }
  return s;
}

const CORNERS = [[8, 9, 20], [6, 18, 38], [0, 36, 47], [2, 45, 11], [29, 26, 15], [27, 44, 24], [33, 53, 42], [35, 17, 51]];
const EDGES = [[5, 10], [7, 19], [3, 37], [1, 46], [32, 16], [28, 25], [30, 43], [34, 52], [23, 12], [21, 41], [50, 39], [48, 14]];
// The home facelet (sticker identity) shown at `at`, read from the colors.
function identityAt(colors, at) {
  if (at % 9 === 4) return colors[at] * 9 + 4;
  const shown = colors[at];
  for (const slot of CORNERS) {
    if (!slot.includes(at)) continue;
    const seen = slot.map((f) => colors[f]).sort();
    const piece = CORNERS.find((p) => p.map((f) => Math.floor(f / 9)).sort().join() === seen.join());
    return piece.find((f) => Math.floor(f / 9) === shown);
  }
  for (const slot of EDGES) {
    if (!slot.includes(at)) continue;
    const seen = [colors[slot[0]], colors[slot[1]]];
    const piece = EDGES.find((p) => (Math.floor(p[0] / 9) === seen[0] && Math.floor(p[1] / 9) === seen[1])
      || (Math.floor(p[1] / 9) === seen[0] && Math.floor(p[0] / 9) === seen[1]));
    return piece.find((f) => Math.floor(f / 9) === shown);
  }
  throw new Error('facelet on no piece');
}
// null when the route meets the accepted lists, else the reason.
function replay(c, moves, rotation) {
  let s = [...c.facelets].map((ch) => FACE.indexOf(ch));
  s = applyWord(s, moves);
  s = applyWord(s, rotation);
  const lists = c.accepted.split(' ').map((g) => g.split(',').map(Number));
  for (let i = 0; i < 9; i++) {
    const id = identityAt(s, 18 + i);
    if (!lists[i].includes(id)) return `position ${i} shows ${id}, accepted ${lists[i]}`;
  }
  return null;
}

// ---- module ----
const bytes = fs.readFileSync(path);
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
function frontSolve(ex, text) {
  const input = enc.encode(text);
  const p = ex.cubacle_alloc(input.length);
  new Uint8Array(ex.memory.buffer, p, input.length).set(input);
  const r = ex.cubacle_front_solve(p, input.length);
  ex.cubacle_free(p, input.length);
  const total = ex.cubacle_result_len(r);
  const out = new Uint8Array(ex.memory.buffer, r, total).slice();
  ex.cubacle_free(r, total);
  return { status: out[4], text: dec.decode(out.subarray(5)) };
}
function request(c, limits = [0, 0, 0, 0], version = '1') {
  if (mutate === 'ignore-limits') limits = [0, 0, 0, 0];
  let accepted = c.accepted;
  let up = c.centerUp ? 1 : 0;
  if (mutate === 'version') version = '2';
  if (mutate === 'truncate-accepted') accepted = accepted.split(' ').slice(0, 8).join(' ');
  if (mutate === 'flip-center-up') up = 1 - up;
  return [version, c.facelets, c.twists, accepted, up, ...limits].join('\t');
}

let failures = 0;
const fail = (msg) => { failures++; console.error(`FAIL ${msg}`); };
const ex = instantiate();
if (ex.cubacle_abi_version() !== 3) fail(`abi version ${ex.cubacle_abi_version()}, expected 3`);

console.log('print\tstep\texpected_htm\tgot_htm\trotation\tnodes\ttables_ms\twall_ms');
let rotations = 0;
for (const c of cases) {
  const t0 = performance.now();
  const r = frontSolve(ex, request(c));
  const wall = performance.now() - t0;
  const label = `${c.print} step ${c.step}`;
  if (r.status !== 0) { fail(`${label}: status ${r.status} ${r.text}`); continue; }
  const f = r.text.split('\t');
  if (f.length !== 5 || f[0] !== '1') { fail(`${label}: answer shape ${r.text}`); continue; }
  const [, moves, rotationText, nodes, tablesMs] = f;
  const rotation = mutate === 'drop-rotation' ? '' : rotationText;
  const got = moves.split(/\s+/).filter(Boolean).length;
  const rot = rotation.split(/\s+/).filter(Boolean).length;
  if (got !== c.expected) fail(`${label}: ${got} moves, expected ${c.expected}`);
  if (rot > 2) fail(`${label}: rotation has ${rot} tokens`);
  if (rot > 0) rotations++;
  const why = replay(c, moves, rotation);
  if (why) fail(`${label}: replay: ${why}`);
  console.log([c.print, c.step, c.expected, got, rotation || '-', nodes, tablesMs, wall.toFixed(1)].join('\t'));
}

// ---- error and limit answers, on the first case ----
const c0 = cases[0];
const limited = frontSolve(ex, request(c0, [0, 0, 10, 0]));
if (limited.status !== 3 || limited.text !== '1\tnodes=11') fail(`node cap: ${limited.status} ${limited.text}`);
const tabled = frontSolve(ex, request(c0, [1, 0, 0, 0]));
if (tabled.status !== 4 || !tabled.text.startsWith('1\tpermille=')) fail(`table budget: ${tabled.status} ${tabled.text}`);
const bounded = frontSolve(ex, request(c0, [0, 0, 0, 1]));
if (bounded.status !== 2) fail(`move limit: ${bounded.status} ${bounded.text}`);
const wrong = frontSolve(ex, request(c0, [0, 0, 0, 0], '2'));
if (wrong.status !== 1 || !wrong.text.includes('version')) fail(`version: ${wrong.status} ${wrong.text}`);
const short = frontSolve(ex, request(c0).split('\t').slice(0, 8).join('\t'));
if (short.status !== 1) fail(`field count: ${short.status} ${short.text}`);
// Determinism: a second instance gives the same moves, rotation and nodes.
const again = frontSolve(instantiate(), request(c0, [0, 0, 0, 0]));
const first = frontSolve(ex, request(c0));
const cut = (t) => t.split('\t').slice(0, 4).join('\t');
if (again.status !== first.status || cut(again.text) !== cut(first.text)) fail('answers differ between instances');

console.error(`front-host: ${cases.length} cases, ${rotations} with a rotation, ${failures} failures, module ${bytes.length} bytes`);
process.exit(failures === 0 ? 0 : 1);
