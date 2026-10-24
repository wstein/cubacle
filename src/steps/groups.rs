//! Icons in hand units: `moves --icons` with a `·` or a `( )` in the input.
//!
//! The moves of a group stay separate icons, close together; groups are
//! separated by a larger gap with a thin bracket and the group's number above.
//! A parenthesised block of groups gets an outer, thicker bracket. Rows break
//! between groups; a group wider than a row is cut as the last resort.

use std::ops::Range;

use cubacle::notation::{Group, Item};
use cubacle::puzzle::{Facelets, Move, MoveSequence};
use cubacle::render::{brackets, icon, Action, BlockSpan, Raster, UnitSpan};

use super::{lone_quarters, row_icon, Pictures};
use crate::icons::{numbered, show, Graphics};
use crate::views::kitty_block;

/// Pixels between two icons of one group.
const GAP_IN: u32 = 2;
/// Pixels between two groups of one block.
const GAP_UNIT: u32 = 16;
/// Pixels between two blocks.
const GAP_BLOCK: u32 = 28;

/// The hand units of a move list, as ranges of move indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    blocks: Vec<BlockRange>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BlockRange {
    /// Drawn under an outer bracket (it came from parentheses).
    outer: bool,
    units: Vec<Range<usize>>,
}

impl Layout {
    pub fn of(tree: &Group) -> Layout {
        let mut at = 0;
        let mut blocks = Vec::new();
        for item in &tree.items {
            let Item::Group(group) = item else { continue };
            let mut units = Vec::new();
            if group.is_unit() {
                units.push(at..at + group.items.len());
                at += group.items.len();
            } else {
                for inner in &group.items {
                    if let Item::Group(unit) = inner {
                        let len = unit.flatten().len();
                        units.push(at..at + len);
                        at += len;
                    }
                }
            }
            if !units.is_empty() {
                blocks.push(BlockRange {
                    outer: !group.is_unit(),
                    units,
                });
            }
        }
        Layout { blocks }
    }

    /// The number of groups.
    pub fn units(&self) -> usize {
        self.blocks.iter().map(|b| b.units.len()).sum()
    }

    fn has_outer(&self) -> bool {
        self.blocks.iter().any(|b| b.outer)
    }

    /// The moves with ` · ` between groups and a block in parentheses.
    pub fn line(&self, moves: &[Move], write: &dyn Fn(&Move) -> String) -> String {
        let words = |unit: &Range<usize>| {
            moves[unit.clone()]
                .iter()
                .map(write)
                .collect::<Vec<_>>()
                .join(" ")
        };
        let blocks: Vec<String> = self
            .blocks
            .iter()
            .map(|b| {
                let inner: Vec<String> = b.units.iter().map(words).collect();
                if b.outer {
                    format!("({})", inner.join(" · "))
                } else {
                    inner.join(" · ")
                }
            })
            .collect();
        blocks.join(" · ")
    }
}

/// A run of moves of one group that sits in a row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Piece {
    /// Index of the group, from 0.
    pub unit: usize,
    pub moves: Range<usize>,
    /// The piece starts its group (otherwise the group goes on from the row before).
    pub first: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RowBlock {
    id: usize,
    outer: bool,
    pub pieces: Vec<Piece>,
}

/// What a row can hold.
#[derive(Clone, Copy)]
pub(super) struct Fit {
    /// Pixel width of one icon.
    pub icon: u32,
    /// Pixel width of the row.
    pub width: u32,
    /// Most icons in a row (`--columns`).
    pub icons: usize,
}

impl Fit {
    fn piece_width(&self, n: usize) -> u32 {
        let n = u32::try_from(n).unwrap_or(u32::MAX);
        n * self.icon + n.saturating_sub(1) * GAP_IN
    }

    fn row_width(&self, row: &[RowBlock]) -> u32 {
        let mut width = 0;
        for (b, block) in row.iter().enumerate() {
            if b > 0 {
                width += GAP_BLOCK;
            }
            for (p, piece) in block.pieces.iter().enumerate() {
                if p > 0 {
                    width += GAP_UNIT;
                }
                width += self.piece_width(piece.moves.len());
            }
        }
        width
    }

    fn fits(&self, row: &[RowBlock]) -> bool {
        let icons: usize = row
            .iter()
            .flat_map(|b| &b.pieces)
            .map(|p| p.moves.len())
            .sum();
        icons <= self.icons && self.row_width(row) <= self.width
    }
}

/// `block` added to `row`; fragments of one block in a row join up.
fn joined(mut row: Vec<RowBlock>, block: RowBlock) -> Vec<RowBlock> {
    match row.last_mut() {
        Some(last) if last.id == block.id => last.pieces.extend(block.pieces),
        _ => row.push(block),
    }
    row
}

struct Packer {
    fit: Fit,
    rows: Vec<Vec<RowBlock>>,
    current: Vec<RowBlock>,
}

impl Packer {
    fn flush(&mut self) {
        if !self.current.is_empty() {
            self.rows.push(std::mem::take(&mut self.current));
        }
    }

    /// Puts `block` in the current row, or in a new one when it does not fit.
    fn place(&mut self, block: &RowBlock) -> bool {
        let trial = joined(self.current.clone(), block.clone());
        if self.fit.fits(&trial) {
            self.current = trial;
            return true;
        }
        if !self.current.is_empty() {
            self.flush();
            let trial = vec![block.clone()];
            if self.fit.fits(&trial) {
                self.current = trial;
                return true;
            }
        }
        false
    }
}

/// Cuts the groups into rows. Whole blocks stay together when they fit a row,
/// else groups do; a group wider than a row is cut into icons last.
pub(super) fn pack(layout: &Layout, fit: Fit) -> Vec<Vec<RowBlock>> {
    let mut packer = Packer {
        fit,
        rows: Vec::new(),
        current: Vec::new(),
    };
    let per_row = {
        let by_width = (fit.width + GAP_IN) / (fit.icon + GAP_IN).max(1);
        fit.icons.min(by_width as usize).max(1)
    };
    let mut unit = 0;
    for (id, block) in layout.blocks.iter().enumerate() {
        let pieces: Vec<Piece> = block
            .units
            .iter()
            .enumerate()
            .map(|(i, moves)| Piece {
                unit: unit + i,
                moves: moves.clone(),
                first: true,
            })
            .collect();
        unit += block.units.len();
        let whole = RowBlock {
            id,
            outer: block.outer,
            pieces: pieces.clone(),
        };
        if packer.place(&whole) {
            continue;
        }
        for piece in pieces {
            let one = RowBlock {
                id,
                outer: block.outer,
                pieces: vec![piece.clone()],
            };
            if packer.place(&one) {
                continue;
            }
            // wider than a row: icon chunks
            let mut start = piece.moves.start;
            let mut first = true;
            while start < piece.moves.end {
                let end = (start + per_row).min(piece.moves.end);
                let chunk = RowBlock {
                    id,
                    outer: block.outer,
                    pieces: vec![Piece {
                        unit: piece.unit,
                        moves: start..end,
                        first,
                    }],
                };
                if !packer.place(&chunk) {
                    packer.flush();
                    packer.current = vec![chunk];
                }
                first = false;
                start = end;
            }
        }
    }
    packer.flush();
    packer.rows
}

/// The line above a row: `group 3 of 10`, `groups 3-6 of 10`.
pub(super) fn heading(row: &[RowBlock], total: usize) -> String {
    let pieces: Vec<&Piece> = row.iter().flat_map(|b| &b.pieces).collect();
    let (Some(first), Some(last)) = (pieces.first(), pieces.last()) else {
        return String::new();
    };
    let more = if first.first { "" } else { " (continued)" };
    if first.unit == last.unit {
        format!("group {} of {total}{more}", first.unit + 1)
    } else {
        format!(
            "groups {}-{} of {total}{more}",
            first.unit + 1,
            last.unit + 1
        )
    }
}

/// The step pictures of a grouped move list.
pub(super) fn steps(
    pictures: &Pictures<'_>,
    start: &Facelets,
    solution: &MoveSequence,
    layout: &Layout,
) -> Result<String, String> {
    let moves = solution.moves();
    if pictures.graphics == Graphics::Kitty {
        kitty(pictures, start, moves, layout)
    } else {
        text(pictures, start, moves, layout)
    }
}

fn kitty(
    pictures: &Pictures<'_>,
    start: &Facelets,
    moves: &[Move],
    layout: &Layout,
) -> Result<String, String> {
    let style = &pictures.style;
    let mut state = start.clone();
    let mut out = String::new();
    let width = match moves.first() {
        Some(first) => {
            icon(&state, &Action::Layer(*first), style)
                .map_err(|error| error.to_string())?
                .to_rgba(style.size)
                .width
        }
        None => return Ok(out),
    };
    let fit = Fit {
        icon: width,
        width: pictures.terminal.width_px,
        icons: pictures.options.columns.unwrap_or(usize::MAX),
    };
    let lone = lone_quarters(moves);
    let total = layout.units();
    let outer = layout.has_outer();
    for row in pack(layout, fit) {
        let mut parts: Vec<Raster> = Vec::new();
        let mut spans: Vec<BlockSpan> = Vec::new();
        let mut x = 0;
        for (b, block) in row.iter().enumerate() {
            if b > 0 {
                parts.push(spacer(GAP_BLOCK));
                x += GAP_BLOCK;
            }
            let mut units = Vec::new();
            for (p, piece) in block.pieces.iter().enumerate() {
                if p > 0 {
                    parts.push(spacer(GAP_UNIT));
                    x += GAP_UNIT;
                }
                let mut icons = Vec::new();
                for at in piece.moves.clone() {
                    let action = Action::Layer(moves[at]);
                    let drawn = icon(&state, &action, style).map_err(|error| error.to_string())?;
                    let drawn = numbered(drawn, at + 1, pictures.options.numbers, style.backdrop);
                    icons.push(row_icon(drawn.to_rgba(style.size), lone[at]));
                    state = action.apply(&state).map_err(|error| error.to_string())?;
                }
                let unit = Raster::hstack(&icons, GAP_IN);
                units.push(UnitSpan {
                    x,
                    width: unit.width,
                    label: piece.first.then(|| (piece.unit + 1).to_string()),
                });
                x += unit.width;
                parts.push(unit);
            }
            spans.push(BlockSpan {
                outer: block.outer,
                units,
            });
        }
        let strip = Raster::hstack(&parts, 0);
        let header = brackets(&spans, strip.width, style.size, outer, style.backdrop);
        out.push_str(&heading(&row, total));
        out.push('\n');
        out.push_str(&kitty_block(
            &Raster::vstack(&header, &strip),
            pictures.terminal.cell.1,
        ));
    }
    Ok(out)
}

fn spacer(width: u32) -> Raster {
    Raster {
        width,
        height: 1,
        data: vec![0; width as usize * 4],
    }
}

fn text(
    pictures: &Pictures<'_>,
    start: &Facelets,
    moves: &[Move],
    layout: &Layout,
) -> Result<String, String> {
    let notation = pictures.options.notation;
    let kind = start.kind();
    let write = |mv: &Move| notation.write_move(kind, mv);
    let total = layout.units();
    let mut out = layout.line(moves, &write);
    out.push('\n');
    let mut state = start.clone();
    let mut number = 0;
    for block in &layout.blocks {
        for (i, unit) in block.units.iter().enumerate() {
            number += 1;
            let words: Vec<String> = moves[unit.clone()].iter().map(write).collect();
            let open = if block.outer && i == 0 { "(" } else { "" };
            let close = if block.outer && i + 1 == block.units.len() {
                ")"
            } else {
                ""
            };
            out.push_str(&format!(
                "group {number} of {total}: {open}{}{close}\n",
                words.join(" ")
            ));
            for mv in &moves[unit.clone()] {
                let action = Action::Layer(*mv);
                let drawn = icon(&state, &action, &pictures.style).map_err(|e| e.to_string())?;
                out.push_str(&show(&drawn, pictures.graphics, pictures.style.size));
                out.push('\n');
                state = action.apply(&state).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
