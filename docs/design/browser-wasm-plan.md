---
title: "Browser WebAssembly plan (C1 to C5)"
description: "Plan: state, target architecture, slices and open owner decisions for issues #21 to #25 (browser facade, 3x3x3 target modes, pattern algebra, table loading, reproducible build)."
weight: 50
---

# Browser WebAssembly plan (C1 to C5)

Status: **proposal**. This page is the plan for issues
[#21](https://github.com/wstein/cubacle/issues/21) (C1),
[#22](https://github.com/wstein/cubacle/issues/22) (C2),
[#23](https://github.com/wstein/cubacle/issues/23) (C3),
[#24](https://github.com/wstein/cubacle/issues/24) (C4) and
[#25](https://github.com/wstein/cubacle/issues/25) (C5). It approves nothing. Every slice that
section 6 lists under an open decision waits for that decision. The issues stay the place for
status; this page holds the approach.

Markers (the legend of the Cubistry plans): **O** owner decision with date and source, **V**
verified in a file or on GitHub at the revision named, **P** proposed here, **U** UNKNOWN (not
measured, not found or not run). Base of all file references: `develop` at `de12c68`.

## 1. Sources and work in flight

- **V** Issues #21 to #25: one-line bodies. #21 acceptance: "builds without native-only crates;
  benchmark committed". #22: "Replay-exact target solving using Calendar fixtures". #23: "Cover
  the algebra used by cubistry Calendar planner; property tests". #24: "Deterministic bytes,
  pinned SHA-256". #25: "Stable hash on two clean machines".
- **O** Owner comments of 2026-11-03 on #21 and #22: a planning gate, not an implementation
  approval. C2 needs its own priority, license and interface approval before implementation; no
  new Rust engine.
- **O** Gate G1 "conditionally passed", owner, 2026-11-06 (Cubistry
  `knowledge/plans/cubacle-wasm-replacement.md`, section "G1 result"): steps 2 and 3 of that plan
  go ahead (shadow parity, 2x2x2 adapter). It is not a decision on steps 4 to 6; C2, C3 and C4 are
  step 4. Open there: whether the 2x2x2 cold start (about 93 ms in Node) and 21.4 MB of linear
  memory are acceptable.
- **O** Cubistry Picture Cube host plan (`knowledge/plans/unified-application.md`, D3,
  2026-11-11): engine option E0, the existing engines stay; slice S5 delivers the 2x2x2 and 3x3x3
  Solve with the current providers. Cubacle is not a dependency of that stack.
- **V** [PuzlWerk #23](https://github.com/wstein/puzlwerk/issues/23) (P1, low priority): whether
  `@puzlwerk/solver-2x2` moves to the cubacle 2x2x2 engine is to be evaluated after G1/G2. Not part
  of this plan.
- **V** Merged: #33 (facade and G1 measurement), #34 (G1 gaps: bundle comparison, browser run,
  move counts), #32 (`CenterTwists`, `src/puzzle/twists.rs`), #36 and #37 (picture renderer).
- **V** Work in flight, 2026-11-12: no open pull request (`gh pr list -R wstein/cubacle --state
  all`), no remote branch other than `develop` and `main` (`git ls-remote origin`), no fork
  (`gh api repos/wstein/cubacle/forks`). Work an author started locally is not visible here; it is
  pushed as a Draft pull request on the branch name of its slice (section 5) before the next slice
  starts, and this plan is aligned with it in review.

## 2. Current state per issue

| Issue | State | Evidence |
| --- | --- | --- |
| C1 #21 facade and benchmark | **partial** | **V** done: `crates/wasm/src/lib.rs` (C ABI `cubacle_alloc`, `cubacle_solve`, `cubacle_result_len`, `cubacle_free`; status 0, 1, 2), `crates/wasm/tests/{facade,golden}.rs`, `tools/wasm/{bench,browser,bundle-size,cubing-moves}.mjs`, numbers and commands in [Browser WebAssembly build](../wasm.md). **V** open: the `cubacle` crate without `fetch` still pulls `clap`, `sysinfo`, `toml`, `rand`, `serde`, `serde_json` and `sha2` into the wasm graph (`cargo tree -p cubacle --no-default-features --target wasm32-unknown-unknown -e normal --depth 1`); `sysinfo` and `clap` are native or CLI crates. **U** worker start, warm code cache, Firefox, Safari, resident memory (listed as unmeasured in `docs/wasm.md`). G1 decision: **O** conditional (above); closing #21 is the owner's. |
| C2 #22 exact-frame and ignore-centers 3x3x3 targets | **open** | **V** the facade solves a scramble to the solved cube only. **V** related code: 3x3x3 target sets with sticker identity, `SolveFrame::{Fixed, AnyOrientation}` (`src/engines/three/partial/`, `src/puzzle/proof.rs`); center twists are "slice 2" of the target sets, not built ([Don't-care targets](dont-care-targets.md), end of section 11). **V** `CenterTwists` and the Orbit64 V2 picture model (`rotations` 0..=3, `src/notation/orbit64/v2/cube.rs`) carry center orientation. **U** which Calendar fixtures (file, Cubistry commit) are the acceptance set. |
| C3 #23 pattern/transformation API | **open** | **V** no permutation-orientation algebra is exported. **V** closest existing model: `notation::orbit64::v2::Cube` (corner pieces and twists, midge pieces and flips, center rotations, frame). **V** the Cubistry Calendar uses from Cubing.js: `KPuzzle.defaultPattern`, `KPattern` construction from `{CORNERS, EDGES, CENTERS}` with `pieces` and `orientation`, `applyMove`, `applyAlg`, `applyTransformation`, `isIdentical`, `patternData`, `algToTransformation`, transformation `invert` and `toKPattern` (`src/apps/calendar/transition/planner.mjs:30-93`, `src/adapters/cubing-2x2.ts:47-147` at Cubistry `e7ab2830`). |
| C4 #24 table loading | **open** | **V** tables are built in linear memory on the first solve of each size (`docs/wasm.md`, Interface); no load, cancel or pin layer. **V** 2x2x2 tables are a process-wide `OnceLock` (`src/engines/two/mod.rs:268`), 3x3x3 tables an `Arc<OnceLock<Tables>>` per solver (`src/engines/three/tables.rs:599`). **V** no serialized table format for these two engines (the fixed-frame 2x2x2 table has a SHA-256 of its distances, `src/engines/two/fixed/table.rs:131`, a different table). |
| C5 #25 reproducible build and tarball | **open** | **V** build command in `docs/wasm.md`; no toolchain pin file, no manifest, no tarball, no hash. **V** `release.yml` packs native binaries per target on tag push, not the wasm module. |

## 3. Target architecture (P unless marked)

### 3.1 Boundaries

- **Root crate `cubacle`.** New module `src/kpuzzle/` (P name) with the pattern and
  transformation algebra for the 2x2x2 and 3x3x3. It depends on `puzzle` only (rule of
  `ARCHITECTURE.md`); the bridge to `Facelets`, `CenterTwists` and Orbit64 V2 sits in
  `notation`, which may name `puzzle`. Table serialization (C4) is a crate-private function per
  engine plus one `pub` entry each in `engines::two` and `engines::three`; the search code is not
  changed (AGENTS.md, Code).
- **Feature split (D8).** `clap`, `sysinfo` and `toml` become optional behind a default feature
  (P name `cli`); `cubacle-wasm` depends on `cubacle` with `default-features = false`. The
  server keeps what it needs.
- **`crates/wasm`.** Rust facade: the existing C ABI stays (ABI 1) and gains ABI 2 (section 3.6).
  `lib.rs` is split into `abi.rs`, `solve.rs`, `host.rs` before new exports land (slice A1), so
  later slices add files instead of editing one file.
- **`crates/wasm/js/`.** ESM glue without npm dependencies and without a bundler: `index.mjs`
  (client in the page or worker), `worker.mjs` (the module in a dedicated worker), `index.d.ts`.
  Tests with `node --test` (Node built-ins only).
- **`xtask`.** `cargo xtask wasm` builds the module, writes the manifest and the tarball
  (section 3.5). No Python (repo rule for new tools; `release.yml` keeps its existing `python3`
  line).
- **Non-dependencies.** Cubacle does not depend on Cubing.js, PuzlWerk or the viewer. Cubing.js
  stays a test-only oracle on the Cubistry side (Cubistry plan, "No runtime fallback").

### 3.2 Pattern and transformation API (C3)

Data shape modeled on the KPuzzle data of Cubing.js, so the Calendar planner can switch with few
edits; names in camelCase:

| Item | Shape |
| --- | --- |
| Definition | `{name, orbits: [{orbitName, numPieces, numOrientations}], defaultPattern, moves: {name: TransformationData}}` for `2x2x2` (`CORNERS`) and `3x3x3` (`CORNERS`, `EDGES`, `CENTERS` with 4 orientations) |
| TransformationData | per orbit `{permutation: [..], orientationDelta: [..]}` |
| PatternData | per orbit `{pieces: [..], orientation: [..]}` |

Operations (Rust and JS): `identity`, `compose(a, b)` (apply `a`, then `b`; the Cubing.js
`a.applyTransformation(b)`), `invert`, `move(name)`, `alg(text)` (parsed by `notation`; outer
face turns, rotations and slices as the 3x3x3 notation accepts them), `toPattern(t)` (default
pattern after `t`), `applyTransformation(pattern, t)`, `isIdentical(p, q, ignore)` with `ignore`
a set of orbits or `centerOrientation`, conversion to and from `Facelets` plus `CenterTwists`.
Values are immutable; every input is validated (lengths, permutation, orientation range, parity
only where the definition demands it) and refused with `bad_input`.

Index convention (D3): either the numbering of the Cubing.js `3x3x3` and `2x2x2` definitions or a
cubacle numbering with a documented mapping. Properties the tests state, independent of D3:
group laws (identity, inverse, associativity on random words), move orders (`R` 4, `R U` 105,
`R U R' U'` 6), `alg(w)` equals the composition of its moves, `toPattern(alg(w))` equals
`Facelets::solved(kind).apply(w)` through the bridge (differential test against the existing
model), center orientation of `toPattern` equals the `CenterTwists` of the Orbit64 V2 picture
model for the same word.

### 3.3 Target modes (C2, 3x3x3 only)

A solve request is `{start: PatternData, target: PatternData, mode, maxMoves, budgetMs}`; the
answer is a move sequence `w` in outer face turns (HTM), no whole-cube rotation.

| Mode | Met when `applyTransformation(start, alg(w))` ... | Route (D9) | Claim |
| --- | --- | --- | --- |
| `ignore-centers` | equals `target` on `CORNERS` and `EDGES` and on center positions; center orientation free | relative state `start^-1 * target`, solved by `Min2PhaseSolver` | best found, not shortest |
| `exact-frame` | equals `target` on all three orbits, center orientation included | `ignore-centers`, then a center fix-up for the remaining center twists | best found, not shortest |

Centers of a 3x3x3 never move, so a target whose center pieces differ from the start is refused
(`unsupported`), as `src/engines/three/partial/mod.rs:94` does today. The proven-shortest route
(target sets with center twists as a view condition) stays out of the browser module: its tables
are 44 MB and more and their build takes about 2 s natively (Don't-care targets, results block),
a size the G1 numbers do not cover. Acceptance: every answer on the agreed Calendar fixtures
replays exactly to the target in the cubacle model (native test) and in the JS glue (Node test).
Parity with Cubing.js (gate G2, no more moves) is measured in Cubistry, not claimed here.

### 3.4 Worker and table loading (C4)

- **Lazy.** Nothing is fetched or built before the first call of a size. State per size:
  `idle`, `loading`, `ready`, `failed`; `failed` allows a retry (the pattern of the Cubistry 2x2x2
  table loader, `src/apps/playground/solver.ts` at `e7ab2830`).
- **Hash-pinned module.** The glue fetches the `.wasm` bytes, refuses a body larger than the
  pinned size, checks length and SHA-256 (`crypto.subtle.digest`) against the manifest, then
  compiles with `WebAssembly.compile`. `compileStreaming` is not used, because it compiles before
  the hash is known; the cost is the lost streaming overlap (**U**, 1.5 to 1.7 ms compile in
  `docs/wasm.md` is the upper bound of what can be lost on that machine).
- **Tables (D4).** Recommended: tables stay built inside the module (no download of 20 MB);
  "deterministic bytes" is met by a serialization of the built tables whose SHA-256 is pinned in
  a native test and printed by the Node bench, so a different build or architecture that builds
  different tables fails. Alternative: prebuilt table files loaded with
  `cubacle_tables_load(kind, ptr, len)`, header `{magic, formatVersion, engine, tableId, length}`
  plus pinned SHA-256; it needs a size and time measurement first.
- **Cancel (D11).** A wasm call cannot be interrupted from outside. Before a call starts, an
  `AbortSignal` rejects with `cancelled`. During a call, cancel terminates the worker; the client
  starts a new worker on the next call and the tables are built again (the price). Every call
  also carries `budgetMs`, passed to `SolveOptions::with_time_budget` (the module reads the clock
  through `cubacle_host_now`). A cooperative stop flag over `SharedArrayBuffer` needs
  cross-origin isolation, which GitHub Pages cannot set (**U** for other hosts); not planned.
- **Protocol.** Request `{id, op, args}`, answer `{id, ok: true, value}` or
  `{id, ok: false, error: {code, message}}`; one call at a time per worker, later calls queue.

### 3.5 Build, manifest and tarball (C5)

- **Command.** `cargo xtask wasm` (P): refuses a dirty tree; runs
  `cargo build --release --locked -p cubacle-wasm --target wasm32-unknown-unknown` with
  `--remap-path-prefix` for the repository and the Cargo home. The flags go through
  `.cargo/config.toml` or `CARGO_ENCODED_RUSTFLAGS` together with the existing
  `getrandom_backend` flag: a plain `RUSTFLAGS` replaces the target `rustflags` of
  `.cargo/config.toml` and would drop that flag.
- **Toolchain (D5).** The manifest records `rustc -Vv`; the xtask refuses a toolchain other
  than the one named in the xtask source. No `wasm-opt` (no extra tool).
- **Manifest** `cubacle-wasm.manifest.json`: `{format: "cubacle.wasm-manifest", formatVersion: 1,
  version, commit, rustc, files: {name: {bytes, sha256}}, tableSha256: {...}}`. Keys sorted,
  no time stamps.
- **Tarball** `cubacle-wasm-<version>.tgz`, npm layout (`package/` prefix): `package.json`
  (`"license": "AGPL-3.0-or-later"`, `"type": "module"`, `exports`, no scripts),
  `cubacle_wasm.wasm`, `index.mjs`, `worker.mjs`, `index.d.ts`, the manifest, `LICENSE`,
  `LICENSE-MIT`, `NOTICE.md`, a short `README.md`. Entries sorted by name, ustar, mtime from the
  commit time, uid and gid 0, modes 0644 and 0755, gzip without name and time. Writer: D6.
- **Pin (D2).** The consumer pins repository, commit, package, version and `sha512` integrity,
  like `puzlwerk-source.json` in Cubistry. Nothing is published to npm.
- **Reproducibility.** Claimed only when two machines (different hosts, same toolchain) produce
  the same SHA-256 for the `.wasm` and the `.tgz` from the same commit (**U** today).

### 3.6 Errors and versions

- Codes, one list for the C ABI and JS: `ok` 0, `bad_input` 1, `no_solution` 2 (the existing three
  bytes keep their values), `unsupported` 3, `limit` 4 (time or moves), `table_mismatch` 5
  (hash, length or format), `internal` 6; JS only: `cancelled`, `load_failed` (network, size,
  hash of the module). A trap poisons the instance; the client drops the worker.
- `cubacle_abi_version() -> u32` (1 today, 2 with the JSON entry `cubacle_call(ptr, len)` that
  takes `{op, args}` and returns the result buffer of ABI 1 with JSON text). Every JSON message
  carries `formatVersion`; unknown versions are refused with `unsupported`.
- Package version 0.x; any change of the JSON shape or the codes raises the `formatVersion` of
  the message, and the manifest names the ABI version.

### 3.7 How a consumer calls it

A Cubistry or PuzlWerk worker imports `index.mjs` from the pinned tarball and calls, in this
order: `createCubacle({wasmUrl, manifest})` (nothing loaded yet), `kpuzzle("3x3x3")`
(definition, loads the module on first use), pattern operations, and
`solvePattern({start, target, mode, maxMoves, budgetMs, signal})`, which returns
`{moves, length}` or rejects with a `CubacleError {code, message}`. `dispose()` terminates the
worker. The Calendar planner keeps its own logic and replaces only the Cubing.js handles
`kpuzzle`, `KPattern` and `solve` (`createTransitionOptimizer` arguments). That change is a
Cubistry slice (B5, B6) after G2, not part of this plan.

### 3.8 Non-goals

4x4x4 and larger in the browser; a proven-shortest 3x3x3 browser route; a runtime fallback to
Cubing.js; npm publication; a viewer or shared cube core in cubacle (AGENTS.md, PuzlWerk
boundary); REST changes; threads in the module; a change to search code or recorded fixtures;
the PuzlWerk 2x2x2 consolidation (#23 there).

## 4. Dependencies

```
A1 split facade --> A2 feature trim (C1) --> A3 build+manifest (C5) --> A4 tarball (C5)
                                                   |
                                                   v
B1 algebra core (C3) --> B2 notation bridge --> B3 wasm+JS algebra --> L1 loader+worker (C4)
                                                                            |
                                     M1 target modes (C2) --> M2 C2 export --+--> T1 table hashes (C4)
```

C4 needs the manifest (A3) for its pins and an ABI 2 export (B3) to call. C2 needs the algebra
(B1, B2). Stack limit: at most 4 open pull requests per stack and 3 open units per repository,
so the work runs as four stacks one after another or two in parallel: stack A, stack B (B1 to B3 and L1), stack M (M1, M2, T1).

## 5. Slices

Branch names follow the stack numbering (`feat/<stack>-<n>`); each pull request is based on the branch of the slice before it in its stack. Rules for every slice: one topic, at most about 10 files, branch from the named parent, test
first (the named test fails on the parent, then passes), Conventional Commits, Draft pull
request with a named blocker, independent review of the exact head. Local checks per AGENTS.md
on the final tree: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace --release`, `typos --locale en-us`, `git diff --check`. No single
invocation over 5 minutes: a longer one is stopped and reported "not run, CI"; targeted runs
(`cargo test --release -p cubacle-wasm`, `cargo test --release --lib kpuzzle`) come first. The
wasm target is added once with `rustup target add wasm32-unknown-unknown`.

| Slice | Branch | Issue | Content | Fails before | Done when |
| --- | --- | --- | --- | --- | --- |
| A1 | `feat/wasm-build-1` | C1 | Move `crates/wasm/src/lib.rs` into `abi.rs`, `solve.rs`, `host.rs` (`git mv` first); add `cubacle_abi_version` returning 1 | `crates/wasm/tests/facade.rs`: new test calls `cubacle_abi_version` (does not compile) | existing facade and golden tests unchanged and green; bench output equal to golden |
| A2 | `feat/wasm-build-2` | C1 (D8) | Optional `cli` feature for `clap`, `sysinfo`, `toml`; `cargo xtask wasm-deps` fails when the normal wasm graph holds `clap`, `sysinfo`, `toml` or `ureq` | `cargo xtask wasm-deps` reports `clap` | the check passes; CLI and server tests unchanged; `.wasm` size before and after recorded with command |
| A3 | `feat/wasm-build-3` | C5 (D5) | `cargo xtask wasm`: clean-tree check, pinned flags, manifest | xtask unit tests: manifest key order, dirty tree refused, toolchain mismatch refused | two builds of one commit on one machine give the same SHA-256; manifest committed as an example in docs |
| A4 | `feat/wasm-build-4` | C5 (D2, D6, D7) | Deterministic tarball writer and package layout | xtask unit tests: entry order, mtime, owner, mode, gzip header without time | two packs of one commit give the same SHA-512; `tar -tvzf` listing in the PR |
| B1 | `feat/kpuzzle-1` | C3 (D3) | `src/kpuzzle/`: definitions, transformation, pattern, compose, invert, validation | property tests in `src/kpuzzle/tests.rs` | group laws and move orders hold; `tests/architecture_test.rs` green |
| B2 | `feat/kpuzzle-2` | C3 | `alg(text)`, bridge to `Facelets` and `CenterTwists` in `notation` | differential test against `Facelets::apply` on fixed random words | bridge round trip and center twists equal the Orbit64 V2 picture model |
| B3 | `feat/kpuzzle-3` | C3 | ABI 2 `cubacle_call`, JSON algebra ops, `index.d.ts` types | `crates/wasm/tests/` JSON cases; `node --test` on the built module | Node test replays the Calendar planner operations listed in section 2 |
| L1 | `feat/kpuzzle-4` | C4 (D11) | `crates/wasm/js/`: client, worker, hash-pinned load, abort, budget, retry | `node --test`: wrong hash, oversize body, abort before call, terminate during call, retry after failure | all pass; the bench runs through the glue |
| M1 | `feat/target-modes-1` | C2 (D1, D9) | `ignore-centers` and `exact-frame` on patterns, native | replay tests on the agreed Calendar fixtures (provenance: Cubistry commit, path, SHA-256) | every fixture replays exactly; refused cases named |
| M2 | `feat/target-modes-2` | C2 | `solvePattern` in ABI 2 and the glue | Node replay test on the same fixtures | the same answers as native (byte-equal move text) |
| T1 | `feat/target-modes-3` | C4 (D4) | Serialization of the built 2x2x2 and 3x3x3 tables, SHA-256 pinned in a native test, printed by the bench | the pinned-hash test (fails with a wrong constant; mutation: change one table entry) | native and wasm print the same hashes |

Stacking points and restack risks:

- Stack A and stack B both touch `crates/wasm/` and root `Cargo.toml`. Start B after A1 is merged
  or base B3 on A's top; B1 and B2 touch only `src/`.
- A2 changes root features used by `crates/server`; a later change of `develop` there forces a
  restack of A2 to A4 (new review of the changed head; a pure rebase carries over by patch-id).
- M1 waits for decisions D1, D9 and the fixture list (D12); its fixtures come from Cubistry and need a commit pin, so a
  Cubistry change of those files does not move the test.
- A workflow change (D10: CI job for the wasm build and `node --test`) is its own slice after A3;
  until then CI does not run the wasm or Node tests, and the PR text says so.

## 6. Open owner decisions

Each with recommendation (R) and price. None is taken by this plan.

1. **D1 Approval.** Are C2, C3 and C4 approved for implementation (the comments of 2026-11-03
   ask for a separate approval; G1 covered steps 2 and 3 only)? R: approve A1 to A4 and B1 to B3
   now, L1, M1, M2 and T1 after D4 and D9. Price: no cubacle answer for exact-frame targets until
   then; Cubistry keeps Cubing.js.
2. **D2 Pin by commit or by tag.** The fixed-frame design records "other repositories pin only
   release tags with green CI" (**O**, [Fixed-frame proofs](fixed-frame-proofs.md) 1.3); #25 and
   the Cubistry plan say commit plus integrity. R: commit plus `sha512` for the wasm tarball,
   tag pin unchanged for native binaries. Price: an exception to an existing decision.
3. **D3 Piece numbering.** Cubing.js numbering or cubacle numbering with a mapping? R: Cubing.js
   numbering for `2x2x2` and `3x3x3`, checked by a test fixture produced from the pinned Cubing.js
   in Cubistry, after a license check of that data (Cubing.js license of the definitions: **U**).
   Price: a fixture with provenance and a NOTICE entry if data is copied.
4. **D4 Tables built or downloaded.** R: built in the module, hashes pinned in tests. Price: the
   2x2x2 cold start of about 93 ms (Node) stays; the G1 open point is then decided as accepted or
   needs a smaller table (separate work).
5. **D5 Toolchain pin.** R: the xtask checks one rustc version; no repo-wide
   `rust-toolchain.toml` (it would conflict with "latest releases" and the msrv job). Price: a
   version bump of the pin with each Rust update that changes bytes.
6. **D6 Tarball writer.** R: Rust in `xtask` with the `tar` crate and a gzip crate (xtask only, not
   shipped). Alternative: `npm pack` (needs Node in the build; byte stability across npm
   versions **U**). Price: two dependencies in `xtask`.
7. **D7 Package name.** R: `cubacle-wasm`, unscoped, unpublished. Price: none now; a later npm
   scope is a rename.
8. **D8 Feature split of the root crate.** R: yes (acceptance of #21). Price: a `cli` feature in
   the root crate; the server and CLI builds name it.
9. **D9 Exact-frame route.** R: fix-up after `ignore-centers` (fast, not shortest), and the center
   fix-up words from a table computed in cubacle, not copied from Cubistry's
   `center-corrections.json`. Price: answers can be longer than the Cubing.js plus PuzlWerk route;
   G2 then fails on length and needs a decision there.
10. **D10 CI jobs.** Add a `wasm` job (build, golden replay, `node --test`) to `ci.yml`? R: yes,
    light lane, after A3. Price: about one more minute per code PR (**U** until measured).
11. **D11 Cancel.** R: terminate the worker plus `budgetMs`; no shared-memory flag. Price: tables
    are built again after a cancel during a solve.
12. **D12 Fixtures for C2.** Which Calendar fixtures (Cubistry commit and paths) are the acceptance
    set? R: the owner names them or approves a list the author proposes in the M1 pull request.

## 7. Claims that need proof

No pull request may state the following without the evidence named; otherwise the text says
UNKNOWN.

| Claim | Evidence required |
| --- | --- |
| `.wasm` size (raw, gzip, brotli) | `node tools/wasm/bench.mjs <wasm>` output, commit, rustc version, flags |
| Cold or warm time, memory | the bench or `tools/wasm/browser.mjs` command, machine, CPU, OS, Node or browser version, number of runs, range over runs (as in `docs/wasm.md`) |
| "No native-only crates" | `cargo xtask wasm-deps` output at the head |
| Reproducible `.wasm` or `.tgz` | SHA-256 from two named machines, same commit and toolchain |
| Exact replay (C2) | the native and Node replay tests, fixture list with provenance |
| Fewer or equal moves than Cubing.js | not claimable in cubacle; G2 runs in Cubistry |
| Shortest | not claimable for the browser routes of section 3.3 |
| Table hash stability | the pinned-hash test on two architectures (native aarch64 or x86_64 and wasm32) |
| Cancel works in a browser | a browser run of the glue (Playwright command as in `docs/wasm.md`); Node tests alone show the protocol |
| API covers the Calendar planner | a Node test that runs each operation listed in section 2 with the planner's data shapes |

Not verified for this plan: no build or benchmark was run; numbers above are quoted from
`docs/wasm.md` and the don't-care design. Firefox, Safari and resident memory stay UNKNOWN.
