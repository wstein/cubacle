# cubacle-wasm

The browser WebAssembly build of the cubacle 2x2x2 and 3x3x3 solvers.

The package holds `cubacle_wasm.wasm` (a C ABI over linear memory), the manifest
`cubacle-wasm.manifest.json` (size and SHA-256 of each file, the `rustc` that built it) and, once
they exist, the JavaScript glue. It is not published to npm: a consumer pins the repository, the
commit, the package name, the version and the `sha512` integrity of the tarball.

Interface and build: `docs/wasm.md` in https://github.com/wstein/cubacle.

License: AGPL-3.0-or-later (`LICENSE`); the original MIT grant of Chen Shuang is in `LICENSE-MIT`,
provenance in `NOTICE.md`.
