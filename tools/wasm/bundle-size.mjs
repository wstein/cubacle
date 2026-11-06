// Comparison input for G1: raw, gzip (level 9) and brotli (quality 11) size of the
// files in a directory whose names match a pattern, plus every embedded
// WebAssembly module (a template string that starts with the base64 of "\0asm").
//
//   node tools/wasm/bundle-size.mjs <directory> <file name regex>
import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';

const [dir, pattern] = process.argv.slice(2);
if (!dir || !pattern) {
  console.error('usage: node tools/wasm/bundle-size.mjs <directory> <file name regex>');
  process.exit(2);
}
const sizes = (bytes) => ({
  raw: bytes.length,
  gzip: zlib.gzipSync(bytes, { level: 9 }).length,
  brotli: zlib.brotliCompressSync(bytes, {
    params: { [zlib.constants.BROTLI_PARAM_QUALITY]: 11, [zlib.constants.BROTLI_PARAM_SIZE_HINT]: bytes.length },
  }).length,
});
const files = fs.readdirSync(dir).filter((name) => new RegExp(pattern).test(name)).sort();
if (files.length === 0) {
  console.error('no file matches');
  process.exit(1);
}
const out = { files: {}, total: { raw: 0, gzip: 0, brotli: 0 } };
for (const name of files) {
  const bytes = fs.readFileSync(path.join(dir, name));
  const own = sizes(bytes);
  const embedded = [...bytes.toString('utf8').matchAll(/[`'"](AGFzbQ[A-Za-z0-9+/=]+)[`'"]/g)].map((m) => {
    const wasm = Buffer.from(m[1], 'base64');
    return { magic: wasm.subarray(0, 4).toString('hex'), ...sizes(wasm) };
  });
  out.files[name] = { ...own, embedded_wasm: embedded };
  for (const key of Object.keys(out.total)) out.total[key] += own[key];
}
console.log(JSON.stringify(out, null, 2));
