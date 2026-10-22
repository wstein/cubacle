//! Origin: Rust re-implementation of `Orbit64.Coord`, `Orbit64.Rank` and the
//! mixed-radix packing of `Orbit64.State` (flix-orbit64, Apache-2.0, Werner Stein).
//!
//! A cube state is a list of digits with fixed radices. Corners are a
//! permutation (Lehmer digits) and seven base-3 twists; midges a permutation of
//! the parity the corners dictate (the last two digits are implied) and eleven
//! base-2 flips; each wing orbit a permutation of 24; each centre orbit five
//! combination ranks. An odd cube ends with its frame, radix 24. Written most
//! significant digit first, the digits are one big mixed-radix number.

use super::frame::FRAME_COUNT;

/// Twist and flip digits are little-endian: the first slot is the units digit.
const TWIST_STATES: u32 = 2187;
const FLIP_STATES: u32 = 2048;

/// The coordinates of every orbit, as the pieces sitting in each slot.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct Coords {
    pub(super) corner_perm: Vec<usize>,
    pub(super) corner_twist: Vec<usize>,
    pub(super) midge_perm: Vec<usize>,
    pub(super) midge_flip: Vec<usize>,
    pub(super) wings: Vec<Vec<usize>>,
    pub(super) centres: Vec<Vec<usize>>,
}

fn choose(n: usize, k: usize) -> u32 {
    if k > n {
        return 0;
    }
    (0..k).fold(1u64, |acc, i| acc * (n - i) as u64 / (i as u64 + 1)) as u32
}

fn permutation_radices(n: usize, free: usize) -> impl Iterator<Item = u32> {
    (0..free).map(move |i| (n - i) as u32)
}

fn centre_radices() -> impl Iterator<Item = u32> {
    (0..5).map(|t| choose(24 - 4 * t, 4))
}

/// The radix of every digit of an `n x n x n` state, most significant first.
pub(super) fn radices(n: usize) -> Vec<u32> {
    let mut out: Vec<u32> = permutation_radices(8, 8).collect();
    out.push(TWIST_STATES);
    if n % 2 == 1 {
        out.extend(permutation_radices(12, 10));
        out.push(FLIP_STATES);
    }
    for _ in 0..(n - 2) / 2 {
        out.extend(permutation_radices(24, 24));
    }
    for _ in 0..(n - 2) * (n - 2) / 4 {
        out.extend(centre_radices());
    }
    if n % 2 == 1 {
        out.push(FRAME_COUNT as u32);
    }
    out
}

fn parity(permutation: &[usize]) -> usize {
    let inversions: usize = (0..permutation.len())
        .map(|i| {
            permutation[i + 1..]
                .iter()
                .filter(|&&q| q < permutation[i])
                .count()
        })
        .sum();
    inversions % 2
}

fn is_permutation(values: &[usize], n: usize) -> bool {
    let mut seen = vec![false; n];
    values.len() == n
        && values
            .iter()
            .all(|&v| v < n && !std::mem::replace(&mut seen[v], true))
}

fn lehmer(permutation: &[usize], free: usize, out: &mut Vec<u32>) {
    for i in 0..free {
        let smaller = permutation[i + 1..]
            .iter()
            .filter(|&&q| q < permutation[i])
            .count();
        out.push(smaller as u32);
    }
}

/// Rebuilds a permutation of `n` from `free` Lehmer digits. With `wanted`
/// the last two entries are the order that gives that parity.
fn unlehmer(digits: &mut impl Iterator<Item = u32>, n: usize, wanted: Option<usize>) -> Vec<usize> {
    let mut available: Vec<usize> = (0..n).collect();
    let free = if wanted.is_some() { n - 2 } else { n };
    let mut out = Vec::with_capacity(n);
    for _ in 0..free {
        let digit = digits.next().unwrap_or(0) as usize;
        out.push(available.remove(digit));
    }
    out.extend(available);
    if let Some(parity_wanted) = wanted {
        if parity(&out) != parity_wanted {
            out.swap(n - 2, n - 1);
        }
    }
    out
}

fn orientation_value(values: &[usize], radix: usize) -> u32 {
    let free = values.len() - 1;
    values[..free]
        .iter()
        .rev()
        .fold(0, |acc, &v| acc * radix as u32 + v as u32)
}

fn orientations(mut value: u32, count: usize, radix: u32) -> Vec<usize> {
    let mut out = Vec::with_capacity(count + 1);
    for _ in 0..count {
        out.push((value % radix) as usize);
        value /= radix;
    }
    let sum: usize = out.iter().sum();
    out.push((radix as usize - sum % radix as usize) % radix as usize);
    out
}

fn centre_digits(colours: &[usize], out: &mut Vec<u32>) -> Result<(), String> {
    let counts_ok = (0..6).all(|c| colours.iter().filter(|&&x| x == c).count() == 4);
    if colours.len() != 24 || !counts_ok {
        return Err("a centre orbit does not hold each colour exactly four times".into());
    }
    let mut remaining: Vec<usize> = (0..24).collect();
    for colour in 0..5 {
        let hits: Vec<usize> = (0..remaining.len())
            .filter(|&p| colours[remaining[p]] == colour)
            .collect();
        let rank: u32 = hits
            .iter()
            .enumerate()
            .map(|(i, &p)| choose(p, i + 1))
            .sum();
        out.push(rank);
        remaining.retain(|&slot| colours[slot] != colour);
    }
    Ok(())
}

fn unrank_centres(digits: &mut impl Iterator<Item = u32>) -> Vec<usize> {
    let mut colours = vec![5; 24];
    let mut remaining: Vec<usize> = (0..24).collect();
    for colour in 0..5 {
        let mut rank = digits.next().unwrap_or(0);
        let mut positions = [0usize; 4];
        for i in (1..=4).rev() {
            let mut c = i - 1;
            while choose(c + 1, i) <= rank {
                c += 1;
            }
            rank -= choose(c, i);
            positions[i - 1] = c;
        }
        for &p in &positions {
            colours[remaining[p]] = colour;
        }
        remaining = remaining
            .iter()
            .enumerate()
            .filter(|(p, _)| !positions.contains(p))
            .map(|(_, &slot)| slot)
            .collect();
    }
    colours
}

impl Coords {
    /// The digits of the state in the order of [`radices`], without the frame.
    /// `Err` says in plain words why the pieces are not those of a cube.
    pub(super) fn digits(&self, n: usize) -> Result<Vec<u32>, String> {
        if !is_permutation(&self.corner_perm, 8) {
            return Err("the corners are not eight different pieces".into());
        }
        if self.corner_twist.len() != 8 || self.corner_twist.iter().sum::<usize>() % 3 != 0 {
            return Err("the corner twists do not add up to a multiple of three".into());
        }
        let mut out = Vec::new();
        lehmer(&self.corner_perm, 8, &mut out);
        out.push(orientation_value(&self.corner_twist, 3));
        if n % 2 == 1 {
            if !is_permutation(&self.midge_perm, 12) {
                return Err("the edges are not twelve different pieces".into());
            }
            if self.midge_flip.len() != 12 || self.midge_flip.iter().sum::<usize>() % 2 != 0 {
                return Err("the edge flips do not add up to an even number".into());
            }
            if parity(&self.corner_perm) != parity(&self.midge_perm) {
                return Err("the corner and edge permutations have different parity".into());
            }
            lehmer(&self.midge_perm, 10, &mut out);
            out.push(orientation_value(&self.midge_flip, 2));
        }
        for wing in &self.wings {
            if !is_permutation(wing, 24) {
                return Err("the wings are not twenty-four different pieces".into());
            }
            lehmer(wing, 24, &mut out);
        }
        for centre in &self.centres {
            centre_digits(centre, &mut out)?;
        }
        Ok(out)
    }

    /// The inverse of [`Coords::digits`], for digits within their radices.
    pub(super) fn from_digits(n: usize, digits: &[u32]) -> Coords {
        let mut digits = digits.iter().copied();
        let corner_perm = unlehmer(&mut digits, 8, None);
        let corner_twist = orientations(digits.next().unwrap_or(0), 7, 3);
        let mut coords = Coords {
            corner_perm,
            corner_twist,
            ..Coords::default()
        };
        if n % 2 == 1 {
            let wanted = parity(&coords.corner_perm);
            coords.midge_perm = unlehmer(&mut digits, 12, Some(wanted));
            coords.midge_flip = orientations(digits.next().unwrap_or(0), 11, 2);
        }
        coords.wings = (0..(n - 2) / 2)
            .map(|_| unlehmer(&mut digits, 24, None))
            .collect();
        coords.centres = (0..(n - 2) * (n - 2) / 4)
            .map(|_| unrank_centres(&mut digits))
            .collect();
        coords
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_spaces_have_the_bit_sizes_of_the_format() {
        let bits = |n: usize| radices(n).iter().map(|&r| f64::from(r).log2()).sum::<f64>();
        for (n, expected) in [
            (2, 26.39),
            (3, 69.82),
            (4, 156.96),
            (5, 251.90),
            (6, 390.58),
            (7, 537.05),
        ] {
            assert!((bits(n) - expected).abs() < 0.01, "{n}: {}", bits(n));
        }
    }

    #[test]
    fn every_digit_lies_below_its_radix_for_a_scrambled_state() {
        let mut coords = Coords::from_digits(5, &vec![0; radices(5).len()]);
        coords.corner_perm.swap(0, 3);
        coords.midge_perm.swap(0, 3);
        coords.corner_twist[0] = 2;
        coords.corner_twist[1] = 1;
        coords.wings[0].swap(2, 20);
        coords.centres[1].swap(0, 23);
        let digits = coords.digits(5).unwrap();
        let radices = radices(5);
        assert_eq!(digits.len() + 1, radices.len());
        for (digit, radix) in digits.iter().zip(&radices) {
            assert!(digit < radix);
        }
        assert_eq!(Coords::from_digits(5, &digits).digits(5).unwrap(), digits);
    }

    #[test]
    fn faults_are_reported_in_plain_words() {
        let mut coords = Coords::from_digits(3, &vec![0; radices(3).len()]);
        coords.corner_perm.swap(0, 1);
        assert!(coords.digits(3).unwrap_err().contains("parity"));
        coords.corner_twist[0] = 1;
        assert!(coords.digits(3).unwrap_err().contains("twists"));
        coords.corner_perm[0] = 2;
        assert!(coords.digits(3).unwrap_err().contains("corners"));
    }
}
