//! The frame of a state icon: four thin strips along the edge of the canvas.

use super::scene::{Part, Shape, HEIGHT, WIDTH};
use super::style::Backdrop;

/// Width of the line in canvas units, about 1.5 pixels at the default size.
const LINE: f64 = 1.6;
/// How much of the line is see-through, so the frame stays quiet.
const OPACITY: f64 = 0.7;

fn strip(x0: f64, y0: f64, x1: f64, y1: f64, backdrop: Backdrop) -> Shape {
    let mut shape = Shape::new(
        Part::Frame,
        vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]],
        backdrop.ink(),
    );
    shape.opacity = OPACITY;
    shape
}

/// The frame in the ink color of the backdrop.
pub(crate) fn frame(backdrop: Backdrop) -> Vec<Shape> {
    vec![
        strip(0.0, 0.0, WIDTH, LINE, backdrop),
        strip(0.0, HEIGHT - LINE, WIDTH, HEIGHT, backdrop),
        strip(0.0, LINE, LINE, HEIGHT - LINE, backdrop),
        strip(WIDTH - LINE, LINE, WIDTH, HEIGHT - LINE, backdrop),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{Facelets, PuzzleKind};
    use crate::render::{icon, state_icon, Action, Style};

    #[test]
    fn a_state_icon_has_every_sticker_bright_and_no_arrow_or_label() {
        let cube = Facelets::solved(PuzzleKind::Cube3);
        let plain = state_icon(&cube, &Style::default());
        let svg = plain.to_svg();
        // 3 faces * 9 stickers + 3 body plates + 4 frame strips
        assert_eq!(svg.matches("<polygon").count(), 27 + 3 + 4);
        assert_eq!(plain.label(), "state");
        assert_eq!(plain.direction(), None);
        let moving = icon(&cube, &"R".parse::<Action>().unwrap(), &Style::default()).unwrap();
        // the resting stickers of a move icon are darker than in the state icon
        assert_ne!(svg, moving.to_svg());
        assert!(svg.contains("#ffffff"), "the white center keeps full color");
        // white on the Up face, dimmed to 25 percent in the move icon only
        assert!(moving.to_svg().contains("#404040"));
        assert!(!svg.contains("#404040"));
    }

    #[test]
    fn the_frame_follows_the_backdrop_and_the_rows_line_up() {
        let cube = Facelets::solved(PuzzleKind::Cube4);
        let dark = state_icon(&cube, &Style::default());
        let light = state_icon(
            &cube,
            &Style {
                backdrop: Backdrop::Light,
                ..Style::default()
            },
        );
        assert!(dark.to_svg().contains("#f4f4f4"));
        assert!(light.to_svg().contains("#14161a"));
        let moving = icon(&cube, &"R".parse::<Action>().unwrap(), &Style::default()).unwrap();
        let (a, b) = (dark.to_rgba(96), moving.to_rgba(96));
        assert_eq!((a.width, a.height), (b.width, b.height));
    }
}
