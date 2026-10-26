//! Colors and the options that change how an icon looks.

use crate::puzzle::Face;

/// An opaque sRGB color.
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

/// The color and the letter shown for each face, in `URFDLB` order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    colors: [Rgb; 6],
    letters: [char; 6],
}

impl Palette {
    pub fn new(colors: [Rgb; 6], letters: [char; 6]) -> Palette {
        Palette { colors, letters }
    }

    /// Bright, saturated tile colors for sticker letters `W R G Y O B`; other letters become grey.
    pub fn from_letters(letters: [char; 6]) -> Palette {
        let color = |letter: char| match letter.to_ascii_uppercase() {
            'W' => Rgb(0xff, 0xff, 0xff),
            'R' => Rgb(0xe8, 0x12, 0x1e),
            'G' => Rgb(0x00, 0xc2, 0x4a),
            'Y' => Rgb(0xff, 0xe0, 0x00),
            'O' => Rgb(0xff, 0x7a, 0x00),
            'B' => Rgb(0x00, 0x5c, 0xff),
            _ => Rgb(0x96, 0x96, 0x96),
        };
        Palette::new(letters.map(color), letters)
    }

    pub(crate) fn color(&self, face: Face) -> Rgb {
        self.colors[face as usize]
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
    /// Real colors on the moving layer, everything else darkened (see [`Backdrop`]).
    #[default]
    DimRest,
    /// The moving layer turns grey, the rest keeps its colors.
    GreyMoving,
}

/// The backdrop the icon will be seen on. It decides how strongly the resting
/// pieces are dimmed and the color of the notation: light glyphs on a dark
/// backdrop, dark glyphs on a light one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Backdrop {
    #[default]
    Dark,
    Light,
}

impl Backdrop {
    /// The brightness of the pieces that do not turn.
    pub(crate) fn dimming(self) -> f64 {
        match self {
            Backdrop::Dark => 0.25,
            Backdrop::Light => 0.40,
        }
    }

    pub(crate) fn ink(self) -> Rgb {
        match self {
            Backdrop::Dark => Rgb(0xf4, 0xf4, 0xf4),
            Backdrop::Light => Rgb(0x14, 0x16, 0x1a),
        }
    }
}

/// Everything that is not the cube state or the move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    pub palette: Palette,
    pub shading: Shading,
    /// The height in pixels this icon is meant for. It only sets minimum
    /// widths (gaps, arrows) so small icons stay crisp; vector output scales.
    pub size: u32,
    /// A solid backdrop color; `None` leaves it transparent.
    pub background: Option<Rgb>,
    /// What the icon will be seen on; see [`Backdrop`].
    pub backdrop: Backdrop,
}

impl Default for Style {
    fn default() -> Style {
        Style {
            palette: Palette::default(),
            shading: Shading::default(),
            size: 96,
            background: None,
            backdrop: Backdrop::default(),
        }
    }
}

/// Fixed brightness per visible face, giving the cube its depth.
pub(crate) fn face_light(face: Face) -> f64 {
    match face {
        Face::U | Face::D => 1.0,
        Face::F => 0.93,
        _ => 0.82,
    }
}

/// The darkening of resting stickers in the letter grid.
pub(crate) const DIM: f64 = 0.45;
pub(crate) const GREY: Rgb = Rgb(0x9a, 0xa0, 0xa6);
pub(crate) const BODY: Rgb = Rgb(0x0e, 0x10, 0x14);
pub(crate) const ARROW_VIOLET: Rgb = Rgb(0x93, 0x33, 0xea);
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
    fn letters_pick_colors() {
        let palette = Palette::default();
        assert_eq!(palette.color(Face::U), Rgb(0xff, 0xff, 0xff));
        assert_eq!(palette.letter(Face::F), 'G');
        assert_eq!(
            Palette::from_letters(['?'; 6]).color(Face::R),
            Rgb(0x96, 0x96, 0x96)
        );
    }
}
