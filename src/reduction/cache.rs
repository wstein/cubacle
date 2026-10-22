//! The on-disk cache of tables.
//!
//! One file per table: a 32-byte header (magic, format version, word count,
//! checksum of the payload) and the words in little-endian order. A file that
//! is missing, foreign, of another version, of the wrong size or corrupt is
//! treated as absent and rebuilt; it is never trusted. Writing goes to a
//! temporary name and is renamed into place, so a reader sees a whole file or
//! none. Failing to write is not an error: the tables stay in memory.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const MAGIC: &[u8; 8] = b"CUBACLE\x01";
const HEADER: usize = 32;

/// A directory of table files of one format version.
#[derive(Debug, Clone)]
pub(crate) struct Cache {
    dir: Option<PathBuf>,
    version: u32,
    /// Writes that failed, shared by the clones.
    failures: Arc<AtomicUsize>,
}

/// The default directory: `CUBACLE_CACHE_DIR`, else `$XDG_CACHE_HOME/cubacle`,
/// else `~/.cache/cubacle`.
pub(crate) fn default_dir() -> Option<PathBuf> {
    dir_from(|name| std::env::var_os(name))
}

fn dir_from(get: impl Fn(&str) -> Option<std::ffi::OsString>) -> Option<PathBuf> {
    let var = |name: &str| get(name).filter(|v| !v.is_empty());
    if let Some(dir) = var("CUBACLE_CACHE_DIR") {
        return Some(PathBuf::from(dir));
    }
    // A relative XDG_CACHE_HOME is invalid by the specification; ignore it.
    if let Some(dir) = var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute())
    {
        return Some(dir.join("cubacle"));
    }
    var("HOME").map(|home| PathBuf::from(home).join(".cache").join("cubacle"))
}

const SEED: u64 = 0xcbf2_9ce4_8422_2325;

fn mix(hash: u64, word: u32) -> u64 {
    (hash ^ u64::from(word))
        .wrapping_mul(0x0000_0100_0000_01b3)
        .rotate_left(5)
}

fn checksum(words: &[u32]) -> u64 {
    words.iter().fold(SEED, |hash, &word| mix(hash, word))
}

impl Cache {
    pub(crate) fn new(dir: Option<PathBuf>, version: u32) -> Cache {
        Cache {
            dir,
            version,
            failures: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn path(&self, id: &str) -> Option<PathBuf> {
        self.dir
            .as_ref()
            .map(|dir| dir.join(format!("{id}.v{}.tbl", self.version)))
    }

    /// The words of table `id`, if a valid file of exactly `words` words exists.
    pub(crate) fn load(&self, id: &str, words: usize) -> Option<Vec<u32>> {
        let mut file = fs::File::open(self.path(id)?).ok()?;
        if file.metadata().ok()?.len() != (HEADER + words * 4) as u64 {
            return None;
        }
        let mut header = [0u8; HEADER];
        file.read_exact(&mut header).ok()?;
        if &header[..8] != MAGIC
            || header[8..12] != self.version.to_le_bytes()
            || header[16..24] != (words as u64).to_le_bytes()
        {
            return None;
        }
        // Streamed in blocks, so the file is never held next to the table.
        let mut payload = Vec::with_capacity(words);
        let mut block = vec![0u8; 1 << 20];
        let mut hash = SEED;
        while payload.len() < words {
            let take = ((words - payload.len()) * 4).min(block.len());
            file.read_exact(&mut block[..take]).ok()?;
            for chunk in block[..take].chunks_exact(4) {
                let word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                hash = mix(hash, word);
                payload.push(word);
            }
        }
        (header[24..] == hash.to_le_bytes()).then_some(payload)
    }

    /// How many writes failed so far (a table that could not be saved is
    /// still in memory, but will be built again next time).
    pub(crate) fn write_failures(&self) -> usize {
        self.failures.load(Ordering::Relaxed)
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
        let written = write().is_ok();
        if !written {
            self.failures.fetch_add(1, Ordering::Relaxed);
        }
        written
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
    fn the_directory_follows_the_environment() {
        let env = |pairs: &'static [(&str, &str)]| {
            move |name: &str| {
                pairs
                    .iter()
                    .find(|(key, _)| *key == name)
                    .map(|(_, value)| std::ffi::OsString::from(value))
            }
        };
        let all = &[
            ("CUBACLE_CACHE_DIR", "/own"),
            ("XDG_CACHE_HOME", "/xdg"),
            ("HOME", "/home/u"),
        ];
        assert_eq!(dir_from(env(all)), Some(PathBuf::from("/own")));
        assert_eq!(
            dir_from(env(&all[1..])),
            Some(PathBuf::from("/xdg/cubacle"))
        );
        assert_eq!(
            dir_from(env(&[("XDG_CACHE_HOME", "relative"), ("HOME", "/home/u")])),
            Some(PathBuf::from("/home/u/.cache/cubacle")),
            "a relative XDG_CACHE_HOME is ignored"
        );
        assert_eq!(dir_from(env(&[])), None);
    }

    #[test]
    fn a_failed_write_is_counted() {
        let dir = scratch("unwritable");
        fs::create_dir_all(&dir).unwrap();
        let blocker = dir.join("file");
        fs::write(&blocker, b"x").unwrap();
        // A directory below a regular file cannot be created.
        let cache = Cache::new(Some(blocker.join("sub")), 1);
        assert!(!cache.store("t", &[1, 2]));
        assert_eq!(cache.write_failures(), 1);
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
