//! Whole-cube pictures on a graphics terminal: one state seen from the
//! Up-Right-Front corner and from the opposite Down-Left-Back corner, side by
//! side in one image when they fit the row.

use cubacle::render::{Icon, KittyPlacement, Raster, Style};

use crate::rows;
use crate::terminal::Terminal;

/// The corners a whole-cube picture is seen from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Views {
    /// Up-Right-Front and the opposite Down-Left-Back corner, so that all six faces show.
    #[default]
    Both,
    /// Up-Right-Front only, as the move icons.
    Urf,
}

impl Views {
    /// How many of the two pictures of a state are drawn.
    pub(crate) fn count(self) -> usize {
        match self {
            Views::Both => 2,
            Views::Urf => 1,
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
    pair: &[Icon; 2],
    style: &Style,
    terminal: &Terminal,
    views: Views,
    columns: Option<usize>,
) -> String {
    let rasters: Vec<Raster> = pair
        .iter()
        .take(views.count())
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

    #[test]
    fn two_views_share_a_row_only_when_they_fit_and_columns_allow() {
        // 2 * 167 + 8 pixels
        assert!(pair_fits(342, 342, None));
        assert!(!pair_fits(341, 342, None));
        assert!(pair_fits(1000, 342, Some(2)));
        assert!(!pair_fits(1000, 342, Some(1)));
    }

    #[test]
    fn one_view_or_two() {
        assert_eq!(Views::Both.count(), 2);
        assert_eq!(Views::Urf.count(), 1);
        assert_eq!(Views::default(), Views::Both);
    }
}
