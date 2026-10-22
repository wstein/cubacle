//! Colours and the options that change how an icon looks.

use crate::puzzle::Face;

/// An opaque sRGB colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub(crate) fn scale(self, factor: f64) -> Rgb {
        let channel = |v: u8| (f64::from(v) * factor).round().clamp(0.0, 255.0) as u8;
        Rgb(channel(self.0), channel(self.1), channel(self.2))
    }

    pub(crate) fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

/// The colour and the letter shown for each face, in `URFDLB` order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    colours: [Rgb; 6],
    letters: [char; 6],
}

impl Palette {
    pub fn new(colours: [Rgb; 6], letters: [char; 6]) -> Palette {
        Palette { colours, letters }
    }

    /// Colours for sticker letters `W R G Y O B`; other letters become grey.
    pub fn from_letters(letters: [char; 6]) -> Palette {
        let colour = |letter: char| match letter.to_ascii_uppercase() {
            'W' => Rgb(0xf2, 0xf2, 0xee),
            'R' => Rgb(0xd6, 0x28, 0x28),
            'G' => Rgb(0x2a, 0x9d, 0x4e),
            'Y' => Rgb(0xf6, 0xc9, 0x0e),
            'O' => Rgb(0xf7, 0x7f, 0x00),
            'B' => Rgb(0x1f, 0x5f, 0xbf),
            _ => Rgb(0x88, 0x88, 0x88),
        };
        Palette::new(letters.map(colour), letters)
    }

    pub(crate) fn colour(&self, face: Face) -> Rgb {
        self.colours[face as usize]
    }

    pub(crate) fn letter(&self, face: Face) -> char {
        self.letters[face as usize]
    }
}

impl Default for Palette {
    /// The western scheme: white up, green front, red right.
    fn default() -> Palette {
        Palette::from_letters(['W', 'R', 'G', 'Y', 'O', 'B'])
    }
}

/// How the moving layer is set apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shading {
    /// Real colours on the moving layer, everything else darkened to 45 %.
    #[default]
    DimRest,
    /// The moving layer turns grey, the rest keeps its colours.
    GreyMoving,
}

/// Everything that is not the cube state or the move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    pub palette: Palette,
    pub shading: Shading,
    /// The height in pixels this icon is meant for. It only sets minimum
    /// widths (gaps, arrows) so small icons stay crisp; vector output scales.
    pub size: u32,
    /// A soft shadow under the cube.
    pub shadow: bool,
    /// A solid backdrop; `None` leaves it transparent.
    pub background: Option<Rgb>,
}

impl Default for Style {
    fn default() -> Style {
        Style {
            palette: Palette::default(),
            shading: Shading::default(),
            size: 96,
            shadow: true,
            background: None,
        }
    }
}

/// Fixed brightness per visible face, giving the cube its depth.
pub(crate) fn face_light(face: Face) -> f64 {
    match face {
        Face::U => 1.0,
        Face::F => 0.84,
        _ => 0.68,
    }
}

pub(crate) const DIM: f64 = 0.45;
pub(crate) const GREY: Rgb = Rgb(0x9a, 0xa0, 0xa6);
pub(crate) const BODY: Rgb = Rgb(0x0e, 0x10, 0x14);
pub(crate) const ARROW_RED: Rgb = Rgb(0xe0, 0x20, 0x2a);
pub(crate) const ARROW_CYAN: Rgb = Rgb(0x2f, 0xc4, 0xf2);
pub(crate) const WHITE: Rgb = Rgb(0xff, 0xff, 0xff);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_and_mix_round_and_clamp() {
        assert_eq!(Rgb(200, 100, 0).scale(0.45), Rgb(90, 45, 0));
        assert_eq!(Rgb(200, 100, 0).scale(2.0), Rgb(255, 200, 0));
        assert_eq!(Rgb(1, 2, 255).hex(), "#0102ff");
    }

    #[test]
    fn letters_pick_colours() {
        let palette = Palette::default();
        assert_eq!(palette.colour(Face::U), Rgb(0xf2, 0xf2, 0xee));
        assert_eq!(palette.letter(Face::F), 'G');
        assert_eq!(
            Palette::from_letters(['?'; 6]).colour(Face::R),
            Rgb(0x88, 0x88, 0x88)
        );
    }
}
