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
            Backdrop::Dark => 0.325,
            Backdrop::Light => 0.46,
        }
    }

    pub(crate) fn ink(self) -> Rgb {
        match self {
            Backdrop::Dark => Rgb(0xf4, 0xf4, 0xf4),
            Backdrop::Light => Rgb(0x14, 0x16, 0x1a),
        }
    }

    /// The rotation axis of a whole-cube rotation: near white on a dark
    /// backdrop, dark slate on a light one.
    pub(crate) fn axis_ink(self) -> Rgb {
        match self {
            Backdrop::Dark => Rgb(0xe6, 0xeb, 0xf2),
            Backdrop::Light => Rgb(0x4a, 0x54, 0x68),
        }
    }

    /// The orbit ring of a whole-cube rotation: a mid tone that is at least
    /// 3:1 against the backdrop.
    pub(crate) fn ring_ink(self) -> Rgb {
        match self {
            Backdrop::Dark => Rgb(0x9a, 0xa4, 0xb2),
            Backdrop::Light => Rgb(0x6b, 0x75, 0x87),
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
    #[test]
    fn inactive_layers_reduce_darkening_by_ten_percent() {
        use super::Backdrop;
        assert!((Backdrop::Dark.dimming() - 0.325).abs() < 1e-12);
        assert!((Backdrop::Light.dimming() - 0.46).abs() < 1e-12);
    }

    use super::*;

    #[test]
    fn scale_and_mix_round_and_clamp() {
        assert_eq!(Rgb(200, 100, 0).scale(0.45), Rgb(90, 45, 0));
        assert_eq!(Rgb(200, 100, 0).scale(2.0), Rgb(255, 200, 0));
        assert_eq!(Rgb(1, 2, 255).hex(), "#0102ff");
    }

    fn luminance(c: Rgb) -> f64 {
        let lin = |v: u8| {
            let s = f64::from(v) / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(c.0) + 0.7152 * lin(c.1) + 0.0722 * lin(c.2)
    }

    fn contrast(a: Rgb, b: Rgb) -> f64 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// The axis and the ring of a rotation icon are at least 3:1 against the
    /// backdrops they are meant for: pure black and a dark panel, pure white and
    /// a light panel.
    #[test]
    fn the_axis_and_the_ring_read_on_their_backdrop() {
        let dark = [Rgb(0, 0, 0), Rgb(0x1c, 0x1e, 0x24)];
        let light = [Rgb(0xff, 0xff, 0xff), Rgb(0xf4, 0xf4, 0xf4)];
        for (backdrop, grounds) in [(Backdrop::Dark, dark), (Backdrop::Light, light)] {
            for ground in grounds {
                for (name, ink) in [("axis", backdrop.axis_ink()), ("ring", backdrop.ring_ink())] {
                    let ratio = contrast(ink, ground);
                    assert!(
                        ratio >= 3.0,
                        "{name} on {backdrop:?} {}: contrast {ratio:.2}",
                        ground.hex()
                    );
                }
            }
        }
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
