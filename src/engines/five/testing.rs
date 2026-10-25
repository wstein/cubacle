//! Helpers for tests: the tables written by the Java solver, when a copy is at
//! hand, as an oracle for the tables built here.

use std::fs;
use std::path::PathBuf;

/// The words of the Java table `name` (for example `Phase1TCenterSymPrun.jhdata`)
/// from the directory in `CUBE555_TABLES`, if set and present.
///
/// A serialized `int[]` is a 27-byte header and the big-endian ints.
pub(super) fn java_table(name: &str) -> Option<Vec<u32>> {
    let dir = PathBuf::from(std::env::var_os("CUBE555_TABLES")?);
    let bytes = fs::read(dir.join(name)).ok()?;
    let body = bytes.get(27..)?;
    assert_eq!(body.len() % 4, 0, "{name}");
    Some(
        body.as_chunks::<4>()
            .0
            .iter()
            .map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]))
            .collect(),
    )
}

/// Compares a built table with the Java one, naming the first difference.
pub(super) fn assert_same(name: &str, built: &[u32], java: &[u32]) {
    assert_eq!(built.len(), java.len(), "{name}: length");
    if let Some(at) = built.iter().zip(java).position(|(a, b)| a != b) {
        panic!(
            "{name}: word {at} differs: built {:#010x}, java {:#010x}",
            built[at], java[at]
        );
    }
}
