//! All tables of the five-phase solver, owned by one value.
//!
//! New code; the contents follow cs0x7f/cube555 `Search.init`.

use std::path::PathBuf;

use super::build::Build;
use super::cubie_sym::Syms;
use super::p1::P1;
use super::p2::P2;
use super::p3::P3;
use super::p4::P4;
use super::p5::P5;
use crate::reduction::cache::Cache;

/// Bumped whenever a table changes its layout, so old cache files are ignored.
const FORMAT: u32 = 1;

pub(crate) struct Tables {
    pub(super) syms: Syms,
    pub(super) p1: P1,
    pub(super) p2: P2,
    pub(super) p3: P3,
    pub(super) p4: P4,
    pub(super) p5: P5,
    /// Whether a table could not be written to the cache directory.
    pub(super) cache_write_failed: bool,
}

/// How to make the tables.
#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub threads: usize,
    pub cache_dir: Option<PathBuf>,
    /// Deepest layer of the tables; 15 means the full tables.
    pub cap: u32,
}

impl Tables {
    pub(crate) fn build(config: &Config) -> Tables {
        let b = Build {
            threads: config.threads.max(1),
            cache: Cache::new(config.cache_dir.clone(), FORMAT),
            cap: config.cap,
        };
        let syms = Syms::new();
        let p1 = P1::build(&syms, &b);
        let p2 = P2::build(&b);
        let p3 = P3::build(&syms, &b);
        let p4 = P4::build(&b);
        let p5 = P5::build(&syms, &b);
        Tables {
            cache_write_failed: b.cache.write_failures() > 0,
            syms,
            p1,
            p2,
            p3,
            p4,
            p5,
        }
    }

    /// Bytes held by the pruning tables.
    pub(crate) fn pruning_bytes(&self) -> usize {
        self.p1.bytes() + self.p2.bytes() + self.p3.bytes() + self.p4.bytes() + self.p5.bytes()
    }
}
