//! The block alphabet, the walk over block sequences and the stored table
//! that meet-in-the-middle searches share (pattern repair, `solve --within`).

mod alphabet;
mod search;

pub(crate) use alphabet::{perm_of, Alphabet};
pub(crate) use search::{hash_of, permuted, position_keys, sums_of, Table, Walker, ENTRY_BYTES};
