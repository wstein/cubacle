//! How the tables of the five-phase solver are made: loaded from the cache
//! if a valid file is there, else built (and then cached).
//!
//! New code; the tables themselves follow cs0x7f/cube555.

use crate::reduction::bfs::{self, Space};
use crate::reduction::cache::Cache;
use crate::reduction::hashed::{self, Hashed};
use crate::reduction::nibbles::Nibbles;

/// What the builders of the phases need.
pub(super) struct Build {
    pub threads: usize,
    pub cache: Cache,
    /// Deepest layer of a table, 15 for the full tables. A smaller value
    /// gives small tables for tests; unknown entries then read as `cap + 1`.
    pub cap: u32,
}

impl Build {
    fn id(&self, name: &str) -> String {
        if self.cap < 15 {
            format!("{name}.cap{}", self.cap)
        } else {
            name.to_owned()
        }
    }

    /// The table of exact distances in `space`.
    pub(super) fn dense<S: Space>(&self, name: &str, space: &S) -> Nibbles {
        let id = self.id(name);
        let size = space.size();
        if let Some(words) = self.cache.load(&id, Nibbles::words_for(size)) {
            return Nibbles::from_words(words);
        }
        let mut table = bfs::build(space, self.threads, self.cap);
        if self.cap < 15 {
            table.fill_unknown(size, self.cap + 1);
        }
        self.cache.store(&id, table.words());
        table
    }

    /// The hashed table of `1 << bits` entries, `depth` deep (less if capped).
    pub(super) fn hashed<S: Hashed>(
        &self,
        name: &str,
        space: &S,
        depth: u32,
        bits: u32,
    ) -> Nibbles {
        let id = self.id(name);
        if let Some(words) = self.cache.load(&id, (1usize << bits) / 8) {
            return Nibbles::from_words(words);
        }
        let table = hashed::build(space, depth.min(self.cap), bits, self.threads);
        self.cache.store(&id, table.words());
        table
    }
}
