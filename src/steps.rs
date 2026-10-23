//! `solve --icons`: the steps of a solution as icons, in phases and rows.
//!
//! On a Kitty terminal every phase has a heading line, then its icons in rows
//! as wide as the terminal (one image per row, see `rows`), then a framed
//! picture of the cube after the phase. Elsewhere there is one text picture per
//! step, as before.

use std::io::Write;

use cubacle::notation::{orbit64, ColourScheme, Notation};
use cubacle::phases::Phase;
use cubacle::puzzle::{Facelets, Move, MoveSequence, Turn};
use cubacle::render::{
    icon, state_icon, state_icon_against, Action, Icon, KittyPlacement, Raster, Style,
};

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
    /// How the step lines and the final solution line write moves (the icon
    /// labels stay as they are).
    pub notation: Notation,
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

/// The heading of `found[at]`. With `collapse`, a run of empty phases gets one
/// line, `Phases 1-5: nothing to do`, on its first phase and none on the
/// others (`None`); a lone empty phase keeps its own heading.
pub(crate) fn heading_at(found: &[Phase], at: usize, collapse: bool) -> Option<String> {
    let phase = &found[at];
    if !collapse || !phase.moves.is_empty() {
        return Some(phase_heading(phase));
    }
    if at > 0 && found[at - 1].moves.is_empty() {
        return None;
    }
    let run = found[at..]
        .iter()
        .take_while(|p| p.moves.is_empty())
        .count();
    if run == 1 {
        return Some(phase_heading(phase));
    }
    Some(format!(
        "Phases {}-{}: nothing to do",
        phase.number,
        found[at + run - 1].number
    ))
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
/// `cubacle recover --state <start> --current <token>`.
pub(crate) fn token_line(phase: &Phase, state: &Facelets) -> Result<String, String> {
    let token = orbit64::encode(state).map_err(|error| error.to_string())?;
    Ok(format!(
        "State after phase {} as token: {token}\n",
        phase.number
    ))
}

/// The text marker for a quarter turn that stands between two half turns, the
/// easiest one to do as a half turn.
const QUARTER_NOTE: &str = "   <-- quarter turn, not a half turn";

/// The frame colour of a lone quarter turn in the Kitty rows (amber).
#[cfg(feature = "quarter-frame")]
const QUARTER_FRAME: cubacle::render::Rgb = cubacle::render::Rgb(255, 176, 0);

/// The icon of a move as it goes into a Kitty row: with the `quarter-frame`
/// feature a lone quarter turn gets an amber frame, otherwise the icon as is.
#[cfg_attr(not(feature = "quarter-frame"), allow(unused_variables))]
fn row_icon(raster: Raster, lone: bool) -> Raster {
    #[cfg(feature = "quarter-frame")]
    if lone {
        return raster.framed(QUARTER_FRAME);
    }
    raster
}

/// Whether each move is a quarter turn with a half turn right before and right
/// after it.
pub(crate) fn lone_quarters(moves: &[Move]) -> Vec<bool> {
    let half = |mv: Option<&Move>| mv.is_some_and(|m| m.turn == Turn::Half);
    (0..moves.len())
        .map(|i| {
            moves[i].turn != Turn::Half && i > 0 && half(moves.get(i - 1)) && half(moves.get(i + 1))
        })
        .collect()
}

/// The line above the text picture of a step: `step 3 of 9: F`, with a note
/// when the move is a lone quarter turn.
pub(crate) fn step_line(index: usize, total: usize, mv: &str, lone: bool) -> String {
    let note = if lone { QUARTER_NOTE } else { "" };
    format!("step {} of {total}: {mv}{note}\n", index + 1)
}

/// The picture options `solve` and `recover` share.
#[derive(clap::Args)]
pub struct PictureArgs {
    /// Show each step with a picture of the cube (graphics on a capable terminal)
    #[arg(long)]
    pub icons: bool,
    /// Icon height in pixels for --icons (default: 32 per cube layer, at least 96)
    #[arg(long, requires = "icons", value_name = "PIXELS")]
    pub icon_size: Option<u32>,
    /// What the --icons pictures are seen on: sets the notation colour and the
    /// dimming (default: ask the terminal, dark when it does not answer)
    #[arg(long, requires = "icons", value_enum)]
    pub background: Option<Background>,
    /// Icons per row for --icons (default: as many as fit the terminal width)
    #[arg(long, requires = "icons", value_name = "N", value_parser = clap::value_parser!(u32).range(1..))]
    pub columns: Option<u32>,
}

impl PictureArgs {
    pub fn options(&self, scheme: Option<ColourScheme>, notation: Notation) -> SolutionIcons {
        SolutionIcons {
            scheme,
            height: self.icon_size,
            background: self.background,
            columns: self.columns.map(|c| c as usize),
            notation,
        }
    }
}

/// The terminal, the style and the options of one run, set up once.
pub struct Pictures<'a> {
    graphics: Graphics,
    terminal: Terminal,
    style: Style,
    options: &'a SolutionIcons,
    collapse_empty: bool,
}

impl<'a> Pictures<'a> {
    pub fn new(cube_size: usize, options: &'a SolutionIcons) -> Pictures<'a> {
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
        let height = options.height.unwrap_or_else(|| default_height(cube_size));
        let style = style(options.scheme, height, false, background);
        Pictures {
            graphics,
            terminal,
            style,
            options,
            collapse_empty: false,
        }
    }

    /// Runs of phases with nothing to do get one heading line (`recover`).
    pub fn collapsing_empty_phases(mut self) -> Pictures<'a> {
        self.collapse_empty = true;
        self
    }

    /// The step pictures of a solution (no start line, no final moves line).
    pub fn steps(
        &self,
        start: &Facelets,
        solution: &MoveSequence,
        found: &[Phase],
    ) -> Result<String, String> {
        if self.graphics == Graphics::Kitty {
            kitty_steps(
                start,
                solution,
                found,
                self.options.columns,
                &self.terminal,
                &self.style,
                self.collapse_empty,
            )
        } else {
            text_steps(
                start,
                solution,
                found,
                self.graphics,
                &self.style,
                self.options.notation,
            )
        }
    }

    /// The cube as it should be and as it is, the second marked where it
    /// differs from the first.
    pub fn states_against(
        &self,
        should: (&str, &Facelets),
        now: (&str, &Facelets),
    ) -> Result<String, String> {
        self.states_with(&[
            (should.0, state_icon(should.1, &self.style), should.1),
            (
                now.0,
                state_icon_against(now.1, should.1, &self.style),
                now.1,
            ),
        ])
    }

    /// Whole-cube pictures, each under its label and followed by its token.
    fn states_with(&self, states: &[(&str, Icon, &Facelets)]) -> Result<String, String> {
        let mut out = String::new();
        for (label, drawn, state) in states {
            out.push_str(label);
            out.push('\n');
            if self.graphics == Graphics::Kitty {
                out.push_str(&kitty_block(
                    &drawn.to_rgba(self.style.size),
                    self.terminal.cell.1,
                ));
            } else {
                out.push_str(&show(drawn, self.graphics, self.style.size));
                out.push('\n');
            }
            let token = orbit64::encode(state).map_err(|error| error.to_string())?;
            out.push_str(&format!("token: {token}\n"));
        }
        Ok(out)
    }
}

/// Each step of the solution with the picture of the cube before it.
pub fn print_solution(
    start: &Facelets,
    solution: &MoveSequence,
    found: &[Phase],
    options: &SolutionIcons,
) -> Result<(), String> {
    let pictures = Pictures::new(start.kind().size(), options);
    let mut out = start_line(start)?;
    out.push_str(&pictures.steps(start, solution, found)?);
    let mut stdout = std::io::stdout().lock();
    stdout
        .write_all(out.as_bytes())
        .and_then(|()| writeln!(stdout, "{}", options.notation.write(solution)))
        .map_err(|e| format!("cannot write to standard output: {e}"))
}

fn text_steps(
    start: &Facelets,
    solution: &MoveSequence,
    found: &[Phase],
    graphics: Graphics,
    style: &Style,
    notation: Notation,
) -> Result<String, String> {
    let mut state = start.clone();
    let total = solution.len();
    let mut out = String::new();
    let lone = lone_quarters(solution.moves());
    for (index, mv) in solution.moves().iter().enumerate() {
        let action = Action::Layer(*mv);
        let drawn = icon(&state, &action, style).map_err(|error| error.to_string())?;
        out.push_str(&step_line(
            index,
            total,
            &notation.write_move(mv),
            lone[index],
        ));
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
    collapse_empty: bool,
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
    let lone = lone_quarters(moves);
    let per_row = rows::per_row(terminal.width_px, width, rows::GAP, columns);
    // Without phases (a route that is not from solver stages) the moves form
    // one segment that has no heading and no state picture.
    let whole = [Phase {
        number: 0,
        name: "",
        moves: 0..total,
    }];
    let named = !found.is_empty();
    let phases = if named { found } else { &whole };
    for (at, phase) in phases.iter().enumerate() {
        if named {
            if let Some(heading) = heading_at(phases, at, collapse_empty) {
                out.push_str(&heading);
                out.push('\n');
            }
        }
        if phase.moves.is_empty() {
            continue;
        }
        for row in rows::rows(phase.moves.clone(), per_row) {
            let mut rasters = Vec::with_capacity(row.len());
            for (at, mv) in row.clone().zip(&moves[row.clone()]) {
                let action = Action::Layer(*mv);
                let drawn = icon(&state, &action, style).map_err(|error| error.to_string())?;
                let raster = drawn.to_rgba(style.size);
                rasters.push(row_icon(raster, lone[at]));
                state = action.apply(&state).map_err(|error| error.to_string())?;
            }
            out.push_str(&rows::heading(&row, total));
            out.push('\n');
            let strip = Raster::hstack(&rasters, rows::GAP);
            out.push_str(&kitty_block(&strip, terminal.cell.1));
        }
        if named {
            out.push_str(&state_heading(phase));
            out.push('\n');
            out.push_str(&kitty_block(
                &state_icon(&state, style).to_rgba(style.size),
                terminal.cell.1,
            ));
            out.push_str(&token_line(phase, &state)?);
        }
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
    fn empty_phases_collapse_into_one_line() {
        let stages = [
            cubacle::puzzle::SolveStage { name: "a", end: 0 },
            cubacle::puzzle::SolveStage { name: "b", end: 0 },
            cubacle::puzzle::SolveStage { name: "c", end: 0 },
            cubacle::puzzle::SolveStage { name: "d", end: 4 },
            cubacle::puzzle::SolveStage { name: "e", end: 4 },
            cubacle::puzzle::SolveStage { name: "f", end: 9 },
        ];
        let found = cubacle::phases::from_stages(&stages, 9).unwrap();
        let lines: Vec<Option<String>> = (0..found.len())
            .map(|i| heading_at(&found, i, true))
            .collect();
        let want = [
            Some("Phases 1-3: nothing to do"),
            None,
            None,
            Some("Phase 4: d (4 moves)"),
            Some("Phase 5: e (nothing to do)"),
            Some("Phase 6: f (5 moves)"),
        ];
        assert_eq!(lines, want.map(|l| l.map(str::to_owned)));
        assert_eq!(
            heading_at(&found, 0, false).as_deref(),
            Some("Phase 1: a (nothing to do)")
        );
    }

    #[test]
    fn a_quarter_turn_between_half_turns_is_flagged() {
        let seq: MoveSequence = "3Rw2 3Dw2 F 3Uw2 R U R2 B R2 F".parse().unwrap();
        let lone = lone_quarters(seq.moves());
        let flagged: Vec<usize> = (0..lone.len()).filter(|&i| lone[i]).collect();
        assert_eq!(flagged, [2, 7]);
        let line = step_line(2, 10, &seq.moves()[2].to_string(), true);
        assert_eq!(
            line,
            "step 3 of 10: F   <-- quarter turn, not a half turn\n"
        );
        assert_eq!(
            step_line(0, 10, &seq.moves()[0].to_string(), false),
            "step 1 of 10: 3Rw2\n"
        );
    }

    #[cfg(not(feature = "quarter-frame"))]
    #[test]
    fn kitty_rows_have_no_amber_frame_by_default() {
        let plain = Raster {
            width: 40,
            height: 40,
            data: vec![7; 40 * 40 * 4],
        };
        let shown = row_icon(plain.clone(), true);
        assert_eq!(shown.data, plain.data);
    }

    #[cfg(feature = "quarter-frame")]
    #[test]
    fn kitty_rows_frame_a_lone_quarter_turn_with_the_feature() {
        let plain = Raster {
            width: 40,
            height: 40,
            data: vec![7; 40 * 40 * 4],
        };
        assert_eq!(row_icon(plain.clone(), false).data, plain.data);
        let shown = row_icon(plain, true);
        assert_eq!(&shown.data[..4], [255, 176, 0, 255]);
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
