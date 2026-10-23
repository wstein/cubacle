//! `solve --icons`: the steps of a solution as icons, in phases and rows.
//!
//! On a Kitty terminal every phase has a heading line, then its icons in rows
//! as wide as the terminal (one image per row, see `rows`), then a framed
//! picture of the cube after the phase. Elsewhere there is one text picture per
//! step, as before.

use std::io::Write;

use cubacle::notation::{orbit64, ColourScheme};
use cubacle::phases::Phase;
use cubacle::puzzle::{Facelets, MoveSequence};
use cubacle::render::{icon, state_icon, Action, KittyPlacement, Raster, Style};

use crate::icons::{default_height, detect_stdout, show, style, Background, Graphics};
use crate::rows;
use crate::terminal::{self, Terminal};

/// What `solve --icons` was asked for.
pub struct SolutionIcons {
    pub scheme: Option<ColourScheme>,
    /// Icon height in pixels.
    pub height: Option<u32>,
    /// `None`: ask the terminal.
    pub background: Option<Background>,
    /// Icons per row; `None`: as many as fit.
    pub columns: Option<usize>,
}

/// A raster as a block of terminal rows: the rows are cleared so that older
/// text cannot show through, then the image is drawn at its own pixel size
/// without moving the cursor, which then steps below it.
fn kitty_block(raster: &Raster, cell_height: u32) -> String {
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

/// The line above the icons of a phase: `Phase 2: edges (14 moves)`.
pub(crate) fn phase_heading(phase: &Phase) -> String {
    let count = phase.moves.len();
    if count == 0 {
        return format!("Phase {}: {} (nothing to do)", phase.number, phase.name);
    }
    let unit = if count == 1 { "move" } else { "moves" };
    format!("Phase {}: {} ({count} {unit})", phase.number, phase.name)
}

/// The `--verbose` line of a phase: `phase 2: edges, moves 15-27`, or
/// `phase 2: edges, nothing to do`. It says what `phase_heading` says.
pub(crate) fn phase_line(phase: &Phase) -> String {
    if phase.moves.is_empty() {
        return format!("phase {}: {}, nothing to do", phase.number, phase.name);
    }
    format!(
        "phase {}: {}, moves {}-{}",
        phase.number,
        phase.name,
        phase.moves.start + 1,
        phase.moves.end
    )
}

/// The line above the picture of the cube after a phase.
pub(crate) fn state_heading(phase: &Phase) -> String {
    format!(
        "State after phase {} - compare it with your cube",
        phase.number
    )
}

/// The line that gives the start state as an Orbit64 token.
pub(crate) fn start_line(state: &Facelets) -> Result<String, String> {
    let token = orbit64::encode(state).map_err(|error| error.to_string())?;
    Ok(format!("Start state: {token}\n"))
}

/// The line that gives the cube after a phase as an Orbit64 token, for
/// `cubacle repair --state <start> --current <token>`.
pub(crate) fn token_line(phase: &Phase, state: &Facelets) -> Result<String, String> {
    let token = orbit64::encode(state).map_err(|error| error.to_string())?;
    Ok(format!(
        "State after phase {} as token: {token}\n",
        phase.number
    ))
}

/// Each step of the solution with the picture of the cube before it.
pub fn print_solution(
    start: &Facelets,
    solution: &MoveSequence,
    found: &[Phase],
    options: &SolutionIcons,
) -> Result<(), String> {
    let graphics = detect_stdout();
    let terminal = if graphics == Graphics::Kitty {
        terminal::probe(options.background.is_none())
    } else {
        Terminal::default()
    };
    let background = options.background.unwrap_or(match terminal.background {
        Some(colour) if colour.is_light() => Background::Light,
        _ => Background::Dark,
    });
    let height = options
        .height
        .unwrap_or_else(|| default_height(start.kind().size()));
    let style = style(options.scheme, height, false, background);
    let mut out = start_line(start)?;
    out.push_str(&if graphics == Graphics::Kitty {
        kitty_steps(start, solution, found, options.columns, &terminal, &style)?
    } else {
        text_steps(start, solution, found, graphics, &style)?
    });
    let mut stdout = std::io::stdout().lock();
    stdout
        .write_all(out.as_bytes())
        .and_then(|()| writeln!(stdout, "{solution}"))
        .map_err(|e| format!("cannot write to standard output: {e}"))
}

fn text_steps(
    start: &Facelets,
    solution: &MoveSequence,
    found: &[Phase],
    graphics: Graphics,
    style: &Style,
) -> Result<String, String> {
    let mut state = start.clone();
    let total = solution.len();
    let mut out = String::new();
    for (index, mv) in solution.moves().iter().enumerate() {
        let action = Action::Layer(*mv);
        let drawn = icon(&state, &action, style).map_err(|error| error.to_string())?;
        out.push_str(&format!("step {} of {total}: {mv}\n", index + 1));
        out.push_str(&show(&drawn, graphics, style.size));
        out.push('\n');
        state = action.apply(&state).map_err(|error| error.to_string())?;
        for phase in found.iter().filter(|p| p.moves.end == index + 1) {
            if !phase.moves.is_empty() {
                out.push_str(&token_line(phase, &state)?);
            }
        }
    }
    Ok(out)
}

fn kitty_steps(
    start: &Facelets,
    solution: &MoveSequence,
    found: &[Phase],
    columns: Option<usize>,
    terminal: &Terminal,
    style: &Style,
) -> Result<String, String> {
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
    let per_row = rows::per_row(terminal.width_px, width, rows::GAP, columns);
    for phase in found {
        out.push_str(&phase_heading(phase));
        out.push('\n');
        if phase.moves.is_empty() {
            continue;
        }
        for row in rows::rows(phase.moves.clone(), per_row) {
            let mut rasters = Vec::with_capacity(row.len());
            for mv in &moves[row.clone()] {
                let action = Action::Layer(*mv);
                let drawn = icon(&state, &action, style).map_err(|error| error.to_string())?;
                rasters.push(drawn.to_rgba(style.size));
                state = action.apply(&state).map_err(|error| error.to_string())?;
            }
            out.push_str(&rows::heading(&row, total));
            out.push('\n');
            let strip = Raster::hstack(&rasters, rows::GAP);
            out.push_str(&kitty_block(&strip, terminal.cell.1));
        }
        out.push_str(&state_heading(phase));
        out.push('\n');
        out.push_str(&kitty_block(
            &state_icon(&state, style).to_rgba(style.size),
            terminal.cell.1,
        ));
        out.push_str(&token_line(phase, &state)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbose_lines_and_headings_agree() {
        let stages = [
            cubacle::puzzle::SolveStage { name: "a", end: 0 },
            cubacle::puzzle::SolveStage { name: "b", end: 4 },
            cubacle::puzzle::SolveStage { name: "c", end: 9 },
        ];
        let found = cubacle::phases::from_stages(&stages, 9).unwrap();
        let lines: Vec<String> = found.iter().map(phase_line).collect();
        assert_eq!(
            lines,
            [
                "phase 1: a, nothing to do",
                "phase 2: b, moves 1-4",
                "phase 3: c, moves 5-9"
            ]
        );
        let headings: Vec<String> = found.iter().map(phase_heading).collect();
        assert_eq!(
            headings,
            [
                "Phase 1: a (nothing to do)",
                "Phase 2: b (4 moves)",
                "Phase 3: c (5 moves)"
            ]
        );
    }

    #[test]
    fn headings_name_the_phase_and_count_its_moves() {
        let phase = Phase {
            number: 2,
            name: "edges",
            moves: 13..27,
        };
        assert_eq!(phase_heading(&phase), "Phase 2: edges (14 moves)");
        let one = Phase {
            number: 1,
            name: "solve",
            moves: 0..1,
        };
        assert_eq!(phase_heading(&one), "Phase 1: solve (1 move)");
        assert_eq!(
            state_heading(&phase),
            "State after phase 2 - compare it with your cube"
        );
    }
}
