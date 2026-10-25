//! Whether a cube state fits a template.
//!
//! A state matches when some assignment of colors to the placeholders makes
//! every sticker agree with its slot and the guard true. A placeholder is one
//! color on all its stickers, so the color of its first sticker is the only
//! candidate: there is nothing to search. One pass over the stickers fixes the
//! assignment or finds a disagreement, then the guard is evaluated once.

use crate::puzzle::Facelets;

use super::schema::Slot;
use super::Template;

/// Marks a placeholder that has not been seen yet.
const UNSET: u8 = u8::MAX;

impl Template {
    /// Whether `cube` fits the template. A cube of another size never does.
    pub fn matches(&self, cube: &Facelets) -> bool {
        if cube.kind() != self.kind {
            return false;
        }
        let mut colors = vec![UNSET; self.names.len()];
        for (slot, &face) in self.slots.iter().zip(cube.stickers()) {
            match *slot {
                Slot::Any => {}
                Slot::Fixed(wanted) => {
                    if wanted != face {
                        return false;
                    }
                }
                Slot::Var(var) => {
                    let color = face.index() as u8;
                    match colors[var] {
                        UNSET => colors[var] = color,
                        seen if seen != color => return false,
                        _ => {}
                    }
                }
            }
        }
        self.guard.holds(&colors)
    }
}
