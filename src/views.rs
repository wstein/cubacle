//! Whole-cube pictures on a graphics terminal: one state seen from the
//! Up-Right-Front corner and from the opposite Down-Left-Back corner, or as a
//! flat net, side by side in one image when they fit the row.

use cubacle::puzzle::Facelets;
use cubacle::render::{
    state_icon_net, state_icon_net_against, state_icon_pair, state_icon_pair_against, Icon,
    KittyPlacement, Raster, Style,
};

use crate::rows;
use crate::terminal::Terminal;

/// The pictures a whole-cube state is drawn as.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Views {
    /// Up-Right-Front and the opposite Down-Left-Back corner, so that all six faces show.
    #[default]
    Both,
    /// Up-Right-Front only, as the move icons.
    Urf,
    /// The flat net: U above F, L F R B in a row, D below; every sticker once, no projection.
    Net,
    /// The net and the two corners.
    All,
}

impl Views {
    /// Whether every sticker of the cube is in one of the pictures.
    pub(crate) fn shows_every_face(self) -> bool {
        self != Views::Urf
    }

    /// The pictures of `state`, with amber marks on the stickers that differ
    /// from `reference` when there is one.
    pub(crate) fn icons(
        self,
        state: &Facelets,
        reference: Option<&Facelets>,
        style: &Style,
    ) -> Vec<Icon> {
        let pair = || match reference {
            Some(r) => state_icon_pair_against(state, r, style),
            None => state_icon_pair(state, style),
        };
        let net = || match reference {
            Some(r) => state_icon_net_against(state, r, style),
            None => state_icon_net(state, style),
        };
        match self {
            Views::Both => pair().into(),
            Views::Urf => {
                let [urf, _] = pair();
                vec![urf]
            }
            Views::Net => vec![net()],
            Views::All => {
                let [urf, dbl] = pair();
                vec![net(), urf, dbl]
            }
        }
    }
}

/// Whether the pictures of a state go side by side in one image: they must
/// fit the terminal width and `--columns` must allow two icons in a row.
fn pair_fits(width_px: u32, pair_px: u32, columns: Option<usize>) -> bool {
    columns.is_none_or(|c| c >= 2) && pair_px <= width_px
}

/// The pictures of one state on a graphics terminal: one image with the views
/// side by side, or one image per view when they do not fit a row.
pub(crate) fn kitty_state(
    icons: &[Icon],
    style: &Style,
    terminal: &Terminal,
    columns: Option<usize>,
) -> String {
    let rasters: Vec<Raster> = icons
        .iter()
        .map(|drawn| drawn.to_rgba(style.size))
        .collect();
    let joined = Raster::hstack(&rasters, rows::GAP);
    if rasters.len() < 2 || pair_fits(terminal.width_px, joined.width, columns) {
        return kitty_block(&joined, terminal.cell.1);
    }
    rasters
        .iter()
        .map(|raster| kitty_block(raster, terminal.cell.1))
        .collect()
}

/// A raster as a block of terminal rows: the rows are cleared so that older
/// text cannot show through, then the image is drawn at its own pixel size
/// without moving the cursor, which then steps below it.
pub(crate) fn kitty_block(raster: &Raster, cell_height: u32) -> String {
    let rows = raster.height.div_ceil(cell_height.max(1)).max(1);
    let placement = KittyPlacement {
        rows: None,
        keep_cursor: true,
    };
    let mut text = "\x1b[2K\n".repeat(rows as usize);
    text.push_str(&format!("\x1b[{rows}A"));
    text.push_str(&raster.to_kitty(&placement));
    text.push_str(&format!("\x1b[{rows}B\r"));
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::Terminal;

    #[test]
    fn two_views_share_a_row_only_when_they_fit_and_columns_allow() {
        // 2 * 167 + 8 pixels
        assert!(pair_fits(342, 342, None));
        assert!(!pair_fits(341, 342, None));
        assert!(pair_fits(1000, 342, Some(2)));
        assert!(!pair_fits(1000, 342, Some(1)));
    }

    #[test]
    fn the_views_draw_one_to_three_pictures() {
        let cube = Facelets::solved(cubacle::puzzle::PuzzleKind::Cube3);
        let style = Style::default();
        let count = |v: Views| v.icons(&cube, None, &style).len();
        assert_eq!(
            [
                count(Views::Both),
                count(Views::Urf),
                count(Views::Net),
                count(Views::All)
            ],
            [2, 1, 1, 3]
        );
        assert_eq!(Views::default(), Views::Both);
        assert!(!Views::Urf.shows_every_face() && Views::Net.shows_every_face());
    }

    #[test]
    fn narrow_terminals_stack_the_net_and_the_corners() {
        let cube = Facelets::solved(cubacle::puzzle::PuzzleKind::Cube3);
        let style = Style::default();
        let icons = Views::All.icons(&cube, None, &style);
        let wide = Terminal {
            width_px: 5000,
            cell: (8, 16),
            background: None,
        };
        let narrow = Terminal {
            width_px: 200,
            cell: (8, 16),
            background: None,
        };
        let images = |t: &Terminal, columns| {
            kitty_state(&icons, &style, t, columns)
                .matches("a=T")
                .count()
        };
        assert_eq!(images(&wide, None), 1);
        assert_eq!(images(&wide, Some(1)), 3);
        assert_eq!(images(&narrow, None), 3);
        let one = Views::Net.icons(&cube, None, &style);
        assert_eq!(
            kitty_state(&one, &style, &narrow, None)
                .matches("a=T")
                .count(),
            1
        );
    }
}
