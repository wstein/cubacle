//! Origin: Rust re-implementation of the base64url integer arithmetic of
//! flix-orbit64 (`Orbit64.Internal.Encoding`, Apache-2.0, Werner Stein).
//!
//! A token is one integer of up to 540 bits. Every digit of its mixed-radix
//! form has a radix below 2^32, so a small unsigned big integer with
//! multiply-add and divide-by-small is all the codec needs.

use super::ALPHABET;

/// An unsigned integer as little-endian 32-bit limbs, without leading zero limbs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Big {
    limbs: Vec<u32>,
}

impl Big {
    pub(super) fn zero() -> Big {
        Big { limbs: Vec::new() }
    }

    pub(super) fn one() -> Big {
        Big { limbs: vec![1] }
    }

    pub(super) fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }

    /// `self = self * factor + add`.
    pub(super) fn mul_add(&mut self, factor: u32, add: u32) {
        let mut carry = u64::from(add);
        for limb in &mut self.limbs {
            let value = u64::from(*limb) * u64::from(factor) + carry;
            *limb = value as u32;
            carry = value >> 32;
        }
        if carry != 0 {
            self.limbs.push(carry as u32);
        }
    }

    /// `self = self / divisor`, returning the remainder. `divisor` is not zero.
    pub(super) fn div_rem(&mut self, divisor: u32) -> u32 {
        let mut rest = 0u64;
        for limb in self.limbs.iter_mut().rev() {
            let value = (rest << 32) | u64::from(*limb);
            *limb = (value / u64::from(divisor)) as u32;
            rest = value % u64::from(divisor);
        }
        while self.limbs.last() == Some(&0) {
            self.limbs.pop();
        }
        rest as u32
    }

    /// `self - 1`; a zero stays zero.
    fn decrement(&mut self) {
        for limb in &mut self.limbs {
            if *limb == 0 {
                *limb = u32::MAX;
            } else {
                *limb -= 1;
                break;
            }
        }
        while self.limbs.last() == Some(&0) {
            self.limbs.pop();
        }
    }

    fn bit_len(&self) -> usize {
        match self.limbs.last() {
            None => 0,
            Some(top) => 32 * (self.limbs.len() - 1) + (32 - top.leading_zeros() as usize),
        }
    }

    /// Whether `self <= 2^bits`.
    pub(super) fn at_most_power_of_two(&self, bits: usize) -> bool {
        let mut below = self.clone();
        below.decrement();
        below.bit_len() <= bits
    }

    /// Reads base64url text; `Err` is the position of the first foreign byte.
    pub(super) fn from_base64(text: &str) -> Result<Big, (usize, char)> {
        let mut value = Big::zero();
        for (position, found) in text.chars().enumerate() {
            let digit = u8::try_from(found)
                .ok()
                .and_then(|byte| ALPHABET.iter().position(|&known| known == byte))
                .ok_or((position, found))?;
            value.mul_add(64, digit as u32);
        }
        Ok(value)
    }

    /// Writes exactly `width` characters, or `None` when the value needs more.
    pub(super) fn to_base64(&self, width: usize) -> Option<String> {
        let mut rest = self.clone();
        let mut out = vec![ALPHABET[0]; width];
        for slot in out.iter_mut().rev() {
            *slot = ALPHABET[rest.div_rem(64) as usize];
        }
        rest.is_zero()
            .then(|| out.into_iter().map(char::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mul_add_and_div_rem_are_inverse() {
        let mut value = Big::zero();
        let radices = [7u32, 4_000_000_000, 3, 2_187, 10_626];
        let digits = [5u32, 123_456_789, 2, 2_000, 10_000];
        for (radix, digit) in radices.iter().zip(digits) {
            value.mul_add(*radix, digit);
        }
        for (radix, digit) in radices.iter().zip(digits).rev() {
            assert_eq!(value.div_rem(*radix), digit);
        }
        assert!(value.is_zero());
    }

    #[test]
    fn base64_round_trips_with_leading_zeros() {
        let value = Big::from_base64("AAB-_").unwrap();
        assert_eq!(value.to_base64(5).as_deref(), Some("AAB-_"));
        assert_eq!(value.to_base64(2), None);
        assert_eq!(Big::from_base64("AA+"), Err((2, '+')));
    }

    #[test]
    fn power_of_two_bound_is_inclusive() {
        let mut value = Big::one();
        for _ in 0..40 {
            value.mul_add(2, 0);
        }
        assert!(value.at_most_power_of_two(40));
        assert!(!value.at_most_power_of_two(39));
        value.mul_add(1, 1);
        assert!(!value.at_most_power_of_two(40));
    }
}
