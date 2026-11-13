//! The twists a face center of a 3x3x3 picture cube may show in a target.
//!
//! One mask per center in `URFDLB` order; bit `k` of an entry set means the
//! center may show twist `k` (see [`super::CenterTwists`] for the convention).

use super::twists::TwistError;

/// The twists each center accepts: bit `k` of entry `f` set means the center
/// `f` may show twist `k`.
pub type TwistMasks = [u8; 6];

/// Every twist accepted on every center.
pub const FREE_TWIST: TwistMasks = [0b1111; 6];

/// Reads a twist condition: six items separated by commas, each a twist 0 to
/// 3, several joined with `|` (`0|2`), or `X` or `?` for any: `0,X,2,X,X,X`.
pub fn parse_twist_masks(text: &str) -> Result<TwistMasks, TwistError> {
    let bad = |why: &str| {
        TwistError::Syntax(format!(
            "a twist condition is six items for U,R,F,D,L,B separated by commas, each 0 to 3 (several joined by |) or X for any, like 0,X,2|3,X,X,X ({why}; got {text:?})"
        ))
    };
    let parts: Vec<&str> = text.split(',').map(str::trim).collect();
    if parts.len() != 6 {
        return Err(bad("not six items"));
    }
    let mut masks = [0u8; 6];
    for (slot, part) in masks.iter_mut().zip(&parts) {
        if matches!(*part, "X" | "?") {
            *slot = 0b1111;
            continue;
        }
        for item in part.split('|') {
            match item.trim() {
                "0" => *slot |= 1,
                "1" => *slot |= 2,
                "2" => *slot |= 4,
                "3" => *slot |= 8,
                _ => return Err(bad("an item is no twist")),
            }
        }
    }
    Ok(masks)
}

/// The text form of a twist condition, the inverse of [`parse_twist_masks`].
pub fn twist_masks_text(masks: &TwistMasks) -> String {
    masks
        .iter()
        .map(|&mask| {
            if mask & 0b1111 == 0b1111 {
                "X".to_owned()
            } else {
                (0..4)
                    .filter(|k| mask >> k & 1 == 1)
                    .map(|k| k.to_string())
                    .collect::<Vec<_>>()
                    .join("|")
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}
