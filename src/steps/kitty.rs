//! The Kitty rows of `solve --icons` and `recover --icons`: phase blocks of
//! icon rows with the picture of the cube after each phase in the row flow.

use cubacle::phases::Phase;
use cubacle::puzzle::{Facelets, MoveSequence};
use cubacle::render::{icon, Action, Icon, Raster, Style};

use super::{groups, heading_at, lone_quarters, row_icon, token_line, Pictures, SolutionIcons};
use crate::icons::numbered;
use crate::rows;
use crate::terminal::Terminal;
use crate::views::{kitty_block, kitty_state};

/// The phase blocks of a solution as Kitty rows; with `with_start` the start
/// picture is the first item of the first row.
pub(super) fn kitty_steps(
    pictures: &Pictures<'_>,
    start: &Facelets,
    solution: &MoveSequence,
    found: &[Phase],
    mut with_start: bool,
) -> Result<String, String> {
    let (options, terminal, style) = (pictures.options, &pictures.terminal, &pictures.style);
    let collapse_empty = pictures.collapse_empty;
    let total = solution.len();
    let moves = solution.moves();
    let mut state = start.clone();
    let mut out = String::new();
    // all icons have one size; the first one tells the width
    let width = match moves.first() {
        Some(first) => {
            icon(&state, &Action::Layer(*first), style)
                .map_err(|error| error.to_string())?
                .to_rgba(style.size)
                .width
        }
        None => 0,
    };
    let lone = lone_quarters(moves);
    let per_row = rows::per_row(terminal.width_px, width, rows::GAP, options.columns);
    // Without phases (a route that is not from solver stages) the moves form
    // one segment that has no heading and no state picture.
    let whole = [Phase {
        number: 0,
        name: "",
        moves: 0..total,
    }];
    let named = !found.is_empty();
    let phases = if named { found } else { &whole };
    let mut blank = false;
    for (at, phase) in phases.iter().enumerate() {
        if named {
            if let Some(heading) = heading_at(phases, at, collapse_empty, total) {
                if std::mem::take(&mut blank) {
                    out.push('\n');
                }
                out.push_str(&heading);
                out.push('\n');
            }
        }
        if phase.moves.is_empty() {
            continue;
        }
        // the items of the phase in row order: the start picture before the
        // first moves, the icons, the picture of the state after the phase
        if std::mem::take(&mut blank) {
            out.push('\n');
        }
        let mut items: Vec<Item> = Vec::new();
        if with_start {
            with_start = false;
            items.push(state_item(start, options, style, width, per_row, terminal));
        }
        for at in phase.moves.clone() {
            let action = Action::Layer(moves[at]);
            let drawn = icon(&state, &action, style).map_err(|error| error.to_string())?;
            let drawn = numbered(drawn, at + 1, options.numbers, style.backdrop);
            let raster = row_icon(drawn.to_rgba(style.size), lone[at]);
            items.push(Item::cell(raster, 1));
            state = action.apply(&state).map_err(|error| error.to_string())?;
        }
        if named {
            items.push(state_item(&state, options, style, width, per_row, terminal));
        }
        for row in pack(items, per_row) {
            match row.as_slice() {
                [Item {
                    block: Some(text), ..
                }] => out.push_str(text),
                _ => {
                    let rasters: Vec<Raster> = row.into_iter().filter_map(|i| i.raster).collect();
                    let strip = Raster::hstack(&rasters, rows::GAP);
                    out.push_str(&kitty_block(&strip, terminal.cell.1));
                }
            }
        }
        if named {
            // the phase block ends with its token on a line of its own, and
            // one blank line separates it from the next block
            out.push_str(&token_line(phase, &state)?);
            blank = true;
        }
    }
    Ok(out)
}

/// One thing in a row of the Kitty output: a move icon (one cell), or a
/// picture of the cube that is as wide as `cells` icons and the gaps between
/// them. A picture that is too
/// wide for a row is `block`, drawn as it is on rows of its own.
struct Item {
    cells: usize,
    raster: Option<Raster>,
    block: Option<String>,
}

impl Item {
    fn cell(raster: Raster, cells: usize) -> Item {
        Item {
            cells,
            raster: Some(raster),
            block: None,
        }
    }
}

/// The picture of the cube `state`, as tall as a move icon, taking as many
/// icon cells of the row as its width needs (two for the folded view); it has
/// no frame.
fn state_item(
    state: &Facelets,
    options: &SolutionIcons,
    style: &Style,
    cell: u32,
    per_row: usize,
    terminal: &Terminal,
) -> Item {
    let drawn: Vec<Icon> = options
        .views
        .icons(state, None, style)
        .into_iter()
        .map(Icon::unframed)
        .collect();
    // as tall as a move icon with its label; the width follows from the shape
    let rasters: Vec<Raster> = drawn.iter().map(|d| d.to_rgba(style.size)).collect();
    let natural = Raster::hstack(&rasters, rows::GAP);
    // the row cells it needs, rounded up
    let cells = (natural.width + rows::GAP).div_ceil(cell + rows::GAP) as usize;
    if cells > per_row {
        let text = kitty_state(&drawn, style, terminal, options.columns);
        return Item {
            cells: per_row,
            raster: None,
            block: Some(text),
        };
    }
    // centred in the cells it takes
    let slot = cells as u32 * cell + (cells as u32 - 1) * rows::GAP;
    let left = (slot - natural.width) / 2;
    let right = slot - natural.width - left;
    let padded = Raster::hstack(&[groups::spacer(left), natural, groups::spacer(right)], 0);
    Item::cell(padded, cells)
}

/// Fills rows with at most `per_row` cells; a picture drawn as a block has a
/// row of its own.
fn pack(items: Vec<Item>, per_row: usize) -> Vec<Vec<Item>> {
    let mut rows: Vec<Vec<Item>> = Vec::new();
    let mut used = 0;
    for item in items {
        let alone = item.block.is_some();
        let room = used > 0 && !alone && used + item.cells <= per_row;
        used = if alone {
            per_row
        } else if room {
            used + item.cells
        } else {
            item.cells
        };
        match rows.last_mut() {
            Some(row) if room => row.push(item),
            _ => rows.push(vec![item]),
        }
    }
    rows
}
