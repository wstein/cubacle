//! A minimal PNG writer for RGBA8 images: zlib "stored" (uncompressed)
//! deflate blocks, so no compression code and no dependency are needed.
//! The files are about as large as the raw pixels, which is fine for the
//! small test photos of `cubacle photo`.

/// The CRC-32 (IEEE) of `bytes`, as PNG chunks use it.
pub fn crc32(bytes: &[u8]) -> u32 {
    crc32_update(0xFFFF_FFFF, bytes) ^ 0xFFFF_FFFF
}

fn crc32_update(mut crc: u32, bytes: &[u8]) -> u32 {
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    crc
}

/// The Adler-32 checksum of `bytes`, as zlib streams use it.
pub fn adler32(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for chunk in bytes.chunks(5552) {
        for &byte in chunk {
            a += u32::from(byte);
            b += a;
        }
        a %= 65521;
        b %= 65521;
    }
    (b << 16) | a
}

/// A zlib stream that stores `data` in uncompressed deflate blocks.
fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut blocks = data.chunks(65535).peekable();
    if blocks.peek().is_none() {
        out.extend_from_slice(&[1, 0, 0, 0xFF, 0xFF]);
    }
    while let Some(block) = blocks.next() {
        out.push(u8::from(blocks.peek().is_none()));
        let len = block.len() as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(block);
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

/// Encodes `rgba` (`width * height * 4` bytes, top row first) as a PNG file.
pub fn encode_rgba(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    assert_eq!(rgba.len(), width as usize * height as usize * 4);
    let mut raw = Vec::with_capacity(rgba.len() + height as usize);
    for row in rgba.chunks(width as usize * 4) {
        raw.push(0); // filter type: none
        raw.extend_from_slice(row);
    }
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]); // 8 bit, RGBA, no interlace
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    chunk(&mut out, b"IHDR", &header);
    chunk(&mut out, b"IDAT", &zlib_stored(&raw));
    chunk(&mut out, b"IEND", &[]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksums_match_known_values() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
        assert_eq!(adler32(b""), 1);
        // Long input crosses the modulo batches.
        let long = vec![0xFFu8; 100_000];
        let (mut a, mut b) = (1u64, 0u64);
        for &x in &long {
            a = (a + u64::from(x)) % 65521;
            b = (b + a) % 65521;
        }
        assert_eq!(adler32(&long), ((b << 16) | a) as u32);
    }

    /// Reads the stored blocks back out of a PNG made by `encode_rgba`.
    fn decode(png: &[u8]) -> (u32, u32, Vec<u8>) {
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        let mut pos = 8;
        let (mut width, mut height) = (0, 0);
        let mut idat = Vec::new();
        while pos < png.len() {
            let len = u32::from_be_bytes(png[pos..pos + 4].try_into().unwrap()) as usize;
            let kind = &png[pos + 4..pos + 8];
            let data = &png[pos + 8..pos + 8 + len];
            let crc = u32::from_be_bytes(png[pos + 8 + len..pos + 12 + len].try_into().unwrap());
            assert_eq!(crc, crc32(&png[pos + 4..pos + 8 + len]), "crc of a chunk");
            match kind {
                b"IHDR" => {
                    width = u32::from_be_bytes(data[..4].try_into().unwrap());
                    height = u32::from_be_bytes(data[4..8].try_into().unwrap());
                }
                b"IDAT" => idat.extend_from_slice(data),
                _ => {}
            }
            pos += 12 + len;
        }
        let mut raw = Vec::new();
        let mut at = 2;
        loop {
            let last = idat[at] == 1;
            let len = u16::from_le_bytes([idat[at + 1], idat[at + 2]]) as usize;
            let nlen = u16::from_le_bytes([idat[at + 3], idat[at + 4]]);
            assert_eq!(nlen, !(len as u16));
            raw.extend_from_slice(&idat[at + 5..at + 5 + len]);
            at += 5 + len;
            if last {
                break;
            }
        }
        assert_eq!(
            u32::from_be_bytes(idat[at..at + 4].try_into().unwrap()),
            adler32(&raw)
        );
        (width, height, raw)
    }

    #[test]
    fn round_trips_through_the_stored_blocks() {
        // 200 x 200 pixels span more than one 65535-byte block.
        let (w, h) = (200u32, 200u32);
        let rgba: Vec<u8> = (0..w * h * 4).map(|i| (i % 251) as u8).collect();
        let (dw, dh, raw) = decode(&encode_rgba(w, h, &rgba));
        assert_eq!((dw, dh), (w, h));
        assert!(raw.len() > 65535);
        let rows: Vec<u8> = raw
            .chunks(w as usize * 4 + 1)
            .flat_map(|row| {
                assert_eq!(row[0], 0);
                row[1..].to_vec()
            })
            .collect();
        assert_eq!(rows, rgba);
    }

    #[test]
    fn a_one_pixel_image_is_valid() {
        let (w, h, raw) = decode(&encode_rgba(1, 1, &[1, 2, 3, 255]));
        assert_eq!((w, h), (1, 1));
        assert_eq!(raw, vec![0, 1, 2, 3, 255]);
    }
}
