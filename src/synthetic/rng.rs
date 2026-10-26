//! Counter-based random numbers: every value is a pure function of
//! `(seed, index)`, so the generator has no state.
//!
//! The hash is a 64-bit integer mixer (the finalizer of SplitMix64) with
//! wrapping arithmetic, identical on every platform. [`gauss`] uses the
//! standard library's `ln` and `cos`, which may differ in the last bit between
//! platforms; see the guarantee in `mod.rs`.

/// The 32-bit hash of `(seed, i)`.
pub(crate) fn hash(seed: i32, i: u64) -> u32 {
    let mut z = (u64::from(seed as u32) << 32) ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((z ^ (z >> 31)) >> 32) as u32
}

/// A uniform value in `[0, 1)` with 24 bits.
pub(crate) fn unit(seed: i32, i: u64) -> f64 {
    f64::from(hash(seed, i) & 0xFF_FFFF) / 16_777_216.0
}

/// A standard normal value (Box-Muller, one cosine branch).
pub(crate) fn gauss(seed: i32, i: u64) -> f64 {
    let u1 = unit(seed, 2 * i) + 1e-9;
    let u2 = unit(seed, 2 * i + 1);
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_pinned() {
        // Pinned values: a change here changes every photo.
        assert_eq!(hash(0, 0), 0xE220_A839);
        assert_eq!(hash(7, 6000), 3_927_243_653);
    }

    #[test]
    fn hash_depends_on_seed_and_index() {
        assert_ne!(hash(1, 0), hash(2, 0));
        assert_ne!(hash(1, 0), hash(1, 1));
        assert_eq!(hash(5, 9), hash(5, 9));
    }

    #[test]
    fn unit_is_in_range_and_spread() {
        let values: Vec<f64> = (0..10_000).map(|i| unit(3, i)).collect();
        assert!(values.iter().all(|&v| (0.0..1.0).contains(&v)));
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        assert!((mean - 0.5).abs() < 0.02, "mean {mean}");
    }

    #[test]
    fn gauss_is_roughly_standard_normal() {
        let n = 20_000;
        let values: Vec<f64> = (0..n).map(|i| gauss(11, i)).collect();
        let mean = values.iter().sum::<f64>() / n as f64;
        let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
        assert!(mean.abs() < 0.05, "mean {mean}");
        assert!((var - 1.0).abs() < 0.08, "variance {var}");
    }
}
