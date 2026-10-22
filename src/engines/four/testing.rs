//! Helpers for tests: the checksums the reference prints for its tables.

/// The 64-bit FNV-1 style hash of a sequence of integers, as the checking
/// harness of the reference computes it (`x ^= v; x *= 1099511628211`, the
/// values sign-extended).
pub(super) fn fnv(values: impl IntoIterator<Item = i64>) -> i64 {
    let mut x: u64 = 1_469_598_103_934_665_603;
    for v in values {
        x ^= v as u64;
        x = x.wrapping_mul(1_099_511_628_211);
    }
    x as i64
}
