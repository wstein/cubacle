//! The Kitty graphics protocol, transmitting raw RGBA and displaying it at once.
//!
//! Syntax per <https://sw.kovidgoyal.net/kitty/graphics-protocol/>: every
//! command is `ESC _ G <keys> ; <base64 payload> ESC \`, with `a=T` (transmit
//! and display), `f=32` (RGBA), `s` and `v` (pixel width and height), `q=2`
//! (suppress replies), `c` and `r` (the size in cells). The payload is cut into
//! chunks of at most 4096 base64 bytes (a multiple of 4 for all but the last),
//! `m=1` on every chunk but the last, which carries `m=0`; later chunks carry
//! only `m`.

use super::raster::Raster;

const CHUNK: usize = 4096;
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64 with padding.
pub(crate) fn base64(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, &b)| n | u32::from(b) << (16 - 8 * i));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// The escape sequences that show `raster`, optionally scaled to `cells` columns by rows.
pub(crate) fn encode(raster: &Raster, cells: Option<(u32, u32)>) -> String {
    let payload = base64(&raster.data);
    let mut keys = format!("a=T,f=32,s={},v={},q=2", raster.width, raster.height);
    if let Some((columns, rows)) = cells {
        keys.push_str(&format!(",c={columns},r={rows}"));
    }
    let mut out = String::new();
    let chunks: Vec<&str> = payload
        .as_bytes()
        .chunks(CHUNK)
        .map(|c| std::str::from_utf8(c).expect("base64 is ascii"))
        .collect();
    for (i, chunk) in chunks.iter().enumerate() {
        let more = u8::from(i + 1 < chunks.len());
        if i == 0 {
            out.push_str(&format!("\x1b_G{keys},m={more};{chunk}\x1b\\"));
        } else {
            out.push_str(&format!("\x1b_Gm={more};{chunk}\x1b\\"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn unbase64(text: &str) -> Vec<u8> {
        let mut out = Vec::new();
        let digits: Vec<u32> = text
            .bytes()
            .filter(|&b| b != b'=')
            .map(|b| ALPHABET.iter().position(|&a| a == b).unwrap() as u32)
            .collect();
        for group in digits.chunks(4) {
            let n = group
                .iter()
                .enumerate()
                .fold(0u32, |n, (i, &d)| n | d << (18 - 6 * i));
            for i in 0..group.len() - 1 {
                out.push((n >> (16 - 8 * i)) as u8);
            }
        }
        out
    }

    #[test]
    fn base64_matches_the_rfc_vectors() {
        for (plain, coded) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(plain.as_bytes()), coded);
        }
    }

    fn payloads(sequence: &str) -> Vec<(String, String)> {
        sequence
            .split("\x1b\\")
            .filter(|s| !s.is_empty())
            .map(|s| {
                let body = s.strip_prefix("\x1b_G").expect("starts with ESC _ G");
                let (keys, data) = body.split_once(';').expect("keys;payload");
                (keys.to_owned(), data.to_owned())
            })
            .collect()
    }

    #[test]
    fn large_images_are_chunked_and_round_trip() {
        let data: Vec<u8> = (0..20_000u32).map(|i| (i * 7 % 251) as u8).collect();
        let raster = Raster {
            width: 100,
            height: 50,
            data: data[..20_000].to_vec(),
        };
        let parts = payloads(&encode(&raster, Some((12, 6))));
        assert!(parts.len() > 2);
        assert_eq!(parts[0].0, "a=T,f=32,s=100,v=50,q=2,c=12,r=6,m=1");
        let mut joined = String::new();
        for (i, (keys, chunk)) in parts.iter().enumerate() {
            assert!(chunk.len() <= 4096);
            let last = i + 1 == parts.len();
            if i > 0 {
                assert_eq!(keys, if last { "m=0" } else { "m=1" });
            }
            if !last {
                assert_eq!(chunk.len() % 4, 0);
                assert!(keys.ends_with("m=1"));
            } else {
                assert!(keys.ends_with("m=0"));
            }
            joined.push_str(chunk);
        }
        assert_eq!(unbase64(&joined), raster.data);
    }

    #[test]
    fn a_small_image_is_one_final_chunk() {
        let raster = Raster {
            width: 1,
            height: 1,
            data: vec![1, 2, 3, 4],
        };
        assert_eq!(
            encode(&raster, None),
            "\x1b_Ga=T,f=32,s=1,v=1,q=2,m=0;AQIDBA==\x1b\\"
        );
    }
}
