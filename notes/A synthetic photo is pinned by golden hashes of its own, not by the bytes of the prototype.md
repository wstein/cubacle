---
id: 20261026150000
aliases: []
tags: ["type/decision", "status/decided", "category/architecture"]
---
The synthetic face generator began as a prototype in another language whose draft demanded the same bytes as JavaScript: a double product with `ToInt32` in the hash, round-half-even stores and libm results of V8. The owner released the draft, so cubacle does not copy those quirks. The hash is a 64-bit integer mixer with wrapping arithmetic, identical on every platform; pixels are stored by plain round-to-nearest; the label is clipped instead of wrapping into the next row. What stays is the frame: grid in, RGBA out, eight presets, the options, seed, the `SYNTHETIC <PRESET>` label and the OKLCH scatter. Determinism is promised for one binary and one platform family, because `ln`, `cos`, `cbrt` and `powf` may differ in the last bit between libm versions. The tests pin the output of this implementation with a SHA-256 per preset and option, so a change of the photos is a visible, deliberate edit of those values, together with a new revision line on the specification page.

`cubacle photo` writes the faces as PNG with stored zlib blocks: about 80 lines with CRC-32 and Adler-32 are cheaper than a dependency, and the files are small test photos.

## Links

- [[A flat net shows every sticker undistorted]]

Source: src/synthetic/rng.rs, lines 1-20; tests/synthetic_test.rs, lines 1-50
