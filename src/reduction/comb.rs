//! Coordinates of permutations and combinations of pieces.
//!
//! Slot arrays hold `i32`: a piece number, or `-1` for a slot that is not one
//! of the pieces being tracked. This is the numbering of `Util` in
//! cs0x7f/cube555 (Ported from cs0x7f/cube555 src/Util.java, MIT, Copyright
//! (c) 2023 Chen Shuang; this file is AGPL-3.0-or-later, see NOTICE.md), kept
//! exactly so that coordinates agree with that solver.

/// Binomial coefficients `C(n, k)` for `n, k < 25`.
const fn binomials() -> [[i32; 25]; 25] {
    let mut table = [[0i32; 25]; 25];
    let mut i = 0;
    while i < 25 {
        table[i][0] = 1;
        table[i][i] = 1;
        i += 1;
    }
    i = 1;
    while i < 25 {
        let mut j = 1;
        while j <= i {
            table[i][j] = table[i - 1][j] + table[i - 1][j - 1];
            j += 1;
        }
        i += 1;
    }
    table
}

pub(crate) const CNK: [[i32; 25]; 25] = binomials();

const FACT: [i32; 13] = {
    let mut f = [1i32; 13];
    let mut i = 1;
    while i < 13 {
        f[i] = f[i - 1] * i as i32;
        i += 1;
    }
    f
};

/// Writes permutation number `idx` of `n` items into `arr` (of the even
/// permutations only, when `even`).
pub(crate) fn set_perm(arr: &mut [i32], mut idx: i32, n: usize, even: bool) {
    let mut val: u64 = 0xFEDC_BA98_7654_3210;
    let mut parity = 0;
    if even {
        idx <<= 1;
    }
    let n = n - 1;
    for i in 0..n {
        let p = FACT[n - i];
        let v = idx / p;
        parity ^= v;
        idx %= p;
        let shift = (v << 2) as u32;
        arr[i] = ((val >> shift) & 0xf) as i32;
        let m = (1u64 << shift) - 1;
        val = (val & m) + ((val >> 4) & !m);
    }
    if even && parity & 1 != 0 {
        arr[n] = arr[n - 1];
        arr[n - 1] = (val & 0xf) as i32;
    } else {
        arr[n] = (val & 0xf) as i32;
    }
}

/// The number of the permutation in `arr[..n]`.
pub(crate) fn get_perm(arr: &[i32], n: usize, even: bool) -> i32 {
    let mut idx = 0;
    let mut val: u64 = 0xFEDC_BA98_7654_3210;
    for (i, &piece) in arr.iter().enumerate().take(n - 1) {
        let shift = (piece << 2) as u32;
        idx = (n - i) as i32 * idx + ((val >> shift) & 0xf) as i32;
        val = val.wrapping_sub(0x1111_1111_1111_1110u64 << shift);
    }
    if even {
        idx >> 1
    } else {
        idx
    }
}

/// Marks `r` of the first `n` slots with `0` and the others with `-1`.
pub(crate) fn set_comb(arr: &mut [i32], mut idx: i32, mut r: usize, n: usize) {
    for i in (0..n).rev() {
        if idx >= CNK[i][r] {
            idx -= CNK[i][r];
            r = r.wrapping_sub(1);
            arr[i] = 0;
        } else {
            arr[i] = -1;
        }
    }
}

/// The number of the combination of marked slots in `arr[..n]`.
pub(crate) fn get_comb(arr: &[i32], mut r: usize, n: usize) -> i32 {
    let mut idx = 0;
    for i in (0..n).rev() {
        if arr[i] != -1 {
            idx += CNK[i][r];
            r = r.wrapping_sub(1);
        }
    }
    idx
}

/// Like [`get_comb`] for `n` slots holding two kinds of `n / 2` each: the
/// slots that differ from the last one are the marked ones.
pub(crate) fn get_scomb(arr: &[i32]) -> i32 {
    let n = arr.len();
    let mut r = n / 2;
    let mut idx = 0;
    for i in (0..n).rev() {
        if arr[i] != arr[n - 1] {
            idx += CNK[i][r];
            r = r.wrapping_sub(1);
        }
    }
    idx
}

/// Copies the pieces of the marked slots of `src` to `dst`, in slot order.
pub(crate) fn copy_from_comb(src: &[i32], dst: &mut [i32]) {
    let mut r = 0;
    for &v in src {
        if v != -1 {
            dst[r] = v;
            r += 1;
        }
    }
}

/// Puts `src` into the marked slots of `dst`, in slot order.
pub(crate) fn copy_to_comb(src: &[i32], dst: &mut [i32]) {
    let mut r = 0;
    for slot in dst.iter_mut() {
        if *slot != -1 {
            *slot = src[r];
            r += 1;
        }
    }
}

/// The parity (0 even, 1 odd) of a permutation.
pub(crate) fn parity(arr: &[i32]) -> i32 {
    let mut p = 0;
    for i in 0..arr.len() {
        for j in i + 1..arr.len() {
            if arr[i] > arr[j] {
                p ^= 1;
            }
        }
    }
    p
}

/// Exchanges two slots.
pub(crate) fn swap2(arr: &mut [i32], a: usize, b: usize) {
    arr.swap(a, b);
}

/// Turns four slots `a b c d` by `pow` (0 forwards, 1 half, 2 backwards),
/// xoring `1` into each moved piece when `flip`.
pub(crate) fn swap4(arr: &mut [i32], [a, b, c, d]: [usize; 4], pow: i32, flip: bool) {
    let x = i32::from(flip);
    match pow {
        0 => {
            let t = arr[d] ^ x;
            arr[d] = arr[c] ^ x;
            arr[c] = arr[b] ^ x;
            arr[b] = arr[a] ^ x;
            arr[a] = t;
        }
        1 => {
            arr.swap(a, c);
            arr.swap(b, d);
        }
        _ => {
            let t = arr[a] ^ x;
            arr[a] = arr[b] ^ x;
            arr[b] = arr[c] ^ x;
            arr[c] = arr[d] ^ x;
            arr[d] = t;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permutation_numbers_round_trip() {
        let mut arr = [0; 8];
        for idx in (0..40320).step_by(7) {
            set_perm(&mut arr, idx, 8, false);
            assert_eq!(get_perm(&arr, 8, false), idx);
        }
        for idx in (0..20160).step_by(5) {
            set_perm(&mut arr, idx, 8, true);
            assert_eq!(get_perm(&arr, 8, true), idx);
            assert_eq!(parity(&arr), 0);
        }
    }

    #[test]
    fn combination_numbers_round_trip() {
        let mut arr = [0; 24];
        for idx in (0..735_471).step_by(997) {
            set_comb(&mut arr, idx, 8, 24);
            assert_eq!(arr.iter().filter(|&&v| v == 0).count(), 8);
            assert_eq!(get_comb(&arr, 8, 24), idx);
        }
    }

    #[test]
    fn four_cycles_compose() {
        let mut arr = [0, 1, 2, 3];
        swap4(&mut arr, [0, 1, 2, 3], 0, false);
        swap4(&mut arr, [0, 1, 2, 3], 2, false);
        assert_eq!(arr, [0, 1, 2, 3]);
        swap4(&mut arr, [0, 1, 2, 3], 1, false);
        swap4(&mut arr, [0, 1, 2, 3], 1, false);
        assert_eq!(arr, [0, 1, 2, 3]);
    }
}
