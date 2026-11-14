//! A deterministic `.tgz`: entries sorted by name, ustar headers with a fixed owner and
//! mode, one modification time for every entry, and a gzip header without name or time.

use std::io::Write;

use flate2::{Compression, GzBuilder};
use sha2::{Digest, Sha512};

/// The gzip header "operating system" byte: 255, unknown, on every host.
const GZIP_OS_UNKNOWN: u8 = 255;

/// One file of the package. `name` is relative to the `package/` prefix.
pub struct Entry {
    pub name: String,
    pub data: Vec<u8>,
    /// 0o644, or 0o755 for an executable.
    pub mode: u32,
}

/// The `.tgz` bytes of `entries` under the prefix `package/`; `mtime` (seconds) is the
/// commit time. Refuses a repeated name.
pub fn tgz(entries: &[Entry], mtime: u64) -> Result<Vec<u8>, String> {
    let mut sorted: Vec<&Entry> = entries.iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    if let Some(pair) = sorted.windows(2).find(|pair| pair[0].name == pair[1].name) {
        return Err(format!("the package lists {} twice", pair[0].name));
    }
    let mut builder = tar::Builder::new(Vec::new());
    for entry in sorted {
        let mut header = tar::Header::new_ustar();
        header
            .set_path(format!("package/{}", entry.name))
            .map_err(|error| format!("{}: {error}", entry.name))?;
        header.set_size(entry.data.len() as u64);
        header.set_mode(entry.mode);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mtime(mtime);
        header.set_entry_type(tar::EntryType::Regular);
        header.set_cksum();
        builder
            .append(&header, entry.data.as_slice())
            .map_err(|error| error.to_string())?;
    }
    let tar_bytes = builder.into_inner().map_err(|error| error.to_string())?;
    let mut encoder = GzBuilder::new()
        .mtime(0)
        .operating_system(GZIP_OS_UNKNOWN)
        .write(Vec::new(), Compression::best());
    encoder
        .write_all(&tar_bytes)
        .map_err(|error| error.to_string())?;
    encoder.finish().map_err(|error| error.to_string())
}

/// Standard base64 with padding.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, b)| n | (*b as u32) << (16 - 8 * i));
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

/// The npm integrity string of `bytes`: `sha512-` and the base64 of the digest.
pub fn integrity(bytes: &[u8]) -> String {
    format!("sha512-{}", base64(&Sha512::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::GzDecoder;
    use std::io::Read;

    fn entry(name: &str, data: &str, mode: u32) -> Entry {
        Entry {
            name: name.into(),
            data: data.as_bytes().to_vec(),
            mode,
        }
    }

    fn sample() -> Vec<Entry> {
        vec![
            entry("b.txt", "bb", 0o644),
            entry("a.txt", "a", 0o755),
            entry("c/d.txt", "", 0o644),
        ]
    }

    fn untar(tgz_bytes: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        GzDecoder::new(tgz_bytes).read_to_end(&mut out).unwrap();
        out
    }

    #[test]
    fn entries_are_sorted_under_the_package_prefix() {
        let bytes = tgz(&sample(), 1_700_000_000).unwrap();
        let raw = untar(&bytes);
        let mut archive = tar::Archive::new(raw.as_slice());
        let names: Vec<String> = archive
            .entries()
            .unwrap()
            .map(|e| e.unwrap().path().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["package/a.txt", "package/b.txt", "package/c/d.txt"]);
    }

    #[test]
    fn mtime_owner_mode_and_format_are_fixed() {
        let bytes = tgz(&sample(), 1_700_000_000).unwrap();
        let raw = untar(&bytes);
        let mut archive = tar::Archive::new(raw.as_slice());
        let mut modes = Vec::new();
        for e in archive.entries().unwrap() {
            let e = e.unwrap();
            let h = e.header();
            assert_eq!(h.mtime().unwrap(), 1_700_000_000);
            assert_eq!((h.uid().unwrap(), h.gid().unwrap()), (0, 0));
            assert!(h.username().unwrap().is_none_or(str::is_empty));
            assert!(h.groupname().unwrap().is_none_or(str::is_empty));
            assert_eq!(&h.as_bytes()[257..263], b"ustar\0");
            modes.push(h.mode().unwrap());
        }
        assert_eq!(modes, [0o755, 0o644, 0o644]);
    }

    #[test]
    fn the_gzip_header_has_no_name_no_time_and_a_fixed_os_byte() {
        let bytes = tgz(&sample(), 1).unwrap();
        assert_eq!(
            &bytes[..4],
            &[0x1f, 0x8b, 8, 0],
            "magic, deflate, no flags (no name)"
        );
        assert_eq!(&bytes[4..8], &[0, 0, 0, 0], "no modification time");
        assert_eq!(bytes[9], GZIP_OS_UNKNOWN);
        assert_eq!(
            &bytes[..10],
            &[0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 2, 255],
            "XFL 2 marks the best compression"
        );
    }

    #[test]
    fn input_order_does_not_matter_and_two_packs_are_equal() {
        let mut reversed = sample();
        reversed.reverse();
        assert_eq!(tgz(&sample(), 5).unwrap(), tgz(&reversed, 5).unwrap());
        assert_eq!(
            integrity(&tgz(&sample(), 5).unwrap()),
            integrity(&tgz(&sample(), 5).unwrap())
        );
        assert_ne!(tgz(&sample(), 5).unwrap(), tgz(&sample(), 6).unwrap());
    }

    #[test]
    fn a_repeated_name_is_refused() {
        let entries = vec![entry("a", "1", 0o644), entry("a", "2", 0o644)];
        assert!(tgz(&entries, 0).unwrap_err().contains("twice"));
    }

    #[test]
    fn integrity_matches_a_known_digest() {
        // sha512 of the empty input, base64.
        assert_eq!(
            integrity(b""),
            "sha512-z4PhNX7vuL3xVChQ1m2AB9Yg5AULVxXcg/SpIdNs6c5H0NE8XYXysP+DGNKHfuwvY7kxvUdBeoGlODJ6+SfaPg=="
        );
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
    }
}
