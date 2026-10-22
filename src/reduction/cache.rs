//! The on-disk cache of tables.
//!
//! One file per table: a 32-byte header (magic, format version, word count,
//! checksum of the payload) and the words in little-endian order. A file that
//! is missing, foreign, of another version, of the wrong size or corrupt is
//! treated as absent and rebuilt; it is never trusted. Writing goes to a
//! temporary name and is renamed into place, so a reader sees a whole file or
//! none. Failing to write is not an error: the tables stay in memory.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 8] = b"CUBACLE\x01";
const HEADER: usize = 32;

/// A directory of table files of one format version.
#[derive(Debug, Clone)]
pub(crate) struct Cache {
    dir: Option<PathBuf>,
    version: u32,
}

/// The default directory: `CUBACLE_CACHE_DIR`, else `$XDG_CACHE_HOME/cubacle`,
/// else `~/.cache/cubacle`.
pub(crate) fn default_dir() -> Option<PathBuf> {
    let var = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
    if let Some(dir) = var("CUBACLE_CACHE_DIR") {
        return Some(PathBuf::from(dir));
    }
    if let Some(dir) = var("XDG_CACHE_HOME") {
        return Some(PathBuf::from(dir).join("cubacle"));
    }
    var("HOME").map(|home| PathBuf::from(home).join(".cache").join("cubacle"))
}

fn checksum(words: &[u32]) -> u64 {
    words.iter().fold(0xcbf2_9ce4_8422_2325u64, |hash, &word| {
        (hash ^ u64::from(word))
            .wrapping_mul(0x0000_0100_0000_01b3)
            .rotate_left(5)
    })
}

impl Cache {
    pub(crate) fn new(dir: Option<PathBuf>, version: u32) -> Cache {
        Cache { dir, version }
    }

    pub(crate) fn dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    fn path(&self, id: &str) -> Option<PathBuf> {
        self.dir
            .as_ref()
            .map(|dir| dir.join(format!("{id}.v{}.tbl", self.version)))
    }

    /// The words of table `id`, if a valid file of exactly `words` words exists.
    pub(crate) fn load(&self, id: &str, words: usize) -> Option<Vec<u32>> {
        let path = self.path(id)?;
        let bytes = fs::read(path).ok()?;
        if bytes.len() != HEADER + words * 4 || &bytes[..8] != MAGIC {
            return None;
        }
        let field = |at: usize, len: usize| &bytes[at..at + len];
        if field(8, 4) != self.version.to_le_bytes() || field(16, 8) != (words as u64).to_le_bytes()
        {
            return None;
        }
        let payload: Vec<u32> = bytes[HEADER..]
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();
        (field(24, 8) == checksum(&payload).to_le_bytes()).then_some(payload)
    }

    /// Writes table `id`; returns whether it was written.
    pub(crate) fn store(&self, id: &str, words: &[u32]) -> bool {
        let Some(path) = self.path(id) else {
            return false;
        };
        let write = || -> std::io::Result<()> {
            fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))?;
            let temp = path.with_extension(format!("tmp{}", std::process::id()));
            let mut out = std::io::BufWriter::new(fs::File::create(&temp)?);
            let mut header = [0u8; HEADER];
            header[..8].copy_from_slice(MAGIC);
            header[8..12].copy_from_slice(&self.version.to_le_bytes());
            header[16..24].copy_from_slice(&(words.len() as u64).to_le_bytes());
            header[24..].copy_from_slice(&checksum(words).to_le_bytes());
            out.write_all(&header)?;
            for word in words {
                out.write_all(&word.to_le_bytes())?;
            }
            out.flush()?;
            drop(out);
            fs::rename(&temp, &path).inspect_err(|_| {
                let _ = fs::remove_file(&temp);
            })
        };
        write().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cubacle-cache-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_stored_table_comes_back() {
        let dir = scratch("roundtrip");
        let cache = Cache::new(Some(dir.clone()), 1);
        let words: Vec<u32> = (0..1000).map(|i| i * 2_654_435_761u32).collect();
        assert!(cache.store("t", &words));
        assert_eq!(cache.load("t", 1000), Some(words));
        assert_eq!(cache.load("t", 999), None, "wrong size");
        assert_eq!(
            Cache::new(Some(dir.clone()), 2).load("t", 1000),
            None,
            "other version"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn a_damaged_file_is_not_trusted() {
        let dir = scratch("damaged");
        let cache = Cache::new(Some(dir.clone()), 1);
        assert!(cache.store("t", &[1, 2, 3, 4]));
        let path = cache.path("t").unwrap();
        let mut bytes = fs::read(&path).unwrap();
        bytes[HEADER + 5] ^= 1;
        fs::write(&path, bytes).unwrap();
        assert_eq!(cache.load("t", 4), None);
        fs::write(&path, b"junk").unwrap();
        assert_eq!(cache.load("t", 4), None);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn without_a_directory_nothing_is_stored() {
        let cache = Cache::new(None, 1);
        assert!(!cache.store("t", &[1]));
        assert_eq!(cache.load("t", 1), None);
    }
}
