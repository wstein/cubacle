//! Embedded glyph outlines of the label font.
//!
//! The module holds polygons for `0123456789URFDLBwx'` only, derived from
//! Quicksand under the SIL Open Font License 1.1 (see `NOTICE.md` and
//! `licenses/`).

mod quicksand;

/// A character, its advance and its filled polygons, in glyph units: the cap
/// height is 1.4 and `y` points down from the cap line.
pub(crate) type Glyph = (char, f64, &'static [&'static [[f64; 2]]]);

pub(crate) use quicksand::GLYPHS as QUICKSAND;
